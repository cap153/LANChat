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
use tokio::io::AsyncWriteExt;
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
    peer_id: String,
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
        .allow_headers(Any);
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/*path", get(serve_assets))
        .route("/api/get_my_name", get(get_name_http))
        .route("/api/get_my_id", get(get_id_http))
        .route("/api/update_my_name", post(update_name_http))
        .route("/api/get_peers", get(get_peers_http))
        .route("/api/send_message", post(send_message_http))
        .route("/api/chat_history/:peer_id", get(get_chat_history_http))
        .route("/api/upload", post(upload_file_http))
        .route("/api/download/:file_id", get(download_file_http))
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

async fn update_name_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateNameRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 收到改名请求: {}", payload.name);
    
    // 使用数据库的更新函数（包含验证逻辑）
    match crate::db::update_username(&state.pool, payload.name.clone()).await {
        Ok(_) => Json(NameResponse {
            name: payload.name,
        })
        .into_response(),
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
        "INSERT INTO messages (sender_id, content, msg_type, timestamp) VALUES ('me', ?, 'text', ?)"
    )
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


// 上传文件 - 使用流式处理
async fn upload_file_http(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    println!("[Web Server] 收到文件上传请求");
    
    let mut sender_id = String::new();  // 改名：这是发送者的 ID
    let mut file_name = String::new();
    let mut file_path: Option<std::path::PathBuf> = None;
    let mut file_size: usize = 0;
    
    // 获取下载目录
    let download_dir = get_download_dir(&state.pool).await;
    if let Err(e) = fs::create_dir_all(&download_dir).await {
        eprintln!("[Web Server] 创建目录失败: {}", e);
    }
    
    // 解析 multipart 数据 - 使用流式处理
    while let Some(mut field) = multipart.next_field().await.ok().flatten() {
        let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
        println!("[Web Server] 解析字段: {}", field_name);
        
        match field_name.as_str() {
            "peer_id" => {
                if let Ok(text) = field.text().await {
                    sender_id = text;
                    println!("[Web Server] sender_id (发送者): {}", sender_id);
                }
            }
            "file" => {
                file_name = field.file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                println!("[Web Server] 文件名: {}", file_name);
                
                // 生成文件 ID 和路径
                let file_id = uuid::Uuid::new_v4().to_string();
                let safe_filename = format!("{}_{}", file_id, file_name);
                let path = download_dir.join(&safe_filename);
                
                println!("[Web Server] 保存文件到: {:?}", path);
                
                // 创建文件并流式写入
                match fs::File::create(&path).await {
                    Ok(mut file) => {
                        // 使用 chunk() 流式读取
                        while let Ok(Some(chunk)) = field.chunk().await {
                            if let Err(e) = file.write_all(&chunk).await {
                                eprintln!("[Web Server] 写入文件失败: {}", e);
                                return (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(ErrorResponse { 
                                        error: format!("写入文件失败: {}", e) 
                                    }),
                                ).into_response();
                            }
                            file_size += chunk.len();
                        }
                        
                        println!("[Web Server] 文件大小: {} 字节", file_size);
                        file_path = Some(path);
                    }
                    Err(e) => {
                        eprintln!("[Web Server] 创建文件失败: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse { 
                                error: format!("创建文件失败: {}", e) 
                            }),
                        ).into_response();
                    }
                }
            }
            _ => {
                println!("[Web Server] 忽略未知字段: {}", field_name);
            }
        }
    }
    
    // 验证必需字段
    if file_name.is_empty() || file_path.is_none() || file_size == 0 {
        eprintln!("[Web Server] 文件验证失败: file_name={}, size={}", 
                  file_name, file_size);
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { 
                error: format!("缺少文件或文件为空 (name={}, size={})", file_name, file_size) 
            }),
        ).into_response();
    }
    
    let file_path = file_path.unwrap();
    println!("[Web Server] 接收文件: {}, 大小: {} 字节, sender_id: {}", 
             file_name, file_size, sender_id);
    
    // 保存到数据库
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 从文件路径提取 file_id
    let file_id = file_path.file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.split('_').next())
        .unwrap_or("unknown")
        .to_string();
    
    if let Err(e) = sqlx::query(
        "INSERT INTO messages (sender_id, content, msg_type, timestamp, file_path, file_status) VALUES (?, ?, 'file', ?, ?, ?)"
    )
    .bind(&sender_id)
    .bind(&file_name)
    .bind(timestamp)
    .bind(file_path.to_str().unwrap())
    .bind(file_size.to_string())
    .execute(&state.pool)
    .await {
        eprintln!("[Web Server] 保存数据库失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("保存记录失败: {}", e) }),
        ).into_response();
    }
    
    println!("[Web Server] 文件保存成功: {} ({}字节)", file_id, file_size);
    
    // 桌面端: 发送 Tauri 事件通知前端
    #[cfg(feature = "desktop")]
    if let Some(ref app) = state.app_handle {
        use tauri::Emitter;
        let _ = app.emit("new-message", serde_json::json!({
            "from_id": sender_id,
            "from_name": "Unknown",  // 文件上传时没有传递用户名
            "content": file_name.clone(),
            "timestamp": timestamp,
            "msg_type": "file",
            "file_id": file_id.clone(),
            "file_name": file_name.clone(),
            "file_size": file_size,
        }));
        println!("[Web Server] 已发送文件接收 Tauri 事件: new-message");
    }
    
    Json(serde_json::json!({
        "success": true,
        "file_id": file_id,
        "file_name": file_name,
        "file_size": file_size,
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
