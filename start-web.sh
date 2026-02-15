#!/bin/bash

# LANChat Web 端启动脚本
# 用法: ./start-web.sh [端口] [数据库路径]

PORT=${1:-8888}
DB_PATH=${2:-""}

# 检查可执行文件是否存在
if [ ! -f "./src-tauri/target/debug/lanchat-web" ] && [ ! -f "./src-tauri/target/release/lanchat-web" ]; then
    echo "错误: 找不到 lanchat-web 可执行文件"
    echo "请先编译 Web 端:"
    echo "  cd src-tauri"
    echo "  cargo build --bin lanchat-web --features web --no-default-features"
    exit 1
fi

# 优先使用 release 版本
if [ -f "./src-tauri/target/release/lanchat-web" ]; then
    BINARY="./src-tauri/target/release/lanchat-web"
else
    BINARY="./src-tauri/target/debug/lanchat-web"
fi

echo "正在启动 LANChat Web 服务器..."
echo "使用可执行文件: $BINARY"
echo "端口: $PORT"

if [ -n "$DB_PATH" ]; then
    echo "数据库路径: $DB_PATH"
    $BINARY --port $PORT --db-path "$DB_PATH"
else
    echo "数据库路径: ~/.local/share/com.lanchat.app/ (默认，与桌面端共享)"
    $BINARY --port $PORT
fi
