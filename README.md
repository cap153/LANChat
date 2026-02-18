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

### aur

```bash
paru -S lanchat-bin
```

### Releases

[https://github.com/cap153/LANChat/releases](https://github.com/cap153/LANChat/releases) 

### 编译

前置要求：

[https://v2.tauri.app/start/](https://v2.tauri.app/start/)  
[https://v2.tauri.app/start/prerequisites/](https://v2.tauri.app/start/prerequisites/)   

```bash
# 桌面端
cargo tauri build --bundles deb
cargo tauri build --bundles rpm

# apk
cargo tauri android build --target aarch64
./sign-apk.sh

# windows桌面端
cd src-tauri
cargo xwin build --release --bin lanchat --target x86_64-pc-windows-msvc

# Web 端（精简版，无 GUI 依赖）
cd src-tauri
cargo build --release --bin lanchat-web --features web --no-default-features
```

## 主题

支持自定义`css`，文件名称随意，存在路径：

- **Linux**: `~/.config/lanchat/`
- **Windows**: `%APPDATA%\.config\lanchat`

可以参考内置的主题：[https://github.com/cap153/LANChat/tree/main/src/css](https://github.com/cap153/LANChat/tree/main/src/css) 

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
- [x] Windows 端测试

### 🚧 进行中
- [ ] 删除历史聊天记录

### 📋 计划中
- [ ] 文件重新下载

## 运行

1. 在服务器上运行(不指定参数将默认使用`8888`端口):
```bash
lanchat-web --port 8888
```

2. 配置防火墙示例:
```bash
sudo ufw allow 8888/tcp
sudo ufw allow 8888/udp
```

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

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 致谢

- [Tauri](https://tauri.app/) - 跨平台应用框架
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包
