// 消息发送和接收模块
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(feature = "desktop")]
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessage {
    pub msg_type: String,      // "text"
    pub from_id: String,        // 发送者 UUID
    pub from_name: String,      // 发送者名字
    pub content: String,        // 消息内容
    pub timestamp: u64,         // Unix 时间戳
}

// 发送文本消息
pub async fn send_text_message(
    peer_addr: &str,
    from_id: String,
    from_name: String,
    content: String,
) -> Result<(), String> {
    println!("[Messaging] 正在连接到 {}...", peer_addr);
    
    // 构造消息
    let message = TextMessage {
        msg_type: "text".to_string(),
        from_id,
        from_name,
        content,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    // 序列化为 JSON
    let json = serde_json::to_string(&message)
        .map_err(|e| format!("序列化失败: {}", e))?;
    
    // 尝试通过 WebSocket 发送
    let ws_url = format!("ws://{}/ws", peer_addr);
    
    match tokio_tungstenite::connect_async(&ws_url).await {
        Ok((mut ws_stream, _)) => {
            println!("[Messaging] WebSocket 连接成功");
            
            use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
            use futures_util::SinkExt;
            
            ws_stream.send(WsMessage::Text(json))
                .await
                .map_err(|e| format!("发送失败: {}", e))?;
            
            // 优雅地关闭连接
            let _ = ws_stream.close(None).await;
            
            println!("[Messaging] 消息发送成功");
            Ok(())
        }
        Err(e) => {
            eprintln!("[Messaging] WebSocket 连接失败: {}, 尝试 TCP", e);
            // 回退到 TCP
            send_via_tcp(peer_addr, message).await
        }
    }
}

// 通过 TCP 发送(回退方案)
async fn send_via_tcp(peer_addr: &str, message: TextMessage) -> Result<(), String> {
    use tokio::net::TcpStream;
    
    let mut stream = TcpStream::connect(peer_addr)
        .await
        .map_err(|e| format!("TCP 连接失败: {}", e))?;
    
    let json = serde_json::to_string(&message)
        .map_err(|e| format!("序列化失败: {}", e))?;
    
    // 发送消息长度(4字节)
    let len = json.len() as u32;
    stream.write_all(&len.to_be_bytes())
        .await
        .map_err(|e| format!("发送长度失败: {}", e))?;
    
    // 发送消息内容
    stream.write_all(json.as_bytes())
        .await
        .map_err(|e| format!("发送消息失败: {}", e))?;
    
    println!("[Messaging] TCP 消息发送成功");
    Ok(())
}

// 启动消息接收服务器
pub async fn start_message_server(
    port: u16,
    db_pool: sqlx::Pool<sqlx::Sqlite>,
    #[cfg(feature = "desktop")]
    app_handle: Option<tauri::AppHandle>,
) {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("无法绑定消息服务器端口");
    
    println!("[Messaging] 消息服务器启动在端口 {}", port);
    
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("[Messaging] 收到来自 {} 的连接", addr);
                
                let pool = db_pool.clone();
                #[cfg(feature = "desktop")]
                let app = app_handle.clone();
                
                tokio::spawn(async move {
                    #[cfg(feature = "desktop")]
                    let result = handle_message_connection(stream, pool, app).await;
                    
                    #[cfg(feature = "web")]
                    let result = handle_message_connection(stream, pool).await;
                    
                    if let Err(e) = result {
                        eprintln!("[Messaging] 处理消息失败: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[Messaging] 接受连接失败: {}", e);
            }
        }
    }
}

// 处理单个消息连接 - 桌面端版本
#[cfg(all(feature = "desktop", not(feature = "web")))]
async fn handle_message_connection(
    mut stream: tokio::net::TcpStream,
    db_pool: sqlx::Pool<sqlx::Sqlite>,
    app_handle: Option<tauri::AppHandle>,
) -> Result<(), String> {
    // 读取消息长度(4字节)
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)
        .await
        .map_err(|e| format!("读取长度失败: {}", e))?;
    
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    if len > 1024 * 1024 {
        return Err("消息过大".to_string());
    }
    
    // 读取消息内容
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)
        .await
        .map_err(|e| format!("读取消息失败: {}", e))?;
    
    // 解析 JSON
    let json_str = String::from_utf8(buffer)
        .map_err(|e| format!("UTF-8 解析失败: {}", e))?;
    
    let message: TextMessage = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    
    println!("[Messaging] 收到消息: {} 说: {}", message.from_name, message.content);
    
    // 保存到数据库
    save_message_to_db(&db_pool, &message).await?;
    
    // 发送事件通知前端
    if let Some(app) = app_handle {
        let _ = app.emit("new-message", serde_json::json!({
            "from_id": message.from_id,
            "from_name": message.from_name,
            "content": message.content,
            "timestamp": message.timestamp,
        }));
    }
    
    Ok(())
}

