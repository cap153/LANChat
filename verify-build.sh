#!/bin/bash
# 验证编译结果

set -e

echo "======================================"
echo "验证 LANChat 编译和功能"
echo "======================================"

cd src-tauri

echo ""
echo "1. 清理旧的编译..."
cargo clean

echo ""
echo "2. 编译 Web 端..."
cargo build --bin lanchat-web --features web --no-default-features
echo "✓ Web 端编译成功"

echo ""
echo "3. 编译桌面端..."
cargo build --bin lanchat --features desktop
echo "✓ 桌面端编译成功"

echo ""
echo "4. 检查 Web 端依赖..."
WEB_GUI=$(ldd target/debug/lanchat-web | grep -E "(webkit|gtk)" || true)
if [ -z "$WEB_GUI" ]; then
    echo "✓ Web 端无 GUI 依赖"
else
    echo "✗ Web 端包含 GUI 依赖:"
    echo "$WEB_GUI"
    exit 1
fi

echo ""
echo "5. 检查桌面端依赖..."
DESKTOP_GUI=$(ldd target/debug/lanchat | grep -E "(webkit|gtk)" || true)
if [ -n "$DESKTOP_GUI" ]; then
    echo "✓ 桌面端包含 GUI 依赖"
else
    echo "✗ 桌面端缺少 GUI 依赖"
    exit 1
fi

echo ""
echo "6. 检查文件大小..."
WEB_SIZE=$(du -h target/debug/lanchat-web | cut -f1)
DESKTOP_SIZE=$(du -h target/debug/lanchat | cut -f1)
echo "  Web 端: $WEB_SIZE"
echo "  桌面端: $DESKTOP_SIZE"

echo ""
echo "======================================"
echo "所有验证通过!"
echo "======================================"
echo ""
echo "可执行文件位置:"
echo "  Web 端: src-tauri/target/debug/lanchat-web"
echo "  桌面端: src-tauri/target/debug/lanchat"
echo ""
echo "运行示例:"
echo "  Web 端: ./src-tauri/target/debug/lanchat-web --port 8888"
echo "  桌面端: ./src-tauri/target/debug/lanchat"
