use serde::{Deserialize, Serialize};

// 消息结构体 - 对应 messages 表
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,  // SQLite 的 INTEGER PRIMARY KEY 是 i64
    pub sender_id: String,
    pub content: String,
    pub msg_type: String,
    pub timestamp: i64,
    pub file_path: Option<String>,
    pub file_status: Option<String>,
}

// API 响应用的消息结构体（字段名适配前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub from_id: String,
    pub content: String,
    pub timestamp: i64,
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
}

impl From<Message> for MessageResponse {
    fn from(msg: Message) -> Self {
        let mut response = MessageResponse {
            from_id: msg.sender_id,
            content: msg.content.clone(),
            timestamp: msg.timestamp,
            msg_type: msg.msg_type.clone(),
            file_id: None,
            file_name: None,
            file_path: None,
            file_status: None,
            file_size: None,
        };
        
        // 如果是文件消息，添加文件信息
        if msg.msg_type == "file" {
            if let Some(ref path) = msg.file_path {
                // 从路径提取文件名
                let filename = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // 提取文件 ID（如果路径包含 UUID）
                let file_id = filename.split('_').next().unwrap_or(filename);
                
                response.file_id = Some(file_id.to_string());
                response.file_name = Some(msg.content.clone());  // content 存储的是文件名
                response.file_path = Some(path.clone());
                response.file_status = msg.file_status.clone();
                
                // 尝试获取文件大小
                if let Ok(metadata) = std::fs::metadata(path) {
                    response.file_size = Some(metadata.len());
                }
            }
        }
        
        response
    }
}

// 设置结构体 - 对应 settings 表
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
}
