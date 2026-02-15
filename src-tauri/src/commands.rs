use crate::db::DbState;
use tauri::State;

#[tauri::command]
pub async fn get_my_name(state: State<'_, DbState>) -> Result<String, String> {
    println!("[Command] 收到前端请求: get_my_name");
    crate::db::get_username(&state.pool).await
}

#[tauri::command]
pub async fn update_my_name(state: State<'_, DbState>, new_name: String) -> Result<String, String> {
    println!("[Command] 收到前端请求: update_my_name, 新名字: {}", new_name);
    
    // 更新数据库
    crate::db::update_username(&state.pool, new_name.clone()).await?;
    
    // 返回更新后的名字
    Ok(new_name)
}
