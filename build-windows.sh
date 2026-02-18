#!/bin/bash
# Windows 桌面端交叉编译脚本（在 Linux 上编译 Windows 版本）

set -e

echo "=== 编译 Windows 桌面端 ==="
echo "使用 cargo-xwin 交叉编译 MSVC target"
echo ""

cd src-tauri

# 使用 cargo-xwin 编译 Windows MSVC 版本
cargo xwin build --release --bin lanchat --target x86_64-pc-windows-msvc

echo ""
echo "=== 编译完成 ==="
ls -lh target/x86_64-pc-windows-msvc/release/lanchat.exe

# 复制到根目录
cp target/x86_64-pc-windows-msvc/release/lanchat.exe ../LANChat-Windows-x64.exe

echo ""
echo "✓ Windows 可执行文件: LANChat-Windows-x64.exe"
file ../LANChat-Windows-x64.exe
