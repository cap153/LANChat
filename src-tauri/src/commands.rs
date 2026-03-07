// commands.rs - Tauri 命令（桌面端和移动端共享）
#[cfg(feature = "desktop")]
use crate::db::DbState;

#[cfg(feature = "desktop")]
use crate::peers::{Peer, PeerManager};

#[cfg(feature = "desktop")]
use std::sync::Arc;

#[cfg(feature = "desktop")]
use tauri::{Emitter, Manager, State};

// 用于管理 PeerManager 的状态
#[cfg(feature = "desktop")]
pub struct PeerState {
    pub manager: Arc<PeerManager>,
}

#[cfg(feature = "desktop")]

/// 根据设备内存和文件大小计算最优分块大小
fn calculate_optimal_chunk_size(_file_size: usize) -> usize {
    #[cfg(feature = "desktop")]
    {
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();

        // 获取可用内存（字节）
        let available_memory = sys.available_memory() as usize * 1024; // sysinfo 返回的是 KB

        // 使用可用内存的 80%（大胆使用内存以获得更快的速度）
        let max_chunk_memory = available_memory * 80 / 100;

        // 动态计算分块大小：使用可用内存的 80%，但最小 50MB，最大 500MB
        let chunk_size = std::cmp::max(
            50 * 1024 * 1024, // 最小 50MB
            std::cmp::min(
                max_chunk_memory,  // 使用可用内存的 80%
                500 * 1024 * 1024, // 最大 500MB
            ),
        );

        println!(
            "[Command] 系统可用内存: {} MB",
            available_memory / (1024 * 1024)
        );
        println!(
            "[Command] 内存预算: {} MB",
            max_chunk_memory / (1024 * 1024)
        );
        println!(
            "[Command] 计算的分块大小: {} MB",
            chunk_size / (1024 * 1024)
        );

        chunk_size
    }
    
    #[cfg(not(feature = "desktop"))]
    {
        // Web 端：使用保守的固定值
        println!("[Command] Web 端使用固定分块大小: 100 MB");
        100 * 1024 * 1024
    }
}

