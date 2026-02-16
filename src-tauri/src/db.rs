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
            let default_path = std::env::temp_dir().join("lanchat_downloads");
            Ok(default_path.to_str().unwrap().to_string())
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

        // 初始保存路径（Web 端使用临时目录）
        let download_dir = std::env::temp_dir().join("lanchat_downloads");
        sqlx::query("INSERT INTO settings (key, value) VALUES ('download_path', ?)")
            .bind(download_dir.to_str().unwrap())
            .execute(&pool)
            .await?;
    }

    Ok(pool)
}
