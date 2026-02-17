// commands.rs - Tauri 命令（桌面端和移动端共享）
use crate::db::DbState;
use crate::peers::{Peer, PeerManager};
use std::sync::Arc;
use tauri::State;

// 用于管理 PeerManager 的状态
pub struct PeerState {
    pub manager: Arc<PeerManager>,
}

#[tauri::command]
pub async fn get_my_name(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_my_name");
    crate::db::get_username(&state.pool).await
}

#[tauri::command]
pub async fn get_my_id(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_my_id");
    crate::db::get_user_id(&state.pool).await
}

#[tauri::command]
pub async fn get_settings(state: State<'_, DbState>) -> Result<serde_json::Value, String> {
    println!("[Command] 收到前端请求: get_settings");
    
    let download_path = crate::db::get_download_path(&state.pool).await?;
    
    Ok(serde_json::json!({
        "download_path": download_path,
    }))
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, DbState>,
    download_path: Option<String>,
) -> Result<(), String> {
    println!("[Command] 收到前端请求: update_settings");
    
    if let Some(path) = download_path {
        crate::db::update_download_path(&state.pool, path).await?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn update_my_name(state: State<'_, DbState>, new_name: String) -> Result<String, String> {
    println!("[Command] 收到前端请求: update_my_name, 新名字: {}", new_name);
    
    // 更新数据库
    crate::db::update_username(&state.pool, new_name.clone()).await?;
    
    // 数据库更新后，定时广播线程会自动使用新名称
    println!("[Command] 用户名已更新，广播线程将使用新名称");
    
    // 返回更新后的名字
    Ok(new_name)
}

#[tauri::command]
pub async fn get_peers(state: State<'_, PeerState>) -> Result<Vec<Peer>, String> {
    Ok(state.manager.get_all_peers())
}

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
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp) VALUES ('me', ?, ?, 'text', ?)"
    )
    .bind(&peer_id)  // 接收者ID
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

#[tauri::command]
pub async fn get_chat_history(
    state: State<'_, DbState>,
    peer_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    crate::network::messaging::get_chat_history(&state.pool, &peer_id, 100).await
}

#[tauri::command]
pub async fn send_file(
    state: State<'_, DbState>,
    peer_id: String,
    peer_addr: String,
    file_path: String,
) -> Result<serde_json::Value, String> {
    println!("[Command] 收到发送文件请求: {} -> {} ({})", file_path, peer_addr, peer_id);
    
    // 获取文件名和大小
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无效的文件名")?
        .to_string();
    
    let file_metadata = std::fs::metadata(&file_path)
        .map_err(|e| format!("读取文件信息失败: {}", e))?;
    let file_size = file_metadata.len() as usize;
    
    println!("[Command] 文件: {}, 大小: {} 字节", file_name, file_size);
    
    // 立即创建数据库记录（状态为 uploading）
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let result = sqlx::query(
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp, file_path, file_status) VALUES ('me', ?, ?, 'file', ?, ?, 'uploading')"
    )
    .bind(&peer_id)      // 接收者ID
    .bind(&file_name)
    .bind(timestamp)
    .bind(&file_path)
    .execute(&state.pool)
    .await;
    
    let message_id = match result {
        Ok(res) => {
            let id = res.last_insert_rowid();
            println!("[Command] ✓ 已创建上传中记录，ID: {}", id);
            Some(id)
        }
        Err(e) => {
            eprintln!("[Command] ✗ 创建上传记录失败: {}", e);
            None
        }
    };
    
    // 读取文件
    let file_data = tokio::fs::read(&file_path)
        .await
        .map_err(|e| {
            // 删除失败的数据库记录
            if let Some(id) = message_id {
                let pool = state.pool.clone();
                tokio::spawn(async move {
                    let _ = sqlx::query("DELETE FROM messages WHERE id = ?")
                        .bind(id)
                        .execute(&pool)
                        .await;
                });
            }
            format!("读取文件失败: {}", e)
        })?;
    
    // 构造 multipart 请求
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))  // 增加超时时间到 5 分钟
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
        .map_err(|e| {
            // 删除失败的数据库记录
            if let Some(id) = message_id {
                let pool = state.pool.clone();
                tokio::spawn(async move {
                    let _ = sqlx::query("DELETE FROM messages WHERE id = ?")
                        .bind(id)
                        .execute(&pool)
                        .await;
                });
            }
            format!("上传失败: {}", e)
        })?;
    
    let status = response.status();
    println!("[Command] 响应状态: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误信息".to_string());
        eprintln!("[Command] 错误响应: {}", error_text);
        
        // 删除失败的数据库记录
        if let Some(id) = message_id {
            let _ = sqlx::query("DELETE FROM messages WHERE id = ?")
                .bind(id)
                .execute(&state.pool)
                .await;
        }
        
        return Err(format!("上传失败: HTTP {} - {}", status, error_text));
    }
    
    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    // 更新数据库状态为 "sent"
    if let Some(id) = message_id {
        if let Err(e) = sqlx::query(
            "UPDATE messages SET file_status = 'sent' WHERE id = ?"
        )
        .bind(id)
        .execute(&state.pool)
        .await {
            eprintln!("[Command] ⚠ 更新数据库状态失败: {}", e);
        } else {
            println!("[Command] ✓ 文件状态已更新为 sent");
        }
    }
    
    println!("[Command] 文件上传成功");
    
    // 返回文件信息
    Ok(serde_json::json!({
        "success": true,
        "file_id": result.get("file_id").and_then(|v| v.as_str()).unwrap_or(""),
        "file_name": file_name,
        "file_size": file_size,
    }))
}

