# 简化文件上传修复方案

## 问题根源

Axum 的 multipart 解析器在处理大文件时有问题，特别是：
1. 默认body限制太小（2MB）
2. multipart 流解析容易出错

## 解决方案

在 `src-tauri/src/web_server.rs` 的 `start_server` 函数中，在 `.layer(cors)` 后面添加：

```rust
.layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024))
```

完整的修改位置（第63-75行左右）：

```rust
let app = Router::new()
    .route("/", get(serve_index))
    .route("/*path", get(serve_assets))
    .route("/api/get_my_name", get(get_name_http))
    .route("/api/update_my_name", post(update_name_http))
    .route("/api/get_peers", get(get_peers_http))
    .route("/api/send_message", post(send_message_http))
    .route("/api/chat_history/:peer_id", get(get_chat_history_http))
    .route("/api/upload", post(upload_file_http))
    .route("/api/download/:file_id", get(download_file_http))
    .route("/ws", get(websocket_handler))
    .layer(cors)
    .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024))  // 添加这一行
    .with_state(state);
```

## 手动修改步骤

1. 打开 `src-tauri/src/web_server.rs`
2. 找到 `start_server` 函数
3. 在 `.layer(cors)` 后面添加 `.layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024))`
4. 保存文件
5. 重新编译：
   ```bash
   cd src-tauri
   cargo build --bin lanchat-web --features web --no-default-features
   cargo build --bin lanchat --features desktop
   ```

## 为什么这么复杂？

Python 的 `http.server` 只是一个简单的静态文件服务器，不处理：
- WebSocket 连接
- 文件上传
- 数据库操作
- 局域网发现
- 消息传输

我们的应用需要所有这些功能，所以比简单的文件服务器复杂得多。

## 替代方案：使用更简单的库

如果你想要更简单的实现，可以考虑：
1. 使用 `warp` 而不是 `axum`（更简单但功能少）
2. 使用 `actix-web`（更成熟但学习曲线陡）
3. 直接使用 `hyper`（最底层，完全控制）

但这些都需要重写大部分代码。
