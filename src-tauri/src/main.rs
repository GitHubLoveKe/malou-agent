use tauri::Manager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppInfo {
    name: String,
    version: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    id: String,
    content: String,
    timestamp: String,
    is_user: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAIConfig {
    api_key: String,
    api_base: String,
    model: String,
    temperature: f32,
}

struct AppState {
    messages: Arc<Mutex<Vec<Message>>>,
    openai_config: Arc<Mutex<OpenAIConfig>>,
}

// Tauri命令：获取应用信息
#[tauri::command]
async fn get_app_info() -> Result<AppInfo, String> {
    Ok(AppInfo {
        name: "Malou Agent".to_string(),
        version: "0.1.0".to_string(),
        description: "Windows桌面AI助手".to_string(),
    })
}

// Tauri命令：发送消息
#[tauri::command]
async fn send_message(
    content: String,
    state: tauri::State<'_, AppState>
) -> Result<Message, String> {
    let mut messages = state.messages.lock().await;
    
    // 创建用户消息
    let user_message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        content: content.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_user: true,
    };
    
    messages.push(user_message.clone());
    
    // 创建AI回复（简单echo，后续接入OpenAI）
    let ai_message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        content: format!("AI回复: {}", content),
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_user: false,
    };
    
    messages.push(ai_message.clone());
    
    Ok(ai_message)
}

// Tauri命令：获取消息历史
#[tauri::command]
async fn get_messages(
    state: tauri::State<'_, AppState>
) -> Result<Vec<Message>, String> {
    let messages = state.messages.lock().await;
    Ok(messages.clone())
}

// Tauri命令：清空消息历史
#[tauri::command]
async fn clear_messages(
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut messages = state.messages.lock().await;
    messages.clear();
    Ok(())
}

// Tauri命令：获取OpenAI配置
#[tauri::command]
async fn get_openai_config(
    state: tauri::State<'_, AppState>
) -> Result<OpenAIConfig, String> {
    let config = state.openai_config.lock().await;
    Ok(config.clone())
}

// Tauri命令：保存OpenAI配置
#[tauri::command]
async fn save_openai_config(
    config: OpenAIConfig,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut openai_config = state.openai_config.lock().await;
    *openai_config = config;
    println!("OpenAI配置已保存: {:?}", openai_config);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            messages: Arc::new(Mutex::new(Vec::new())),
            openai_config: Arc::new(Mutex::new(OpenAIConfig {
                api_key: "".to_string(),
                api_base: "https://api.openai.com/v1".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                temperature: 0.7,
            })),
        })
        .setup(|app| {
            println!("Malou Agent桌面应用启动成功!");
            println!("应用路径: {:?}", app.path().app_data_dir());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            send_message,
            get_messages,
            clear_messages,
            get_openai_config,
            save_openai_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}