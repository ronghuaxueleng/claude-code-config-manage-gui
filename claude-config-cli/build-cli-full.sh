#!/bin/bash

# Claude Config CLI - 详细构建脚本
# 自动检查环境、安装依赖、配置镜像、编译和安装

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# 打印函数
print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${MAGENTA}▶ $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# 打印横幅
print_banner() {
    clear
    echo -e "${CYAN}"
    cat << "EOF"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║     Claude Code 配置管理器 - CLI 版本构建脚本                ║
║     Claude Code Configuration Manager - CLI Build Script     ║
║                                                               ║
║     版本: v1.2.0                                              ║
║     平台: Linux / macOS / WSL                                 ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}\n"
}

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 获取系统信息
get_system_info() {
    print_step "步骤 1/8: 系统信息检测"

    print_info "操作系统: $(uname -s)"
    print_info "内核版本: $(uname -r)"
    print_info "架构: $(uname -m)"

    if [ -f /etc/os-release ]; then
        . /etc/os-release
        print_info "发行版: $PRETTY_NAME"
    fi

    print_success "系统信息检测完成"
}

# 检查并安装 Rust
check_and_install_rust() {
    print_step "步骤 2/8: 检查 Rust 环境"

    if command_exists cargo && command_exists rustc; then
        RUST_VERSION=$(rustc --version)
        CARGO_VERSION=$(cargo --version)
        print_success "Rust 已安装"
        print_info "  $RUST_VERSION"
        print_info "  $CARGO_VERSION"
        return 0
    fi

    print_warning "Rust 未安装"
    echo ""
    read -p "是否自动安装 Rust? (推荐) [Y/n]: " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        print_info "正在下载并安装 Rust..."
        print_info "这可能需要几分钟时间..."

        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

        # 加载 Rust 环境
        export PATH="$HOME/.cargo/bin:$PATH"
        source "$HOME/.cargo/env" 2>/dev/null || true

        if command_exists cargo; then
            print_success "Rust 安装成功"
            rustc --version
            cargo --version
        else
            print_error "Rust 安装失败，请手动安装"
            echo ""
            echo "访问: https://rustup.rs/"
            exit 1
        fi
    else
        print_error "需要 Rust 环境才能继续"
        echo ""
        echo "请访问 https://rustup.rs/ 安装 Rust"
        exit 1
    fi
}

