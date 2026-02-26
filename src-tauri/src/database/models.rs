use serde::{Deserialize, Serialize};

/// 会话数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub model_id: Option<String>,
    pub total_tokens: i32,
    pub message_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建会话请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversation {
    pub title: String,
    pub model_id: Option<String>,
}

/// 消息数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String, // "user" | "assistant" | "system"
    pub content: String,
    pub model_id: Option<String>,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub created_at: String,
}

/// 创建消息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub model_id: Option<String>,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
}

/// Token 使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub id: i64,
    pub model_id: String,
    pub date: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub request_count: i32,
    pub created_at: String,
}

/// Token 使用汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_tokens: i64,
    pub total_requests: i64,
}

/// 每日 Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTokenUsage {
    pub date: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub request_count: i32,
}

/// 模型配置数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub model_type: String, // "local" | "remote"
    pub config: String,     // JSON string
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建模型配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub model_type: String,
    pub config: String,
    pub enabled: Option<bool>,
}

/// 聊天消息响应（包含 Token 信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub message: Message,
    pub conversation: Conversation,
}
