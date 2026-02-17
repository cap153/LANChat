我想写一个跨平台的、不用注册的、支持传输文件的聊天软件，项目名称想好了，就叫作LANChat
打算使用rust后端，前端原生html，然后tauri2.0实现跨平台。
没有什么经验，所以请你先帮我制定完善的计划。
1. 代码提前安装功能规划好放在哪些文件，例如数据库查询语句可以单独放`db.rs`
2. 需要支持linux桌面端、windows桌面端、安卓app、web端
3. 我的开发环境是arch linux，现在已经安装好了rust环境，使用cargo安装好了tauri。
4. 对于`不用注册`参考locaclsend自动生成用户名，需要支持用户手动改名保存到数据库
5. 需要支持历史聊天记录，暂定保存sqlite，保存路径遵循社区规范就行了，例如linux应该是保存在`~/.local/share`
6. 不要什么东西都往js文件里面写，前后端分离，各个文件一定要分工明确
7. 传输文件需要支持设置是否自动接收、支持设置自动接收的保存路径，如果没有设置自动接收，手动点击接收时要支持选择路径
8. 传输文件的接收端如果把已经接收的文件删除了，如果发送端还没有删除或移动本地文件，需要支持从历史记录点击重新下载
9. 不要把接收的文件存到数据库，那样数据库会变的庞大
10. 我的开发环境可以直接调试linux桌面端；我还有一台局域网没有安装图形界面的debian服务器，可以方便的跑web端，我的防火墙开放了8888端口打算作为默认的web访问端口，默认的广播端口、文件传输复用该端口；我希望可以在web端的可执行文件指定端口，如果没有指定就用默认值；对应的桌面端在接收广播或者访问对方api接口肯定是需要对应，在设置页面需要有相关的端口选项
11. 启动桌面端时不要启动web端，启动web端时不要启动桌面端，我希望它们可以编译成独立的可执行文件，因为我不想在桌面端额外启动web端消耗资源，我的debian服务器也跑不了桌面端，所以需要独立的web端
12. 编译web端要打包前端代码，方便我直接扔到服务器运行测试
13. 在合适的地方都加上println语句方便检查错误
14. 消息删除功能，支持多选消息然后删除
15. 正确显示文件大小
16. offline主题修改

其他的需求还没有想到，开发的时候我想到了会提出来  

---

### 第一阶段：项目结构与技术选型

为了保证代码整洁（前后端分离，逻辑拆分），我们需要明确文件职责。

#### 1. 后端文件规划 (`src-tauri/src/`)
*   `main.rs`: 程序入口，初始化 Tauri 窗口、数据库和系统托盘。
*   `db.rs`: 数据库逻辑。负责 SQLite 的初始化、增删改查（消息记录、用户信息、设置）。
*   `network/`: (文件夹)
    *   `mod.rs`: 网络模块入口。
    *   `discovery.rs`: 局域网设备发现（UDP 广播或 mDNS）。
    *   `protocol.rs`: 定义数据包格式（JSON），处理消息解析。
    *   `transfer.rs`: 文件传输逻辑（流式读写，支持大文件）。
*   `commands.rs`: 定义所有通过 `invoke` 给前端调用的 Rust 函数。
*   `models.rs`: 定义 Rust 中的结构体（User, Message, FileMetadata）。
*   `utils.rs`: 工具函数（获取系统路径、生成随机用户名、图标处理等）。

#### 2. 前端文件规划 (`src/`)
*   `index.html`: 主界面结构。
*   `css/style.css`: 样式。
*   `js/`:
    *   `api.js`: 封装 `invoke` 调用，所有的后端请求都写在这里。
    *   `ui.js`: 负责 DOM 操作，渲染消息列表、用户列表。
    *   `app.js`: 逻辑主入口，监听后端推送的事件（如收到新消息）。

---

### 第二阶段：核心功能逻辑设计

#### 1. 用户身份与“不注册”实现
*   **首次启动**：Rust 后端检查数据库是否有 `settings` 表。如果没有，利用 `utils.rs` 随机生成一个（例如：`勇敢的螃蟹`），生成一个唯一 UUID，存入 SQLite。
*   **改名**：前端调用 `update_username` 命令，后端更新数据库并广播给局域网其他设备。

#### 2. 数据库设计 (SQLite)
使用 `rusqlite` 或 `sqlx` 库。
*   `users`: 存储局域网发现的用户（UID, 用户名, IP, 最后在线时间）。
*   `messages`: 存储聊天记录（发送者 UID, 接收者 UID, 内容, 类型[文本/文件], 时间戳, 文件状态）。
*   `settings`: 存储个人配置（用户名, 自动接收开关, 默认保存路径）。