# 配置 Cargo 国内镜像
configure_cargo_mirrors() {
    print_step "步骤 3/8: 配置 Cargo 镜像源"

    local config_file="$HOME/.cargo/config.toml"

    if [ -f "$config_file" ]; then
        print_info "Cargo 配置文件已存在: $config_file"

        if grep -q "rsproxy" "$config_file"; then
            print_success "国内镜像已配置"
            return 0
        fi

        print_warning "配置文件存在但未配置镜像"
        read -p "是否备份并重新配置? [y/N]: " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            cp "$config_file" "$config_file.backup.$(date +%Y%m%d_%H%M%S)"
            print_info "已备份原配置文件"
        else
            print_warning "跳过镜像配置"
            return 0
        fi
    fi

    print_info "正在配置国内镜像源..."
    mkdir -p "$HOME/.cargo"

    cat > "$config_file" << 'EOF'
# Claude Config CLI - Cargo 配置
# 字节跳动镜像源 (rsproxy.cn)

[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

# 网络设置
[net]
git-fetch-with-cli = true
retry = 3

[http]
timeout = 60

# 编译优化
[build]
jobs = 4

# 增量编译
[build]
incremental = true
EOF

    print_success "国内镜像配置完成"
    print_info "  镜像源: https://rsproxy.cn/"
    print_info "  配置文件: $config_file"
}

# 检查系统依赖
check_system_dependencies() {
    print_step "步骤 4/8: 检查系统依赖"

    local missing_deps=()
    local optional_deps=()

    # 检查必需工具
    if ! command_exists pkg-config; then
        missing_deps+=("pkg-config")
    else
        print_success "pkg-config 已安装: $(pkg-config --version)"
    fi

    # 检查 OpenSSL 开发包
    if pkg-config --exists openssl 2>/dev/null; then
        local ssl_version=$(pkg-config --modversion openssl)
        print_success "OpenSSL 开发包已安装: $ssl_version"
    else
        if [ -f /etc/debian_version ]; then
            missing_deps+=("libssl-dev")
        elif [ -f /etc/redhat-release ]; then
            missing_deps+=("openssl-devel")
        elif [ -f /etc/arch-release ]; then
            missing_deps+=("openssl")
        else
            missing_deps+=("openssl-dev")
        fi
    fi

    # 检查编译器
    if command_exists gcc; then
        print_success "GCC 已安装: $(gcc --version | head -n1)"
    elif command_exists clang; then
        print_success "Clang 已安装: $(clang --version | head -n1)"
    else
        if [ -f /etc/debian_version ]; then
            missing_deps+=("build-essential")
        elif [ -f /etc/redhat-release ]; then
            missing_deps+=("gcc" "gcc-c++" "make")
        elif [ -f /etc/arch-release ]; then
            missing_deps+=("base-devel")
        else
            missing_deps+=("gcc")
        fi
    fi

    # 检查可选依赖
    if ! command_exists git; then
        optional_deps+=("git")
    fi

    # 处理缺失的依赖
    if [ ${#missing_deps[@]} -eq 0 ]; then
        print_success "所有必需依赖已安装"
    else
        print_warning "缺少以下必需依赖: ${missing_deps[*]}"
        echo ""
        read -p "是否自动安装? (需要 sudo 权限) [Y/n]: " -n 1 -r
        echo ""

        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            install_system_dependencies "${missing_deps[@]}"
        else
            print_error "缺少必需依赖，无法继续"
            exit 1
        fi
    fi

    # 处理可选依赖
    if [ ${#optional_deps[@]} -gt 0 ]; then
        print_info "建议安装以下工具: ${optional_deps[*]}"
    fi
}

# 安装系统依赖
install_system_dependencies() {
    local deps=("$@")

    print_info "正在安装系统依赖..."

    if [ -f /etc/debian_version ]; then
        # Debian/Ubuntu
        print_info "检测到 Debian/Ubuntu 系统"
        sudo apt update
        sudo apt install -y "${deps[@]}"
    elif [ -f /etc/redhat-release ]; then
        # RedHat/CentOS/Fedora
        print_info "检测到 RedHat/CentOS/Fedora 系统"
        if command_exists dnf; then
            sudo dnf install -y "${deps[@]}"
        else
            sudo yum install -y "${deps[@]}"
        fi
    elif [ -f /etc/arch-release ]; then
        # Arch Linux
        print_info "检测到 Arch Linux 系统"
        sudo pacman -S --noconfirm "${deps[@]}"
    else
        print_error "不支持的 Linux 发行版"
        print_info "请手动安装: ${deps[*]}"
        exit 1
    fi

    print_success "系统依赖安装完成"
}

# 检查项目目录
check_project_directory() {
    print_step "步骤 5/8: 检查项目目录"

    # 获取脚本所在目录（即 claude-config-cli 目录）
    local script_dir="$(cd "$(dirname "$0")" && pwd)"

    # 检查是否在正确的目录
    if [ ! -f "$script_dir/Cargo.toml" ]; then
        print_error "Cargo.toml 不存在"
        print_error "请确保从 claude-config-cli 目录运行此脚本"
        exit 1
    fi

    print_success "项目目录检查通过"
    print_info "  项目路径: $script_dir"

    # 显示项目信息
    cd "$script_dir"
    local project_name=$(grep '^name' Cargo.toml | head -n1 | cut -d'"' -f2)
    local project_version=$(grep '^version' Cargo.toml | head -n1 | cut -d'"' -f2)

    print_info "  项目名称: $project_name"
    print_info "  项目版本: $project_version"
}

# 清理旧的构建
clean_old_build() {
    print_step "步骤 6/8: 清理旧构建 (可选)"

    # 获取脚本所在目录
    local script_dir="$(cd "$(dirname "$0")" && pwd)"
    cd "$script_dir"

    if [ -d "target" ]; then
        local target_size=$(du -sh target 2>/dev/null | awk '{print $1}')
        print_info "发现旧的构建目录 (大小: $target_size)"

        read -p "是否清理? (可加快编译但会丢失缓存) [y/N]: " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_info "正在清理..."
            cargo clean
            print_success "清理完成"
        else
            print_info "保留旧构建 (利用增量编译)"
        fi
    else
        print_info "无需清理"
    fi
}

# 编译项目
build_project() {
    print_step "步骤 7/8: 编译项目"

    # 获取脚本所在目录
    local script_dir="$(cd "$(dirname "$0")" && pwd)"
    cd "$script_dir"

    # 确保环境变量已加载
    export PATH="$HOME/.cargo/bin:$PATH"
    source "$HOME/.cargo/env" 2>/dev/null || true

    echo ""
    print_info "编译配置:"
    print_info "  模式: Release (优化版本)"
    print_info "  目标: $(rustc -vV | grep host | awk '{print $2}')"
    print_info "  并行任务: 4"
    echo ""

    print_warning "首次编译可能需要 3-5 分钟，请耐心等待..."
    echo ""

    # 显示编译进度
    local start_time=$(date +%s)

    if cargo build --release 2>&1 | tee /tmp/cargo-build.log; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))

        echo ""
        print_success "编译成功！"
        print_info "  耗时: ${duration} 秒"

        # 显示可执行文件信息
        if [ -f "target/release/claude-config" ]; then
            local exe_size=$(du -h target/release/claude-config | awk '{print $1}')
            local exe_path=$(realpath target/release/claude-config)

            echo ""
            print_info "可执行文件信息:"
            print_info "  路径: $exe_path"
            print_info "  大小: $exe_size"
            print_info "  类型: $(file -b target/release/claude-config)"
        fi

        return 0
    else
        echo ""
        print_error "编译失败"
        print_info "查看详细日志: /tmp/cargo-build.log"

        echo ""
        print_warning "故障排除建议:"
        echo "  1. 检查网络连接"
        echo "  2. 重新配置镜像: rm ~/.cargo/config.toml"
        echo "  3. 清理缓存: cargo clean"
        echo "  4. 更新工具链: rustup update"

        exit 1
    fi
}

# 安装到系统
install_to_system() {
    print_step "步骤 8/8: 安装 (可选)"

    # 获取脚本所在目录
    local script_dir="$(cd "$(dirname "$0")" && pwd)"
    local exe_path="$script_dir/target/release/claude-config"

    if [ ! -f "$exe_path" ]; then
        print_error "可执行文件不存在"
        return 1
    fi

    echo ""
    print_info "安装选项:"
    echo "  1. 安装到系统 (/usr/local/bin) - 全局可用"
    echo "  2. 安装到用户目录 (~/.local/bin) - 仅当前用户"
    echo "  3. 跳过安装 - 手动使用"
    echo ""

    read -p "请选择 [1/2/3]: " -n 1 -r
    echo ""

    case $REPLY in
        1)
            print_info "正在安装到 /usr/local/bin..."
            if sudo cp "$exe_path" /usr/local/bin/claude-config; then
                sudo chmod +x /usr/local/bin/claude-config
                print_success "安装完成！"
                print_info "现在可以在任何地方运行: claude-config"
            else
                print_error "安装失败"
            fi
            ;;
        2)
            print_info "正在安装到 ~/.local/bin..."
            mkdir -p ~/.local/bin
            cp "$exe_path" ~/.local/bin/claude-config
            chmod +x ~/.local/bin/claude-config
            print_success "安装完成！"
            print_info "确保 ~/.local/bin 在 PATH 中"

            if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                print_warning "请添加以下行到 ~/.bashrc 或 ~/.zshrc:"
                echo '  export PATH="$HOME/.local/bin:$PATH"'
            fi
            ;;
        3)
            print_info "跳过安装"
            ;;
        *)
            print_warning "无效选择，跳过安装"
            ;;
    esac
}

