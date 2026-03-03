// commands.rs - Tauri 命令（桌面端和移动端共享）
use crate::db::DbState;
use crate::peers::{Peer, PeerManager};
use std::sync::Arc;
use tauri::{State, Manager};


// 用于管理 PeerManager 的状态
pub struct PeerState {
    pub manager: Arc<PeerManager>,
}

/// 根据设备内存和文件大小计算最优分块大小
#[cfg(feature = "desktop")]
fn calculate_optimal_chunk_size(_file_size: usize) -> usize {
    use sysinfo::System;
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // 获取可用内存（字节）
    let available_memory = sys.available_memory() as usize * 1024; // sysinfo 返回的是 KB
    
    // 使用可用内存的 80%（大胆使用内存以获得更快的速度）
    let max_chunk_memory = available_memory * 80 / 100;
    
    // 动态计算分块大小：使用可用内存的 80%，但最小 50MB，最大 500MB
    let chunk_size = std::cmp::max(
        50 * 1024 * 1024,  // 最小 50MB
        std::cmp::min(
            max_chunk_memory,  // 使用可用内存的 80%
            500 * 1024 * 1024  // 最大 500MB
        )
    );
    
    println!("[Command] 系统可用内存: {} MB", available_memory / (1024 * 1024));
    println!("[Command] 内存预算: {} MB", max_chunk_memory / (1024 * 1024));
    println!("[Command] 计算的分块大小: {} MB", chunk_size / (1024 * 1024));
    
    chunk_size
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
    crate::db::save_text_message(&state.pool, peer_id, content).await?;
    
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
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    peer_id: String,
    peer_addr: String,
    file_path: String,
) -> Result<serde_json::Value, String> {
    println!("[Command] 收到发送文件请求: {} -> {} ({})", file_path, peer_addr, peer_id);
    
    // 检测是否是 Android content URI
    let is_content_uri = file_path.starts_with("content://");
    println!("[Command] 文件路径类型: {}", if is_content_uri { "content URI" } else { "普通路径" });
    
    // 获取文件名和大小
    let (file_name, file_size) = if is_content_uri {
        // Android content URI 处理 - 使用 Tauri 的 fs 插件
        println!("[Command] 处理 Android content URI: {}", file_path);
        
        // 从 URI 中提取文件名
        let file_name = file_path
            .split('/')
            .last()
            .and_then(|s| urlencoding::decode(s).ok())
            .map(|s| {
                // 移除可能的扩展名编码
                let decoded = s.to_string();
                // 如果包含 : 说明是 Android 的 document ID 格式，提取实际文件名
                if let Some(idx) = decoded.rfind(':') {
                    decoded[idx+1..].to_string()
                } else {
                    decoded
                }
            })
            .unwrap_or_else(|| format!("file_{}.dat", chrono::Utc::now().timestamp()));
        
        println!("[Command] 提取的文件名: {}", file_name);
        
        // 获取文件大小
        let size = match tokio::fs::metadata(&file_path).await {
            Ok(metadata) => metadata.len() as usize,
            Err(e) => {
                println!("[Command] 无法获取文件大小: {}", e);
                0
            }
        };
        
        (file_name, size)
    } else {
        // 普通文件路径处理
        let file_name = std::path::Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("无效的文件名")?
            .to_string();
        
        let file_metadata = std::fs::metadata(&file_path)
            .map_err(|e| format!("读取文件信息失败: {}", e))?;
        let file_size = file_metadata.len() as usize;
        
        println!("[Command] 文件: {}, 大小: {} 字节", file_name, file_size);
        
        (file_name, file_size)
    };
    
    // 立即创建数据库记录（状态为 uploading）
    let message_id = match crate::db::save_file_message(
        &state.pool,
        peer_id.clone(),
        file_name.clone(),
        file_size,
        file_path.clone(),
        "uploading".to_string(),
    ).await {
        Ok(id) => {
            println!("[Command] ✓ 已创建上传中记录，ID: {}", id);
            Some(id)
        }
        Err(e) => {
            eprintln!("[Command] ✗ 创建上传记录失败: {}", e);
            None
        }
    };
    
    // 获取自己的 ID（发送者 ID）
    let my_id = crate::db::get_user_id(&state.pool).await?;
    
    // 获取接收方的可用内存
    let receiver_memory_mb = if let Some(peer_state) = app.try_state::<PeerState>() {
        let peers = peer_state.manager.get_all_peers();
        peers.iter()
            .find(|p| p.addr.starts_with(&peer_addr.split(':').next().unwrap_or("")))
            .map(|p| p.available_memory_mb)
            .unwrap_or(1024) // 默认 1GB
    } else {
        1024 // 默认 1GB
    };
    
    println!("[Command] 接收方可用内存: {} MB", receiver_memory_mb);
    
    // 分块上传
    let chunk_size = calculate_optimal_chunk_size(file_size);
    
    // 根据接收方内存调整分块大小（取发送方和接收方的最小值）
    let max_chunk_for_receiver = std::cmp::max(50 * 1024 * 1024, receiver_memory_mb as usize * 1024 * 1024 / 4); // 接收方内存的 1/4
    let adjusted_chunk_size = std::cmp::min(chunk_size, max_chunk_for_receiver);
    
    println!("[Command] 原始分块大小: {} MB, 调整后: {} MB", chunk_size / (1024 * 1024), adjusted_chunk_size / (1024 * 1024));
    
    let total_chunks = (file_size + adjusted_chunk_size - 1) / adjusted_chunk_size;
    
    println!("[Command] 开始分块上传: 文件大小={}, 分块大小={}, 总分块数={}", 
             file_size, adjusted_chunk_size, total_chunks);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建客户端失败: {}", e))?;
    
    let upload_url = format!("http://{}/api/upload", peer_addr);
    
    // 打开文件进行流式读取
    let mut file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("打开文件失败: {}", e))?;
    
    let mut offset = 0;
    let mut chunk_index = 0;
    let start_time = std::time::Instant::now();
    
    loop {
        // 读取分块（循环读取直到填满缓冲区或文件结束）
        let mut buf = vec![0u8; adjusted_chunk_size];
        let mut bytes_read = 0;
        
        while bytes_read < adjusted_chunk_size {
            let n = tokio::io::AsyncReadExt::read(&mut file, &mut buf[bytes_read..])
                .await
                .map_err(|e| format!("读取文件失败: {}", e))?;
            
            if n == 0 {
                break; // 文件读取完毕
            }
            
            bytes_read += n;
        }
        
        if bytes_read == 0 {
            break; // 文件读取完毕
        }
        
        buf.truncate(bytes_read);
        let n = bytes_read;
        
        // 构造 multipart 请求
        let form = reqwest::multipart::Form::new()
            .text("peer_id", my_id.clone())
            .text("file_name", file_name.clone())
            .text("file_size", file_size.to_string())
            .text("chunk_index", chunk_index.to_string())
            .text("chunk_total", total_chunks.to_string())
            .part(
                "chunk",
                reqwest::multipart::Part::bytes(buf.clone())
                    .mime_str("application/octet-stream")
                    .map_err(|e| format!("设置 MIME 类型失败: {}", e))?
            );
        
        println!("[Command] 上传分块 {}/{}, 大小: {} 字节", 
                 chunk_index + 1, total_chunks, n);
        
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
                        let _ = crate::db::delete_message_by_id(&pool, id).await;
                    });
                }
                format!("上传分块失败: {}", e)
            })?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误信息".to_string());
            eprintln!("[Command] ✗ 上传分块失败: {}", error_text);
            
            // 删除失败的数据库记录
            if let Some(id) = message_id {
                let _ = crate::db::delete_message_by_id(&state.pool, id).await;
            }
            
            return Err(format!("上传分块失败: {}", error_text));
        }
        
        offset += n;
        chunk_index += 1;
        
        // 打印进度
        let elapsed = start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let speed = offset as f64 / (1024.0 * 1024.0) / elapsed;
            println!("[Command] 已上传: {} MB, 速度: {:.2} MB/s", 
                     offset / (1024 * 1024), speed);
        }
    }
    
    let total_time = start_time.elapsed().as_secs_f64();
    let avg_speed = file_size as f64 / (1024.0 * 1024.0) / total_time;
    println!("[Command] ✓ 文件上传完成，耗时: {:.2}s, 平均速度: {:.2} MB/s", 
             total_time, avg_speed);
    
    // 更新数据库状态为 "sent"
    if let Some(id) = message_id {
        if let Err(e) = crate::db::update_file_status_by_id(&state.pool, id, "sent").await {
            eprintln!("[Command] ⚠ 更新数据库状态失败: {}", e);
        } else {
            println!("[Command] ✓ 文件状态已更新为 sent");
        }
    }
    
    println!("[Command] 文件上传成功");
    
    // 返回文件信息
    Ok(serde_json::json!({
        "success": true,
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
            "is_custom": false,
            "is_builtin": true
        }),
        serde_json::json!({
            "name": "vscode",
            "display_name": "VSCode 主题",
            "is_custom": false,
            "is_builtin": true
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
                                "is_builtin": false,
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
    
    // 检查是否是内置主题
    if theme_name == "vscode" {
        // 从嵌入的资源中读取 vscode.css
        let css_content = include_str!("../../src/css/vscode.css");
        println!("[Command] 加载内置主题: vscode ({} 字节)", css_content.len());
        return Ok(css_content.to_string());
    }
    
    // 自定义主题从用户目录读取
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
    crate::db::save_current_theme(&state.pool, theme_name.clone()).await?;
    
    println!("[Command] 主题设置已保存: {}", theme_name);
    Ok(())
}

#[tauri::command]
pub async fn get_current_theme(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_current_theme");
    
    let result = crate::db::get_current_theme(&state.pool).await?;
    
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

#[tauri::command]
pub async fn save_file_message(
    state: State<'_, DbState>,
    peer_id: String,
    file_name: String,
    file_size: usize,
    file_path: String,
    status: String,
) -> Result<i64, String> {
    println!("[Command] 收到前端请求: save_file_message");
    println!("[Command] 文件: {}, 大小: {}, 状态: {}", file_name, file_size, status);
    
    // 使用数据库层的函数
    crate::db::save_file_message(
        &state.pool,
        peer_id,
        file_name,
        file_size,
        file_path,
        status,
    ).await
}