#### 3. 局域网发现 (类似 LocalSend)
*   **UDP 广播**：每隔几秒发送一个包含自己 ID、名字和端口的 UDP 包。
*   **监听器**：维护一个内存中的在线用户列表，实时推送给前端。

#### 4. 文件传输逻辑 (重点)
*   **协议**：使用 TCP 进行文件传输。
*   **自动接收**：在 `db.rs` 中查询配置。如果开启，Rust 直接创建文件流开始写入。
*   **手动选择**：如果未开启，Rust 触发 Tauri 的 `dialog` 插件，弹出原生文件夹选择框。
*   **重发机制**：
    *   数据库记录文件的**原始本地路径**。
    *   当接收端请求重发时，发送端校验该路径文件是否存在，存在则重新开启流。

---

### 第三阶段：跨平台与权限适配 (Tauri 2.0 规范)

根据你提供的文档，我们需要在 `src-tauri/capabilities/` 中精细化配置权限。

#### 1. 权限配置文件规划
创建 `src-tauri/capabilities/default.json`：
```json
{
  "identifier": "main-capability",
  "windows": ["main"],
  "permissions": [
    "core:path:default",
    "core:event:default",
    "core:window:default",
    "core:dialog:allow-save-file",
    "core:dialog:allow-open-file",
    "fs:allow-read-text-file",
    "fs:allow-write-file",
    "fs:allow-exists",
    "http:default",
    "network:allow-start-server",
    "network:allow-connect"
  ]
}
```

#### 2. 路径处理 (遵循规范)
使用 `tauri-plugin-path`：
*   **Linux**: `~/.local/share/lanchat/` (数据库)
*   **Windows**: `APPDATA/lanchat/`
*   **Android**: 内部存储路径。

---

### 第四阶段：开发路线图 (Todo List)

1.  **环境初始化**:
    *   `cargo tauri init`
    *   安装插件: `sql`, `fs`, `dialog`, `path`, `network` (Tauri 2.0 插件)。
2.  **后端基础**:
    *   在 `db.rs` 中实现 SQLite 自动创建表。
    *   在 `utils.rs` 中实现获取机器指纹和随机名。
3.  **网络通讯**:
    *   实现 UDP 发现逻辑，让两个窗口能互相看到。
    *   实现简单的文本消息传递。
4.  **文件功能**:
    *   实现文件选择 -> 发送元数据 -> 接收端确认 -> TCP 流传输。
    *   集成自动接收逻辑。
5.  **UI 完善**:
    *   原生 HTML 编写简洁的侧边栏和聊天框。
    *   **Android 适配**: 调整 CSS 媒体查询，确保手机端布局正常。

---

### 给你的特别提醒 (基于 Tauri 2.0 文档)

1.  **Android 权限**: 在 Tauri 2.0 中，Android 的文件访问需要在 `gen/android` 下的 `AndroidManifest.xml` 手动添加权限声明（如 `READ_EXTERNAL_STORAGE`），并在 Rust 中请求运行时权限。
2.  **前后端分离**: 绝对**不要**在 HTML 里写 `<script>window.ipc = ...</script>`。建议所有的 Rust 交互都封装在 `js/api.js` 中，前端只管调用函数并处理 Promise。
3.  **文件安全**: Tauri 2.0 的 `fs` 权限非常严格。如果你要让用户选择任意路径保存，必须通过 `dialog` 插件获取路径，或者在 `scope` 中预设权限。