// Web 端的消息处理 - 不带 AppHandle
#[cfg(all(feature = "web", not(feature = "desktop")))]
async fn handle_message_connection(
    mut stream: tokio::net::TcpStream,
    db_pool: sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), String> {
    // 读取消息长度(4字节)
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)
        .await
        .map_err(|e| format!("读取长度失败: {}", e))?;
    
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    if len > 1024 * 1024 {
        return Err("消息过大".to_string());
    }
    
    // 读取消息内容
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)
        .await
        .map_err(|e| format!("读取消息失败: {}", e))?;
    
    // 解析 JSON
    let json_str = String::from_utf8(buffer)
        .map_err(|e| format!("UTF-8 解析失败: {}", e))?;
    
    let message: TextMessage = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    
    println!("[Messaging] 收到消息: {} 说: {}", message.from_name, message.content);
    
    // 保存到数据库
    save_message_to_db(&db_pool, &message).await?;
    
    // Web 端暂时只保存,不通知前端(前端会轮询)
    
    Ok(())
}

// 保存消息到数据库
async fn save_message_to_db(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    message: &TextMessage,
) -> Result<(), String> {
    // 获取当前用户ID作为接收者
    let my_id = crate::db::get_user_id(pool).await?;
    
    sqlx::query(
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&message.from_id)  // 发送者ID
    .bind(&my_id)            // 接收者ID（当前用户）
    .bind(&message.content)
    .bind(&message.msg_type)
    .bind(message.timestamp as i64)
    .execute(pool)
    .await
    .map_err(|e| format!("保存消息失败: {}", e))?;
    
    println!("[Messaging] 消息已保存到数据库");
    Ok(())
}

// 查询聊天历史
pub async fn get_chat_history(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    peer_id: &str,
    limit: i32,
) -> Result<Vec<serde_json::Value>, String> {
    // 获取当前用户ID
    let my_id = crate::db::get_user_id(pool).await?;
    
    // 查询双向对话：
    // 1. 我发送给对方的消息 (sender_id = my_id AND receiver_id = peer_id)
    // 2. 对方发送给我的消息 (sender_id = peer_id AND (receiver_id = my_id OR receiver_id IS NULL))
    // 3. 兼容旧数据：sender_id = 'me' 的消息
    let messages = sqlx::query_as::<_, crate::models::Message>(
        "SELECT id, sender_id, receiver_id, content, msg_type, timestamp, file_path, file_status 
         FROM messages 
         WHERE 
            (sender_id = ? AND receiver_id = ?) OR 
            (sender_id = ? AND (receiver_id = ? OR receiver_id IS NULL)) OR
            (sender_id = 'me' AND receiver_id = ?)
         ORDER BY timestamp ASC LIMIT ?"
    )
    .bind(&my_id)        // 我发送的消息
    .bind(peer_id)       // 发送给对方
    .bind(peer_id)       // 对方发送的消息
    .bind(&my_id)        // 发送给我
    .bind(peer_id)       // 兼容旧数据
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("查询历史失败: {}", e))?;
    
    // 转换为 MessageResponse 并序列化为 JSON
    let responses: Vec<serde_json::Value> = messages
        .into_iter()
        .map(|msg| {
            let response = crate::models::MessageResponse::from(msg);
            serde_json::to_value(response).unwrap_or(serde_json::json!({}))
        })
        .collect();
    
    Ok(responses)
}
