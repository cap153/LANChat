#!/bin/bash

# APK 签名脚本

KEYSTORE="lanchat-release.keystore"
KEYSTORE_ALIAS="lanchat"
APK_UNSIGNED="src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"
APK_SIGNED="lanchat-release.apk"

echo "=== LANChat APK 签名工具 ==="
echo ""

# 检查是否存在密钥库
if [ ! -f "$KEYSTORE" ]; then
    echo "密钥库不存在，正在创建..."
    echo "请按提示输入信息（密码建议记住，后续签名需要用到）"
    echo ""
    
    keytool -genkey -v \
        -keystore "$KEYSTORE" \
        -alias "$KEYSTORE_ALIAS" \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -storepass android \
        -keypass android \
        -dname "CN=LANChat, OU=Dev, O=LANChat, L=City, S=State, C=CN"
    
    if [ $? -ne 0 ]; then
        echo "❌ 密钥库创建失败"
        exit 1
    fi
    
    echo "✓ 密钥库创建成功: $KEYSTORE"
    echo ""
fi

# 检查 unsigned APK 是否存在
if [ ! -f "$APK_UNSIGNED" ]; then
    echo "❌ 找不到未签名的 APK: $APK_UNSIGNED"
    echo "请先运行: cargo tauri android build"
    exit 1
fi

echo "正在签名 APK..."
echo "输入文件: $APK_UNSIGNED"
echo "输出文件: $APK_SIGNED"
echo ""

# 使用 apksigner 签名（Android SDK 自带）
APKSIGNER="$ANDROID_HOME/build-tools/$(ls $ANDROID_HOME/build-tools | tail -1)/apksigner"

if [ ! -f "$APKSIGNER" ]; then
    echo "❌ 找不到 apksigner 工具"
    echo "请确保已安装 Android SDK build-tools"
    exit 1
fi

# 签名 APK
$APKSIGNER sign \
    --ks "$KEYSTORE" \
    --ks-key-alias "$KEYSTORE_ALIAS" \
    --ks-pass pass:android \
    --key-pass pass:android \
    --out "$APK_SIGNED" \
    "$APK_UNSIGNED"

if [ $? -ne 0 ]; then
    echo "❌ APK 签名失败"
    exit 1
fi

echo ""
echo "✓ APK 签名成功！"
echo ""
echo "签名后的 APK: $APK_SIGNED"
echo "文件大小: $(du -h $APK_SIGNED | cut -f1)"
echo ""
echo "现在可以安装到设备："
echo "  adb install $APK_SIGNED"
echo ""
echo "或者直接推送到设备："
echo "  adb push $APK_SIGNED /sdcard/Download/"