附上tauri2.0最新的部分文档：
```
Permissions

Permissions are descriptions of explicit privileges of commands.

[[permission]]
identifier = "my-identifier"
description = "This describes the impact and more."
commands.allow = [
    "read_file"
]

[[scope.allow]]
my-scope = "$HOME/*"

[[scope.deny]]
my-scope = "$HOME/secret"

It can enable commands to be accessible in the frontend of a Tauri application. It can map scopes to commands and defines which commands are enabled. Permissions can enable or deny certain commands, define scopes or combine both.

To grant or deny a permission to your app’s window or webview, you must reference the permission in a capability.

Permissions can be grouped as a set under a new identifier. This is called a permission set. This allows you to combine scope related permissions with command related permissions. It also allows to group or bundle operating specific permissions into more usable sets.

As a plugin developer you can ship multiple, pre-defined, well named permissions for all of your exposed commands.

As an application developer you can extend existing plugin permissions or define them for your own commands. They can be grouped or extended in a set to be re-used or to simplify the main configuration files later.
Permission Identifier

The permissions identifier is used to ensure that permissions can be re-used and have unique names.

Tip

With name we refer to the plugin crate name without the tauri-plugin- prefix. This is meant as namespacing to reduce likelihood of naming conflicts. When referencing permissions of the application itself it is not necessary.

    <name>:default Indicates the permission is the default for a plugin or application
    <name>:<command-name> Indicates the permission is for an individual command

The plugin prefix tauri-plugin- will be automatically prepended to the identifier of plugins at compile time and is not required to be manually specified.

Identifiers are limited to ASCII lower case alphabetic characters [a-z] and the maximum length of the identifier is currently limited to 116 due to the following constants:

const IDENTIFIER_SEPARATOR: u8 = b':';
const PLUGIN_PREFIX: &str = "tauri-plugin-";

// https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
const MAX_LEN_PREFIX: usize = 64 - PLUGIN_PREFIX.len();
const MAX_LEN_BASE: usize = 64;
const MAX_LEN_IDENTIFIER: usize = MAX_LEN_PREFIX + 1 + MAX_LEN_BASE;

Configuration Files

Simplified example of an example Tauri plugin directory structure:
Terminal window

tauri-plugin
├── README.md
├── src
│  └── lib.rs
├── build.rs
├── Cargo.toml
├── permissions
│  └── <identifier>.json/toml
│  └── default.json/toml

The default permission is handled in a special way, as it is automatically added to the application configuration, as long as the Tauri CLI is used to add plugins to a Tauri application.

For application developers the structure is similar:
Terminal window

tauri-app
├── index.html
├── package.json
├── src
├── src-tauri
│   ├── Cargo.toml
│   ├── permissions
│      └── <identifier>.toml
|   ├── capabilities
│      └── <identifier>.json/.toml
│   ├── src
│   ├── tauri.conf.json

Note

As an application developer the capability files can be written in json/json5 or toml, whereas permissions only can be defined in toml.
Examples

Example permissions from the File System plugin.
plugins/fs/permissions/autogenerated/base-directories/home.toml

[[permission]]
identifier = "scope-home"
description = """This scope permits access to all files and
list content of top level directories in the `$HOME`folder."""

[[scope.allow]]
path = "$HOME/*"

plugins/fs/permissions/read-files.toml

[[permission]]
identifier = "read-files"
description = """This enables all file read related
commands without any pre-configured accessible paths."""
commands.allow = [
    "read_file",
    "read",
    "open",
    "read_text_file",
    "read_text_file_lines",
    "read_text_file_lines_next"
]

plugins/fs/permissions/autogenerated/commands/mkdir.toml

[[permission]]
identifier = "allow-mkdir"
description = "This enables the mkdir command."
commands.allow = [
    "mkdir"
]

Example implementation extending above plugin permissions in your app:
my-app/src-tauri/permissions/home-read-extends.toml

[[set]]
identifier = "allow-home-read-extended"
description = """ This allows non-recursive read access to files and to create directories
in the `$HOME` folder.
"""
permissions = [
    "fs:read-files",
    "fs:scope-home",
    "fs:allow-mkdir"
]

Capabilities

Tauri provides application and plugin developers with a capabilities system, to granually enable and constrain the core exposure to the application frontend running in the system WebView.

Capabilities define which permissions are granted or denied for which windows or webviews.

Capabilities can affect multiple windows and webviews and these can be referenced in multiple capabilities.

Security Tip

Windows and WebViews which are part of more than one capability effectively merge the security boundaries and permissions of all involved capabilities.

Capability files are either defined as a JSON or a TOML file inside the src-tauri/capabilities directory.

It is good practice to use individual files and only reference them by identifier in the tauri.conf.json but it is also possible to define them directly in the capabilities field.

All capabilities inside the capabilities directory are automatically enabled by default. Once capabilities are explicitly enabled in the tauri.conf.json, only these are used in the application build.

For a full reference of the configuration scheme please see the references section.

The following example JSON defines a capability that allows the main window use the default functionality of core plugins and the window.setTitle API.
src-tauri/capabilities/default.json

{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:path:default",
    "core:event:default",
    "core:window:default",
    "core:app:default",
    "core:resources:default",
    "core:menu:default",
    "core:tray:default",
    "core:window:allow-set-title"
  ]
}

These snippets are part of the Tauri configuration file.

This is likely the most common configuration method, where the individual capabilities are inlined and only permissions are referenced by identifier.

This requires well defined capability files in the capabilities directory.
src-tauri/tauri.conf.json

{
  "app": {
    "security": {
      "capabilities": ["my-capability", "main-capability"]
    }
  }
}

Inline capabilities can be mixed with pre-defined capabilities.
src-tauri/tauri.conf.json

{
  "app": {
    "security": {
      "capabilities": [
        {
          "identifier": "my-capability",
          "description": "My application capability used for all windows",
          "windows": ["*"],
          "permissions": ["fs:default", "allow-home-read-extended"]
        },
        "my-second-capability"
      ]
    }
  }
}

By default, all commands that you registered in your app (using the tauri::Builder::invoke_handler function) are allowed to be used by all the windows and webviews of the app. To change that, consider using AppManifest::commands.
src-tauri/build.rs

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["your_command"])),
    )
    .unwrap();
}

Target Platform

Capabilities can be platform-specific by defining the platforms array. By default the capability is applied to all targets, but you can select a subset of the linux, macOS, windows, iOS and android targets.

For example a capability for desktop operating systems. Note it enables permissions on plugins that are only available on desktop:
src-tauri/capabilities/desktop.json

{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "desktop-capability",
  "windows": ["main"],
  "platforms": ["linux", "macOS", "windows"],
  "permissions": ["global-shortcut:allow-register"]
}

And another example of a capability for mobile. Note it enables permissions on plugins that are only available on mobile:
src-tauri/capabilities/mobile.json

{
  "$schema": "../gen/schemas/mobile-schema.json",
  "identifier": "mobile-capability",
  "windows": ["main"],
  "platforms": ["iOS", "android"],
  "permissions": [
    "nfc:allow-scan",
    "biometric:allow-authenticate",
    "barcode-scanner:allow-scan"
  ]
}

Remote API Access

By default the API is only accessible to bundled code shipped with the Tauri App. To allow remote sources access to certain Tauri Commands it is possible to define this in the capability configuration file.

This example would allow to scan for NFC tags and to use the barcode scanner from all subdomains of tauri.app.
src-tauri/capabilities/remote-tags.json

{
  "$schema": "../gen/schemas/remote-schema.json",
  "identifier": "remote-tag-capability",
  "windows": ["main"],
  "remote": {
    "urls": ["https://*.tauri.app"]
  },
  "platforms": ["iOS", "android"],
  "permissions": ["nfc:allow-scan", "barcode-scanner:allow-scan"]
}

Caution

On Linux and Android, Tauri is unable to distinguish between requests from an embedded <iframe> and the window itself.

Please consider usage of this feature very carefully and read more into the specific security implications for your targeted operating system in the reference section of this feature.
Security Boundaries

What does it protect against?

Depending on the permissions and capabilities it is able to:

    Minimize impact of frontend compromise
    Prevent or reduce (accidential) exposure of local system interfaces and data
    Prevent or reduce possible privilege escalation from frontend to backend/system

What does it not protect against?

    Malicious or insecure Rust code
    Too lax scopes and configuration
    Incorrect scope checks in the command implementation
    Intentional bypasses from Rust code
    Basically anything which was written in the rust core of an application
    0-days or unpatched 1-days in the system WebView
    Supply chain attacks or otherwise compromised developer systems

Security Tip

The security boundaries are depending on window labels (not titles). We recommend to only expose of the window creation functionality to higher privileged windows.
Schema Files

Tauri generates JSON schemas with all the permissions available to your application through tauri-build, allowing autocompletion in your IDE. To use a schema, set the $schema property in your configuration file (either .json or .toml) to one of the platform-specific schemas located in the gen/schemas directory. Usually you will set it to ../gen/schemas/desktop-schema.json or ../gen/schemas/mobile-schema.json though you can also define a capability for a specific target platform.
Configuration Files

Simplified example of an example Tauri application directory structure:
Terminal window

tauri-app
├── index.html
├── package.json
├── src/
├── src-tauri/
│   ├── Cargo.toml
│   ├── capabilities/
│   │  └── <identifier>.json/toml
│   ├── src/
│   ├── tauri.conf.json

Everything can be inlined into the tauri.conf.json but even a little more advanced configuration would bloat this file and the goal of this approach is that the permissions are abstracted away whenever possible and simple to understand.
```


