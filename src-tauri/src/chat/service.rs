use crate::database::{
    Database, Message, Conversation, CreateMessage, CreateConversation, ChatMessageResponse,
};
use super::conversation_repo::ConversationRepo;
use super::message_repo::MessageRepo;
use super::openai_client::{OpenAIClient, ChatMessage};
use crate::token_tracker::TokenTracker;

/// 聊天服务
pub struct ChatService {
    conversation_repo: ConversationRepo,
    message_repo: MessageRepo,
    openai_client: OpenAIClient,
    token_tracker: TokenTracker,
}

impl ChatService {
    /// 创建新的聊天服务
    pub fn new(db: Database) -> Self {
        Self {
            conversation_repo: ConversationRepo::new(db.clone()),
            message_repo: MessageRepo::new(db.clone()),
            openai_client: super::openai_client::create_default_client(),
            token_tracker: TokenTracker::new(db),
        }
    }

    /// 更新 OpenAI 配置
    pub fn update_openai_config(&mut self, api_key: String, api_base: String, model: String, temperature: f32) {
        self.openai_client.update_config(api_key, api_base, model, temperature);
    }

    /// 创建新会话
    pub fn create_conversation(&self, title: String, model_id: Option<String>) -> Result<Conversation, String> {
        self.conversation_repo
            .create(CreateConversation { title, model_id })
            .map_err(|e| format!("创建会话失败: {}", e))
    }

    /// 获取会话列表
    pub fn list_conversations(&self, limit: i32, offset: i32) -> Result<Vec<Conversation>, String> {
        self.conversation_repo
            .list(limit, offset)
            .map_err(|e| format!("获取会话列表失败: {}", e))
    }

    /// 获取单个会话
    pub fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, String> {
        self.conversation_repo
            .get(id)
            .map_err(|e| format!("获取会话失败: {}", e))
    }

    /// 更新会话标题
    pub fn update_conversation_title(&self, id: &str, title: &str) -> Result<(), String> {
        self.conversation_repo
            .update_title(id, title)
            .map_err(|e| format!("更新会话标题失败: {}", e))
    }

    /// 删除会话
    pub fn delete_conversation(&self, id: &str) -> Result<bool, String> {
        self.conversation_repo
            .delete(id)
            .map_err(|e| format!("删除会话失败: {}", e))
    }

    /// 获取会话消息
    pub fn get_messages(&self, conversation_id: &str, limit: i32, offset: i32) -> Result<Vec<Message>, String> {
        self.message_repo
            .list_by_conversation(conversation_id, limit, offset)
            .map_err(|e| format!("获取消息失败: {}", e))
    }

    /// 清空会话消息
    pub fn clear_conversation(&self, conversation_id: &str) -> Result<i32, String> {
        // 删除消息
        let deleted = self.message_repo
            .delete_by_conversation(conversation_id)
            .map_err(|e| format!("清空消息失败: {}", e))?;

        // 重置会话统计为 0
        self.conversation_repo
            .reset_stats(conversation_id)
            .ok(); // 忽略错误

        Ok(deleted)
    }

    /// 发送消息并获取 AI 回复
    pub async fn send_message(&self, conversation_id: &str, content: &str) -> Result<ChatMessageResponse, String> {
        // 1. 保存用户消息
        let _user_message = self.message_repo
            .create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "user".to_string(),
                content: content.to_string(),
                model_id: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            })
            .map_err(|e| format!("保存用户消息失败: {}", e))?;
    
        // 更新会话统计（+1 消息）
        self.conversation_repo
            .update_stats(conversation_id, 0, 1)
            .ok();
    
        // 2. 获取历史消息构建上下文
        let history = self.message_repo
            .get_recent(conversation_id, 20) // 最近 20 条消息
            .map_err(|e| format!("获取历史消息失败: {}", e))?;
    
        let chat_messages: Vec<ChatMessage> = history
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
    
        // 3. 调用 OpenAI API
        let model_id = self.openai_client.get_model().to_string();
        let chat_result = self.openai_client.chat(chat_messages).await?;
    
        // 4. 保存 AI 回复
        let (prompt_tokens, completion_tokens, total_tokens) = match &chat_result.usage {
            Some(usage) => (usage.prompt_tokens, usage.completion_tokens, usage.total_tokens),
            None => (0, 0, 0),
        };
    
        let ai_message = self.message_repo
            .create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "assistant".to_string(),
                content: chat_result.content.clone(),
                model_id: Some(model_id.clone()),
                prompt_tokens: Some(prompt_tokens),
                completion_tokens: Some(completion_tokens),
                total_tokens: Some(total_tokens),
            })
            .map_err(|e| format!("保存 AI 消息失败: {}", e))?;
    
        // 5. 更新会话统计（+tokens, +1 消息）
        self.conversation_repo
            .update_stats(conversation_id, total_tokens, 1)
            .ok();
    
        // 6. 记录 Token 使用
        if total_tokens > 0 {
            self.token_tracker
                .record_usage(&model_id, prompt_tokens, completion_tokens, total_tokens)
                .ok();
        }
    
        // 7. 获取更新后的会话信息
        let conversation = self.conversation_repo
            .get(conversation_id)
            .map_err(|e| format!("获取会话信息失败: {}", e))?
            .ok_or_else(|| "会话不存在".to_string())?;
    
        Ok(ChatMessageResponse {
            message: ai_message,
            conversation,
        })
    }

    /// 发送消息（简单版本，不调用 AI）
    pub fn send_message_simple(&self, conversation_id: &str, content: &str) -> Result<Message, String> {
        // 保存用户消息
        let user_message = self.message_repo
            .create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "user".to_string(),
                content: content.to_string(),
                model_id: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            })
            .map_err(|e| format!("保存消息失败: {}", e))?;

        // 更新会话统计
        self.conversation_repo
            .update_stats(conversation_id, 0, 1)
            .ok();

        // 返回 echo 消息（用于测试）
        let echo_message = self.message_repo
            .create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "assistant".to_string(),
                content: format!("AI回复: {}", content),
                model_id: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            })
            .map_err(|e| format!("保存回复失败: {}", e))?;

        self.conversation_repo
            .update_stats(conversation_id, 0, 1)
            .ok();

        Ok(echo_message)
    }

    /// 获取 Token 统计
    pub fn get_token_tracker(&self) -> &TokenTracker {
        &self.token_tracker
    }
}