/// 统一的文件上传实现
/// 接受一个实现了 AsyncRead 的文件对象
async fn upload_file_internal<R: tokio::io::AsyncRead + Unpin>(
    app: &tauri::AppHandle,
    state: &State<'_, DbState>,
    peer_state: Option<&State<'_, PeerState>>,
    peer_id: String,
    peer_addr: String,
    file_name: String,
    file_size: usize,
    file_path_for_db: String,
    mut file: R,
) -> Result<serde_json::Value, String> {
    // 立即创建数据库记录（状态为 uploading）
    let message_id = match crate::db::save_file_message(
        &state.pool,
        peer_id.clone(),
        file_name.clone(),
        file_size,
        file_path_for_db.clone(),
        "uploading".to_string(),
    )
    .await
    {
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
    let receiver_memory_mb = if let Some(ps) = peer_state {
        let peers = ps.manager.get_all_peers();
        peers
            .iter()
            .find(|p| {
                p.addr
                    .starts_with(&peer_addr.split(':').next().unwrap_or(""))
            })
            .map(|p| p.available_memory_mb)
            .unwrap_or(1024)
    } else {
        1024
    };

    println!("[Command] 接收方可用内存: {} MB", receiver_memory_mb);

    // 分块上传
    let chunk_size = calculate_optimal_chunk_size(file_size);

    // 根据接收方内存调整分块大小（取发送方和接收方的最小值）
    let max_chunk_for_receiver = std::cmp::max(
        50 * 1024 * 1024,
        receiver_memory_mb as usize * 1024 * 1024 / 4,
    );
    let adjusted_chunk_size = std::cmp::min(chunk_size, max_chunk_for_receiver);

    println!(
        "[Command] 原始分块大小: {} MB, 调整后: {} MB",
        chunk_size / (1024 * 1024),
        adjusted_chunk_size / (1024 * 1024)
    );

    let total_chunks = (file_size + adjusted_chunk_size - 1) / adjusted_chunk_size;

    println!(
        "[Command] 开始分块上传: 文件大小={}, 分块大小={}, 总分块数={}",
        file_size, adjusted_chunk_size, total_chunks
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建客户端失败: {}", e))?;

    let upload_url = format!("http://{}/api/upload", peer_addr);

    let mut offset = 0;
    let mut chunk_index = 0;
    let start_time = std::time::Instant::now();

    loop {
        // 读取分块
        let mut buf = vec![0u8; adjusted_chunk_size];
        let mut bytes_read = 0;

        while bytes_read < adjusted_chunk_size {
            let n = tokio::io::AsyncReadExt::read(&mut file, &mut buf[bytes_read..])
                .await
                .map_err(|e| format!("读取文件失败: {}", e))?;

            if n == 0 {
                break;
            }

            bytes_read += n;
        }

        if bytes_read == 0 {
            break;
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
                    .map_err(|e| format!("设置 MIME 类型失败: {}", e))?,
            );

        println!(
            "[Command] 上传分块 {}/{}, 大小: {} 字节",
            chunk_index + 1,
            total_chunks,
            n
        );

        let response = client.post(&upload_url).multipart(form).send().await.map_err(|e| {
            if let Some(id) = message_id {
                let pool = state.pool.clone();
                tokio::spawn(async move {
                    let _ = crate::db::delete_message_by_id(&pool, id).await;
                });
            }
            format!("上传分块失败: {}", e)
        })?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            eprintln!("[Command] ✗ 上传分块失败: {}", error_text);

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
            println!(
                "[Command] 已上传: {} MB, 速度: {:.2} MB/s",
                offset / (1024 * 1024),
                speed
            );
            let _ = app.emit(
                "upload_progress",
                serde_json::json!({
                    "file_name": file_name.clone(),
                    "speed_mb_s": speed
                }),
            );
        }
    }

    let total_time = start_time.elapsed().as_secs_f64();
    let avg_speed = file_size as f64 / (1024.0 * 1024.0) / total_time;
    println!(
        "[Command] ✓ 文件上传完成，耗时: {:.2}s, 平均速度: {:.2} MB/s",
        total_time, avg_speed
    );

    // 更新数据库状态为 "sent"
    if let Some(id) = message_id {
        if let Err(e) = crate::db::update_file_status_by_id(&state.pool, id, "sent").await {
            eprintln!("[Command] ⚠ 更新数据库状态失败: {}", e);
        } else {
            println!("[Command] ✓ 文件状态已更新为 sent");
        }
    }

    Ok(serde_json::json!({
        "success": true,
        "file_name": file_name,
        "file_size": file_size,
    }))
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
    println!(
        "[Command] 收到前端请求: update_my_name, 新名字: {}",
        new_name
    );

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
    crate::network::messaging::send_text_message(&peer_addr, my_id, my_name, content.clone())
        .await?;

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
    println!(
        "[Command] 收到发送文件请求: {} -> {} ({})",
        file_path, peer_addr, peer_id
    );

    // 检测是否是 Android content URI
    if file_path.starts_with("content://") {
        #[cfg(target_os = "android")]
        {
            println!("[Command] 检测到 Android content URI，使用 FD 方式");
            
            // 使用 JNI 调用 Android ContentResolver 获取 FD
            use crate::android_fd::AndroidFile;
            
            // 从 content URI 获取文件描述符
            let android_file = AndroidFile::from_content_uri(&file_path)?;
            let std_file = android_file.into_file();
            let file = tokio::fs::File::from_std(std_file);
            
            // 从 URI 中提取文件名
            let file_name = file_path
                .split('/')
                .last()
                .and_then(|s| urlencoding::decode(s).ok())
                .map(|s| {
                    let decoded = s.to_string();
                    if let Some(idx) = decoded.rfind(':') {
                        decoded[idx + 1..].to_string()
                    } else {
                        decoded
                    }
                })
                .unwrap_or_else(|| format!("file_{}.dat", chrono::Utc::now().timestamp()));
            
            // 获取文件大小
            let file_size = match tokio::fs::metadata(&file_path).await {
                Ok(metadata) => metadata.len() as usize,
                Err(_) => {
                    // 如果无法通过 metadata 获取，尝试通过 stat 系统调用
                    println!("[Command] 无法通过 metadata 获取文件大小，使用默认值");
                    0
                }
            };
            
            println!("[Command] 文件名: {}, 大小: {} 字节", file_name, file_size);
            
            // 使用统一的上传函数
            let peer_state = app.try_state::<PeerState>();
            return upload_file_internal(
                &app,
                &state,
                peer_state.as_ref(),
                peer_id,
                peer_addr,
                file_name,
                file_size,
                file_path,
                file,
            )
            .await;
        }
        
        #[cfg(not(target_os = "android"))]
        {
            return Err("content:// URI 仅在 Android 上支持".to_string());
        }
    }

    // 普通文件路径处理
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无效的文件名")?
        .to_string();

    let file_metadata =
        std::fs::metadata(&file_path).map_err(|e| format!("读取文件信息失败: {}", e))?;
    let file_size = file_metadata.len() as usize;

    println!("[Command] 文件: {}, 大小: {} 字节", file_name, file_size);

    // 打开文件
    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("打开文件失败: {}", e))?;

    // 使用统一的上传函数
    let peer_state = app.try_state::<PeerState>();
    upload_file_internal(
        &app,
        &state,
        peer_state.as_ref(),
        peer_id,
        peer_addr,
        file_name,
        file_size,
        file_path,
        file,
    )
    .await
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
        }),
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
    println!(
        "[Command] 收到前端请求: get_theme_css, 主题: {}",
        theme_name
    );

    if theme_name == "default" {
        return Ok(String::new()); // 默认主题返回空字符串
    }

    // 检查是否是内置主题
    if theme_name == "vscode" {
        // 从嵌入的资源中读取 vscode.css
        let css_content = include_str!("../../src/css/vscode.css");
        println!(
            "[Command] 加载内置主题: vscode ({} 字节)",
            css_content.len()
        );
        return Ok(css_content.to_string());
    }

    // 自定义主题从用户目录读取
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let theme_path = home_dir
        .join(".config")
        .join("lanchat")
        .join(format!("{}.css", theme_name));

    if !theme_path.exists() {
        return Err(format!("主题文件不存在: {}", theme_path.display()));
    }

    let css_content =
        std::fs::read_to_string(&theme_path).map_err(|e| format!("读取主题文件失败: {}", e))?;

    println!(
        "[Command] 成功读取主题文件: {} ({} 字节)",
        theme_path.display(),
        css_content.len()
    );
    Ok(css_content)
}

