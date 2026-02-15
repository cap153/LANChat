// src/main.rs

use lanchat::db;
use lanchat::peers::PeerManager;
use std::sync::Arc;
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
            lanchat::commands::get_my_id,
            lanchat::commands::update_my_name,
            lanchat::commands::get_peers,
            lanchat::commands::send_message,
            lanchat::commands::get_chat_history,
            lanchat::commands::send_file
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
                
                let my_id = db::get_user_id(&pool)
                    .await
                    .expect("无法获取或生成用户 ID");

                handle.manage(db::DbState { pool: pool.clone() });
                println!("[Main] 我的用户名: {}", my_name);
                println!("[Main] 我的 ID: {}", my_id);

                // 创建全局用户管理器
                let peer_manager = Arc::new(PeerManager::new());
                
                // 将 PeerManager 注册到 Tauri 状态管理
                handle.manage(lanchat::commands::PeerState {
                    manager: peer_manager.clone(),
                });

                let h1 = handle.clone();
                let id1 = my_id.clone();
                let name1 = my_name.clone();
                let peer_manager_clone = peer_manager.clone();
                tokio::spawn(async move {
                    println!("[Main] 开启监听线程...");
                    lanchat::network::discovery::start_listening(port, id1, name1, Some(h1), peer_manager_clone).await;
                });

                let id2 = my_id.clone();
                let name2 = my_name.clone();
                tokio::spawn(async move {
                    println!("[Main] 开启广播线程...");
                    lanchat::network::discovery::start_announcing(port, id2, name2).await;
                });

                // 桌面端也启动 HTTP 服务器（用于接收文件和 WebSocket 消息）
                let pool_clone = pool.clone();
                let peer_manager_clone = peer_manager.clone();
                let handle_clone = handle.clone();
                tokio::spawn(async move {
                    println!("[Main] 启动 HTTP 服务器在端口 {}...", port);
                    lanchat::web_server::start_server(port, port, pool_clone, peer_manager_clone, Some(handle_clone)).await;
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