#[tauri::command]
pub async fn get_theme_list() -> Result<Vec<serde_json::Value>, String> {
    println!("[Command] 收到前端请求: get_theme_list");
    
    let mut themes = vec![
        serde_json::json!({
            "name": "default",
            "display_name": "默认主题",
            "is_custom": false
        })
    ];
    
    // 检查自定义主题目录
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let theme_dir = home_dir.join(".config").join("lanchat");
    
    if theme_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&theme_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("css") {
                        if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                            themes.push(serde_json::json!({
                                "name": file_name,
                                "display_name": file_name,
                                "is_custom": true,
                                "path": path.to_string_lossy()
                            }));
                        }
                    }
                }
            }
        }
    }
    
    println!("[Command] 找到 {} 个主题", themes.len());
    Ok(themes)
}

#[tauri::command]
pub async fn get_theme_css(theme_name: String) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_theme_css, 主题: {}", theme_name);
    
    if theme_name == "default" {
        return Ok(String::new()); // 默认主题返回空字符串
    }
    
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let theme_path = home_dir.join(".config").join("lanchat").join(format!("{}.css", theme_name));
    
    if !theme_path.exists() {
        return Err(format!("主题文件不存在: {}", theme_path.display()));
    }
    
    let css_content = std::fs::read_to_string(&theme_path)
        .map_err(|e| format!("读取主题文件失败: {}", e))?;
    
    println!("[Command] 成功读取主题文件: {} ({} 字节)", theme_path.display(), css_content.len());
    Ok(css_content)
}

#[tauri::command]
pub async fn save_current_theme(state: State<'_, DbState>, theme_name: String) -> Result<(), String> {
    println!("[Command] 收到前端请求: save_current_theme, 主题: {}", theme_name);
    
    // 保存当前主题到数据库
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('current_theme', ?)")
        .bind(&theme_name)
        .execute(&state.pool)
        .await
        .map_err(|e| format!("保存主题设置失败: {}", e))?;
    
    println!("[Command] 主题设置已保存: {}", theme_name);
    Ok(())
}

#[tauri::command]
pub async fn get_current_theme(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_current_theme");
    
    let result = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = 'current_theme'")
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| format!("查询主题设置失败: {}", e))?;
    
    let theme = result.unwrap_or_else(|| "default".to_string());
    println!("[Command] 当前主题: {}", theme);
    Ok(theme)
}

#[tauri::command]
pub async fn get_default_download_path() -> Result<String, String> {
    println!("[Command] 收到前端请求: get_default_download_path");
    
    if cfg!(target_os = "android") {
        // Android 的公共下载目录
        let download_path = "/storage/emulated/0/Download/LANChat";
        println!("[Command] Android 默认下载路径: {}", download_path);
        Ok(download_path.to_string())
    } else {
        // 桌面端和 Web 端返回用户下载目录
        let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let download_path = home_dir.join("Downloads").join("LANChat");
        println!("[Command] 默认下载路径: {}", download_path.display());
        Ok(download_path.to_string_lossy().to_string())
    }
}

#[tauri::command]
pub async fn request_storage_permission() -> Result<bool, String> {
    println!("[Command] 收到前端请求: request_storage_permission");
    
    #[cfg(target_os = "android")]
    {
        // Android 上需要请求存储权限
        // 注意：这个功能需要 Tauri 的 Android 插件支持
        // 目前先返回 true，假设权限已授予
        println!("[Command] Android 存储权限检查（假设已授予）");
        return Ok(true);
    }
    
    #[cfg(not(target_os = "android"))]
    {
        // 桌面端不需要权限
        Ok(true)
    }
}