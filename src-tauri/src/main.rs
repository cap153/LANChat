// src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lanchat_lib::db;
use lanchat_lib::peers::PeerManager;
use std::sync::Arc;
use tauri::{Manager, menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent}};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        // --- 重点：添加下面这段代码 ---
        .invoke_handler(tauri::generate_handler![
            lanchat_lib::commands::get_my_name,
            lanchat_lib::commands::get_my_id,
            lanchat_lib::commands::update_my_name,
            lanchat_lib::commands::get_peers,
            lanchat_lib::commands::send_message,
            lanchat_lib::commands::get_chat_history,
            lanchat_lib::commands::send_file,
            lanchat_lib::commands::get_settings,
            lanchat_lib::commands::update_settings,
            lanchat_lib::commands::get_theme_list,
            lanchat_lib::commands::get_theme_css,
            lanchat_lib::commands::save_current_theme,
            lanchat_lib::commands::get_current_theme,
            lanchat_lib::commands::get_default_download_path,
            lanchat_lib::commands::request_storage_permission,
            lanchat_lib::commands::save_file_message
        ])
        // --------------------------
        .setup(|app| {
            let handle = app.handle().clone();
            let port = 8888;

            // 获取主窗口并设置关闭事件处理
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // 阻止默认关闭行为
                        api.prevent_close();
                        // 隐藏窗口而不是关闭
                        let _ = window_clone.hide();
                    }
                });
            }

            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("LANChat")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == tauri::tray::MouseButton::Left 
                            && button_state == tauri::tray::MouseButtonState::Up {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

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
                handle.manage(lanchat_lib::commands::PeerState {
                    manager: peer_manager.clone(),
                });

                let h1 = handle.clone();
                let id1 = my_id.clone();
                let name1 = my_name.clone();
                let peer_manager_clone = peer_manager.clone();
                tokio::spawn(async move {
                    println!("[Main] 开启监听线程...");
                    lanchat_lib::network::discovery::start_listening(port, id1, name1, Some(h1), peer_manager_clone).await;
                });

                let id2 = my_id.clone();
                let pool2 = pool.clone();
                tokio::spawn(async move {
                    println!("[Main] 开启广播线程...");
                    lanchat_lib::network::discovery::start_announcing(port, id2, pool2).await;
                });

                // 桌面端也启动 HTTP 服务器（用于接收文件和 WebSocket 消息）
                let pool_clone = pool.clone();
                let peer_manager_clone = peer_manager.clone();
                let handle_clone = handle.clone();
                tokio::spawn(async move {
                    println!("[Main] 启动 HTTP 服务器在端口 {}...", port);
                    lanchat_lib::web_server::start_server(port, port, pool_clone, peer_manager_clone, Some(handle_clone)).await;
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
