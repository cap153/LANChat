use crate::utils::generate_random_name;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::path::PathBuf;

#[cfg(feature = "desktop")]
use tauri::AppHandle;
#[cfg(feature = "desktop")]
use tauri::Manager;

pub struct DbState {
    pub pool: Pool<Sqlite>,
}

pub async fn get_username(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<String, String> {
    println!("[DB] 正在从数据库读取用户名...");
    let res: (String,) = sqlx::query_as("SELECT value FROM settings WHERE key = 'username'")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            println!("[DB] 读取失败: {}", e);
            e.to_string()
        })?;
    println!("[DB] 读取成功: {}", res.0);
    Ok(res.0)
}

pub async fn get_user_id(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<String, String> {
    let res: Result<(String,), _> = sqlx::query_as("SELECT value FROM settings WHERE key = 'user_id'")
        .fetch_one(pool)
        .await;
    
    match res {
        Ok((id,)) => Ok(id),
        Err(_) => {
            // 如果没有 user_id,生成一个并保存
            let user_id = uuid::Uuid::new_v4().to_string();
            println!("[DB] 生成并保存新的用户 ID: {}", user_id);
            
            sqlx::query("INSERT INTO settings (key, value) VALUES ('user_id', ?)")
                .bind(&user_id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            
            Ok(user_id)
        }
    }
}

pub async fn update_username(pool: &sqlx::Pool<sqlx::Sqlite>, new_name: String) -> Result<(), String> {
    println!("[DB] 正在更新用户名为: {}", new_name);
    
    // 验证用户名不为空
    if new_name.trim().is_empty() {
        return Err("用户名不能为空".to_string());
    }
    
    // 验证用户名长度
    if new_name.len() > 50 {
        return Err("用户名过长（最多50个字符）".to_string());
    }
    
    sqlx::query("UPDATE settings SET value = ? WHERE key = 'username'")
        .bind(new_name.trim())
        .execute(pool)
        .await
        .map_err(|e| {
            println!("[DB] 更新失败: {}", e);
            e.to_string()
        })?;
    
    println!("[DB] 用户名更新成功");
    Ok(())
}

// 获取下载路径
pub async fn get_download_path(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<String, String> {
    let res: Result<(String,), _> = sqlx::query_as("SELECT value FROM settings WHERE key = 'download_path'")
        .fetch_one(pool)
        .await;
    
    match res {
        Ok((path,)) => Ok(path),
        Err(_) => {
            // 如果没有设置，返回默认路径
            if cfg!(target_os = "android") {
                Ok("/storage/emulated/0/Download/LANChat".to_string())
            } else {
                let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
                let default_path = home_dir.join("Downloads").join("LANChat");
                Ok(default_path.to_string_lossy().to_string())
            }
        }
    }
}

// 更新下载路径
pub async fn update_download_path(pool: &sqlx::Pool<sqlx::Sqlite>, new_path: String) -> Result<(), String> {
    println!("[DB] 正在更新下载路径为: {}", new_path);
    
    // 验证路径不为空
    if new_path.trim().is_empty() {
        return Err("路径不能为空".to_string());
    }
    
    // 尝试创建目录
    if let Err(e) = std::fs::create_dir_all(&new_path) {
        return Err(format!("无法创建目录: {}", e));
    }
    
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('download_path', ?)")
        .bind(new_path.trim())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    println!("[DB] 下载路径更新成功");
    Ok(())
}



// 为 Tauri 桌面端初始化数据库
#[cfg(feature = "desktop")]
pub async fn init_db(app_handle: &AppHandle) -> Result<Pool<Sqlite>, sqlx::Error> {
    let app_dir = app_handle.path().app_data_dir().expect("读取路径失败");
    init_db_with_path(app_dir).await
}

// 为 Web 端初始化数据库（使用自定义路径）
pub async fn init_db_standalone(custom_path: Option<PathBuf>) -> Result<Pool<Sqlite>, sqlx::Error> {
    let app_dir = if let Some(path) = custom_path {
        path
    } else {
        // 默认使用与桌面端相同的路径: ~/.local/share/com.lanchat.app/
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share/com.lanchat.app")
    };
    
    init_db_with_path(app_dir).await
}

// 通用的数据库初始化逻辑
async fn init_db_with_path(app_dir: PathBuf) -> Result<Pool<Sqlite>, sqlx::Error> {
    println!("[DB] 数据库路径: {:?}", app_dir);
    
    // 确保目录一定存在
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).unwrap();
    }

    let db_path = app_dir.join("lanchat.db");
    let db_url = format!("sqlite:{}", db_path.to_str().unwrap());

    // 检查文件是否存在，如果不存在，手动创建空文件
    if !db_path.exists() {
        std::fs::File::create(&db_path).unwrap();
    }

    let pool = SqlitePool::connect(&db_url).await?;

    // 创建表结构
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sender_id TEXT,
            receiver_id TEXT,
            content TEXT,
            msg_type TEXT,
            timestamp INTEGER,
            file_path TEXT,
            file_status TEXT
        )",
    )
    .execute(&pool)
    .await?;

    // 数据库迁移：为现有的messages表添加receiver_id字段（如果不存在）
    let _ = sqlx::query("ALTER TABLE messages ADD COLUMN receiver_id TEXT")
        .execute(&pool)
        .await; // 忽略错误，因为字段可能已经存在

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
    )
    .execute(&pool)
    .await?;

    // 初始化配置 (如果没有用户名则生成一个)
    let user_exists = sqlx::query("SELECT value FROM settings WHERE key = 'username'")
        .fetch_optional(&pool)
        .await?;

    if user_exists.is_none() {
        let random_name = generate_random_name();
        println!("[DB] 生成随机用户名: {}", random_name);
        
        // 生成唯一的 UUID
        let user_id = uuid::Uuid::new_v4().to_string();
        println!("[DB] 生成用户 ID: {}", user_id);
        
        sqlx::query("INSERT INTO settings (key, value) VALUES ('username', ?)")
            .bind(random_name)
            .execute(&pool)
            .await?;

        sqlx::query("INSERT INTO settings (key, value) VALUES ('user_id', ?)")
            .bind(user_id)
            .execute(&pool)
            .await?;

        // 初始保存路径 - 统一使用 ~/Downloads/LANChat
        let download_dir = if cfg!(target_os = "android") {
            "/storage/emulated/0/Download/LANChat".to_string()
        } else {
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
            home_dir.join("Downloads").join("LANChat").to_string_lossy().to_string()
        };
        
        println!("[DB] 设置默认下载路径: {}", download_dir);
        
        sqlx::query("INSERT INTO settings (key, value) VALUES ('download_path', ?)")
            .bind(download_dir)
            .execute(&pool)
            .await?;
    }

    Ok(pool)
}


