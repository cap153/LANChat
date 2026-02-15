#!/bin/bash

# LANChat Web 端编译脚本

echo "========================================="
echo "编译 LANChat Web 端（精简版）"
echo "========================================="
echo ""
echo "特性："
echo "  - 无 WebKit/GTK 依赖"
echo "  - 无 Tauri GUI 依赖"
echo "  - 仅包含 Web 服务器和数据库功能"
echo ""

cd src-tauri

if [ "$1" == "release" ]; then
    echo "编译 Release 版本..."
    cargo build --release --bin lanchat-web --features web --no-default-features
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "========================================="
        echo "✅ 编译成功！"
        echo "========================================="
        echo ""
        echo "可执行文件位置:"
        echo "  target/release/lanchat-web"
        echo ""
        echo "文件大小:"
        ls -lh target/release/lanchat-web | awk '{print "  " $5}'
        echo ""
        echo "运行方式:"
        echo "  ./target/release/lanchat-web --port 8888"
        echo ""
        echo "或使用启动脚本:"
        echo "  cd .. && ./start-web.sh"
    fi
else
    echo "编译 Debug 版本..."
    cargo build --bin lanchat-web --features web --no-default-features
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "========================================="
        echo "✅ 编译成功！"
        echo "========================================="
        echo ""
        echo "可执行文件位置:"
        echo "  target/debug/lanchat-web"
        echo ""
        echo "运行方式:"
        echo "  ./target/debug/lanchat-web --port 8888"
        echo ""
        echo "或使用启动脚本:"
        echo "  cd .. && ./start-web.sh"
        echo ""
        echo "提示: 使用 './build-web.sh release' 编译优化版本"
    fi
fi
