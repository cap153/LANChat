use axum::{
    body::Body,
    extract::{Json, Multipart, Path, State, WebSocketUpgrade},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum::extract::ws::{Message, WebSocket};
use futures_util::StreamExt;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::fs;
use tower_http::cors::{CorsLayer, Any};

use crate::peers::PeerManager;

#[derive(RustEmbed)]
#[folder = "../src/"]
struct Asset;

#[derive(Serialize)]
struct NameResponse {
    name: String,
}

#[derive(Deserialize)]
struct UpdateNameRequest {
    name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct SendMessageRequest {
    peer_id: String,    // 新增接收者ID
    peer_addr: String,
    content: String,
}

// Web 服务器的状态
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
    pub peer_manager: Arc<PeerManager>,
    #[cfg(feature = "desktop")]
    pub app_handle: Option<tauri::AppHandle>,
}

pub async fn start_server(
    port: u16, 
    _udp_port: u16, 
    pool: Pool<Sqlite>, 
    peer_manager: Arc<PeerManager>,
    #[cfg(feature = "desktop")]
    app_handle: Option<tauri::AppHandle>,
) {
    let state = Arc::new(AppState { 
        pool, 
        peer_manager,
        #[cfg(feature = "desktop")]
        app_handle,
    });
    
    // 配置 CORS - 允许所有来源（局域网内部使用）
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false);  // 明确设置不需要凭证
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/*path", get(serve_assets))
        .route("/api/get_my_name", get(get_name_http))
        .route("/api/get_my_id", get(get_id_http))
        .route("/api/update_my_name", post(update_name_http))
        .route("/api/get_settings", get(get_settings_http))
        .route("/api/update_settings", post(update_settings_http))
        .route("/api/get_peers", get(get_peers_http))
        .route("/api/send_message", post(send_message_http))
        .route("/api/chat_history/:peer_id", get(get_chat_history_http))
        .route("/api/upload", post(upload_file_http))
        .route("/api/accept_file/:file_id", post(accept_file_http))
        .route("/api/download/:file_id", get(download_file_http))
        .route("/api/create_upload_record", post(create_upload_record_http))
        .route("/api/update_upload_status", post(update_upload_status_http))
        .route("/api/delete_upload_record", post(delete_upload_record_http))
        .route("/api/get_theme_list", get(get_theme_list_http))
        .route("/api/get_theme_css/:theme_name", get(get_theme_css_http))
        .route("/api/save_current_theme", post(save_current_theme_http))
        .route("/api/get_current_theme", get(get_current_theme_http))
        .route("/ws", get(websocket_handler))
        .layer(cors)
        .layer(axum::extract::DefaultBodyLimit::disable())  // 无限制
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("[Web Server] 启动在端口 {} (无文件大小限制)", port);
    axum::serve(listener, app).await.unwrap();
}

async fn get_name_http(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    println!("[Web Server] 收到获取用户名请求");
    
    match crate::db::get_username(&state.pool).await {
        Ok(name) => Json(NameResponse { name }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("读取用户名失败: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn get_id_http(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    println!("[Web Server] 收到获取用户 ID 请求");
    
    match crate::db::get_user_id(&state.pool).await {
        Ok(id) => Json(serde_json::json!({ "id": id })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("读取用户 ID 失败: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn get_settings_http(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    println!("[Web Server] 收到获取设置请求");
    
    let download_path = crate::db::get_download_path(&state.pool).await
        .unwrap_or_else(|_| std::env::temp_dir().join("lanchat_downloads").to_str().unwrap().to_string());
    
    Json(serde_json::json!({
        "download_path": download_path,
    })).into_response()
}

#[derive(Deserialize)]
struct UpdateSettingsRequest {
    download_path: Option<String>,
}

async fn update_settings_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 收到更新设置请求");
    
    if let Some(path) = payload.download_path {
        if let Err(e) = crate::db::update_download_path(&state.pool, path).await {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e }),
            ).into_response();
        }
    }
    
    Json(serde_json::json!({ "success": true })).into_response()
}

async fn update_name_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateNameRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 收到改名请求: {}", payload.name);
    
    // 使用数据库的更新函数（包含验证逻辑）
    match crate::db::update_username(&state.pool, payload.name.clone()).await {
        Ok(_) => {
            // 数据库更新后，定时广播线程会自动使用新名称
            println!("[Web Server] 用户名已更新，广播线程将使用新名称");
            
            Json(NameResponse {
                name: payload.name,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
            .into_response(),
    }
}

async fn get_peers_http(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // 不打印日志,避免刷屏
    let peers = state.peer_manager.get_all_peers();
    Json(peers).into_response()
}

async fn serve_index() -> impl IntoResponse {
    serve_assets(axum::extract::Path("index.html".to_string())).await
}

async fn serve_assets(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404"))
            .unwrap(),
    }
}


async fn send_message_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendMessageRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 收到发送消息请求");
    
    // 获取自己的信息
    let my_id = match crate::db::get_user_id(&state.pool).await {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            ).into_response();
        }
    };
    
    let my_name = match crate::db::get_username(&state.pool).await {
        Ok(name) => name,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            ).into_response();
        }
    };
    
    // 发送消息
    if let Err(e) = crate::network::messaging::send_text_message(
        &payload.peer_addr,
        my_id,
        my_name,
        payload.content.clone(),
    ).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        ).into_response();
    }
    
    // 保存到数据库
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    if let Err(e) = sqlx::query(
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp) VALUES ('me', ?, ?, 'text', ?)"
    )
    .bind(&payload.peer_id)   // 接收者ID
    .bind(&payload.content)
    .bind(timestamp)
    .execute(&state.pool)
    .await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        ).into_response();
    }
    
    Json(serde_json::json!({ "success": true })).into_response()
}

async fn get_chat_history_http(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> impl IntoResponse {
    match crate::network::messaging::get_chat_history(&state.pool, &peer_id, 100).await {
        Ok(messages) => Json(serde_json::json!({ "messages": messages })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        ).into_response(),
    }
}


// WebSocket 处理器
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

// 处理 WebSocket 连接
async fn handle_websocket(socket: WebSocket, state: Arc<AppState>) {
    let (_sender, mut receiver) = socket.split();
    
    println!("[WebSocket] 新的 WebSocket 连接");
    
    // 接收消息
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("[WebSocket] 收到文本消息: {}", text);
                
                // 解析消息
                if let Ok(message) = serde_json::from_str::<crate::network::messaging::TextMessage>(&text) {
                    // 保存到数据库
                    if let Err(e) = save_message_to_db(&state.pool, &message).await {
                        eprintln!("[WebSocket] 保存消息失败: {}", e);
                    } else {
                        println!("[WebSocket] 消息已保存: {} 说: {}", message.from_name, message.content);
                        
                        // 桌面端: 发送 Tauri 事件通知前端
                        #[cfg(feature = "desktop")]
                        if let Some(ref app) = state.app_handle {
                            use tauri::Emitter;
                            let _ = app.emit("new-message", serde_json::json!({
                                "from_id": message.from_id,
                                "from_name": message.from_name,
                                "content": message.content,
                                "timestamp": message.timestamp,
                                "msg_type": message.msg_type,
                            }));
                            println!("[WebSocket] 已发送 Tauri 事件: new-message");
                        }
                    }
                } else {
                    eprintln!("[WebSocket] 无法解析消息");
                }
            }
            Ok(Message::Close(_)) => {
                println!("[WebSocket] 连接关闭");
                break;
            }
            Err(e) => {
                eprintln!("[WebSocket] 错误: {}", e);
                break;
            }
            _ => {}
        }
    }
}

// 保存消息到数据库
async fn save_message_to_db(
    pool: &Pool<Sqlite>,
    message: &crate::network::messaging::TextMessage,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO messages (sender_id, content, msg_type, timestamp) VALUES (?, ?, ?, ?)"
    )
    .bind(&message.from_id)
    .bind(&message.content)
    .bind(&message.msg_type)
    .bind(message.timestamp as i64)
    .execute(pool)
    .await
    .map_err(|e| format!("保存消息失败: {}", e))?;
    
    Ok(())
}

async fn upload_file_http(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    println!("[Web Server] 收到文件上传请求");
    
    let mut sender_id = String::new();
    let mut file_name = String::new();
    let mut file_size: u64 = 0;
    let mut chunk_index: usize = 0;
    let mut chunk_total: usize = 0;
    let mut _file_path: Option<std::path::PathBuf> = None;
    let mut chunk_data: Option<Vec<u8>> = None;
    
    // 获取下载目录
    let download_dir = get_download_dir(&state.pool).await;
    if let Err(e) = fs::create_dir_all(&download_dir).await {
        eprintln!("[Web Server] 创建目录失败: {}", e);
    }
    
    // 解析 multipart 字段
    println!("[Web Server] 开始解析 multipart 字段");
    while let Some(mut field) = multipart.next_field().await.ok().flatten() {
        let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
        
        match field_name.as_str() {
            "peer_id" => {
                if let Ok(text) = field.text().await {
                    sender_id = text;
                    println!("[Web Server] sender_id (发送者): {}", sender_id);
                }
            }
            "file_name" => {
                if let Ok(text) = field.text().await {
                    file_name = text;
                    println!("[Web Server] 文件名: {}", file_name);
                }
            }
            "file_size" => {
                if let Ok(text) = field.text().await {
                    file_size = text.parse().unwrap_or(0);
                    println!("[Web Server] 文件总大小: {}", file_size);
                }
            }
            "chunk_index" => {
                if let Ok(text) = field.text().await {
                    chunk_index = text.parse().unwrap_or(0);
                }
            }
            "chunk_total" => {
                if let Ok(text) = field.text().await {
                    chunk_total = text.parse().unwrap_or(0);
                    println!("[Web Server] 分块信息: {}/{}", chunk_index + 1, chunk_total);
                }
            }
            "chunk" => {
                // 读取分块数据
                let mut data = Vec::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    data.extend_from_slice(&chunk);
                }
                chunk_data = Some(data);
                println!("[Web Server] 收到分块数据，大小: {} 字节", chunk_data.as_ref().map(|d| d.len()).unwrap_or(0));
            }
            _ => {
                println!("[Web Server] 忽略未知字段: {}", field_name);
            }
        }
    }
    
    // 验证必需字段
    // 第一块需要所有字段，后续块只需要 chunk_index 和 chunk
    println!("[Web Server] 验证字段: file_name={}, chunk_index={}, chunk_total={}, has_chunk={}", 
             file_name, chunk_index, chunk_total, chunk_data.is_some());
    
    if chunk_data.is_none() {
        eprintln!("[Web Server] ✗ 缺少 chunk 数据");
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { 
                error: "缺少 chunk 数据".to_string() 
            }),
        ).into_response();
    }
    
    if chunk_index == 0 && file_name.is_empty() {
        eprintln!("[Web Server] ✗ 第一块缺少 file_name");
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { 
                error: "第一块缺少 file_name".to_string() 
            }),
        ).into_response();
    }
    
    // 对于后续块，如果 file_name 为空，从数据库查询
    if chunk_index > 0 && file_name.is_empty() {
        println!("[Web Server] 第 {} 块，file_name 为空，从数据库查询 (sender_id={})", chunk_index + 1, sender_id);
        // 从数据库查询该发送者最近的 downloading 状态的文件
        if let Ok(Some(row)) = sqlx::query("SELECT content FROM messages WHERE sender_id = ? AND msg_type = 'file' AND file_status = 'downloading' ORDER BY id DESC LIMIT 1")
            .bind(&sender_id)
            .fetch_optional(&state.pool)
            .await
        {
            use sqlx::Row;
            file_name = row.get("content");
            println!("[Web Server] 从数据库查询到文件名: {} (sender_id={})", file_name, sender_id);
        } else {
            eprintln!("[Web Server] ✗ 无法确定文件名 (sender_id={})", sender_id);
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { 
                    error: "无法确定文件名".to_string() 
                }),
            ).into_response();
        }
    }
    
    let chunk_data = chunk_data.unwrap();
    let path = download_dir.join(&file_name);
    
    // 第一块时创建文件
    if chunk_index == 0 {
        _file_path = Some(path.clone());
        println!("[Web Server] 创建新文件: {:?}", path);
        
        // 删除旧文件（如果存在）
        let _ = tokio::fs::remove_file(&path).await;
    }
    
    // 以追加模式打开文件并写入
    let file = match tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[Web Server] ✗ 打开文件失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { 
                    error: format!("打开文件失败: {}", e) 
                }),
            ).into_response();
        }
    };
    
    let mut writer = tokio::io::BufWriter::new(file);
    
    // 写入分块数据
    if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut writer, &chunk_data).await {
        eprintln!("[Web Server] ✗ 写入文件失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { 
                error: format!("写入文件失败: {}", e) 
            }),
        ).into_response();
    }
    
    // 刷新缓冲区
    if let Err(e) = tokio::io::AsyncWriteExt::flush(&mut writer).await {
        eprintln!("[Web Server] ✗ 刷新缓冲区失败: {}", e);
    }
    
    println!("[Web Server] ✓ 分块 {}/{} 已保存，大小: {} 字节", chunk_index + 1, chunk_total, chunk_data.len());
    
    // 只在第一块时创建数据库记录
    if chunk_index == 0 && !file_name.is_empty() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let result = sqlx::query(
            "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp, file_path, file_status) VALUES (?, ?, ?, 'file', ?, ?, 'downloading')"
        )
        .bind(&sender_id)
        .bind(&crate::db::get_user_id(&state.pool).await.unwrap_or_else(|_| "unknown".to_string()))
        .bind(&file_name)
        .bind(timestamp)
        .bind(path.to_str().unwrap_or(""))
        .execute(&state.pool)
        .await;
        
        match result {
            Ok(_) => {
                println!("[Web Server] ✓ 文件接收开始，已保存到数据库");
                
                // 发送 Tauri 事件通知前端（桌面端）
                #[cfg(feature = "desktop")]
                if let Some(ref app) = state.app_handle {
                    use tauri::Emitter;
                    let _ = app.emit("new-message", serde_json::json!({
                        "from_id": sender_id,
                        "from_name": "Unknown",
                        "content": file_name.clone(),
                        "timestamp": timestamp,
                        "msg_type": "file",
                        "file_name": file_name.clone(),
                        "file_size": file_size,
                        "file_status": "downloading",
                    }));
                    println!("[Web Server] ✓ Tauri 事件已发送 (downloading)");
                }
            }
            Err(e) => {
                eprintln!("[Web Server] ✗ 保存到数据库失败: {}", e);
            }
        }
    }
    
    // 在最后一块时更新状态为 accepted
    if chunk_index == chunk_total - 1 && !file_name.is_empty() {
        let result = sqlx::query(
            "UPDATE messages SET file_status = 'accepted' WHERE content = ? AND msg_type = 'file' AND file_status = 'downloading'"
        )
        .bind(&file_name)
        .execute(&state.pool)
        .await;
        
        match result {
            Ok(_) => {
                println!("[Web Server] ✓ 文件接收完成，状态已更新为 accepted");
                
                // 发送 Tauri 事件通知前端（桌面端）
                #[cfg(feature = "desktop")]
                if let Some(ref app) = state.app_handle {
                    use tauri::Emitter;
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    let _ = app.emit("new-message", serde_json::json!({
                        "from_id": sender_id,
                        "from_name": "Unknown",
                        "content": file_name.clone(),
                        "timestamp": timestamp,
                        "msg_type": "file",
                        "file_name": file_name.clone(),
                        "file_size": file_size,
                        "file_status": "accepted",
                    }));
                    println!("[Web Server] ✓ Tauri 事件已发送 (accepted)");
                }
            }
            Err(e) => {
                eprintln!("[Web Server] ✗ 更新数据库失败: {}", e);
            }
        }
    }
    
    println!("[Web Server] ========== 文件上传处理完成 ==========");
    Json(serde_json::json!({
        "success": true,
        "file_name": file_name,
        "file_size": file_size,
        "chunk_index": chunk_index,
        "chunk_total": chunk_total,
    })).into_response()
}

// 接受文件（手动接收模式）
#[derive(Deserialize)]
struct AcceptFileRequest {
    save_path: Option<String>,
}

async fn accept_file_http(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
    Json(payload): Json<AcceptFileRequest>,
) -> impl IntoResponse {
    println!("[Web Server] ========== 开始处理文件接收 ==========");
    println!("[Web Server] 收到接受文件请求: file_id={}", file_id);
    println!("[Web Server] save_path: {:?}", payload.save_path);
    
    // 先列出所有文件消息，方便调试
    println!("[Web Server] 查询所有文件消息...");
    if let Ok(rows) = sqlx::query("SELECT id, sender_id, content, file_path, file_status FROM messages WHERE msg_type = 'file' ORDER BY id DESC LIMIT 10")
        .fetch_all(&state.pool)
        .await {
        use sqlx::Row;
        for row in rows {
            let id: i64 = row.get("id");
            let sender: String = row.get("sender_id");
            let content: String = row.get("content");
            let path: String = row.get("file_path");
            let status: String = row.get("file_status");
            println!("[Web Server]   ID={}, sender={}, content={}, path={}, status={}", 
                     id, sender, content, path, status);
        }
    }
    
    // 从数据库查询文件信息 - 使用 file_path LIKE 模糊匹配
    let query_pattern = format!("%{}%", file_id);
    println!("[Web Server] 查询模式: {}", query_pattern);
    
    let row = sqlx::query(
        "SELECT file_path, content FROM messages WHERE file_path LIKE ? AND file_status = 'pending'"
    )
    .bind(&query_pattern)
    .fetch_optional(&state.pool)
    .await;
    
    let row = match row {
        Ok(Some(r)) => {
            println!("[Web Server] ✓ 找到匹配的 pending 文件记录");
            r
        },
        Ok(None) => {
            println!("[Web Server] ✗ 未找到匹配的 pending 文件");
            println!("[Web Server] 可能原因:");
            println!("[Web Server]   1. file_id 不匹配任何文件路径");
            println!("[Web Server]   2. 文件状态不是 'pending'");
            println!("[Web Server]   3. 文件已被接收或删除");
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: format!("文件不存在或已接收 (file_id={})", file_id) }),
            ).into_response();
        }
        Err(e) => {
            println!("[Web Server] ✗ 数据库查询失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("查询文件失败: {}", e) }),
            ).into_response();
        }
    };
    
    use sqlx::Row;
    let temp_path: String = row.get("file_path");
    let file_name: String = row.get("content");
    
    println!("[Web Server] 临时路径: {}", temp_path);
    println!("[Web Server] 文件名: {}", file_name);
    
    // 检查文件是否存在（可能还在上传中）
    if !std::path::Path::new(&temp_path).exists() {
        println!("[Web Server] ⏳ 文件还在下载中");
        return (
            StatusCode::ACCEPTED,  // 202 表示请求已接受但还在处理中
            Json(serde_json::json!({
                "downloading": true,
                "message": "文件正在下载中，请稍候..."
            })),
        ).into_response();
    }
    
    // 确定最终保存路径
    let final_path = if let Some(path) = payload.save_path {
        std::path::PathBuf::from(path).join(&file_name)
    } else {
        let download_path = crate::db::get_download_path(&state.pool).await
            .unwrap_or_else(|_| std::env::temp_dir().join("lanchat_downloads").to_str().unwrap().to_string());
        println!("[Web Server] 使用默认下载路径: {}", download_path);
        std::path::PathBuf::from(download_path).join(&file_name)
    };
    
    println!("[Web Server] 最终路径: {:?}", final_path);
    
    // 确保目标目录存在
    if let Some(parent) = final_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            println!("[Web Server] ✗ 创建目录失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("创建目录失败: {}", e) }),
            ).into_response();
        }
    }
    
    // 移动文件 - 使用复制+删除来支持跨文件系统
    println!("[Web Server] 开始移动文件...");
    if let Err(e) = std::fs::rename(&temp_path, &final_path) {
        // rename 失败（可能是跨文件系统），尝试复制+删除
        println!("[Web Server] rename 失败 ({}), 尝试复制+删除", e);
        
        if let Err(e) = std::fs::copy(&temp_path, &final_path) {
            println!("[Web Server] ✗ 复制文件失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("复制文件失败: {}", e) }),
            ).into_response();
        }
        
        // 复制成功后删除临时文件
        if let Err(e) = std::fs::remove_file(&temp_path) {
            println!("[Web Server] ⚠ 删除临时文件失败: {}", e);
            // 不返回错误，因为文件已经复制成功了
        }
        
        println!("[Web Server] ✓ 文件已复制到目标位置");
    } else {
        println!("[Web Server] ✓ 文件已移动到目标位置");
    }
    
    // 更新数据库状态
    if let Err(e) = sqlx::query(
        "UPDATE messages SET file_status = 'accepted', file_path = ? WHERE file_path = ?"
    )
    .bind(final_path.to_str().unwrap())
    .bind(&temp_path)
    .execute(&state.pool)
    .await {
        println!("[Web Server] ✗ 更新数据库失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("更新数据库失败: {}", e) }),
        ).into_response();
    }
    
    println!("[Web Server] ✓ 文件已接受并保存到: {:?}", final_path);
    Json(serde_json::json!({
        "success": true,
        "path": final_path.to_str().unwrap()
    })).into_response()
}

// 下载文件
async fn download_file_http(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> impl IntoResponse {
    println!("[Web Server] 收到文件下载请求: {}", file_id);
    
    let download_dir = get_download_dir(&state.pool).await;
    let file_path = download_dir.join(&file_id);
    
    match fs::read(&file_path).await {
        Ok(data) => {
            Response::builder()
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_id))
                .body(Body::from(data))
                .unwrap()
        }
        Err(e) => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("文件不存在: {}", e)))
                .unwrap()
        }
    }
}

// 获取下载目录
async fn get_download_dir(pool: &Pool<Sqlite>) -> std::path::PathBuf {
    // 从数据库读取配置
    if let Ok(row) = sqlx::query("SELECT value FROM settings WHERE key = 'download_path'")
        .fetch_one(pool)
        .await
    {
        use sqlx::Row;
        let path: String = row.get("value");
        return std::path::PathBuf::from(path);
    }
    
    // 默认路径
    std::env::temp_dir().join("lanchat_downloads")
}

// 创建上传记录（Web 端发送文件时）
#[derive(Deserialize)]
struct CreateUploadRecordRequest {
    file_name: String,
    timestamp: i64,
    receiver_id: String,  // 新增接收者ID
}

async fn create_upload_record_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUploadRecordRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 创建上传记录: {}", payload.file_name);
    
    let result = sqlx::query(
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp, file_path, file_status) VALUES ('me', ?, ?, 'file', ?, '', 'uploading')"
    )
    .bind(&payload.receiver_id)  // 接收者ID
    .bind(&payload.file_name)
    .bind(payload.timestamp)
    .execute(&state.pool)
    .await;
    
    match result {
        Ok(_) => {
            println!("[Web Server] ✓ 上传记录已创建");
            Json(serde_json::json!({ "success": true })).into_response()
        }
        Err(e) => {
            eprintln!("[Web Server] ✗ 创建上传记录失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("创建记录失败: {}", e) }),
            ).into_response()
        }
    }
}

// 更新上传状态
#[derive(Deserialize)]
struct UpdateUploadStatusRequest {
    file_name: String,
    timestamp: i64,
    status: String,
}

async fn update_upload_status_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateUploadStatusRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 更新上传状态: {} -> {}", payload.file_name, payload.status);
    
    let result = sqlx::query(
        "UPDATE messages SET file_status = ? WHERE sender_id = 'me' AND content = ? AND timestamp = ?"
    )
    .bind(&payload.status)
    .bind(&payload.file_name)
    .bind(payload.timestamp)
    .execute(&state.pool)
    .await;
    
    match result {
        Ok(_) => {
            println!("[Web Server] ✓ 上传状态已更新");
            Json(serde_json::json!({ "success": true })).into_response()
        }
        Err(e) => {
            eprintln!("[Web Server] ✗ 更新上传状态失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("更新状态失败: {}", e) }),
            ).into_response()
        }
    }
}

// 删除上传记录（上传失败时）
#[derive(Deserialize)]
struct DeleteUploadRecordRequest {
    file_name: String,
    timestamp: i64,
}

async fn delete_upload_record_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteUploadRecordRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 删除上传记录: {}", payload.file_name);
    
    let result = sqlx::query(
        "DELETE FROM messages WHERE sender_id = 'me' AND content = ? AND timestamp = ? AND file_status = 'uploading'"
    )
    .bind(&payload.file_name)
    .bind(payload.timestamp)
    .execute(&state.pool)
    .await;
    
    match result {
        Ok(_) => {
            println!("[Web Server] ✓ 上传记录已删除");
            Json(serde_json::json!({ "success": true })).into_response()
        }
        Err(e) => {
            eprintln!("[Web Server] ✗ 删除上传记录失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("删除记录失败: {}", e) }),
            ).into_response()
        }
    }
}

// 主题相关的 HTTP 处理函数
async fn get_theme_list_http() -> impl IntoResponse {
    println!("[Web Server] 收到获取主题列表请求");
    
    let mut themes = vec![
        serde_json::json!({
            "name": "default",
            "display_name": "默认主题",
            "is_custom": false
        })
    ];
    
    // 检查自定义主题目录
    if let Some(home_dir) = dirs::home_dir() {
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
    }
    
    println!("[Web Server] 找到 {} 个主题", themes.len());
    Json(themes).into_response()
}

async fn get_theme_css_http(Path(theme_name): Path<String>) -> impl IntoResponse {
    println!("[Web Server] 收到获取主题CSS请求: {}", theme_name);
    
    if theme_name == "default" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/css")
            .body(Body::from(""))
            .unwrap()
            .into_response();
    }
    
    if let Some(home_dir) = dirs::home_dir() {
        let theme_path = home_dir.join(".config").join("lanchat").join(format!("{}.css", theme_name));
        
        if theme_path.exists() {
            match std::fs::read_to_string(&theme_path) {
                Ok(css_content) => {
                    println!("[Web Server] 成功读取主题文件: {} ({} 字节)", theme_path.display(), css_content.len());
                    return Response::builder()
                        .header(header::CONTENT_TYPE, "text/css")
                        .body(Body::from(css_content))
                        .unwrap()
                        .into_response();
                }
                Err(e) => {
                    eprintln!("[Web Server] 读取主题文件失败: {}", e);
                }
            }
        }
    }
    
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: format!("主题文件不存在: {}", theme_name),
        }),
    ).into_response()
}

#[derive(Deserialize)]
struct SaveThemeRequest {
    theme_name: String,
}

async fn save_current_theme_http(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SaveThemeRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 收到保存主题请求: {}", req.theme_name);
    
    match sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('current_theme', ?)")
        .bind(&req.theme_name)
        .execute(&state.pool)
        .await
    {
        Ok(_) => {
            println!("[Web Server] 主题设置已保存: {}", req.theme_name);
            Json(serde_json::json!({"success": true})).into_response()
        }
        Err(e) => {
            eprintln!("[Web Server] 保存主题设置失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("保存主题设置失败: {}", e),
                }),
            ).into_response()
        }
    }
}

async fn get_current_theme_http(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    println!("[Web Server] 收到获取当前主题请求");
    
    match sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = 'current_theme'")
        .fetch_optional(&state.pool)
        .await
    {
        Ok(result) => {
            let theme = result.unwrap_or_else(|| "default".to_string());
            println!("[Web Server] 当前主题: {}", theme);
            Json(serde_json::json!({"theme": theme})).into_response()
        }
        Err(e) => {
            eprintln!("[Web Server] 查询主题设置失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("查询主题设置失败: {}", e),
                }),
            ).into_response()
        }
    }
}