#[tauri::command]
pub async fn save_current_theme(
    state: State<'_, DbState>,
    theme_name: String,
) -> Result<(), String> {
    println!(
        "[Command] 收到前端请求: save_current_theme, 主题: {}",
        theme_name
    );

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
    println!(
        "[Command] 文件: {}, 大小: {}, 状态: {}",
        file_name, file_size, status
    );

    // 使用数据库层的函数
    crate::db::save_file_message(
        &state.pool,
        peer_id,
        file_name,
        file_size,
        file_path,
        status,
    )
    .await
}
#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn open_file_location(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    
    println!("[Command] 打开文件位置: {}", file_path);
    
    // 使用 opener 插件打开文件所在目录
    app.opener()
        .reveal_item_in_dir(&file_path)
        .map_err(|e| format!("打开文件位置失败: {}", e))?;
    
    println!("[Command] ✓ 文件位置已打开");
    Ok(())
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn set_android_shared_files(
    app: tauri::AppHandle,
    files: Vec<serde_json::Value>,
) -> Result<(), String> {
    println!("[Command] set_android_shared_files 被调用，文件数: {}", files.len());
    
    #[cfg(target_os = "android")]
    {
        use tauri::Manager;
        
        if let Some(share_state) = app.try_state::<AndroidShareState>() {
            share_state.set_files(files);
            println!("[Command] 文件已保存到状态");
            return Ok(());
        }
        
        println!("[Command] 没有找到分享状态");
        Err("分享状态未初始化".to_string())
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, files);
        Err("此功能仅在 Android 上可用".to_string())
    }
}

#[tauri::command]
pub async fn get_android_shared_files(app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    println!("[Command] get_android_shared_files 被调用");
    
    #[cfg(target_os = "android")]
    {
        // 在 Android 上，从 MainActivity 获取分享文件
        // 通过 Tauri 的事件系统或状态管理获取
        // 这里我们使用一个全局状态来存储分享文件
        
        use tauri::Manager;
        
        // 尝试从应用状态获取分享文件
        if let Some(share_state) = app.try_state::<AndroidShareState>() {
            let files = share_state.get_files();
            println!("[Command] 从状态获取到 {} 个文件", files.len());
            return Ok(files);
        }
        
        println!("[Command] 没有找到分享状态");
        Ok(vec![])
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Err("此功能仅在 Android 上可用".to_string())
    }
}

#[tauri::command]
pub async fn clear_android_shared_files(app: tauri::AppHandle) -> Result<(), String> {
    println!("[Command] clear_android_shared_files 被调用");
    
    #[cfg(target_os = "android")]
    {
        use tauri::Manager;
        
        if let Some(share_state) = app.try_state::<AndroidShareState>() {
            share_state.clear_files();
            println!("[Command] 已清除分享文件");
        }
        
        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Err("此功能仅在 Android 上可用".to_string())
    }
}

