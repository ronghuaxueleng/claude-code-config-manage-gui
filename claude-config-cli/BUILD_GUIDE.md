# Claude Config CLI - 构建指南

## 🚀 快速开始

### 使用详细构建脚本（推荐）

这是一个全自动的构建脚本，会检查环境、安装依赖、配置镜像并编译项目。

```bash
# 进入 CLI 项目目录
cd claude-config-cli

# 运行构建脚本
./build-cli-full.sh
```

脚本将自动完成以下 8 个步骤：
1. ✅ 系统信息检测
2. ✅ 检查/安装 Rust 环境
3. ✅ 配置 Cargo 国内镜像
4. ✅ 检查/安装系统依赖
5. ✅ 验证项目目录
6. ✅ 可选清理旧构建
7. ✅ 编译项目（Release 模式）
8. ✅ 可选安装到系统

### 使用简单构建脚本

如果您的环境已经配置好，可以使用简化版本：

```bash
cd claude-config-cli
./build-cli.sh
```

### 手动编译

如果您喜欢手动控制：

```bash
cd claude-config-cli
cargo build --release
```

## 📝 重要说明

### 路径要求

⚠️ **重要**：构建脚本必须从 `claude-config-cli/` 目录内运行！

✅ **正确的做法**：
```bash
cd claude-config-cli
./build-cli-full.sh
```

❌ **错误的做法**：
```bash
# 不要从项目根目录运行
./claude-config-cli/build-cli-full.sh  # 这样会失败！
```

### 首次构建

首次编译可能需要 3-5 分钟，因为需要下载和编译所有依赖。后续编译会快得多（利用增量编译）。

### 系统要求

**Linux/WSL：**
- Rust 1.70+
- pkg-config
- OpenSSL 开发包（libssl-dev）
- GCC 或 Clang

**依赖会自动安装**：构建脚本会自动检测并安装缺失的依赖。

## 🎯 构建选项

### 完整构建（首次推荐）
```bash
cd claude-config-cli
./build-cli-full.sh
```

### 清理后重建
```bash
cd claude-config-cli
./build-cli-full.sh clean
./build-cli-full.sh
```

### 仅编译（已配置环境）
```bash
cd claude-config-cli
cargo build --release
```

### Debug 模式编译
```bash
cd claude-config-cli
cargo build
```

## 📦 编译产物

编译成功后，可执行文件位于：
```
claude-config-cli/target/release/claude-config
```

文件大小约 12MB。

## 🔧 安装选项

构建脚本会在最后询问是否安装：

**选项 1：系统安装（全局可用）**
```bash
# 安装到 /usr/local/bin
sudo cp target/release/claude-config /usr/local/bin/
# 然后在任何地方运行
claude-config
```

**选项 2：用户安装（仅当前用户）**
```bash
# 安装到 ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/claude-config ~/.local/bin/
# 添加到 PATH（如果还没有）
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
# 然后运行
claude-config
```

**选项 3：直接使用（无需安装）**
```bash
# 从项目目录运行
cd claude-config-cli
./target/release/claude-config
```

## 🐛 故障排查

### 问题 1：找不到 Cargo.toml
```
[ERROR] Cargo.toml 不存在
[ERROR] 请确保从 claude-config-cli 目录运行此脚本
```

**解决方案**：确保在正确的目录
```bash
pwd  # 应该显示 .../claude-code-config-manage-gui/claude-config-cli
ls Cargo.toml  # 应该能看到这个文件
```

### 问题 2：编译失败
```
error: linking with `cc` failed
```

**解决方案**：安装构建工具
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# 或者运行完整构建脚本（会自动安装）
./build-cli-full.sh
```

### 问题 3：依赖下载慢
```
Updating crates.io index
```

**解决方案**：配置国内镜像
```bash
# 完整构建脚本会自动配置
./build-cli-full.sh

# 或手动配置
cat > ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
EOF
```

### 问题 4：权限错误
```
Permission denied
```

**解决方案**：添加执行权限
```bash
chmod +x build-cli-full.sh
./build-cli-full.sh
```

## 💡 最佳实践

### 推荐工作流程

**首次构建：**
```bash
cd claude-config-cli
./build-cli-full.sh  # 自动处理一切
```

**日常开发：**
```bash
cd claude-config-cli
cargo build --release  # 快速编译
```

**清理重建：**
```bash
cd claude-config-cli
cargo clean
cargo build --release
```

### 编译优化

已在 Cargo 配置中启用：
- ✅ 增量编译
- ✅ 并行编译（4 个任务）
- ✅ Release 优化
- ✅ 网络重试（3 次）

## 📚 相关文档

- [README.md](README.md) - 完整项目文档
- [QUICKSTART.md](QUICKSTART.md) - 快速入门指南
- [../README.md](../README.md) - GUI 版本文档

## 🎉 构建成功后

运行程序：
```bash
# 方式 1：直接运行
./target/release/claude-config

# 方式 2：如果已安装到系统
claude-config

# 方式 3：从任何地方运行（使用绝对路径）
/path/to/claude-config-cli/target/release/claude-config
```

祝你使用愉快！🚀
