# 文件传输立即显示 - 调试版本

## 最新修改（带详细日志）

### 修改内容
1. 在 `onReceiveMessage` 中添加了详细的调试日志
2. 在 Web 端 `sendFile` 中添加了步骤日志
3. 所有关键步骤都有日志输出

### 测试步骤

#### 1. 启动 Web 端
```bash
cd src-tauri
./target/debug/lanchat-web
```

#### 2. 启动桌面端
```bash
cd src-tauri
cargo tauri dev
```

#### 3. 从 Web 端发送文件到桌面端

**Web 端浏览器 Console 应该显示：**
```
[UI] ========== Web 端发送文件 ==========
[UI] 文件名: test.txt
[UI] 文件大小: 1234
[UI] 目标地址: 192.168.1.x:8888
[UI] 1. 在前端显示上传中消息
[UI] addMessageToChat 被调用
[UI] 消息类型: file
[UI] 是否发送: true
[UI] 渲染文件消息: test.txt
[UI] 文件状态: uploading
[UI] 消息已添加到聊天窗口
[UI] 2. 调用 /api/create_upload_record
[UI] ✓ 上传记录已创建
[UI] 3. 开始上传文件到对方
[UI] ✓ 文件上传成功
[UI] 4. 更新上传状态为 sent
[UI] ✓ 上传状态已更新
[UI] 5. 刷新聊天历史
[UI] 加载了 X 条历史消息
[UI] ========== 文件发送完成 ==========
```

**桌面端 Console 应该显示：**
```
[JS-App] ========== 收到 new-message 事件 ==========
[JS-App] 事件内容: {
  "from_id": "xxx-xxx-xxx",
  "from_name": "Unknown",
  "content": "test.txt",
  "timestamp": 1234567890,
  "msg_type": "file",
  "file_name": "test.txt",
  "file_status": "downloading"
}
[JS-App] 当前聊天对象: { id: "xxx", name: "Web用户", addr: "192.168.1.x:8888" }
[UI] ========== onReceiveMessage 被调用 ==========
[UI] 消息内容: { ... }
[UI] 当前聊天对象: { ... }
[UI] ✓ 匹配当前聊天对象
[UI] 直接显示新消息 (msg_type=file, file_status=downloading)
[UI] addMessageToChat 被调用
[UI] 消息类型: file
[UI] 是否发送: false
[UI] 渲染文件消息: test.txt
[UI] 文件状态: downloading
[UI] 消息已添加到聊天窗口
[UI] ==========================================
```

**桌面端后台应该显示：**
```
[Web Server] 收到文件上传请求
[Web Server] 解析字段: peer_id
[Web Server] sender_id (发送者): xxx-xxx-xxx
[Web Server] 解析字段: file
[Web Server] 文件名: test.txt
[Web Server] ✓ 已创建下载中记录，ID: 123
[Web Server] 自动接收模式：保存到下载目录
[Web Server] 保存文件到: "/home/user/Downloads/test.txt"
[Web Server] 文件大小: 1234 字节
[Web Server] ========== 文件接收完成 ==========
[Web Server] 文件 ID: xxx
[Web Server] 文件名: test.txt
[Web Server] 文件大小: 1234 字节
[Web Server] 文件路径: "/home/user/Downloads/test.txt"
[Web Server] 发送者 ID: xxx-xxx-xxx
[Web Server] 自动接收设置: true
[Web Server] 文件状态: accepted
[Web Server] ✓ 文件记录已更新到数据库
[Web Server] ==========================================
[Web Server] 已发送文件接收完成 Tauri 事件: new-message
```

### 问题诊断

#### 如果 Web 端没有显示上传消息

**检查点 1**: 是否看到 "1. 在前端显示上传中消息"？
- ❌ 没有 → `sendFile` 函数没有被调用
- ✅ 有 → 继续下一步

**检查点 2**: 是否看到 "addMessageToChat 被调用"？
- ❌ 没有 → `addMessageToChat` 函数有问题
- ✅ 有 → 继续下一步

**检查点 3**: 是否看到 "消息已添加到聊天窗口"？
- ❌ 没有 → 消息渲染失败
- ✅ 有但看不到消息 → 检查 CSS 或 DOM 结构

**检查点 4**: 是否看到 "2. 调用 /api/create_upload_record"？
- ❌ 没有 → 代码执行被中断
- ✅ 有 → 继续下一步

**检查点 5**: 是否看到 "✓ 上传记录已创建"？
- ❌ 没有 → API 调用失败，检查网络请求
- ✅ 有 → 数据库记录应该已创建

#### 如果桌面端没有显示下载消息

**检查点 1**: 后台是否有 "✓ 已创建下载中记录"？
- ❌ 没有 → 数据库插入失败
- ✅ 有 → 继续下一步

**检查点 2**: 后台是否有 "已发送文件接收完成 Tauri 事件"？
- ❌ 没有 → Tauri 事件没有发送（可能是 feature 配置问题）
- ✅ 有 → 继续下一步

**检查点 3**: 前端是否有 "收到 new-message 事件"？
- ❌ 没有 → 事件监听器没有注册或事件没有到达前端
- ✅ 有 → 继续下一步

**检查点 4**: 是否有 "✓ 匹配当前聊天对象"？
- ❌ 没有，显示 "✗ 不匹配" → `from_id` 不匹配，检查日志中的 ID
- ✅ 有 → 继续下一步

**检查点 5**: 是否有 "直接显示新消息"？
- ❌ 没有，显示 "文件状态更新，刷新聊天历史" → 状态不是 `downloading`
- ✅ 有 → 应该调用 `addMessageToChat`

**检查点 6**: 是否有 "消息已添加到聊天窗口"？
- ❌ 没有 → `addMessageToChat` 执行失败
- ✅ 有但看不到 → 检查 DOM 或 CSS

### 常见问题

#### Q1: Web 端显示了上传中，但一直不更新
**原因**: 刷新聊天历史时没有从数据库读取到更新后的状态

**解决**:
```bash
# 检查数据库
sqlite3 ~/.local/share/com.lanchat.app/lanchat.db
SELECT * FROM messages WHERE sender_id='me' AND msg_type='file' ORDER BY timestamp DESC LIMIT 5;
```

应该看到 `file_status` 从 `uploading` 变成 `sent`

#### Q2: 桌面端后台有日志但前端没有显示
**原因**: 
1. 聊天窗口没有打开
2. `from_id` 不匹配
3. Tauri 事件没有到达前端

**解决**:
1. 确保打开了与发送者的聊天窗口
2. 检查日志中的 `from_id` 是否匹配
3. 检查是否有 "收到 new-message 事件" 的日志

#### Q3: 两端都没有消息
**原因**: 可能是聊天窗口没有打开

**解决**: 
1. 在发送文件前，先打开与对方的聊天窗口
2. 或者实现未读消息提示功能

### 下一步优化

如果问题依然存在，请提供：
1. Web 端浏览器 Console 的完整日志（从发送文件开始）
2. 桌面端 Console 的完整日志
3. 桌面端后台的完整日志
4. 是否打开了聊天窗口
5. 双方的用户 ID（from_id）