// Android 分享状态管理
#[cfg(all(feature = "desktop", target_os = "android"))]
pub struct AndroidShareState {
    files: std::sync::Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
}

#[cfg(target_os = "android")]
impl AndroidShareState {
    pub fn new() -> Self {
        Self {
            files: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
    
    pub fn set_files(&self, files: Vec<serde_json::Value>) {
        if let Ok(mut f) = self.files.lock() {
            *f = files;
            println!("[AndroidShareState] 已设置 {} 个文件", f.len());
        }
    }
    
    pub fn get_files(&self) -> Vec<serde_json::Value> {
        if let Ok(f) = self.files.lock() {
            println!("[AndroidShareState] 获取 {} 个文件", f.len());
            f.clone()
        } else {
            Vec::new()
        }
    }
    
    pub fn clear_files(&self) {
        if let Ok(mut f) = self.files.lock() {
            f.clear();
            println!("[AndroidShareState] 已清除文件");
        }
    }
}

#[cfg(all(feature = "desktop", not(target_os = "android")))]
pub struct AndroidShareState;

#[cfg(all(feature = "desktop", not(target_os = "android")))]
impl AndroidShareState {
    pub fn new() -> Self {
        Self
    }
}

#[tauri::command]
pub async fn send_file_from_uri(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    #[allow(non_snake_case)]
    peerId: String,
    #[allow(non_snake_case)]
    peerAddr: String,
    uri: String,
    #[allow(non_snake_case)]
    fileName: String,
    #[allow(non_snake_case)]
    fileSize: usize,
) -> Result<serde_json::Value, String> {
    println!(
        "[Command] 收到从 URI 发送文件请求: uri={}, name={}, size={}, to={}",
        uri, fileName, fileSize, peerAddr
    );

    #[cfg(target_os = "android")]
    {
        // 在 Android 上，使用 JNI 从 content:// URI 获取文件描述符
        use crate::android_fd::AndroidFile;
        use tokio::io::AsyncReadExt;

        // 从 content URI 获取文件描述符
        let android_file = match AndroidFile::from_content_uri(&uri) {
            Ok(f) => f,
            Err(e) => {
                println!("[Command] 无法从 URI 获取文件描述符: {}, 错误: {}", uri, e);
                return Err(format!("无法打开文件: {}", e));
            }
        };

        println!("[Command] 成功从 URI 获取文件描述符: {}", uri);
        
        // 转换为 tokio 文件对象
        let std_file = android_file.into_file();
        let mut file = tokio::fs::File::from_std(std_file);

        // 立即创建数据库记录
        let message_id = match crate::db::save_file_message(
            &state.pool,
            peerId.clone(),
            fileName.clone(),
            fileSize,
            uri.clone(),
            "uploading".to_string(),
        )
        .await
        {
            Ok(id) => {
                println!("[Command] ✓ 已创建上传中记录，ID: {}", id);
                Some(id)
            }
            Err(e) => {
                eprintln!("[Command] ✗ 创建上传记录失败: {}", e);
                None
            }
        };

        // 获取自己的 ID
        let my_id = crate::db::get_user_id(&state.pool).await?;

        // 获取接收方的可用内存
        let receiver_memory_mb = if let Some(peer_state) = app.try_state::<PeerState>() {
            let peers = peer_state.manager.get_all_peers();
            peers
                .iter()
                .find(|p| {
                    p.addr
                        .starts_with(&peerAddr.split(':').next().unwrap_or(""))
                })
                .map(|p| p.available_memory_mb)
                .unwrap_or(1024)
        } else {
            1024
        };

        println!("[Command] 接收方可用内存: {} MB", receiver_memory_mb);

        // 计算分块大小
        let chunk_size = calculate_optimal_chunk_size(fileSize);
        let adjusted_chunk_size = std::cmp::min(
            chunk_size,
            std::cmp::max(
                50 * 1024 * 1024,
                receiver_memory_mb as usize * 1024 * 1024 / 4,
            ),
        );

        let total_chunks = (fileSize + adjusted_chunk_size - 1) / adjusted_chunk_size;

        println!(
            "[Command] 开始分块上传: 文件大小={}, 分块大小={}, 总分块数={}",
            fileSize, adjusted_chunk_size, total_chunks
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| format!("创建客户端失败: {}", e))?;

        let upload_url = format!("http://{}/api/upload", peerAddr);

        let mut offset = 0;
        let mut chunk_index = 0;
        let start_time = std::time::Instant::now();

        loop {
            // 读取分块
            let mut buf = vec![0u8; adjusted_chunk_size];
            let mut bytes_read = 0;

            while bytes_read < adjusted_chunk_size {
                let n = file
                    .read(&mut buf[bytes_read..])
                    .await
                    .map_err(|e| format!("读取文件失败: {}", e))?;

                if n == 0 {
                    break;
                }

                bytes_read += n;
            }

            if bytes_read == 0 {
                break;
            }

            buf.truncate(bytes_read);
            let n = bytes_read;

            // 构造 multipart 请求
            let form = reqwest::multipart::Form::new()
                .text("peer_id", my_id.clone())
                .text("file_name", fileName.clone())
                .text("file_size", fileSize.to_string())
                .text("chunk_index", chunk_index.to_string())
                .text("chunk_total", total_chunks.to_string())
                .part(
                    "chunk",
                    reqwest::multipart::Part::bytes(buf.clone())
                        .mime_str("application/octet-stream")
                        .map_err(|e| format!("设置 MIME 类型失败: {}", e))?,
                );

            println!(
                "[Command] 上传分块 {}/{}, 大小: {} 字节",
                chunk_index + 1,
                total_chunks,
                n
            );

            let response = client.post(&upload_url).multipart(form).send().await.map_err(|e| {
                if let Some(id) = message_id {
                    let pool = state.pool.clone();
                    tokio::spawn(async move {
                        let _ = crate::db::delete_message_by_id(&pool, id).await;
                    });
                }
                format!("上传分块失败: {}", e)
            })?;

            if !response.status().is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "无法读取错误信息".to_string());
                eprintln!("[Command] ✗ 上传分块失败: {}", error_text);

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
                println!(
                    "[Command] 已上传: {} MB, 速度: {:.2} MB/s",
                    offset / (1024 * 1024),
                    speed
                );
                let _ = app.emit(
                    "upload_progress",
                    serde_json::json!({
                        "file_name": fileName.clone(),
                        "speed_mb_s": speed
                    }),
                );
            }
        }

        let total_time = start_time.elapsed().as_secs_f64();
        let avg_speed = fileSize as f64 / (1024.0 * 1024.0) / total_time;
        println!(
            "[Command] ✓ 文件上传完成，耗时: {:.2}s, 平均速度: {:.2} MB/s",
            total_time, avg_speed
        );

        // 更新数据库状态
        if let Some(id) = message_id {
            if let Err(e) = crate::db::update_file_status_by_id(&state.pool, id, "sent").await {
                eprintln!("[Command] ⚠ 更新数据库状态失败: {}", e);
            } else {
                println!("[Command] ✓ 文件状态已更新为 sent");
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "file_name": fileName,
            "file_size": fileSize,
        }))
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, state, peerId, peerAddr, uri, fileName, fileSize);
        Err("此功能仅在 Android 上可用".to_string())
    }
}

