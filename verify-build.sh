#!/bin/bash

echo "========================================="
echo "LANChat 编译验证"
echo "========================================="
echo ""

cd src-tauri

echo "1. 检查 lanchat-web 编译..."
if cargo check --bin lanchat-web 2>&1 | grep -q "Finished"; then
    echo "✅ lanchat-web 编译检查通过"
else
    echo "❌ lanchat-web 编译检查失败"
    cargo check --bin lanchat-web
    exit 1
fi

echo ""
echo "2. 检查 lanchat 编译..."
if cargo check --bin lanchat 2>&1 | grep -q "Finished"; then
    echo "✅ lanchat 编译检查通过"
else
    echo "❌ lanchat 编译检查失败"
    cargo check --bin lanchat
    exit 1
fi

echo ""
echo "========================================="
echo "✅ 所有编译检查通过！"
echo "========================================="
echo ""
echo "下一步："
echo "  - 编译桌面端: cd src-tauri && cargo build --bin lanchat"
echo "  - 编译 Web 端: cd src-tauri && cargo build --bin lanchat-web"
echo "  - 运行 Web 端: ./start-web.sh"
