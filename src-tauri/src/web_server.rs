use axum::{
    body::Body,
    extract::{Json, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

#[derive(RustEmbed)]
#[folder = "../src/"]
struct Asset;

#[derive(Serialize)]
struct NameResponse {
    name: String,
}

#[derive(Deserialize)]
struct UpdateNameRequest {
    name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Web 服务器的状态
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
}

pub async fn start_server(port: u16, _udp_port: u16, pool: Pool<Sqlite>) {
    let state = Arc::new(AppState { pool });
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/*path", get(serve_assets))
        .route("/api/get_my_name", get(get_name_http))
        .route("/api/update_my_name", post(update_name_http))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("[Web Server] 启动在端口 {}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn get_name_http(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    println!("[Web Server] 收到获取用户名请求");
    
    match crate::db::get_username(&state.pool).await {
        Ok(name) => Json(NameResponse { name }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("读取用户名失败: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn update_name_http(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateNameRequest>,
) -> impl IntoResponse {
    println!("[Web Server] 收到改名请求: {}", payload.name);
    
    // 使用数据库的更新函数（包含验证逻辑）
    match crate::db::update_username(&state.pool, payload.name.clone()).await {
        Ok(_) => Json(NameResponse {
            name: payload.name,
        })
        .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
            .into_response(),
    }
}

async fn serve_index() -> impl IntoResponse {
    serve_assets(axum::extract::Path("index.html".to_string())).await
}

async fn serve_assets(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404"))
            .unwrap(),
    }
}
