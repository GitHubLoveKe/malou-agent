use tauri::Manager;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

mod config;
mod database;
mod chat;
mod token_tracker;

use config::{ConfigManager, CurrentModel};
use database::{Database, get_default_db_path, Conversation, Message, TokenSummary, DailyTokenUsage};
use chat::ChatService;

// ============ Data Structures ============

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

/// Send message request
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SendMessageRequest {
    conversation_id: String,
    content: String,
}

/// Send message response
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

// ============ App State ============

struct AppState {
    db: Arc<Mutex<Database>>,
    chat_service: Mutex<ChatService>,
    openai_config: Mutex<OpenAIConfig>,
}

// ============ Basic Commands ============

#[tauri::command]
async fn get_app_info() -> Result<AppInfo, String> {
    Ok(AppInfo {
        name: "Malou Agent".to_string(),
        version: "0.1.0".to_string(),
        description: "Windows Desktop AI Assistant".to_string(),
    })
}

// ============ Conversation Management Commands ============

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

// ============ Message Management Commands ============

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
    log::info!("send_message called: conversation_id={}, content_len={}", 
        request.conversation_id, request.content.len());
    
    // Clone data needed in async block
    let conversation_id = request.conversation_id.clone();
    let content = request.content.clone();
    
    // Get database path
    let db_path = get_default_db_path();
    
    // Get OpenAI config
    let openai_config = {
        let config = state.openai_config.lock().unwrap();
        config.clone()
    };
    
    log::debug!("Using OpenAI config: model={}, api_base={}", openai_config.model, openai_config.api_base);
    
    // Execute in blocking thread
    let result = tokio::task::spawn_blocking(move || {
        // Create new database connection
        let db = Database::new(db_path)
            .map_err(|e| {
                log::error!("Database connection failed: {}", e);
                format!("Database connection failed: {}", e)
            })?;
        
        let chat_service = ChatService::new(db);
        
        // Update OpenAI config
        let mut chat_service = chat_service;
        chat_service.update_openai_config(
            openai_config.api_key,
            openai_config.api_base,
            openai_config.model,
            openai_config.temperature,
        );
        
        // Use block_on to execute async send_message
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            chat_service.send_message(&conversation_id, &content).await
        })
    }).await.map_err(|e| {
        log::error!("Task execution failed: {}", e);
        format!("Task execution failed: {}", e)
    })?;
    
    match result {
        Ok(response) => {
            log::info!("Message sent successfully: id={}, tokens={}", 
                response.message.id, response.message.total_tokens);
            Ok(SendMessageResponse {
                id: response.message.id,
                content: response.message.content,
                role: response.message.role,
                conversation_id: response.message.conversation_id,
                timestamp: response.message.created_at,
                prompt_tokens: response.message.prompt_tokens,
                completion_tokens: response.message.completion_tokens,
                total_tokens: response.message.total_tokens,
            })
        },
        Err(e) => {
            log::error!("Send message failed: {}", e);
            Err(e)
        }
    }
}

// ============ Token Statistics Commands ============

#[tauri::command]
fn get_token_usage_summary(
    start_date: Option<String>,
    end_date: Option<String>,
    state: tauri::State<'_, AppState>
) -> Result<TokenSummary, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.get_token_tracker()
        .get_summary(start_date.as_deref(), end_date.as_deref())
        .map_err(|e| format!("Get token summary failed: {}", e))
}

#[tauri::command]
fn get_token_usage_trend(
    days: Option<i32>,
    state: tauri::State<'_, AppState>
) -> Result<Vec<DailyTokenUsage>, String> {
    let chat_service = state.chat_service.lock().unwrap();
    chat_service.get_token_tracker()
        .get_daily_trend(days.unwrap_or(30))
        .map_err(|e| format!("Get token trend failed: {}", e))
}

