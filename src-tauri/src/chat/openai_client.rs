use reqwest::Client;
use serde::{Deserialize, Serialize};

/// OpenAI API 客户端
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    temperature: f32,
}

/// OpenAI 同步阻塞客户端
pub struct OpenAIBlockingClient {
    client: reqwest::blocking::Client,
    api_key: String,
    api_base: String,
    model: String,
    temperature: f32,
}

/// OpenAI 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI 聊天请求
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

/// OpenAI 使用量统计
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

/// OpenAI 聊天响应
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    role: String,
    content: String,
}

/// OpenAI 聊天结果
#[derive(Debug, Clone)]
pub struct ChatResult {
    pub content: String,
    pub role: String,
    pub usage: Option<Usage>,
}

impl OpenAIClient {
    /// 创建新的 OpenAI 客户端
    pub fn new(api_key: String, api_base: String, model: String, temperature: f32) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_base,
            model,
            temperature,
        }
    }

    /// 发送聊天请求
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResult, String> {
        let url = format!("{}/chat/completions", self.api_base);

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
        };

        log::debug!("OpenAI API 异步请求: url={}, model={}", url, self.model);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("OpenAI API 错误 ({}): {}", status, error_text);
            return Err(format!("API 错误 ({}): {}", status, error_text));
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        let choice = chat_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| "没有返回内容".to_string())?;

        log::info!("OpenAI API 异步响应成功, tokens: {:?}", chat_response.usage.as_ref().map(|u| u.total_tokens));

        Ok(ChatResult {
            content: choice.message.content,
            role: choice.message.role,
            usage: chat_response.usage,
        })
    }

    /// 更新配置
    pub fn update_config(&mut self, api_key: String, api_base: String, model: String, temperature: f32) {
        self.api_key = api_key;
        self.api_base = api_base;
        self.model = model;
        self.temperature = temperature;
    }

    /// 获取当前模型名称
    pub fn get_model(&self) -> &str {
        &self.model
    }
}

impl OpenAIBlockingClient {
    /// 创建新的同步 OpenAI 客户端
    pub fn new(api_key: String, api_base: String, model: String, temperature: f32) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            api_key,
            api_base,
            model,
            temperature,
        }
    }

    /// 同步发送聊天请求
    pub fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResult, String> {
        if self.api_key.is_empty() {
            log::warn!("OpenAI API key 为空，请在设置中配置 API key");
            return Err("API key 未配置，请在设置页面配置 API key".to_string());
        }

        let url = format!("{}/chat/completions", self.api_base);

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
        };

        log::info!("OpenAI API 请求: url={}, model={}", url, self.model);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .map_err(|e| {
                log::error!("OpenAI API 网络请求失败: {}", e);
                format!("请求失败: {}", e)
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().unwrap_or_default();
            log::error!("OpenAI API 错误 ({}): {}", status, error_text);
            return Err(format!("API 错误 ({}): {}", status, error_text));
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .map_err(|e| {
                log::error!("解析 OpenAI 响应失败: {}", e);
                format!("解析响应失败: {}", e)
            })?;

        let choice = chat_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| "没有返回内容".to_string())?;

        log::info!("OpenAI API 响应成功, tokens: {:?}", chat_response.usage.as_ref().map(|u| u.total_tokens));

        Ok(ChatResult {
            content: choice.message.content,
            role: choice.message.role,
            usage: chat_response.usage,
        })
    }

    /// 更新配置
    pub fn update_config(&mut self, api_key: String, api_base: String, model: String, temperature: f32) {
        self.api_key = api_key;
        self.api_base = api_base;
        self.model = model;
        self.temperature = temperature;
    }

    /// 获取当前模型名称
    pub fn get_model(&self) -> &str {
        &self.model
    }
}

/// 创建默认配置的 OpenAI 客户端
pub fn create_default_client() -> OpenAIClient {
    OpenAIClient::new(
        String::new(),
        "https://api.openai.com/v1".to_string(),
        "gpt-3.5-turbo".to_string(),
        0.7,
    )
}

/// 创建默认配置的同步 OpenAI 客户端
pub fn create_default_blocking_client() -> OpenAIBlockingClient {
    OpenAIBlockingClient::new(
        String::new(),
        "https://api.openai.com/v1".to_string(),
        "gpt-3.5-turbo".to_string(),
        0.7,
    )
}
