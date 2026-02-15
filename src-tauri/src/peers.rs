// 在线用户管理模块
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,       // UUID
    pub name: String,
    pub addr: String,
    pub last_seen: u64,   // Unix 时间戳
    pub is_offline: bool, // 是否离线
}

// 全局在线用户列表
pub struct PeerManager {
    peers: Arc<RwLock<HashMap<String, Peer>>>, // key 是 UUID
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 添加或更新用户
    pub fn add_or_update(&self, id: String, name: String, addr: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut peers = self.peers.write().unwrap();
        
        if let Some(peer) = peers.get_mut(&id) {
            // 已存在,更新信息
            let was_offline = peer.is_offline;
            peer.name = name;
            peer.addr = addr;
            peer.last_seen = now;
            peer.is_offline = false;
            
            if was_offline {
                println!("[PeerManager] 用户重新上线: {} ({})", peer.name, peer.id);
            } else {
                println!("[PeerManager] 更新用户: {} ({})", peer.name, peer.id);
            }
        } else {
            // 新用户
            let peer = Peer {
                id: id.clone(),
                name: name.clone(),
                addr,
                last_seen: now,
                is_offline: false,
            };
            println!("[PeerManager] 添加新用户: {} ({})", name, id);
            peers.insert(id, peer);
        }
    }

    // 标记所有用户为"待确认"状态,然后检查哪些用户离线
    pub fn mark_stale_as_offline(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut peers = self.peers.write().unwrap();
        
        // 标记超过 6 秒未见的用户为离线
        for peer in peers.values_mut() {
            let time_since_seen = now - peer.last_seen;
            if time_since_seen > 6 && !peer.is_offline {
                println!("[PeerManager] 用户离线: {} ({}) - {}秒未见", peer.name, peer.id, time_since_seen);
                peer.is_offline = true;
            }
        }
        
        // 删除超过 60 秒的用户
        peers.retain(|id, peer| {
            let keep = now - peer.last_seen < 60;
            if !keep {
                println!("[PeerManager] 移除用户: {} ({})", peer.name, id);
            }
            keep
        });
    }

    // 获取所有用户（包括离线的）
    pub fn get_all_peers(&self) -> Vec<Peer> {
        // 先标记离线用户
        self.mark_stale_as_offline();
        
        let peers = self.peers.read().unwrap();
        peers.values().cloned().collect()
    }

    // 获取所有在线用户（过滤掉离线的）
    pub fn get_active_peers(&self) -> Vec<Peer> {
        let peers = self.peers.read().unwrap();
        peers
            .values()
            .filter(|p| !p.is_offline)
            .cloned()
            .collect()
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}
