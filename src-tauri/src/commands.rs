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
pub async fn get_settings(state: State<'_, DbState>) -> Result<serde_json::Value, String> {
    println!("[Command] 收到前端请求: get_settings");
    
    let download_path = crate::db::get_download_path(&state.pool).await?;
    let auto_accept = crate::db::get_auto_accept(&state.pool).await?;
    
    Ok(serde_json::json!({
        "download_path": download_path,
        "auto_accept": auto_accept,
    }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn update_settings(
    state: State<'_, DbState>,
    download_path: Option<String>,
    auto_accept: Option<bool>,
) -> Result<(), String> {
    println!("[Command] 收到前端请求: update_settings");
    
    if let Some(path) = download_path {
        crate::db::update_download_path(&state.pool, path).await?;
    }
    
    if let Some(accept) = auto_accept {
        crate::db::update_auto_accept(&state.pool, accept).await?;
    }
    
    Ok(())
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
    
    // 发送的文件状态标记为 "sent"
    sqlx::query(
        "INSERT INTO messages (sender_id, content, msg_type, timestamp, file_path, file_status) VALUES ('me', ?, 'file', ?, ?, 'sent')"
    )
    .bind(&file_name)
    .bind(timestamp)
    .bind(&file_path)
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


#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn accept_file(
    state: State<'_, DbState>,
    file_id: String,
    save_path: Option<String>,
) -> Result<String, String> {
    println!("[Command] 收到接受文件请求: file_id={}", file_id);
    
    // 从数据库查询文件信息
    let row = sqlx::query(
        "SELECT file_path, content FROM messages WHERE file_path LIKE ? AND file_status = 'pending'"
    )
    .bind(format!("%{}%", file_id))
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| format!("查询文件失败: {}", e))?;
    
    let row = row.ok_or("文件不存在或已接收")?;
    
    use sqlx::Row;
    let temp_path: String = row.get("file_path");
    let file_name: String = row.get("content");
    
    println!("[Command] 临时路径: {}", temp_path);
    println!("[Command] 文件名: {}", file_name);
    
    // 检查文件是否存在（可能还在上传中）
    if !std::path::Path::new(&temp_path).exists() {
        println!("[Command] ⏳ 文件还在下载中");
        return Err("文件还在下载中，请稍候...".to_string());
    }
    
    // 确定最终保存路径
    let final_path = if let Some(path) = save_path {
        // 用户指定了路径
        std::path::PathBuf::from(path).join(&file_name)
    } else {
        // 使用默认下载路径
        let download_path = crate::db::get_download_path(&state.pool).await?;
        std::path::PathBuf::from(download_path).join(&file_name)
    };
    
    println!("[Command] 最终路径: {:?}", final_path);
    
    // 确保目标目录存在
    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    // 移动文件 - 使用复制+删除来支持跨文件系统
    println!("[Command] 开始移动文件...");
    if let Err(e) = std::fs::rename(&temp_path, &final_path) {
        // rename 失败（可能是跨文件系统），尝试复制+删除
        println!("[Command] rename 失败 ({}), 尝试复制+删除", e);
        
        std::fs::copy(&temp_path, &final_path)
            .map_err(|e| format!("复制文件失败: {}", e))?;
        
        // 复制成功后删除临时文件
        if let Err(e) = std::fs::remove_file(&temp_path) {
            println!("[Command] ⚠ 删除临时文件失败: {}", e);
            // 不返回错误，因为文件已经复制成功了
        }
        
        println!("[Command] ✓ 文件已复制到目标位置");
    } else {
        println!("[Command] ✓ 文件已移动到目标位置");
    }
    
    // 更新数据库状态
    sqlx::query(
        "UPDATE messages SET file_status = 'accepted', file_path = ? WHERE file_path = ?"
    )
    .bind(final_path.to_str().unwrap())
    .bind(&temp_path)
    .execute(&state.pool)
    .await
    .map_err(|e| format!("更新数据库失败: {}", e))?;
    
    println!("[Command] ✓ 文件已接受并保存到: {:?}", final_path);
    Ok(final_path.to_str().unwrap().to_string())
}
