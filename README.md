# LANChat

一个跨平台的、无需注册的、支持文件传输的局域网聊天软件。

## 特性

- 🚀 **无需注册** - 自动生成随机用户名，可随时修改
- 💻 **跨平台支持** - Linux 桌面端、Windows 桌面端、Android App、Web 端
- 🔍 **自动发现** - 基于 UDP 广播的局域网设备自动发现
- 💬 **实时聊天** - 支持文本消息和文件传输
- 📁 **文件传输** - 支持大文件传输，可设置自动接收
- 💾 **历史记录** - SQLite 数据库保存聊天记录
- 🌐 **Web 端** - 可部署到无图形界面服务器

## 技术栈

- **后端**: Rust + Tauri 2.0
- **前端**: 原生 HTML + CSS + JavaScript
- **数据库**: SQLite (sqlx)
- **网络**: UDP 广播 + TCP 传输
- **Web 服务器**: Axum

## 快速开始

### 前置要求

- Rust 工具链 (rustc, cargo)
- Tauri CLI: `cargo install tauri-cli`

### 编译

```bash
# 桌面端
cd src-tauri
cargo build --bin lanchat --features desktop

# Web 端（精简版，无 GUI 依赖）
cd src-tauri
cargo build --bin lanchat-web --features web --no-default-features

# Release 版本
cargo build --release --bin lanchat-web --features web --no-default-features

# apk
export RANLIB=$ANDROID_HOME/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ranlib && cargo tauri android build --target aarch64 2>&1 | tail -30
./sign-apk.sh
```

### 运行

文档编写中，后续完善。。。

**注意**: 桌面端和 Web 端默认共享同一个数据库，因此用户名和聊天记录是同步的！

## 项目结构

```
LANChat/
├── src/                      # 前端代码
│   ├── css/
│   │   └── style.css        # 样式文件
│   ├── js/
│   │   ├── api.js           # API 封装
│   │   ├── app.js           # 应用逻辑
│   │   └── ui.js            # UI 交互
│   └── index.html           # 主页面
├── src-tauri/               # 后端代码
│   ├── src/
│   │   ├── main.rs          # 桌面端入口
│   │   ├── server_main.rs   # Web 端入口
│   │   ├── lib.rs           # 库入口
│   │   ├── commands.rs      # Tauri 命令
│   │   ├── db.rs            # 数据库逻辑
│   │   ├── models.rs        # 数据模型
│   │   ├── utils.rs         # 工具函数
│   │   ├── web_server.rs    # Web 服务器
│   │   └── network/         # 网络模块
│   │       ├── discovery.rs # 设备发现
│   │       ├── protocol.rs  # 协议定义
│   │       └── transfer.rs  # 文件传输
│   ├── capabilities/        # Tauri 权限配置
│   ├── permissions/         # 自定义权限
│   └── Cargo.toml
├── start-web.sh             # Web 端启动脚本
├── test-web-api.sh          # API 测试脚本
├── WEB_DEPLOYMENT.md        # Web 端部署文档
├── AGENTS.md                # 开发计划和进度
└── README.md                # 本文件
```

## 数据库

### 默认路径
桌面端和 Web 端共享同一个数据库：
- **Linux**: `~/.local/share/com.lanchat.app/lanchat.db`
- **Windows**: `%APPDATA%\com.lanchat.app\lanchat.db`

### 数据表
- `settings` - 用户配置（用户名、自动接收、保存路径等）
- `messages` - 聊天记录
- `users` - 局域网发现的用户（计划中）

## 功能状态

### ✅ 已完成
- [x] 项目架构搭建
- [x] 数据库初始化
- [x] 自动生成随机用户名
- [x] 用户改名功能（桌面端 + Web 端）
- [x] 局域网设备发现（UDP 广播）
- [x] 实时显示在线用户
- [x] Web 端独立部署
- [x] 桌面端和 Web 端共享数据库
- [x] 设置页面
- [x] 消息历史记录查询
- [x] Android 端适配
- [x] 文本消息传输
- [x] 文件传输功能

### 🚧 进行中
- [ ] Windows 端测试

### 📋 计划中
- [ ] 文件重新下载

## 开发

### 测试 Web API
```bash
# 启动 Web 服务器
./start-web.sh

# 在另一个终端运行测试
./test-web-api.sh
```

### 快速部署到服务器

1. 编译 Release 版本:
```bash
cd src-tauri
cargo build --release --bin lanchat-web
```

2. 复制到服务器:
```bash
scp target/release/lanchat-web user@server:/path/to/deploy/
```

3. 在服务器上运行:
```bash
./lanchat-web --port 8888
```

4. 配置防火墙:
```bash
sudo ufw allow 8888/tcp
sudo ufw allow 8888/udp
```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 致谢

- [Tauri](https://tauri.app/) - 跨平台应用框架
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包
