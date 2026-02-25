mod database;
mod onnx;
mod vector_search;
mod concurrency;
mod db_test;

use specta::Type;
use tauri_specta::Event;
use serde::{Deserialize, Serialize};

#[derive(Type, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageRequest {
    pub content: String,
    pub conversation_id: Option<String>,
}

#[derive(Type, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub content: String,
    pub conversation_id: String,
    pub timestamp: String,
}

#[tauri::command]
#[specta::specta]
async fn send_message(request: MessageRequest) -> Result<MessageResponse, String> {
    // 模拟处理消息
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    Ok(MessageResponse {
        content: format!("Echo: {}", request.content),
        conversation_id: request.conversation_id.unwrap_or_else(|| "default".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
#[specta::specta]
async fn search_knowledge(query: String) -> Result<Vec<String>, String> {
    // 模拟知识库搜索
    Ok(vec![
        format!("搜索结果1: {}", query),
        format!("搜索结果2: {}", query),
    ])
}

#[tauri::command]
#[specta::specta]
async fn run_skill(skill_name: String, params: serde_json::Value) -> Result<serde_json::Value, String> {
    // 模拟技能执行
    Ok(serde_json::json!({
        "result": format!("执行技能: {}", skill_name),
        "params": params
    }))
}

#[tauri::command]
#[specta::specta]
async fn embed_text(text: String) -> Result<Vec<f32>, String> {
    // 模拟文本嵌入
    Ok(vec![0.1, 0.2, 0.3, 0.4])
}

#[tauri::command]
#[specta::specta]
async fn test_database() -> Result<String, String> {
    match db_test::test_database_operations().await {
        Ok(_) => Ok("数据库测试通过".to_string()),
        Err(e) => Err(format!("数据库测试失败: {}", e)),
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            send_message,
            search_knowledge,
            run_skill,
            embed_text,
            test_database
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}