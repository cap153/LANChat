use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "desktop")]
use tauri::{AppHandle, Emitter};

use crate::peers::PeerManager;

const MULTICAST_IP: &str = "224.0.0.167";

// 创建支持广播和组播的 UDP socket
fn create_discovery_socket(
    bind_addr: &str,
    is_listener: bool,
) -> Result<UdpSocket, std::io::Error> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

    #[cfg(target_os = "windows")]
    socket.set_reuse_address(true)?;

    #[cfg(not(target_os = "windows"))]
    {
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;
    }

    let addr: std::net::SocketAddr = bind_addr
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    socket.bind(&addr.into())?;

    let std_socket: UdpSocket = socket.into();

    if is_listener {
        let multi_addr: Ipv4Addr = MULTICAST_IP.parse().unwrap();
        let interface: Ipv4Addr = "0.0.0.0".parse().unwrap();
        let _ = std_socket.join_multicast_v4(&multi_addr, &interface);
    } else {
        std_socket.set_broadcast(true)?;
        let _ = std_socket.set_multicast_ttl_v4(1);
    }

    Ok(std_socket)
}

// 核心黑科技：生成全网段广播地址（绕过 Android 网卡读取限制）
fn get_smart_broadcast_addresses(port: u16) -> Vec<String> {
    let mut addrs = Vec::with_capacity(260);

    // 1. 全局受限广播 (应对普通路由器)
    addrs.push(format!("255.255.255.255:{}", port));
    // 2. 组播 (PC端互联完美生效)
    addrs.push(format!("{}:{}", MULTICAST_IP, port));
    // 3. 苹果 iOS 热点固定广播地址
    addrs.push(format!("172.20.10.15:{}", port));
    // 4. 常见企业路由器网段
    addrs.push(format!("10.0.0.255:{}", port));

    // 5. Android 随机热点网段 "暴力"覆盖 (192.168.0.255 ~ 192.168.255.255)
    // 那些没有对应网卡的地址会在微秒级被内核路由表直接丢弃，不会产生网络风暴
    for i in 0..=255 {
        addrs.push(format!("192.168.{}.255:{}", i, port));
    }

    addrs
}

pub async fn start_announcing(port: u16, user_id: String, pool: sqlx::Pool<sqlx::Sqlite>) {
    let socket = match create_discovery_socket("0.0.0.0:0", false) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[UDP] 创建发送 socket 失败: {}", e);
            return;
        }
    };

    println!("[UDP] 开始通过智能路由遍历发送心跳...");

    use sysinfo::System;
    let mut sys = System::new();
    let target_addrs = get_smart_broadcast_addresses(port);

    loop {
        let username = match crate::db::get_username(&pool).await {
            Ok(name) => name,
            Err(_) => "Unknown".to_string(),
        };

        sys.refresh_memory();
        let available_memory_mb = sys.available_memory() / (1024 * 1024);

        let msg = format!(
            "LANChat|ONLINE|{}|{}|{}|{}",
            user_id, username, port, available_memory_mb
        );

        // 核心：遍历所有可能地址，仅路由存在的网卡能发送成功
        for addr in &target_addrs {
            let _ = socket.send_to(msg.as_bytes(), addr);
        }

        // 成功数量通常是 2~4 个（组播 + 全局 + 刚好撞中的你的 84 热点网段等）
        // println!("[UDP] 心跳发送成功，激活了 {} 个真实路由网段", success_count);

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// 桌面端版本 - 带 AppHandle
#[cfg(all(feature = "desktop", not(feature = "web")))]
pub async fn start_listening(
    port: u16,
    my_id: String,
    _my_name: String,
    app: Option<AppHandle>,
    peer_manager: Arc<PeerManager>,
) {
    let bind_addr = format!("0.0.0.0:{}", port);
    let socket = match create_discovery_socket(&bind_addr, true) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[UDP] 创建监听 socket 失败: {}", e);
            return;
        }
    };

    let mut buf = [0u8; 1024];
    println!("[UDP] 正在端口 {} 监听邻居...", port);

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..size]);
            let parts: Vec<&str> = msg.split('|').collect();

            if parts.len() >= 6 && parts[0] == "LANChat" {
                let peer_id = parts[2].to_string();
                let name = parts[3].to_string();
                let peer_port = parts[4];
                let available_memory_mb: u64 = parts[5].parse().unwrap_or(0);

                if peer_id == my_id {
                    continue;
                }

                let peer_addr = format!("{}:{}", addr.ip(), peer_port);
                
                let is_new_or_reconnected = peer_manager.add_or_update_with_memory(
                    peer_id.clone(),
                    name.clone(),
                    peer_addr.clone(),
                    available_memory_mb,
                );

                // 只在新用户或重新上线时打印日志
                if is_new_or_reconnected {
                    println!(
                        "[UDP] 发现用户: {} ({}) at {} (可用内存: {} MB)",
                        name, peer_id, peer_addr, available_memory_mb
                    );
                }

                if let Some(app_handle) = &app {
                    let _ = app_handle.emit("new-peer", serde_json::json!({
                        "id": peer_id, "name": name, "addr": peer_addr, "available_memory_mb": available_memory_mb
                    }));
                }
            }
        }
    }
}

// Web 端版本 - 不带 AppHandle
#[cfg(all(feature = "web", not(feature = "desktop")))]
pub async fn start_listening(
    port: u16,
    my_id: String,
    _my_name: String,
    peer_manager: Arc<PeerManager>,
) {
    let bind_addr = format!("0.0.0.0:{}", port);
    let socket = match create_discovery_socket(&bind_addr, true) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[UDP] Web端创建监听 socket 失败: {}", e);
            return;
        }
    };

    let mut buf = [0u8; 1024];
    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..size]);
            let parts: Vec<&str> = msg.split('|').collect();

            if parts.len() >= 6 && parts[0] == "LANChat" {
                let peer_id = parts[2].to_string();
                let name = parts[3].to_string();
                let peer_port = parts[4];
                let available_memory_mb: u64 = parts[5].parse().unwrap_or(0);

                if peer_id == my_id {
                    continue;
                }
                let peer_addr = format!("{}:{}", addr.ip(), peer_port);
                peer_manager.add_or_update_with_memory(
                    peer_id,
                    name,
                    peer_addr,
                    available_memory_mb,
                );
            }
        }
    }
}

// 发送单次广播
pub async fn send_single_broadcast(
    port: u16,
    user_id: String,
    username: String,
) -> Result<(), String> {
    let socket = create_discovery_socket("0.0.0.0:0", false)
        .map_err(|e| format!("创建发送socket失败: {}", e))?;

    let msg = format!("LANChat|ONLINE|{}|{}|{}|0", user_id, username, port);
    let target_addrs = get_smart_broadcast_addresses(port);

    for addr in target_addrs {
        let _ = socket.send_to(msg.as_bytes(), &addr);
    }

    Ok(())
}
