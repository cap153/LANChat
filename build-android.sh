#!/bin/bash

# Android 构建脚本
# 用法: ./build-android.sh [dev|build]

set -e

# 设置颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== LANChat Android 构建脚本 ===${NC}"

# 检查 ANDROID_HOME
if [ -z "$ANDROID_HOME" ]; then
    echo -e "${RED}错误: ANDROID_HOME 环境变量未设置${NC}"
    echo "请在 ~/.bashrc 或 ~/.zshrc 中添加:"
    echo "export ANDROID_HOME=\$HOME/.android/Sdk"
    exit 1
fi

echo -e "${GREEN}✓ ANDROID_HOME: $ANDROID_HOME${NC}"

# 查找 NDK
NDK_VERSION="26.1.10909125"
NDK_PATH="$ANDROID_HOME/ndk/$NDK_VERSION"

if [ ! -d "$NDK_PATH" ]; then
    echo -e "${YELLOW}警告: 未找到 NDK $NDK_VERSION${NC}"
    # 尝试查找其他版本
    NDK_PATH=$(find "$ANDROID_HOME/ndk" -maxdepth 1 -type d | grep -v "^$ANDROID_HOME/ndk$" | head -n 1)
    if [ -z "$NDK_PATH" ]; then
        echo -e "${RED}错误: 未找到任何 NDK 版本${NC}"
        echo "请安装 Android NDK"
        exit 1
    fi
    echo -e "${YELLOW}使用 NDK: $NDK_PATH${NC}"
fi

export NDK_HOME="$NDK_PATH"
echo -e "${GREEN}✓ NDK_HOME: $NDK_HOME${NC}"

# 将 NDK 工具链添加到 PATH
TOOLCHAIN_PATH="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
if [ ! -d "$TOOLCHAIN_PATH" ]; then
    echo -e "${RED}错误: 未找到 NDK 工具链: $TOOLCHAIN_PATH${NC}"
    exit 1
fi

export PATH="$TOOLCHAIN_PATH:$PATH"
echo -e "${GREEN}✓ 工具链路径已添加到 PATH${NC}"

# 设置 OpenSSL 为 vendored 模式
export OPENSSL_STATIC=1
echo -e "${GREEN}✓ OpenSSL 配置为 vendored 模式${NC}"

# 检查 Rust Android 目标
echo -e "\n${YELLOW}检查 Rust Android 目标...${NC}"
if ! rustup target list --installed | grep -q "aarch64-linux-android"; then
    echo -e "${YELLOW}安装 aarch64-linux-android 目标...${NC}"
    rustup target add aarch64-linux-android
fi

# 确定构建模式
MODE="${1:-dev}"

echo -e "\n${GREEN}=== 开始构建 (模式: $MODE) ===${NC}"

cd src-tauri

if [ "$MODE" = "build" ]; then
    echo -e "${YELLOW}执行 Release 构建...${NC}"
    cargo tauri android build
elif [ "$MODE" = "dev" ]; then
    echo -e "${YELLOW}执行开发构建...${NC}"
    cargo tauri android dev
else
    echo -e "${RED}错误: 未知模式 '$MODE'${NC}"
    echo "用法: $0 [dev|build]"
    exit 1
fi

echo -e "\n${GREEN}=== 构建完成 ===${NC}"