#[tauri::command]
pub async fn send_file_from_fd(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    #[allow(non_snake_case)]
    peerId: String,
    #[allow(non_snake_case)]
    peerAddr: String,
    #[allow(non_snake_case)]
    fileName: String,
    #[allow(non_snake_case)]
    fileSize: usize,
    fd: i32,
) -> Result<serde_json::Value, String> {
    println!(
        "[Command] 收到从 FD 发送文件请求: fd={}, name={}, size={}, to={}",
        fd, fileName, fileSize, peerAddr
    );

    #[cfg(target_os = "android")]
    {
        use crate::android_fd::AndroidFile;

        // 从 FD 创建文件对象
        let android_file = AndroidFile::from_fd(fd)?;
        let std_file = android_file.into_file();
        let file = tokio::fs::File::from_std(std_file);

        // 使用统一的上传函数
        let peer_state = app.try_state::<PeerState>();
        upload_file_internal(
            &app,
            &state,
            peer_state.as_ref(),
            peerId,
            peerAddr,
            fileName.clone(),
            fileSize,
            format!("fd:{}", fd),
            file,
        )
        .await
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, state, peerId, peerAddr, fileName, fileSize, fd);
        Err("此功能仅在 Android 上可用".to_string())
    }
}


// Web 端的空实现
#[cfg(not(feature = "desktop"))]
pub struct PeerState;

#[cfg(not(feature = "desktop"))]
pub struct AndroidShareState;

#[cfg(not(feature = "desktop"))]
impl AndroidShareState {
    pub fn new() -> Self {
        Self
    }
}