# 显示使用说明
show_usage_guide() {
    echo ""
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}                    构建完成！                              ${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    echo ""

    # 获取脚本所在目录
    local script_dir="$(cd "$(dirname "$0")" && pwd)"
    local exe_path="$script_dir/target/release/claude-config"

    echo -e "${CYAN}📦 可执行文件位置:${NC}"
    echo "   $exe_path"
    echo ""

    echo -e "${CYAN}🚀 运行方式:${NC}"
    echo ""
    echo "   方式 1: 直接运行"
    echo -e "   ${YELLOW}cd $(basename "$script_dir")${NC}"
    echo -e "   ${YELLOW}./target/release/claude-config${NC}"
    echo ""

    if [ -f /usr/local/bin/claude-config ]; then
        echo "   方式 2: 全局命令 (已安装)"
        echo -e "   ${YELLOW}claude-config${NC}"
        echo ""
    fi

    echo -e "${CYAN}✨ 功能特性:${NC}"
    echo "   • 📋 账号管理 - 增删改查 Claude API 账号"
    echo "   • 📁 目录管理 - 管理项目目录"
    echo "   • ⚡ 配置切换 - 一键切换账号和目录"
    echo "   • ☁️  WebDAV 同步 - 云端备份和多设备同步"
    echo "   • 📝 日志查看 - 查看应用日志"
    echo ""

    echo -e "${CYAN}📚 文档:${NC}"
    echo "   • README: $(basename "$script_dir")/README.md"
    echo "   • 快速入门: $(basename "$script_dir")/QUICKSTART.md"
    echo ""

    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
}

# 清理函数
cleanup() {
    print_step "清理构建产物"

    # 获取脚本所在目录
    local script_dir="$(cd "$(dirname "$0")" && pwd)"
    cd "$script_dir"

    if [ -d "target" ]; then
        local size=$(du -sh target 2>/dev/null | awk '{print $1}')
        print_info "将清理 $size 的构建文件"

        cargo clean
        print_success "清理完成"
    else
        print_info "没有需要清理的内容"
    fi
}

# 显示帮助
show_help() {
    print_banner
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  (无)        完整构建流程"
    echo "  clean       清理构建产物"
    echo "  --help, -h  显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0              # 执行完整构建"
    echo "  $0 clean        # 清理构建文件"
    echo ""
}

# 主函数
main() {
    # 解析命令行参数
    case "${1:-}" in
        clean)
            print_banner
            cleanup
            exit 0
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        "")
            # 正常构建流程
            ;;
        *)
            print_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac

    # 执行构建流程
    print_banner
    get_system_info
    check_and_install_rust
    configure_cargo_mirrors
    check_system_dependencies
    check_project_directory
    clean_old_build
    build_project
    install_to_system
    show_usage_guide

    echo ""
    print_success "所有步骤完成！"
    echo ""
}

# 捕获错误
trap 'print_error "脚本执行失败"; exit 1' ERR

# 运行主函数
main "$@"
