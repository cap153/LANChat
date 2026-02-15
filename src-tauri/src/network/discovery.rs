use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "desktop")]
use tauri::{AppHandle, Emitter};

use crate::peers::PeerManager;

// 统一端口广播
pub async fn start_announcing(port: u16, user_id: String, username: String) {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap(); // 发送端
    socket.set_broadcast(true).unwrap();

    let msg = format!("LANChat|ONLINE|{}|{}|{}", user_id, username, port);
    let broadcast_addr = format!("255.255.255.255:{}", port);

    loop {
        let _ = socket.send_to(msg.as_bytes(), &broadcast_addr);
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[cfg(feature = "desktop")]
pub async fn start_listening(port: u16, my_id: String, _my_name: String, app: Option<AppHandle>, peer_manager: Arc<PeerManager>) {
    let socket = std::net::UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();
    let mut buf = [0u8; 1024];
    println!("[UDP] 正在端口 {} 监听邻居...", port);

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..size]);
            let parts: Vec<&str> = msg.split('|').collect();
            
            // 新协议: LANChat|ONLINE|UUID|用户名|端口
            if parts.len() >= 5 && parts[0] == "LANChat" {
                let peer_id = parts[2].to_string();
                let name = parts[3].to_string();
                let peer_port = parts[4];
                
                // 忽略自己
                if peer_id == my_id {
                    continue;
                }

                let peer_addr = format!("{}:{}", addr.ip(), peer_port);

                // 添加到全局用户列表
                peer_manager.add_or_update(peer_id.clone(), name.clone(), peer_addr.clone());

                // 只有当 app 存在时（桌面端）才发送事件
                if let Some(app_handle) = &app {
                    let _ = app_handle.emit(
                        "new-peer",
                        serde_json::json!({
                            "id": peer_id,
                            "name": name,
                            "addr": peer_addr
                        }),
                    );
                }
            }
        }
    }
}

#[cfg(feature = "web")]
pub async fn start_listening(port: u16, my_id: String, _my_name: String, peer_manager: Arc<PeerManager>) {
    let socket = std::net::UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();
    let mut buf = [0u8; 1024];
    println!("[UDP] 正在端口 {} 监听邻居...", port);

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..size]);
            let parts: Vec<&str> = msg.split('|').collect();
            
            // 新协议: LANChat|ONLINE|UUID|用户名|端口
            if parts.len() >= 5 && parts[0] == "LANChat" {
                let peer_id = parts[2].to_string();
                let name = parts[3].to_string();
                let peer_port = parts[4];
                
                // 忽略自己
                if peer_id == my_id {
                    continue;
                }

                let peer_addr = format!("{}:{}", addr.ip(), peer_port);
                
                // 添加到全局用户列表
                peer_manager.add_or_update(peer_id, name, peer_addr);
            }
        }
    }
}
