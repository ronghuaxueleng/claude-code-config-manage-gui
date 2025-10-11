# Claude Code 配置管理器 - 命令行版本

🚀 一个基于 Rust 构建的高性能命令行工具，用于管理 Claude Code 的配置。支持账号管理、目录管理、配置切换和 WebDAV 云同步。

## ✨ 特性

### 🔐 账号管理
- 查看所有账号
- 添加新账号
- 编辑账号信息
- 删除账号

### 📁 目录管理
- 查看所有项目目录
- 添加新目录
- 编辑目录信息
- 删除目录（仅删除数据库记录）

### ⚡ 配置切换
- 快速切换不同账号和目录的配置
- 支持沙盒模式开关
- 自动更新 `.claude/settings.local.json`

### ☁️ WebDAV 云同步
- 管理多个 WebDAV 配置
- 上传配置到云端
- 从云端下载配置
- 查看远程文件列表
- 测试连接状态

### 📝 日志查看
- 查看最近日志
- 查看日志文件信息
- 快速打开日志目录

### 🔓 删除限制代码
- 一键删除 Claude Code 的 Root Check 限制
- 自动创建包装脚本，无侵入式修改
- 脚本内容编译时嵌入二进制，无需外部文件
- 自动处理 Windows/Linux 换行符兼容性
- 支持在服务器上以 root 运行 Claude Code

## 🛠️ 技术栈

- **Rust 2021 Edition** - 高性能系统编程语言
- **Tokio** - 异步运行时
- **SQLx** - 异步 SQL 工具包 (SQLite/MySQL)
- **Dialoguer** - 交互式命令行界面
- **Colored** - 终端彩色输出
- **Comfy-table** - 美观的表格显示
- **Reqwest-DAV** - WebDAV 客户端

## 📋 环境要求

- Rust 1.70+
- Linux / macOS / Windows

## 🚀 快速开始

### 1. 构建项目

**Linux/macOS:**
```bash
chmod +x build-cli.sh
./build-cli.sh
```

**Windows:**
```cmd
build-cli.bat
```

### 2. 运行程序

**Linux/macOS:**
```bash
cd claude-config-cli
./target/release/claude-config
```

**Windows:**
```cmd
cd claude-config-cli
.\target\release\claude-config.exe
```

### 3. 安装到系统（可选）

**Linux/macOS:**
```bash
sudo cp claude-config-cli/target/release/claude-config /usr/local/bin/
# 然后可以在任何地方运行
claude-config
```

**Windows:**
将 `claude-config.exe` 复制到 PATH 环境变量中的任意目录。

## 📖 使用说明

### 主菜单

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║        Claude Code 配置管理器 - 命令行版本 v1.2.0            ║
║        Claude Code Configuration Manager - CLI               ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

请选择操作:
  📋 账号管理
  📁 目录管理
  ⚡ 配置切换
  ☁️  WebDAV 同步
  📝 查看日志
  🔓 删除限制代码
  ❌ 退出程序
```

### 账号管理

1. **查看所有账号** - 以表格形式显示所有账号信息
2. **添加新账号** - 交互式添加新的 Claude API 账号
3. **编辑账号** - 修改现有账号的信息
4. **删除账号** - 删除不需要的账号

### 目录管理

1. **查看所有目录** - 显示所有项目目录及其状态
2. **添加新目录** - 添加新的项目目录到管理列表
3. **编辑目录** - 修改目录信息
4. **删除目录** - 从管理列表中移除目录（不删除实际文件）

### 配置切换

选择账号和目录，一键切换 Claude 配置：
- 自动更新 `.claude/settings.local.json`
- 可选择是否启用沙盒模式
- 支持多账号多目录快速切换

### WebDAV 同步

1. **查看 WebDAV 配置** - 显示所有云端配置
2. **添加 WebDAV 配置** - 配置坚果云、NextCloud 等 WebDAV 服务
3. **测试连接** - 验证 WebDAV 服务器连接状态
4. **上传配置到云端** - 备份当前配置到云端
5. **从云端下载配置** - 从云端恢复配置
6. **查看远程文件** - 列出云端存储的配置文件
7. **删除配置** - 移除 WebDAV 配置

### 删除限制代码

一键删除 Claude Code 的 Root Check 限制：
- 自动查找 `claude` 命令位置
- 创建包装脚本自动删除限制
- 备份原始命令
- 替换为包装脚本
- 支持在 root 用户下运行 Claude Code

## 📁 项目结构

```
claude-config-cli/
├── Cargo.toml              # 项目配置
├── src/
│   ├── main.rs            # 程序入口
│   ├── database.rs        # 数据库操作
│   ├── models.rs          # 数据模型
│   ├── config_manager.rs  # 配置管理
│   ├── claude_config.rs   # Claude 配置处理
│   ├── logger.rs          # 日志系统
│   ├── webdav.rs          # WebDAV 同步
│   └── menu/              # 菜单模块
│       ├── mod.rs
│       ├── account.rs     # 账号管理菜单
│       ├── directory.rs   # 目录管理菜单
│       ├── switch.rs      # 配置切换菜单
│       ├── webdav.rs      # WebDAV 菜单
│       └── logs.rs        # 日志查看菜单
└── resources/             # 资源文件
    ├── config.json
    └── init_db.sql
```

## 🗄️ 数据存储

数据库文件位置：
- Linux/macOS: `~/.claude-config-manager/claude_config.db`
- Windows: `%USERPROFILE%\.claude-config-manager\claude_config.db`

日志文件位置：
- Linux/macOS: `~/.claude-config-manager/logs/`
- Windows: `%USERPROFILE%\.claude-config-manager\logs\`

## 🔧 开发

### 编译 Debug 版本
```bash
cd claude-config-cli
cargo build
```

### 运行 Debug 版本
```bash
cargo run
```

### 运行测试
```bash
cargo test
```

### 代码格式化
```bash
cargo fmt
```

### 代码检查
```bash
cargo clippy
```

## 🆚 与 GUI 版本的区别

| 特性 | GUI 版本 | CLI 版本 |
|------|---------|---------|
| 界面 | 图形化界面 | 命令行交互 |
| 依赖 | Tauri + WebView | 纯 Rust |
| 启动速度 | ~0.5s | ~0.1s |
| 内存占用 | ~15MB | ~5MB |
| 包大小 | ~8MB | ~3MB |
| 跨平台 | ✓ | ✓ |
| 远程使用 | ✗ | ✓ (SSH) |
| 自动化 | 困难 | 容易 |
| 删除 Root 限制 | ✗ | ✓ |

## 📄 许可证

本项目采用 **MIT 许可证**，详见 [LICENSE](../LICENSE) 文件。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 💬 支持

- 🐛 Bug 报告: [GitHub Issues](../../issues)
- 💡 功能建议: [GitHub Discussions](../../discussions)

---

**⭐ 如果这个项目对你有帮助，请给我们一个 Star！**
