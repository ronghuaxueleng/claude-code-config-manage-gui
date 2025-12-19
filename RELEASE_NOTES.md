# Claude Code 配置管理器 v1.7.0

> 🎉 一款基于 Tauri 和 Rust 构建的现代化 Claude Code 配置管理工具，为开发者提供便捷的 API 配置管理解决方案。

## ✨ 本版本更新

### 新增功能

- 📦 **账号批量导入导出**: 支持账号数据的批量导入和导出
  - 添加选择性导出功能，可选择特定账号导出
  - 批量导出时支持 URL 筛选功能
  - 支持自定义保存位置选择
  - 完善账号导入时的重复检测逻辑

- 🌐 **宿主机 IP 功能**: 账号切换时支持使用宿主机 IP
  - GUI 版本添加宿主机 IP 切换选项
  - 优化 URL 显示效果

- 🛡️ **CLAUDE.local.md 文件保护**:
  - 切换账号时检测并询问是否保留现有 CLAUDE.local.md 文件
  - 覆盖前自动备份现有文件

- 🔧 **环境变量增强**:
  - 创建配置文件时自动添加 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC
  - 拆分为独立环境变量：DISABLE_BUG_COMMAND、DISABLE_ERROR_REPORTING、DISABLE_TELEMETRY
  - 不再禁用自动更新功能

- 📁 **Commands 目录支持**: 自动打包和释放 commands 目录下的所有文件

- 🛠️ **Rust 安装脚本**: 添加国内镜像一键安装脚本，支持多镜像源选择

- ⚡ **CLI 增强**:
  - 启动时自动设置 hasCompletedOnboarding
  - 构建脚本自动同步版本号
  - 配置切换时添加 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC

### Bug 修复

- ✅ 修复批量导入账号时参数传递错误导致创建失败的问题
- ✅ 修复导出文件时的 Tauri API 错误
- ✅ 修复批量导出账号功能并优化 URL 显示
- ✅ 修复 token 匹配时因空格导致的匹配失败问题
- ✅ 修复账号切换时未添加 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC 配置的问题
- ✅ 修复 Cargo 源名称冲突问题
- ✅ 添加 Tauri Bundler 工具 GitHub 镜像配置解决 NSIS 下载超时
- ✅ CLI: 修复数据库迁移顺序和版本号显示
- ✅ CLI: 修复 base_urls 表 api_key 字段数据库迁移问题
- ✅ CLI: 添加 accounts 表 custom_env_vars 字段支持

### 改进优化

- 🔄 将 Cargo 镜像源切换为官方源
- 🗑️ 移除未使用的 update_env_config_with_options 方法
- 📝 提交代码时不再附带 Co-Authored-By: Claude 信息

## 📦 下载

### GUI 版本（图形界面）

适合需要可视化界面的用户：

| 平台 | 文件 | 说明 |
|------|------|------|
| 🪟 **Windows** | `claude-code-config-manager_1.7.0_x64_zh-CN.msi` | Windows 安装包（推荐） |
| 🪟 **Windows** | `claude-code-config-manager_1.7.0_x64-setup.exe` | NSIS 安装程序 |
| 🐧 **Linux** | `claude-code-config-manager_1.7.0_amd64.deb` | Debian/Ubuntu 包 |
| 🐧 **Linux** | `claude-code-config-manager_1.7.0_amd64.AppImage` | 通用 AppImage |
| 🍎 **macOS** | `claude-code-config-manager_1.7.0_aarch64.dmg` | macOS 安装镜像 (Apple Silicon) |

**特点**:
- 🎨 现代化的图形界面
- ⚡ 启动时间 ~0.5s
- 💾 内存占用 ~15MB
- 📊 实时数据可视化
- 🔄 自动数据库迁移

### CLI 版本（命令行）

适合服务器环境和命令行爱好者：

| 平台 | 文件 | 说明 |
|------|------|------|
| 🪟 **Windows** | `claude-config-cli-1.7.0-Windows-x86_64.zip` | Windows 可执行文件 |
| 🐧 **Linux** | `claude-config-cli-1.7.0-Linux-x86_64.tar.gz` | Linux 可执行文件 |
| 🍎 **macOS** | `claude-config-cli-1.7.0-Darwin-x86_64.tar.gz` | macOS 可执行文件 |

