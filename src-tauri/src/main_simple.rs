use tauri::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 启动Tauri应用
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

// 简化的Tauri命令处理函数
#[tauri::command]
async fn send_message(message: &str) -> Result<String, String> {
    log::info!("收到消息: {}", message);
    
    // 简单的回显响应
    let response = format!("收到您的消息: \"{}\"。我是Malou Agent，很高兴为您服务！", message);
    Ok(response)
}