// ============ OpenAI Config Commands ============

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
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    log::info!("Saving OpenAI config: model={}, api_base={}", config.model, config.api_base);
    
    // Update memory config
    {
        let mut openai_config = state.openai_config.lock().unwrap();
        *openai_config = config.clone();
    }
    
    // Sync to ConfigManager (update the remote model config)
    let config_manager = app_handle.state::<std::sync::Mutex<ConfigManager>>();
    let mut manager = config_manager.lock().unwrap();
    
    manager.update_section(|app_config| {
        // Find and update the current remote model
        let current_model_id = &app_config.model_selector.current_model;
        if let Some(remote_model) = app_config.remote_models.iter_mut()
            .find(|m| &m.id == current_model_id)
        {
            remote_model.api_key = config.api_key.clone();
            remote_model.api_url = config.api_base.clone();
            remote_model.model = config.model.clone();
            remote_model.temperature = Some(config.temperature);
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    }).map_err(|e| {
        log::error!("Failed to sync config to ConfigManager: {}", e);
        format!("Failed to sync config: {}", e)
    })?;
    
    log::info!("OpenAI config saved and synced to ConfigManager");
    Ok(())
}

// ============ Database Test Command ============

#[tauri::command]
fn test_database(
    state: tauri::State<'_, AppState>
) -> Result<String, String> {
    let chat_service = state.chat_service.lock().unwrap();
    
    // Test create conversation
    let conversation = chat_service.create_conversation("Test Conversation".to_string(), None)?;
    
    // Test send message
    let _message = chat_service.send_message_simple(&conversation.id, "Test message")?;
    
    // Test get messages
    let messages = chat_service.get_messages(&conversation.id, 10, 0)?;
    
    // Delete test conversation
    chat_service.delete_conversation(&conversation.id)?;
    
    Ok(format!("Database test successful! Created conversation, sent {} messages, deleted conversation", messages.len()))
}

// ============ Main Function ============

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .init();
    
    log::info!("Malou Agent starting...");
    
    tauri::Builder::default()
        .setup(|app| {
            // Initialize config manager first
            let config_manager = match ConfigManager::new(app.app_handle()) {
                Ok(cm) => cm,
                Err(e) => {
                    log::error!("Config manager init failed: {}", e);
                    return Err(format!("Config manager init failed: {}", e).into());
                }
            };
            
            // Load OpenAI config from saved remote model config
            let openai_config = if let Some(CurrentModel::Remote(remote_model)) = config_manager.get_current_model() {
                log::info!("Loaded remote model config: id={}, model={}", remote_model.id, remote_model.model);
                OpenAIConfig {
                    api_key: remote_model.api_key,
                    api_base: remote_model.api_url,
                    model: remote_model.model,
                    temperature: remote_model.temperature.unwrap_or(0.7),
                }
            } else {
                log::warn!("No remote model config found, using defaults");
                OpenAIConfig {
                    api_key: String::new(),
                    api_base: "https://api.openai.com/v1".to_string(),
                    model: "gpt-3.5-turbo".to_string(),
                    temperature: 0.7,
                }
            };
            
            log::info!("OpenAI config loaded: model={}, api_base={}", openai_config.model, openai_config.api_base);
            
            // Register config manager
            app.manage(Mutex::new(config_manager));
            
            // Initialize database
            let db_path = get_default_db_path();
            log::info!("Database path: {:?}", db_path);
            
            let db = Database::new(db_path)
                .map_err(|e| {
                    log::error!("Database init failed: {}", e);
                    format!("Database init failed: {}", e)
                })?;
            
            log::info!("Database initialized");
            
            // Create ChatService
            let chat_service = ChatService::new(db.clone());
            
            // Create application state
            let app_state = AppState {
                db: Arc::new(Mutex::new(db)),
                chat_service: Mutex::new(chat_service),
                openai_config: Mutex::new(openai_config),
            };
            
            app.manage(app_state);
            
            log::info!("Malou Agent started successfully!");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Basic commands
            get_app_info,
            test_database,
            // Conversation management
            create_conversation,
            list_conversations,
            get_conversation,
            update_conversation_title,
            delete_conversation,
            // Message management
            send_message,
            get_conversation_messages,
            clear_conversation_messages,
            // Token statistics
            get_token_usage_summary,
            get_token_usage_trend,
            // OpenAI config
            get_openai_config,
            save_openai_config,
            // Config management commands
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
