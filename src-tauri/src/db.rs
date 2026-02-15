use crate::utils::generate_random_name;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::path::PathBuf;
use tauri::AppHandle;
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

// 为 Tauri 桌面端初始化数据库
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
            content TEXT,
            msg_type TEXT,
            timestamp INTEGER,
            file_path TEXT,
            file_status TEXT
        )",
    )
    .execute(&pool)
    .await?;

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
        
        sqlx::query("INSERT INTO settings (key, value) VALUES ('username', ?)")
            .bind(random_name)
            .execute(&pool)
            .await?;

        // 初始保存路径（Web 端使用临时目录）
        let download_dir = std::env::temp_dir().join("lanchat_downloads");
        sqlx::query("INSERT INTO settings (key, value) VALUES ('download_path', ?)")
            .bind(download_dir.to_str().unwrap())
            .execute(&pool)
            .await?;

        sqlx::query("INSERT INTO settings (key, value) VALUES ('auto_accept', 'false')")
            .execute(&pool)
            .await?;
    }

    Ok(pool)
}