// ==================== 文件相关的数据库函数 ====================

/// 保存文件消息到数据库
/// 如果存在相同文件名和状态为 uploading 的消息，则更新；否则新建
pub async fn save_file_message(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    peer_id: String,
    file_name: String,
    file_size: usize,
    file_path: String,
    status: String,
) -> Result<i64, String> {
    println!("[DB] 保存文件消息: 文件={}, 大小={}, 状态={}", file_name, file_size, status);
    
    // 检查是否存在相同文件名和状态为 uploading 的消息
    let existing = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM messages WHERE receiver_id = ? AND content = ? AND msg_type = 'file' AND file_status = 'uploading' ORDER BY id DESC LIMIT 1"
    )
    .bind(&peer_id)
    .bind(&file_name)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("查询消息失败: {}", e))?;
    
    if let Some((msg_id,)) = existing {
        // 更新现有的 uploading 消息
        println!("[DB] 更新现有消息 ID: {}, 状态: {} -> {}", msg_id, "uploading", status);
        sqlx::query(
            "UPDATE messages SET file_path = ?, file_status = ? WHERE id = ?"
        )
        .bind(&file_path)
        .bind(&status)
        .bind(msg_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新消息失败: {}", e))?;
        
        println!("[DB] 消息已更新");
        Ok(msg_id)
    } else {
        // 插入新消息
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let result = sqlx::query(
            "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp, file_path, file_status) VALUES ('me', ?, ?, 'file', ?, ?, ?)"
        )
        .bind(&peer_id)
        .bind(&file_name)
        .bind(timestamp)
        .bind(&file_path)
        .bind(&status)
        .execute(pool)
        .await
        .map_err(|e| format!("保存消息失败: {}", e))?;
        
        let msg_id = result.last_insert_rowid();
        println!("[DB] 新消息已保存，ID: {}", msg_id);
        Ok(msg_id)
    }
}

