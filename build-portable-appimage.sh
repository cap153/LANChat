#!/bin/bash

# 构建更兼容的 AppImage（排除系统库）

set -e

echo "=== 构建便携式 AppImage ==="
echo ""

# 1. 清理旧文件
echo "1. 清理旧文件..."
rm -rf src-tauri/target/release/bundle/appimage
rm -f LANChat-x86_64.AppImage
rm -rf squashfs-root

# 2. 构建 Tauri 应用
echo ""
echo "2. 构建 Tauri 应用..."
cd src-tauri
cargo build --release --features desktop
cd ..

# 3. 创建 AppDir 结构
echo ""
echo "3. 创建 AppDir 结构..."
APPDIR="src-tauri/target/release/bundle/appimage/LANChat.AppDir"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/lib"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

# 4. 复制二进制文件
echo "4. 复制应用文件..."
cp src-tauri/target/release/lanchat "$APPDIR/usr/bin/"

# 5. 创建 desktop 文件
cat > "$APPDIR/usr/share/applications/lanchat.desktop" << 'EOF'
[Desktop Entry]
Name=LANChat
Exec=lanchat
Icon=lanchat
Type=Application
Categories=Network;InstantMessaging;
Terminal=false
EOF

# 6. 复制图标
if [ -f "src-tauri/icons/icon.png" ]; then
    cp src-tauri/icons/icon.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/lanchat.png"
    cp src-tauri/icons/icon.png "$APPDIR/lanchat.png"
elif [ -f "src-tauri/icons/128x128.png" ]; then
    cp src-tauri/icons/128x128.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/lanchat.png"
    cp src-tauri/icons/128x128.png "$APPDIR/lanchat.png"
else
    echo "警告: 未找到图标文件"
fi

# 7. 创建 AppRun
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/lanchat" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# 8. 只复制必要的库（WebKit 相关）
echo ""
echo "5. 复制必要的库..."

# 使用 ldd 查找依赖，但只复制非系统库
LIBS_TO_COPY=(
    "libwebkit2gtk-4.1.so.0"
    "libjavascriptcoregtk-4.1.so.0"
)

for lib in "${LIBS_TO_COPY[@]}"; do
    LIB_PATH=$(ldconfig -p | grep "$lib" | awk '{print $NF}' | head -1)
    if [ -n "$LIB_PATH" ] && [ -f "$LIB_PATH" ]; then
        echo "  复制: $lib"
        cp "$LIB_PATH" "$APPDIR/usr/lib/"
    fi
done

# 9. 使用 linuxdeploy 但排除系统库
echo ""
echo "6. 使用 linuxdeploy 打包（排除系统库）..."

# 创建排除列表
EXCLUDE_LIST="libc.so.6,libm.so.6,libdl.so.2,libpthread.so.0,librt.so.1,libgcc_s.so.1,libstdc++.so.6"
EXCLUDE_LIST="$EXCLUDE_LIST,libz.so.1,libssl.so.3,libcrypto.so.3,libglib-2.0.so.0,libgobject-2.0.so.0"
EXCLUDE_LIST="$EXCLUDE_LIST,libgio-2.0.so.0,libgtk-3.so.0,libgdk-3.so.0,libcairo.so.2,libpango-1.0.so.0"
EXCLUDE_LIST="$EXCLUDE_LIST,libX11.so.6,libXext.so.6,libXrender.so.1,libfontconfig.so.1,libfreetype.so.6"

export LINUXDEPLOY_OUTPUT_VERSION=0.1.0
export ARCH=x86_64

# 使用系统的 linuxdeploy 或缓存的版本
if [ -f "/usr/bin/linuxdeploy" ]; then
    LINUXDEPLOY="/usr/bin/linuxdeploy"
else
    LINUXDEPLOY="$HOME/.cache/tauri/linuxdeploy-x86_64.AppImage"
fi

# 不使用 linuxdeploy，直接打包
echo ""
echo "7. 直接使用 appimagetool 打包..."

# 下载 appimagetool（如果需要）
if [ ! -f "$HOME/.cache/tauri/appimagetool-x86_64.AppImage" ]; then
    echo "  下载 appimagetool..."
    mkdir -p "$HOME/.cache/tauri"
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" \
        -O "$HOME/.cache/tauri/appimagetool-x86_64.AppImage"
    chmod +x "$HOME/.cache/tauri/appimagetool-x86_64.AppImage"
fi

# 打包
ARCH=x86_64 "$HOME/.cache/tauri/appimagetool-x86_64.AppImage" \
    --appimage-extract-and-run \
    "$APPDIR" \
    "LANChat-x86_64.AppImage"

echo ""
echo "=== 完成 ==="
ls -lh LANChat-x86_64.AppImage
echo ""
echo "这个 AppImage 应该在大多数 Linux 发行版上都能运行"
