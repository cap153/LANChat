// commands.rs - 仅用于桌面端的 Tauri 命令
#[cfg(feature = "desktop")]
use crate::db::DbState;
#[cfg(feature = "desktop")]
use crate::peers::{Peer, PeerManager};
#[cfg(feature = "desktop")]
use std::sync::Arc;
#[cfg(feature = "desktop")]
use tauri::State;

// 用于管理 PeerManager 的状态
#[cfg(feature = "desktop")]
pub struct PeerState {
    pub manager: Arc<PeerManager>,
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn get_my_name(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_my_name");
    crate::db::get_username(&state.pool).await
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn get_my_id(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_my_id");
    crate::db::get_user_id(&state.pool).await
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn update_my_name(state: State<'_, DbState>, new_name: String) -> Result<String, String> {
    println!("[Command] 收到前端请求: update_my_name, 新名字: {}", new_name);
    
    // 更新数据库
    crate::db::update_username(&state.pool, new_name.clone()).await?;
    
    // 返回更新后的名字
    Ok(new_name)
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn get_peers(state: State<'_, PeerState>) -> Result<Vec<Peer>, String> {
    Ok(state.manager.get_all_peers())
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn send_message(
    state: State<'_, DbState>,
    peer_id: String,
    peer_addr: String,
    content: String,
) -> Result<(), String> {
    println!("[Command] 收到发送消息请求: 发送给 {}", peer_id);
    
    // 获取自己的信息
    let my_id = crate::db::get_user_id(&state.pool).await?;
    let my_name = crate::db::get_username(&state.pool).await?;
    
    // 发送消息
    crate::network::messaging::send_text_message(&peer_addr, my_id, my_name, content.clone()).await?;
    
    // 保存到数据库(标记为自己发送的)
    sqlx::query(
        "INSERT INTO messages (sender_id, content, msg_type, timestamp) VALUES ('me', ?, 'text', ?)"
    )
    .bind(&content)
    .bind(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64)
    .execute(&state.pool)
    .await
    .map_err(|e| format!("保存消息失败: {}", e))?;
    
    Ok(())
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn get_chat_history(
    state: State<'_, DbState>,
    peer_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    crate::network::messaging::get_chat_history(&state.pool, &peer_id, 100).await
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn send_file(
    state: State<'_, DbState>,
    peer_id: String,
    peer_addr: String,
    file_path: String,
) -> Result<serde_json::Value, String> {
    println!("[Command] 收到发送文件请求: {} -> {}", file_path, peer_addr);
    
    // 读取文件
    let file_data = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 获取文件名
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无效的文件名")?
        .to_string();
    
    let file_size = file_data.len();
    println!("[Command] 文件: {}, 大小: {} 字节", file_name, file_size);
    
    // 构造 multipart 请求
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("创建客户端失败: {}", e))?;
    
    // 获取自己的 ID（发送者 ID）
    let my_id = crate::db::get_user_id(&state.pool).await?;
    
    let form = reqwest::multipart::Form::new()
        .text("peer_id", my_id.clone())  // 传递发送者的 ID
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_data.clone())
                .file_name(file_name.clone())
                .mime_str("application/octet-stream")
                .map_err(|e| format!("设置 MIME 类型失败: {}", e))?
        );
    
    let upload_url = format!("http://{}/api/upload", peer_addr);
    println!("[Command] 上传到: {}", upload_url);
    println!("[Command] sender_id (我的ID): {}", my_id);
    println!("[Command] 文件名: {}", file_name);
    
    let response = client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("上传失败: {}", e))?;
    
    let status = response.status();
    println!("[Command] 响应状态: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误信息".to_string());
        eprintln!("[Command] 错误响应: {}", error_text);
        return Err(format!("上传失败: HTTP {} - {}", status, error_text));
    }
    
    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    // 保存发送记录到数据库
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    sqlx::query(
        "INSERT INTO messages (sender_id, content, msg_type, timestamp, file_path, file_status) VALUES ('me', ?, 'file', ?, ?, ?)"
    )
    .bind(&file_name)
    .bind(timestamp)
    .bind(&file_path)
    .bind(file_size.to_string())
    .execute(&state.pool)
    .await
    .map_err(|e| format!("保存记录失败: {}", e))?;
    
    println!("[Command] 文件上传成功");
    
    // 返回文件信息
    Ok(serde_json::json!({
        "success": true,
        "file_id": result.get("file_id").and_then(|v| v.as_str()).unwrap_or(""),
        "file_name": file_name,
        "file_size": file_size,
    }))
}
