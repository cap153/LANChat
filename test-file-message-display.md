# 文件消息显示测试指南

## 问题诊断

桌面端收到文件后，后台显示：
```
[Web Server] 已发送文件接收 Tauri 事件: new-message
```

但前端没有显示消息。

## 原因

前端的 `onReceiveMessage` 函数只在以下条件下显示消息：
```javascript
if (window.currentChatPeer && window.currentChatPeer.id === message.from_id) {
    addMessageToChat(message, false);
}
```

这意味着：
1. 必须先打开聊天窗口（点击用户列表中的某个用户）
2. 只有当前正在聊天的用户发送的消息才会显示

## 测试步骤

### 桌面端接收文件测试：

1. **启动桌面端**
   ```bash
   ./src-tauri/target/debug/lanchat
   ```

2. **打开浏览器控制台**（F12）

3. **等待发现对方设备**
   - 在用户列表中应该能看到对方

4. **点击用户列表中的发送者**
   - 这会打开聊天窗口
   - 设置 `window.currentChatPeer`

5. **让对方发送文件**

6. **检查控制台日志**：
   ```
   [JS-App] 收到新消息事件
   [JS-App] 事件内容: { from_id: "...", msg_type: "file", ... }
   [UI] 收到新消息: ...
   [UI] 当前聊天对象: { id: "...", name: "...", addr: "..." }
   [UI] 匹配当前聊天对象，显示消息  <-- 应该看到这个
   ```

7. **检查聊天窗口**
   - 应该能看到文件消息

## 如果还是不显示

检查以下内容：

1. **ID 是否匹配**：
   ```javascript
   console.log('发送者 ID:', message.from_id);
   console.log('当前聊天对象 ID:', window.currentChatPeer.id);
   ```

2. **是否打开了聊天窗口**：
   ```javascript
   console.log('聊天窗口状态:', window.currentChatPeer);
   ```

3. **事件是否触发**：
   - 查看控制台是否有 `[JS-App] 收到新消息事件`

## 预期行为

- ✅ Web 端：通过轮询获取消息，每 2 秒检查一次
- ✅ 桌面端：通过 Tauri 事件实时接收消息
- ⚠️ 两端都需要先打开聊天窗口才能看到消息

## 改进建议

未来可以添加：
1. 未读消息提示（红点）
2. 自动打开聊天窗口
3. 消息通知
