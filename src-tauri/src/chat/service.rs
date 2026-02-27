use crate::database::{
    Database, Message, Conversation, ChatMessageResponse, CreateConversation, CreateMessage,
};
use super::conversation_repo::ConversationRepo;
use super::message_repo::MessageRepo;
use super::openai_client::{OpenAIClient, ChatMessage};
use crate::token_tracker::TokenTracker;
use crate::validation::validate_conversation_title;
use crate::{safe_db, error::{AppError, AppResult}};

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
    pub fn create_conversation(&self, title: String, model_id: Option<String>) -> AppResult<Conversation> {
        // 验证会话标题
        let validated_title = validate_conversation_title(&title)
            .map_err(|e| AppError::ValidationError(e.to_string()))?;
        
        let conversation = safe_db!(
            self.conversation_repo.create(CreateConversation { 
                title: validated_title, 
                model_id 
            }),
            "create_conversation"
        );
        Ok(conversation)
    }

    /// 获取会话列表
    pub fn list_conversations(&self, limit: i32, offset: i32) -> AppResult<Vec<Conversation>> {
        Ok(safe_db!(self.conversation_repo.list(limit, offset), "list_conversations"))
    }

    /// 获取单个会话
    pub fn get_conversation(&self, id: &str) -> AppResult<Option<Conversation>> {
        Ok(safe_db!(self.conversation_repo.get(id), "get_conversation"))
    }

    /// 更新会话标题
    pub fn update_conversation_title(&self, id: &str, title: &str) -> AppResult<()> {
        safe_db!(self.conversation_repo.update_title(id, title), "update_conversation_title");
        Ok(())
    }

    /// 删除会话
    pub fn delete_conversation(&self, id: &str) -> AppResult<bool> {
        Ok(safe_db!(self.conversation_repo.delete(id), "delete_conversation"))
    }

    /// 获取会话消息
    pub fn get_messages(&self, conversation_id: &str, limit: i32, offset: i32) -> AppResult<Vec<Message>> {
        Ok(safe_db!(self.message_repo.list_by_conversation(conversation_id, limit, offset), "get_messages"))
    }

    /// 清空会话消息
    pub fn clear_conversation(&self, conversation_id: &str) -> AppResult<i32> {
        // 删除消息
        let deleted = safe_db!(
            self.message_repo.delete_by_conversation(conversation_id), 
            "delete_conversation_messages"
        );

        // 重置会话统计为 0 (忽略错误)
        self.conversation_repo
            .reset_stats(conversation_id)
            .ok();

        Ok(deleted)
    }

    /// 发送消息并获取 AI 回复
    pub async fn send_message(&self, conversation_id: &str, content: &str) -> AppResult<ChatMessageResponse> {
        // 1. 保存用户消息
        let _user_message = safe_db!(
            self.message_repo.create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "user".to_string(),
                content: content.to_string(),
                model_id: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            }),
            "create_user_message"
        );
    
        // 更新会话统计（+1 消息）
        self.conversation_repo
            .update_stats(conversation_id, 0, 1)
            .ok();
    
        // 2. 获取历史消息构建上下文
        let history = safe_db!(
            self.message_repo.get_recent(conversation_id, 20), // 最近 20 条消息
            "get_recent_messages"
        );
    
        let chat_messages: Vec<ChatMessage> = history
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
    
        // 3. 调用 OpenAI API
        let model_id = self.openai_client.get_model().to_string();
        let chat_result = self.openai_client.chat(chat_messages).await
            .map_err(|e| AppError::NetworkError(e))?;
    
        // 4. 保存 AI 回复
        let (prompt_tokens, completion_tokens, total_tokens) = match &chat_result.usage {
            Some(usage) => (usage.prompt_tokens, usage.completion_tokens, usage.total_tokens),
            None => (0, 0, 0),
        };
    
        let ai_message = safe_db!(
            self.message_repo.create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "assistant".to_string(),
                content: chat_result.content.clone(),
                model_id: Some(model_id.clone()),
                prompt_tokens: Some(prompt_tokens),
                completion_tokens: Some(completion_tokens),
                total_tokens: Some(total_tokens),
            }),
            "create_ai_message"
        );
    
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
        let conversation_opt = safe_db!(self.conversation_repo.get(conversation_id), "get_conversation");
        let conversation = conversation_opt.ok_or_else(|| AppError::DatabaseError("会话不存在".to_string()))?;
    
        Ok(ChatMessageResponse {
            message: ai_message,
            conversation,
        })
    }

    /// 发送消息（简单版本，不调用 AI）
    pub fn send_message_simple(&self, conversation_id: &str, content: &str) -> AppResult<Message> {
        // 保存用户消息
        let _user_message = safe_db!(
            self.message_repo.create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "user".to_string(),
                content: content.to_string(),
                model_id: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            }),
            "create_user_message_simple"
        );

        // 更新会话统计
        self.conversation_repo
            .update_stats(conversation_id, 0, 1)
            .ok();

        // 返回 echo 消息（用于测试）
        let echo_message = safe_db!(
            self.message_repo.create(CreateMessage {
                conversation_id: conversation_id.to_string(),
                role: "assistant".to_string(),
                content: format!("AI回复: {}", content),
                model_id: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            }),
            "create_echo_message"
        );

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
