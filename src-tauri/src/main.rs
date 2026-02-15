// src/main.rs

use lanchat::db;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        // --- 重点：添加下面这段代码 ---
        .invoke_handler(tauri::generate_handler![
            lanchat::commands::get_my_name,
            lanchat::commands::update_my_name
        ])
        // --------------------------
        .setup(|app| {
            let handle = app.handle().clone();
            let port = 8888;

            tauri::async_runtime::block_on(async move {
                println!("[Main] 正在初始化数据库...");
                let pool = db::init_db(&handle).await.expect("DB error");
                let my_name = db::get_username(&pool)
                    .await
                    .unwrap_or_else(|_| "Unknown".into());

                handle.manage(db::DbState { pool });
                println!("[Main] 我的用户名: {}", my_name);

                let h1 = handle.clone();
                let name1 = my_name.clone();
                tokio::spawn(async move {
                    println!("[Main] 开启监听线程...");
                    lanchat::network::discovery::start_listening(port, name1, Some(h1)).await;
                });

                let name2 = my_name.clone();
                tokio::spawn(async move {
                    println!("[Main] 开启广播线程...");
                    lanchat::network::discovery::start_announcing(port, name2).await;
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