**特点**:
- 🚀 启动时间 ~0.1s
- 💾 内存占用 ~5MB
- 🔧 完美适配 SSH 远程使用
- 🤖 易于自动化脚本集成
- 🔓 支持删除 Root 限制
- 🔄 自动数据库迁移

## 📝 安装说明

### GUI 版本

**Windows:**
1. 下载 `.msi` 文件
2. 双击运行安装程序
3. 按照向导完成安装
4. 从开始菜单启动应用

**Linux (Debian/Ubuntu):**
```bash
# 下载 .deb 文件后
sudo dpkg -i claude-code-config-manager_1.7.0_amd64.deb

# 如有依赖问题，运行
sudo apt-get install -f
```

**Linux (AppImage):**
```bash
# 下载 .AppImage 文件后
chmod +x claude-code-config-manager_1.7.0_amd64.AppImage
./claude-code-config-manager_1.7.0_amd64.AppImage
```

**macOS:**
1. 下载 `.dmg` 文件
2. 打开 DMG 镜像
3. 将应用拖到 Applications 文件夹
4. 首次运行需要在"系统偏好设置" → "安全性与隐私"中允许

### CLI 版本

**Windows:**
```cmd
# 解压 ZIP 文件
# 将 claude-config.exe 复制到 PATH 中的任意目录
# 或直接运行
.\claude-config.exe
```

**Linux/macOS:**
```bash
# 解压 tar.gz 文件
tar -xzf claude-config-cli-1.7.0-*.tar.gz

# 安装到系统（可选）
sudo cp claude-config /usr/local/bin/claude-config
sudo chmod +x /usr/local/bin/claude-config

# 运行
claude-config
```

## 🔄 升级说明

### 从 v1.6.x 升级

**好消息！** 本版本支持自动迁移，无需任何手动操作：

1. 直接安装 v1.7.0 版本（会覆盖旧版本）
2. 首次启动时会自动检测并迁移数据库
3. 所有数据完全保留，无需备份
4. 迁移完成后即可正常使用

**迁移内容**:
- ✅ 自动添加 custom_env_vars 字段到 accounts 表
- ✅ 保留所有现有账号和配置数据
- ✅ 兼容旧版本 CLAUDE.local.md 文件

## 🚀 快速开始

### 首次使用

1. **启动应用**
2. **添加账号**: 在"账号管理"页面添加你的 Claude API Token
3. **添加目录**: 在"目录管理"页面添加项目目录
4. **配置切换**: 在"配置切换"页面关联账号和目录，一键切换
5. **云端备份**（可选）: 配置 WebDAV 实现多设备同步

### 核心功能

- 🔐 **多账号管理** - 管理多个 Claude API 账号
- 📁 **目录管理** - 管理多个项目目录
- ⚡ **一键切换** - 快速切换不同项目的配置
- 📦 **批量导入导出** - 账号数据的批量管理
- 🌐 **URL 管理** - 管理不同的 API 端点
- 🗄️ **数据库管理** - SQLite/MySQL 双数据库支持
- ☁️ **WebDAV 同步** - 云端备份和多设备同步
- 🛠️ **高级配置** - Claude 权限和环境变量管理

## 🔧 系统要求

### GUI 版本
- **Windows**: Windows 10/11 (64位)
- **Linux**:
  - Debian 10+, Ubuntu 18.04+
  - WebKitGTK 4.0+
- **macOS**: macOS 10.15+

### CLI 版本
- **所有平台**: 64位操作系统
- **Linux**: glibc 2.27+

## 📚 文档

- [完整文档](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/blob/main/README.md)
- [CLI 使用指南](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/blob/main/claude-config-cli/README.md)
- [构建指南](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/blob/main/claude-config-cli/BUILD_GUIDE.md)
- [更新日志](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/blob/main/CHANGELOG.md)

## 🐛 已知问题

- Windows 下首次运行可能需要安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
- Linux 下需要安装 `libwebkit2gtk-4.0-dev` 依赖
- macOS 首次运行需要在系统设置中授权

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/blob/main/LICENSE) 文件。

---

## 🙏 感谢

感谢所有为这个项目做出贡献的开发者和用户！

如果这个项目对你有帮助，请给我们一个 ⭐ Star！

## 📞 支持

- 🐛 **Bug 报告**: [GitHub Issues](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/issues)
- 💡 **功能建议**: [GitHub Discussions](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/discussions)
