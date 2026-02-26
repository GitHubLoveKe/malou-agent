use tauri::Manager;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

mod config;
mod database;
mod chat;
mod token_tracker;

use config::ConfigManager;
use database::{Database, get_default_db_path, Conversation, Message, TokenSummary, DailyTokenUsage};
use chat::ChatService;

// ============ 数据结构 ============

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppInfo {
    name: String,
    version: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAIConfig {
    api_key: String,
    api_base: String,
    model: String,
    temperature: f32,
}

/// 发送消息请求
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SendMessageRequest {
    conversation_id: String,
    content: String,
}

/// 发送消息响应
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SendMessageResponse {
    id: String,
    content: String,
    role: String,
    conversation_id: String,
    timestamp: String,
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

// ============ 应用状态 ============

struct AppState {
    db: Arc<Mutex<Database>>,  // 共享数据库连接
    chat_service: Mutex<ChatService>,
    openai_config: Mutex<OpenAIConfig>,
}

// ============ 基础命令 ============

#[tauri::command]
async fn get_app_info() -> Result<AppInfo, String> {
    Ok(AppInfo {
        name: "Malou Agent".to_string(),
        version: "0.1.0".to_string(),
        description: "Windows桌面AI助手".to_string(),
    })
}

// ============ 会话管理命令 ============

#[tauri::command]
fn create_conversation(
    title: String,
    state: tauri::State<'_, AppState>
) -> Result<Conversation, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.create_conversation(title, None)
}

#[tauri::command]
fn list_conversations(
    limit: Option<i32>,
    offset: Option<i32>,
    state: tauri::State<'_, AppState>
) -> Result<Vec<Conversation>, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.list_conversations(limit.unwrap_or(50), offset.unwrap_or(0))
}

#[tauri::command]
fn get_conversation(
    id: String,
    state: tauri::State<'_, AppState>
) -> Result<Option<Conversation>, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.get_conversation(&id)
}

#[tauri::command]
fn update_conversation_title(
    id: String,
    title: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.update_conversation_title(&id, &title)
}

#[tauri::command]
fn delete_conversation(
    id: String,
    state: tauri::State<'_, AppState>
) -> Result<bool, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.delete_conversation(&id)
}

// ============ 消息管理命令 ============

#[tauri::command]
fn get_conversation_messages(
    conversation_id: String,
    limit: Option<i32>,
    offset: Option<i32>,
    state: tauri::State<'_, AppState>
) -> Result<Vec<Message>, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.get_messages(&conversation_id, limit.unwrap_or(100), offset.unwrap_or(0))
}

#[tauri::command]
fn clear_conversation_messages(
    conversation_id: String,
    state: tauri::State<'_, AppState>
) -> Result<i32, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.clear_conversation(&conversation_id)
}

#[tauri::command]
async fn send_message(
    request: SendMessageRequest,
    state: tauri::State<'_, AppState>
) -> Result<SendMessageResponse, String> {
    // 克隆需要在异步块中使用的数据
    let conversation_id = request.conversation_id.clone();
    let content = request.content.clone();
    
    // 获取数据库路径，用于在 spawn_blocking 中创建新的 ChatService
    let db_path = get_default_db_path();
    
    // 获取 OpenAI 配置
    let openai_config = {
        let config = state.openai_config.lock().unwrap();
        config.clone()
    };
    
    // 在阻塞线程中执行数据库操作和异步 API 调用
    let result = tokio::task::spawn_blocking(move || {
        // 创建新的数据库连接和 ChatService（因为 ChatService 不支持 Send）
        let db = Database::new(db_path)
            .map_err(|e| format!("数据库连接失败: {}", e))?;
        
        let chat_service = ChatService::new(db);
        
        // 更新 OpenAI 配置
        let mut chat_service = chat_service;
        chat_service.update_openai_config(
            openai_config.api_key,
            openai_config.api_base,
            openai_config.model,
            openai_config.temperature,
        );
        
        // 使用 block_on 执行异步的 send_message
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            chat_service.send_message(&conversation_id, &content).await
        })
    }).await.map_err(|e| format!("任务执行失败: {}", e))?;
    
    match result {
        Ok(response) => Ok(SendMessageResponse {
            id: response.message.id,
            content: response.message.content,
            role: response.message.role,
            conversation_id: response.message.conversation_id,
            timestamp: response.message.created_at,
            prompt_tokens: response.message.prompt_tokens,
            completion_tokens: response.message.completion_tokens,
            total_tokens: response.message.total_tokens,
        }),
        Err(e) => Err(e)
    }
}