---

## 开发进度更新 (2025-02-15)

### 已完成功能

#### 1. 用户改名功能 ✅
- **后端实现**:
  - `db.rs`: 添加 `update_username` 函数，支持验证（非空、长度限制）
  - `commands.rs`: 添加 `update_my_name` Tauri 命令
  - 权限配置: 创建 `permissions/commands.toml` 定义权限
  
- **前端实现**:
  - `api.js`: 添加 `apiUpdateMyName` 函数，支持桌面端和 Web 端
  - `ui.js`: 实现改名编辑面板，支持快捷键（回车保存、ESC 取消）
  - `index.html` + `style.css`: 美化界面，添加编辑按钮和弹出式编辑面板

#### 2. Web 端数据库集成 ✅
- **数据库共享**:
  - 重构 `db.rs`，添加 `init_db_standalone` 函数用于 Web 端
  - 支持自定义数据库路径参数 `--db-path`
  - 桌面端和 Web 端可以共享同一个 SQLite 数据库
  
- **Web 服务器更新**:
  - `web_server.rs`: 集成数据库状态，使用 Axum State 管理
  - 实现真实的 `/api/get_my_name` 和 `/api/update_my_name` 接口
  - 从数据库读取和更新用户名
  
- **独立可执行文件**:
  - `server_main.rs`: 完整的 Web 端入口，支持命令行参数
  - 编译为独立的 `lanchat-web` 可执行文件
  - 支持 `--port` 和 `--db-path` 参数

