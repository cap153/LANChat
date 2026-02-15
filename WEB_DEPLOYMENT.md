# LANChat Web 端部署指南

## 概述

LANChat 支持两种运行模式：
1. **桌面端** (`lanchat`) - 使用 Tauri 窗口界面
2. **Web 端** (`lanchat-web`) - 纯 Web 服务器，可部署到无图形界面的服务器

两种模式共享同一个 SQLite 数据库，因此用户名和聊天记录是同步的。

## 编译

### 编译桌面端
```bash
cd src-tauri
cargo build --bin lanchat --features desktop
```

### 编译 Web 端
```bash
cd src-tauri
cargo build --bin lanchat-web --features web --no-default-features
```

### 编译 Release 版本
```bash
# 桌面端
cd src-tauri
cargo build --release --bin lanchat --features desktop

# Web 端（精简版，无 GUI 依赖）
cd src-tauri
cargo build --release --bin lanchat-web --features web --no-default-features
```

## 运行 Web 端

### 基本用法
```bash
# 使用默认端口 8888 和默认数据库路径
./src-tauri/target/debug/lanchat-web

# 指定端口
./src-tauri/target/debug/lanchat-web --port 9000

# 指定数据库路径
./src-tauri/target/debug/lanchat-web --db-path /path/to/db/folder
```

### 使用启动脚本
```bash
# 默认配置（端口 8888）
./start-web.sh

# 指定端口
./start-web.sh 9000

# 指定端口和数据库路径
./start-web.sh 9000 /path/to/db/folder
```

## 数据库位置

### 默认路径
桌面端和 Web 端现在使用相同的默认路径：
- **Linux**: `~/.local/share/com.lanchat.app/lanchat.db`
- **Windows**: `%APPDATA%\com.lanchat.app\lanchat.db`
- **macOS**: `~/Library/Application Support/com.lanchat.app/lanchat.db`

### 共享数据库
桌面端和 Web 端默认就共享同一个数据库，无需额外配置！

如果需要使用自定义路径：
```bash
./src-tauri/target/debug/lanchat-web --db-path /custom/path/to/db
```

## 部署到服务器

### 1. 编译 Release 版本
```bash
cd src-tauri
cargo build --release --bin lanchat-web
```

### 2. 复制到服务器
```bash
# 复制可执行文件
scp target/release/lanchat-web user@server:/path/to/deploy/

# 或者打包整个项目
tar -czf lanchat-web.tar.gz target/release/lanchat-web
scp lanchat-web.tar.gz user@server:/path/to/deploy/
```

### 3. 在服务器上运行
```bash
# 直接运行
./lanchat-web --port 8888

# 或使用 systemd 服务（推荐）
# 创建 /etc/systemd/system/lanchat-web.service
```

### 4. Systemd 服务配置示例
```ini
[Unit]
Description=LANChat Web Server
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/deploy
ExecStart=/path/to/deploy/lanchat-web --port 8888
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启用服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable lanchat-web
sudo systemctl start lanchat-web
sudo systemctl status lanchat-web
```

## 防火墙配置

确保开放以下端口：
- **TCP 8888** (或你指定的端口) - HTTP Web 服务
- **UDP 8888** (或你指定的端口) - 局域网设备发现

```bash
# UFW 示例
sudo ufw allow 8888/tcp
sudo ufw allow 8888/udp

# firewalld 示例
sudo firewall-cmd --permanent --add-port=8888/tcp
sudo firewall-cmd --permanent --add-port=8888/udp
sudo firewall-cmd --reload
```

## 访问 Web 界面

启动后，在浏览器中访问：
```
http://服务器IP:8888
```

例如：
```
http://192.168.1.100:8888
http://localhost:8888
```

## 功能特性

Web 端支持的功能：
- ✅ 自动生成随机用户名
- ✅ 修改用户名（与桌面端共享）
- ✅ 局域网设备发现
- ✅ 实时显示在线用户
- ✅ 与桌面端共享数据库

## 故障排查

### 端口被占用
```bash
# 查看端口占用
sudo lsof -i :8888
# 或
sudo netstat -tulpn | grep 8888
```

### 数据库权限问题
```bash
# 确保数据库目录有写权限
chmod 755 ~/.local/share/lanchat
chmod 644 ~/.local/share/lanchat/lanchat.db
```

### 查看日志
Web 端会在终端输出日志，包括：
- 数据库初始化信息
- 用户名读取/更新
- UDP 广播和监听状态
- HTTP 请求日志

## 开发调试

### 查看编译输出
```bash
cd src-tauri
cargo build --bin lanchat-web 2>&1 | less
```

### 运行并查看详细日志
```bash
RUST_LOG=debug ./src-tauri/target/debug/lanchat-web
```

### 测试 API
```bash
# 获取用户名
curl http://localhost:8888/api/get_my_name

# 更新用户名
curl -X POST http://localhost:8888/api/update_my_name \
  -H "Content-Type: application/json" \
  -d '{"name":"新用户名"}'
```
