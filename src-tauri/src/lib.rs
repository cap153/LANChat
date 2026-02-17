// lib.rs
#[cfg(feature = "desktop")]
pub mod commands;

pub mod db;
pub mod models;
pub mod network;
pub mod peers;
pub mod utils;
pub mod web_server;

// 仅在桌面端编译时包含 Tauri 运行函数
#[cfg(feature = "desktop")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;
    use std::sync::Arc;
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_my_name,
            commands::get_my_id,
            commands::update_my_name,
            commands::get_peers,
            commands::send_message,
            commands::get_chat_history,
            commands::send_file,
            commands::get_settings,
            commands::update_settings,
            commands::get_theme_list,
            commands::get_theme_css,
            commands::save_current_theme,
            commands::get_current_theme,
            commands::get_default_download_path,
            commands::request_storage_permission,
            commands::save_file_message
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let port = 8888;

            tauri::async_runtime::block_on(async move {
                println!("[Lib] 正在初始化数据库...");
                let pool = db::init_db(&handle).await.expect("DB error");
                let my_name = db::get_username(&pool)
                    .await
                    .unwrap_or_else(|_| "Unknown".into());
                
                let my_id = db::get_user_id(&pool)
                    .await
                    .expect("无法获取或生成用户 ID");

                handle.manage(db::DbState { pool: pool.clone() });
                println!("[Lib] 我的用户名: {}", my_name);
                println!("[Lib] 我的 ID: {}", my_id);

                // 创建全局用户管理器
                let peer_manager = Arc::new(peers::PeerManager::new());
                
                // 将 PeerManager 注册到 Tauri 状态管理
                handle.manage(commands::PeerState {
                    manager: peer_manager.clone(),
                });

                let h1 = handle.clone();
                let id1 = my_id.clone();
                let name1 = my_name.clone();
                let peer_manager_clone = peer_manager.clone();
                tokio::spawn(async move {
                    println!("[Lib] 开启监听线程...");
                    network::discovery::start_listening(port, id1, name1, Some(h1), peer_manager_clone).await;
                });

                let id2 = my_id.clone();
                let pool2 = pool.clone();
                tokio::spawn(async move {
                    println!("[Lib] 开启广播线程...");
                    network::discovery::start_announcing(port, id2, pool2).await;
                });

                // 启动 HTTP 服务器（用于接收文件和 WebSocket 消息）
                let pool_clone = pool.clone();
                let peer_manager_clone = peer_manager.clone();
                let handle_clone = handle.clone();
                tokio::spawn(async move {
                    println!("[Lib] 启动 HTTP 服务器在端口 {}...", port);
                    web_server::start_server(port, port, pool_clone, peer_manager_clone, Some(handle_clone)).await;
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
