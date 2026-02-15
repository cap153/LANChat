// commands.rs - 仅用于桌面端的 Tauri 命令
#[cfg(feature = "desktop")]
use crate::db::DbState;
#[cfg(feature = "desktop")]
use crate::peers::{Peer, PeerManager};
#[cfg(feature = "desktop")]
use std::sync::Arc;
#[cfg(feature = "desktop")]
use tauri::State;

// 用于管理 PeerManager 的状态
#[cfg(feature = "desktop")]
pub struct PeerState {
    pub manager: Arc<PeerManager>,
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn get_my_name(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_my_name");
    crate::db::get_username(&state.pool).await
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn update_my_name(state: State<'_, DbState>, new_name: String) -> Result<String, String> {
    println!("[Command] 收到前端请求: update_my_name, 新名字: {}", new_name);
    
    // 更新数据库
    crate::db::update_username(&state.pool, new_name.clone()).await?;
    
    // 返回更新后的名字
    Ok(new_name)
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub async fn get_peers(state: State<'_, PeerState>) -> Result<Vec<Peer>, String> {
    Ok(state.manager.get_all_peers())
}