// ============ Token 统计命令 ============

#[tauri::command]
fn get_token_usage_summary(
    start_date: Option<String>,
    end_date: Option<String>,
    state: tauri::State<'_, AppState>
) -> Result<TokenSummary, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.get_token_tracker()
        .get_summary(start_date.as_deref(), end_date.as_deref())
        .map_err(|e| format!("获取 Token 统计失败: {}", e))
}

#[tauri::command]
fn get_token_usage_trend(
    days: Option<i32>,
    state: tauri::State<'_, AppState>
) -> Result<Vec<DailyTokenUsage>, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.get_token_tracker()
        .get_daily_trend(days.unwrap_or(30))
        .map_err(|e| format!("获取 Token 趋势失败: {}", e))
}

// ============ OpenAI 配置命令 ============

#[tauri::command]
fn get_openai_config(
    state: tauri::State<'_, AppState>
) -> Result<OpenAIConfig, String> {
    let config = state.openai_config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
fn save_openai_config(
    config: OpenAIConfig,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    // 更新内存中的配置
    {
        let mut openai_config = state.openai_config.lock().unwrap();
        *openai_config = config.clone();
    }
    
    // 更新 ChatService 中的 OpenAI 客户端配置
    // 注意：由于 update_openai_config 是异步的，而 ChatService 不支持 Send
    // 这里暂时跳过 OpenAI 客户端配置的更新
    // TODO: 重构 ChatService 以支持 Send 后启用
    println!("OpenAI配置已更新（内存中）");
    Ok(())
}

// ============ 数据库测试命令 ============

#[tauri::command]
fn test_database(
    state: tauri::State<'_, AppState>
) -> Result<String, String> {
    let chat_service = state.chat_service.lock().unwrap();
    
    // 测试创建会话
    let conversation = chat_service.create_conversation("测试会话".to_string(), None)?;
    
    // 测试发送消息
    let _message = chat_service.send_message_simple(&conversation.id, "测试消息")?;
    
    // 测试获取消息
    let messages = chat_service.get_messages(&conversation.id, 10, 0)?;
    
    // 删除测试会话
    chat_service.delete_conversation(&conversation.id)?;
    
    Ok(format!("数据库测试成功! 创建会话、发送 {} 条消息、删除会话均正常", messages.len()))
}

// ============ 主函数 ============

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            println!("Malou Agent桌面应用启动成功!");
            println!("应用路径: {:?}", app.path().app_data_dir());
            
            // 初始化数据库
            let db_path = get_default_db_path();
            println!("数据库路径: {:?}", db_path);
            
            let db = Database::new(db_path)
                .map_err(|e| format!("数据库初始化失败: {}", e))?;
            
            println!("数据库初始化成功");
            
            // 创建 ChatService
            let chat_service = ChatService::new(db.clone());
            
            // 创建应用状态
            let app_state = AppState {
                db: Arc::new(Mutex::new(db)),
                chat_service: Mutex::new(chat_service),
                openai_config: Mutex::new(OpenAIConfig {
                    api_key: String::new(),
                    api_base: "https://api.openai.com/v1".to_string(),
                    model: "gpt-3.5-turbo".to_string(),
                    temperature: 0.7,
                }),
            };
            
            app.manage(app_state);
            
            // 初始化配置管理器（ONNX 配置等）
            match ConfigManager::new(app.app_handle()) {
                Ok(config_manager) => {
                    app.manage(Mutex::new(config_manager));
                    println!("配置管理器初始化成功");
                }
                Err(e) => {
                    eprintln!("配置管理器初始化失败: {}", e);
                }
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 基础命令
            get_app_info,
            test_database,
            // 会话管理
            create_conversation,
            list_conversations,
            get_conversation,
            update_conversation_title,
            delete_conversation,
            // 消息管理
            send_message,
            get_conversation_messages,
            clear_conversation_messages,
            // Token 统计
            get_token_usage_summary,
            get_token_usage_trend,
            // OpenAI 配置
            get_openai_config,
            save_openai_config,
            // 原有配置管理命令
            config::get_app_config,
            config::update_app_config,
            config::get_current_model_info,
            config::update_model_selection,
            config::toggle_local_models,
            config::toggle_onnx_feature
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
