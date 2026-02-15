use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,       // 唯一标识 (UUID)
    pub username: String, // 自动生成的或用户改名后的
    pub is_me: bool,      // 是否是本机用户
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i32,
    pub sender_id: String,
    pub content: String,
    pub msg_type: String, // "text" 或 "file"
    pub timestamp: i64,
    pub file_path: Option<String>,
    pub file_status: Option<String>, // "pending", "downloading", "completed", "deleted"
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Config {
    pub key: String,
    pub value: String,
}
