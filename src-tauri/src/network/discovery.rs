use std::net::UdpSocket;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// 统一端口广播
pub async fn start_announcing(port: u16, username: String) {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap(); // 发送端
    socket.set_broadcast(true).unwrap();

    let msg = format!("LANChat|ONLINE|{}|{}", username, port);
    let broadcast_addr = format!("255.255.255.255:{}", port);

    loop {
        let _ = socket.send_to(msg.as_bytes(), &broadcast_addr);
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

pub async fn start_listening(port: u16, my_name: String, app: Option<AppHandle>) {
    let socket = std::net::UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();
    let mut buf = [0u8; 1024];
    println!("[UDP] 正在端口 {} 监听邻居...", port);

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..size]);
            let parts: Vec<&str> = msg.split('|').collect();
            if parts.len() >= 4 && parts[0] == "LANChat" {
                let name = parts[2].to_string();
                if name == my_name {
                    continue;
                }

                println!("[UDP] 发现新邻居: {} @ {}:{}", name, addr.ip(), parts[3]);

                // 只有当 app 存在时（桌面端）才发送事件
                if let Some(app_handle) = &app {
                    let _ = app_handle.emit(
                        "new-peer",
                        serde_json::json!({
                            "name": name,
                            "addr": format!("{}:{}", addr.ip(), parts[3])
                        }),
                    );
                }
            }
        }
    }
}