/// 获取下载中的文件（根据发送者ID）
pub async fn get_downloading_file(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    sender_id: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query("SELECT content FROM messages WHERE sender_id = ? AND msg_type = 'file' AND file_status = 'downloading' ORDER BY id DESC LIMIT 1")
        .bind(sender_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询文件失败: {}", e))?;
    
    if let Some(row) = row {
        use sqlx::Row;
        let file_name: String = row.get("content");
        Ok(Some(file_name))
    } else {
        Ok(None)
    }
}

/// 更新文件状态（从 downloading 到 accepted）
pub async fn update_file_status(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    file_name: &str,
    new_status: &str,
) -> Result<(), String> {
    println!("[DB] 更新文件状态: {} -> {}", file_name, new_status);
    
    sqlx::query(
        "UPDATE messages SET file_status = ? WHERE content = ? AND msg_type = 'file' AND file_status = 'downloading'"
    )
    .bind(new_status)
    .bind(file_name)
    .execute(pool)
    .await
    .map_err(|e| format!("更新文件状态失败: {}", e))?;
    
    println!("[DB] 文件状态已更新");
    Ok(())
}

/// 创建文件接收记录（Web端发送文件时）
pub async fn create_upload_record(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    receiver_id: String,
    file_name: String,
    timestamp: i64,
) -> Result<i64, String> {
    println!("[DB] 创建上传记录: 接收者={}, 文件={}", receiver_id, file_name);
    
    let result = sqlx::query(
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp, file_path, file_status) VALUES ('me', ?, ?, 'file', ?, '', 'uploading')"
    )
    .bind(&receiver_id)
    .bind(&file_name)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(|e| format!("创建记录失败: {}", e))?;
    
    let msg_id = result.last_insert_rowid();
    println!("[DB] 上传记录已创建，ID: {}", msg_id);
    Ok(msg_id)
}

/// 更新上传状态
pub async fn update_upload_status(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    file_name: String,
    status: String,
) -> Result<(), String> {
    println!("[DB] 更新上传状态: {} -> {}", file_name, status);
    
    sqlx::query(
        "UPDATE messages SET file_status = ? WHERE sender_id = 'me' AND content = ? AND file_status = 'uploading'"
    )
    .bind(&status)
    .bind(&file_name)
    .execute(pool)
    .await
    .map_err(|e| format!("更新状态失败: {}", e))?;
    
    println!("[DB] 上传状态已更新");
    Ok(())
}

/// 删除上传记录（上传失败时）
pub async fn delete_upload_record(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    file_name: String,
    timestamp: i64,
) -> Result<(), String> {
    println!("[DB] 删除上传记录: {}", file_name);
    
    sqlx::query(
        "DELETE FROM messages WHERE sender_id = 'me' AND content = ? AND timestamp = ? AND file_status = 'uploading'"
    )
    .bind(&file_name)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(|e| format!("删除记录失败: {}", e))?;
    
    println!("[DB] 上传记录已删除");
    Ok(())
}

/// 更新文件状态（通过消息ID）
pub async fn update_file_status_by_id(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    msg_id: i64,
    new_status: &str,
) -> Result<(), String> {
    println!("[DB] 更新文件状态（ID: {}）: -> {}", msg_id, new_status);
    
    sqlx::query(
        "UPDATE messages SET file_status = ? WHERE id = ?"
    )
    .bind(new_status)
    .bind(msg_id)
    .execute(pool)
    .await
    .map_err(|e| format!("更新文件状态失败: {}", e))?;
    
    println!("[DB] 文件状态已更新");
    Ok(())
}

/// 删除消息（通过消息ID）
pub async fn delete_message_by_id(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    msg_id: i64,
) -> Result<(), String> {
    println!("[DB] 删除消息: ID {}", msg_id);
    
    sqlx::query(
        "DELETE FROM messages WHERE id = ?"
    )
    .bind(msg_id)
    .execute(pool)
    .await
    .map_err(|e| format!("删除消息失败: {}", e))?;
    
    println!("[DB] 消息已删除");
    Ok(())
}

/// 保存文本消息
pub async fn save_text_message(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    receiver_id: String,
    content: String,
) -> Result<(), String> {
    println!("[DB] 保存文本消息: 接收者={}, 内容长度={}", receiver_id, content.len());
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    sqlx::query(
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp) VALUES ('me', ?, ?, 'text', ?)"
    )
    .bind(&receiver_id)
    .bind(&content)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(|e| format!("保存消息失败: {}", e))?;
    
    println!("[DB] 文本消息已保存");
    Ok(())
}

