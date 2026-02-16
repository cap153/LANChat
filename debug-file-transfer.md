# 文件传输调试指南

## 问题描述
从 Web 端往桌面端上传文件，设置了自动接收：
- ❌ Web 端没有上传的消息
- ❌ 桌面端没有下载的消息
- ✅ 桌面端后台有接收成功的日志

## 调试步骤

### 1. 检查 Web 端发送者

**打开浏览器 Console**，发送文件时应该看到：
```
[UI] Web 端发送文件: filename.ext, size
[UI] 创建上传记录...
[UI] 文件发送成功
```

**检查网络请求**：
- `/api/create_upload_record` - 应该返回 200
- `http://peer_addr/api/upload` - 应该返回 200

**检查数据库**：
```bash
sqlite3 ~/.local/share/com.lanchat.app/lanchat.db
SELECT * FROM messages WHERE sender_id = 'me' AND msg_type = 'file' ORDER BY timestamp DESC LIMIT 5;
```

应该看到：
- `file_status = 'uploading'` (上传中)
- 然后更新为 `file_status = 'sent'` (已发送)

### 2. 检查桌面端接收者

**打开桌面端 Console**，应该看到：
```
[WebSocket] 收到文本消息: ...
或
[Web Server] 收到文件上传请求
[Web Server] ✓ 已创建下载中记录，ID: X
[Web Server] 已发送 Tauri 事件: new-message
```

**检查是否收到 Tauri 事件**：
在 `app.js` 中添加日志：
```javascript
await apiListen('new-message', (event) => {
    console.log("[JS-App] ========== 收到 new-message 事件 ==========");
    console.log("[JS-App] 事件内容:", JSON.stringify(event.payload, null, 2));
    console.log("[JS-App] 当前聊天对象:", window.currentChatPeer);
    onReceiveMessage(event.payload);
});
```

### 3. 可能的问题

#### 问题 A: Web 端没有创建上传记录
**原因**：`/api/create_upload_record` 请求失败

**解决**：
1. 检查浏览器 Console 是否有错误
2. 检查 Web 服务器日志是否有 "创建上传记录" 的日志
3. 确认 API 路由已正确注册

#### 问题 B: 桌面端没有收到 Tauri 事件
**原因**：
1. Tauri 事件没有正确发送
2. 事件监听器没有正确注册
3. `from_id` 不匹配当前聊天对象

**解决**：
1. 检查 `web_server.rs` 中是否有 "已发送 Tauri 事件" 的日志
2. 检查 `app.js` 中是否有 "收到 new-message 事件" 的日志
3. 检查 `from_id` 是否匹配

#### 问题 C: 消息轮询没有工作
**原因**：轮询逻辑可能有问题

**解决**：
在 `app.js` 的 `startMessagePolling` 中添加日志：
```javascript
const checkNewMessages = async () => {
    console.log('[JS-App] 轮询检查新消息...');
    if (!window.currentChatPeer) {
        console.log('[JS-App] 没有打开聊天窗口，跳过');
        return;
    }
    // ...
};
```

## 快速测试

### 测试 1: Web 端上传记录 API
```bash
# 启动 Web 端
./src-tauri/target/debug/lanchat-web

# 在另一个终端测试 API
curl -X POST http://localhost:8888/api/create_upload_record \
  -H "Content-Type: application/json" \
  -d '{"file_name":"test.txt","timestamp":1234567890}'

# 应该返回: {"success":true}
```

### 测试 2: 检查数据库
```bash
# Web 端数据库
sqlite3 ~/.local/share/com.lanchat.app/lanchat.db "SELECT * FROM messages WHERE msg_type='file' ORDER BY timestamp DESC LIMIT 10;"

# 桌面端数据库（如果不同）
sqlite3 ~/.local/share/com.lanchat.app/lanchat.db "SELECT * FROM messages WHERE msg_type='file' ORDER BY timestamp DESC LIMIT 10;"
```

### 测试 3: 检查 Tauri 事件
在桌面端的 `main.rs` 中添加日志：
```rust
println!("[Main] Tauri 应用已启动");
```

## 预期的完整流程

### Web 端发送文件
1. 用户选择文件
2. `sendFile()` 被调用
3. 调用 `/api/create_upload_record` 创建 `uploading` 记录
4. 前端显示"上传中..."消息（橙色闪烁）
5. 调用 `apiSendFile()` 上传文件到对方
6. 上传成功后，调用 `/api/update_upload_status` 更新为 `sent`
7. 刷新聊天历史，显示"已发送"状态

### 桌面端接收文件
1. Web 服务器收到 `/api/upload` 请求
2. 立即创建 `downloading` 记录
3. 发送 Tauri 事件 `new-message`
4. 前端收到事件，调用 `onReceiveMessage()`
5. 显示"下载中..."消息（蓝色闪烁）
6. 文件下载完成，更新为 `accepted`
7. 再次发送 Tauri 事件
8. 前端刷新聊天历史，显示"finish"状态

## 下一步

如果问题依然存在，请提供：
1. Web 端浏览器 Console 的完整日志
2. 桌面端 Console 的完整日志
3. Web 服务器后台的完整日志
4. 数据库查询结果
