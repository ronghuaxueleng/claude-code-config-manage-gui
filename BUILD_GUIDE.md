# build.mjs 使用说明

这是 Claude Config Manager 的 Node.js 构建脚本，提供了比原生 shell 脚本更好的跨平台支持和更多功能。

## 功能特性

- ✅ 跨平台支持（Windows、Linux、macOS）
- ✅ 自动环境检查
- ✅ 国内镜像源配置（可选）
- ✅ 智能依赖管理
- ✅ 多种构建选项
- ✅ 彩色输出和进度提示
- ✅ 详细的错误信息

## 快速开始

### 基本用法

```bash
# 使用 npm
npm run build

# 或直接运行
node build.mjs
```

### 构建选项

```bash
# 构建 debug 版本
npm run build:debug
# 或
node build.mjs --debug

# 清理缓存后构建
npm run build:clean
# 或
node build.mjs --clean

# 只构建 MSI 安装包 (Windows)
npm run build:msi
# 或
node build.mjs --target=msi

# 只构建 NSIS 安装包 (Windows)
npm run build:nsis
# 或
node build.mjs --target=nsis

# 不使用国内镜像源
node build.mjs --no-mirror

# 查看帮助
npm run build:help
# 或
node build.mjs --help
```

## 命令行选项

| 选项 | 说明 |
|------|------|
| `--debug` | 构建 debug 版本（包含调试信息） |
| `--target=<type>` | 指定构建目标（nsis, msi, deb, appimage, dmg 等） |
| `--no-mirror` | 不使用国内镜像源 |
| `--clean` | 清理构建缓存后再构建 |
| `-h, --help` | 显示帮助信息 |

## 构建流程

脚本会自动执行以下步骤：

1. **环境检查** - 验证 Node.js 和 Rust 是否已安装
2. **镜像配置** - 配置 Rust 和 npm 镜像源（国内加速）
3. **依赖安装** - 安装项目所需的 npm 依赖
4. **环境配置** - 设置构建所需的环境变量
5. **执行构建** - 调用 Tauri 进行构建
6. **结果输出** - 显示构建产物的位置

## 构建产物位置

构建完成后，安装包会生成在以下位置：

### Windows
- **NSIS**: `src-tauri/target/release/bundle/nsis/`
- **MSI**: `src-tauri/target/release/bundle/msi/`

### Linux
- **DEB**: `src-tauri/target/release/bundle/deb/`
- **AppImage**: `src-tauri/target/release/bundle/appimage/`

### macOS
- **DMG**: `src-tauri/target/release/bundle/dmg/`
- **App**: `src-tauri/target/release/bundle/macos/`

## 环境要求

- **Node.js** >= 16.0.0
- **Rust** >= 1.70.0
- **Cargo**（随 Rust 一起安装）

### Windows 额外要求
- Visual Studio Build Tools 或 Visual Studio
- WebView2 Runtime（Windows 10/11 已预装）

### Linux 额外要求
```bash
# Debian/Ubuntu
sudo apt-get install -y libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel

# Arch
sudo pacman -S webkit2gtk base-devel curl wget file openssl appmenu-gtk-module gtk3 libappindicator-gtk3 librsvg
```

## 镜像源配置

默认情况下，脚本会自动配置以下镜像源：

- **Rust**: rsproxy.cn
- **npm**: registry.npmmirror.com

如果不需要镜像源，可以使用 `--no-mirror` 选项。

## 故障排除

### 构建失败

1. **网络问题**
   ```bash
   # 使用国内镜像
   node build.mjs
   ```

2. **缓存损坏**
   ```bash
   # 清理缓存后重新构建
   node build.mjs --clean
   ```

3. **Rust 版本过旧**
   ```bash
   # 更新 Rust 工具链
   rustup update stable
   ```

4. **依赖安装失败**
   ```bash
   # 删除 node_modules 重新安装
   rm -rf node_modules
   npm install
   ```

### 常见错误

- **"未找到 Node.js"**: 请安装 Node.js 16 或更高版本
- **"未找到 Rust/Cargo"**: 请安装 Rust 工具链
- **"WebView2 not found"** (Windows): 安装 WebView2 Runtime
- **"Package not found"** (Linux): 安装所需的系统依赖包

## 示例

### 完整构建流程
```bash
# 1. 克隆项目
git clone https://github.com/your-repo/claude-code-config-manage-gui.git
cd claude-code-config-manage-gui

# 2. 运行构建脚本
node build.mjs

# 3. 查找构建产物
# Windows: src-tauri/target/release/bundle/nsis/
# Linux: src-tauri/target/release/bundle/deb/
```

### 快速开发构建
```bash
# 构建 debug 版本（更快，用于测试）
node build.mjs --debug
```

### 发布构建
```bash
# 清理后构建 release 版本
node build.mjs --clean
```

## 与 build.sh 的区别

| 特性 | build.sh | build.mjs |
|------|----------|-----------|
| 跨平台支持 | ❌ (仅 Unix-like) | ✅ 全平台 |
| 彩色输出 | ✅ | ✅ |
| 参数解析 | ⚠️ 有限 | ✅ 完整 |
| 错误处理 | ⚠️ 基础 | ✅ 详细 |
| 包管理器检测 | ❌ | ✅ (pnpm/npm) |
| 构建目标选择 | ❌ | ✅ |
| 帮助文档 | ❌ | ✅ |

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个构建脚本！

## 许可证

与主项目相同