#### 3. 部署支持 ✅
- 创建 `start-web.sh` 启动脚本
- 创建 `WEB_DEPLOYMENT.md` 部署文档
- 包含 systemd 服务配置示例
- 防火墙配置说明

### 数据库路径说明

- **统一默认路径**: `~/.local/share/com.lanchat.app/lanchat.db`
- 桌面端和 Web 端默认共享同一个数据库
- 可通过 `--db-path` 参数自定义路径

### 编译命令

```bash
# 桌面端
cd src-tauri && cargo build --bin lanchat --features desktop

# Web 端（精简版，无 WebKit 等 GUI 依赖）
cd src-tauri && cargo build --bin lanchat-web --features web --no-default-features

# Release 版本
cd src-tauri && cargo build --release --bin lanchat-web --features web --no-default-features
```

### 运行示例

```bash
# Web 端 - 默认配置（与桌面端共享数据库）
./src-tauri/target/debug/lanchat-web

# Web 端 - 指定端口
./src-tauri/target/debug/lanchat-web --port 9000

# Web 端 - 使用自定义数据库路径
./src-tauri/target/debug/lanchat-web --db-path /custom/path
```

### 下一步计划

1. 实现文本消息传输（TCP 协议）
2. 实现文件传输功能
3. 完善聊天界面 UI
4. 添加消息历史记录查询
5. 实现设置页面（端口配置、自动接收等）


---

## 开发进度更新 (2025-02-15 - 用户列表实时更新)

### 已完成功能

#### 1. 在线用户列表管理 ✅
- **后端实现**:
  - 创建 `peers.rs` 模块,实现 `PeerManager` 管理在线用户
  - 支持添加/更新用户、获取活跃用户、清理过期用户
  - 用户 30 秒内活跃视为在线,60 秒后自动清理
  
- **桌面端**:
  - 继续使用 Tauri 事件系统 (`emit("new-peer")`)
  - 前端通过 `apiListen` 实时接收新用户事件
  - 用户列表实时更新
  
- **Web 端**:
  - 添加 `/api/get_peers` API 接口
  - 前端每 3 秒轮询一次用户列表
  - 自动更新用户列表显示

#### 2. 编译优化 ✅
- 修复了 `build.rs` 的条件编译问题
- 修复了 `Cargo.toml` 中 `tauri-build` 的 feature 配置
- Web 端和桌面端都能正常编译
- Web 端无 GUI 依赖,桌面端包含完整 GUI 库

### 技术细节

- **PeerManager**: 使用 `Arc<RwLock<HashMap>>` 实现线程安全的用户列表
- **Web 端轮询**: 使用 `setInterval` 每 3 秒请求一次 `/api/get_peers`
- **桌面端事件**: 使用 Tauri 的 `emit` 和 `listen` 实现实时通信
- **去重逻辑**: 前端 `addUserToList` 检查 `data-addr` 避免重复添加

### 文件变更

- 新增: `src-tauri/src/peers.rs` (用户管理模块)
- 修改: `src-tauri/src/lib.rs` (添加 peers 模块)
- 修改: `src-tauri/src/network/discovery.rs` (集成 PeerManager)
- 修改: `src-tauri/src/web_server.rs` (添加 get_peers API)
- 修改: `src-tauri/src/server_main.rs` (使用 PeerManager)
- 修改: `src-tauri/src/main.rs` (桌面端使用 PeerManager)
- 修改: `src/js/api.js` (添加 apiGetPeers 函数)
- 修改: `src/js/app.js` (添加轮询逻辑)
- 新增: `verify-build.sh` (编译验证脚本)
