#!/bin/bash

# LANChat Web 端启动脚本
# 用法: ./start-web.sh [端口] [数据库路径]

PORT=${1:-8888}
DB_PATH=${2:-""}

echo "正在启动 LANChat Web 服务器..."
echo "端口: $PORT"

if [ -n "$DB_PATH" ]; then
    echo "数据库路径: $DB_PATH"
    ./src-tauri/target/debug/lanchat-web --port $PORT --db-path "$DB_PATH"
else
    echo "数据库路径: ~/.local/share/com.lanchat.app/ (默认，与桌面端共享)"
    ./src-tauri/target/debug/lanchat-web --port $PORT
fi
