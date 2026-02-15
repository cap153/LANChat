# 文件传输测试指南

## 测试环境

### Web 端测试（两个实例）

1. 启动第一个 Web 端实例：
```bash
./src-tauri/target/debug/lanchat-web
```

2. 启动第二个 Web 端实例（不同端口）：
```bash
./src-tauri/target/debug/lanchat-web --port 9999
```

3. 在浏览器中打开：
   - http://localhost:8888
   - http://localhost:9999

4. 等待双方互相发现（用户列表会显示对方）

5. 点击对方用户进入聊天窗口

6. 点击 📎 按钮选择文件发送

7. 对方会在聊天窗口看到文件消息，点击可下载

### 桌面端测试

1. 编译桌面端：
```bash
cd src-tauri
cargo build --bin lanchat --features desktop
```

2. 运行桌面端：
```bash
./target/debug/lanchat
```

3. 同时运行一个 Web 端实例用于测试互通

4. 在桌面端点击 📎 按钮会弹出系统文件选择对话框

5. 选择文件后自动发送

## 已知问题

- Web 端之间发送文件需要确保 CORS 配置正确
- 桌面端需要 Tauri 的 dialog 权限
- 文件保存在临时目录或配置的下载目录

## 调试

查看控制台日志：
- 前端：浏览器开发者工具 Console
- 后端：终端输出

关键日志：
- `[Web Server] 接收文件: xxx, 大小: xxx 字节`
- `[Web Server] 文件保存成功: xxx`
- `[Command] 文件上传成功`
- `[JS-API] 文件上传成功`