/// 保存当前主题
pub async fn save_current_theme(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    theme_name: String,
) -> Result<(), String> {
    println!("[DB] 保存当前主题: {}", theme_name);
    
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('current_theme', ?)")
        .bind(&theme_name)
        .execute(pool)
        .await
        .map_err(|e| format!("保存主题失败: {}", e))?;
    
    println!("[DB] 主题已保存");
    Ok(())
}

/// 获取当前主题
pub async fn get_current_theme(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Option<String>, String> {
    println!("[DB] 获取当前主题");
    
    let result = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = 'current_theme'")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询主题失败: {}", e))?;
    
    if let Some(theme) = &result {
        println!("[DB] 当前主题: {}", theme);
    } else {
        println!("[DB] 未设置主题，使用默认值");
    }
    
    Ok(result)
}

/// 保存接收到的文本消息（来自其他对等体）
pub async fn save_received_text_message(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    sender_id: String,
    content: String,
    msg_type: String,
    timestamp: i64,
) -> Result<(), String> {
    println!("[DB] 保存接收到的文本消息: 发送者={}, 内容长度={}", sender_id, content.len());
    
    sqlx::query(
        "INSERT INTO messages (sender_id, content, msg_type, timestamp) VALUES (?, ?, ?, ?)"
    )
    .bind(&sender_id)
    .bind(&content)
    .bind(&msg_type)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(|e| format!("保存消息失败: {}", e))?;
    
    println!("[DB] 接收到的消息已保存");
    Ok(())
}

/// 更新文件状态（通过文件路径）
pub async fn update_file_status_by_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    old_path: &str,
    new_path: &str,
    new_status: &str,
) -> Result<(), String> {
    println!("[DB] 更新文件状态（路径）: {} -> {}, 状态: {}", old_path, new_path, new_status);
    
    sqlx::query(
        "UPDATE messages SET file_status = ?, file_path = ? WHERE file_path = ?"
    )
    .bind(new_status)
    .bind(new_path)
    .bind(old_path)
    .execute(pool)
    .await
    .map_err(|e| format!("更新文件状态失败: {}", e))?;
    
    println!("[DB] 文件状态已更新");
    Ok(())
}

/// 获取所有文件消息（用于调试）
pub async fn get_all_file_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    limit: i64,
) -> Result<Vec<(i64, String, String, String, String)>, String> {
    println!("[DB] 获取所有文件消息（限制: {}）", limit);
    
    let rows = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT id, sender_id, content, file_path, file_status FROM messages WHERE msg_type = 'file' ORDER BY id DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("查询文件消息失败: {}", e))?;
    
    println!("[DB] 找到 {} 条文件消息", rows.len());
    Ok(rows)
}

/// 查询待接收的文件（通过文件路径模糊匹配）
pub async fn get_pending_file_by_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    path_pattern: &str,
) -> Result<Option<(String, String)>, String> {
    println!("[DB] 查询待接收文件: 路径模式={}", path_pattern);
    
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT file_path, content FROM messages WHERE file_path LIKE ? AND file_status = 'pending'"
    )
    .bind(path_pattern)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("查询文件失败: {}", e))?;
    
    if let Some((path, name)) = &row {
        println!("[DB] 找到待接收文件: {} ({})", name, path);
    } else {
        println!("[DB] 未找到待接收文件");
    }
    
    Ok(row)
}

/// 创建接收文件记录（接收来自其他对等体的文件）
pub async fn create_received_file_record(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    sender_id: String,
    file_name: String,
    file_path: String,
    timestamp: i64,
) -> Result<i64, String> {
    println!("[DB] 创建接收文件记录: 发送者={}, 文件={}", sender_id, file_name);
    
    let result = sqlx::query(
        "INSERT INTO messages (sender_id, receiver_id, content, msg_type, timestamp, file_path, file_status) VALUES (?, 'me', ?, 'file', ?, ?, 'downloading')"
    )
    .bind(&sender_id)
    .bind(&file_name)
    .bind(timestamp)
    .bind(&file_path)
    .execute(pool)
    .await
    .map_err(|e| format!("创建记录失败: {}", e))?;
    
    let msg_id = result.last_insert_rowid();
    println!("[DB] 接收文件记录已创建，ID: {}", msg_id);
    Ok(msg_id)
}
