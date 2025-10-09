#!/bin/bash

echo "============================================="
echo "    Claude Config Manager 国内镜像构建脚本"
echo "============================================="
echo

# 检查基础环境
echo "[1/6] 检查构建环境..."
if ! command -v node &> /dev/null; then
    echo "❌ 错误: 未找到 Node.js，请先安装 Node.js"
    echo "下载地址: https://registry.npmmirror.com/binary.html?path=node/"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到 Rust/Cargo，请先安装 Rust"
    echo "下载地址: https://forge.rust-lang.org/infra/channel-layout.html#mirrors"
    exit 1
fi

# 配置 Rust 国内镜像
echo "[2/6] 配置 Rust 镜像源..."
if [ ! -f "$HOME/.cargo/config.toml" ]; then
    mkdir -p "$HOME/.cargo"
    cat > "$HOME/.cargo/config.toml" << 'EOF'
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

[net]
retry = 2
git-fetch-with-cli = true

[http]
timeout = 60
EOF
    echo "✅ 已配置 Rust 镜像源"
else
    echo "✅ Rust 镜像源已存在"
fi

# 配置 npm 国内镜像
echo "[3/6] 配置 npm 镜像源..."
if ! npm config get registry 2>/dev/null | grep -q "npmmirror.com"; then
    npm config set registry https://registry.npmmirror.com/
    echo "✅ 已配置 npm 镜像源"
else
    echo "✅ npm 镜像源已配置"
fi

# 安装依赖
echo "[4/6] 安装项目依赖..."
if [ ! -d "node_modules" ]; then
    echo "正在安装 npm 依赖..."
    if ! npm install; then
        echo "❌ npm 依赖安装失败"
        exit 1
    fi
    echo "✅ npm 依赖安装完成"
else
    echo "✅ npm 依赖已存在"
fi

# 设置构建环境变量
echo "[5/6] 配置构建环境..."

# WiX 工具下载镜像
export WIX_MIRROR="https://gh-proxy.com/https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip"

# 设置所有可能的 WiX 环境变量
export WIX3_DOWNLOAD_URL="$WIX_MIRROR"
export TAURI_WIX3_DOWNLOAD_URL="$WIX_MIRROR"
export TAURI_BUNDLE_WIX_DOWNLOAD_URL="$WIX_MIRROR"
export WIX_DOWNLOAD_URL="$WIX_MIRROR"

# Cargo 网络优化
export CARGO_HTTP_TIMEOUT=120
export CARGO_NET_RETRY=3
export CARGO_HTTP_MULTIPLEXING=false

echo "✅ 构建环境配置完成"
echo "    WiX 镜像: $WIX_MIRROR"
echo

# 开始构建
echo "[6/6] 开始构建..."
echo "这可能需要几分钟时间，请耐心等待..."
echo

npm run tauri build

# 检查构建结果
if [ $? -eq 0 ]; then
    echo
    echo "=========================================="
    echo "✅ 构建成功！"
    echo "=========================================="
    echo
    echo "📦 构建产物位置:"
    echo "    Linux: src-tauri/target/release/bundle/deb/"
    echo "    AppImage: src-tauri/target/release/bundle/appimage/"
    echo
    echo "🎉 可以在以上目录找到安装程序"
else
    echo
    echo "=========================================="
    echo "❌ 构建失败！"
    echo "=========================================="
    echo
    echo "🔧 故障排除建议:"
    echo "1. 检查网络连接"
    echo "2. 清理缓存: rm -rf node_modules && npm install"
    echo "3. 清理 Rust 缓存: cargo clean"
    echo "4. 检查 Rust 工具链: rustup update"
    echo
    echo "如果问题仍然存在，请查看上面的错误信息"
fi

echo
read -p "按任意键继续..."