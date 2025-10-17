# Claude Code 配置管理器 v1.4.0

> 🎉 一款基于 Tauri 和 Rust 构建的现代化 Claude Code 配置管理工具，为开发者提供便捷的 API 配置管理解决方案。

## ✨ 本版本更新

### 新增功能
- 🔄 **数据库自动迁移**: 智能检测并自动迁移旧版本数据库
  - 自动检测 accounts 表结构
  - 自动添加缺失的 model 字段
  - 零停机迁移，完全兼容旧版本
  - 无需手动操作，启动即自动完成

### 改进优化
- 🎯 **账号模型字段优化**:
  - 移除 model 字段的默认值，改为空字符串
  - 用户可按需为每个账号指定特定模型
  - 灵活配置，更符合实际使用场景

- 🔧 **界面体验改进**:
  - 修复编辑账号时 Base URL 预设下拉框不回填的问题
  - 正确显示当前账号的 Base URL 选项
  - 改进用户交互流程

- 🌍 **多语言支持完善**:
  - 优化 CLI 和 GUI 的多语言翻译
  - 改进语言切换体验
  - 更加一致的界面文案

### Bug 修复
- ✅ 修复编辑账号时 Base URL 下拉框值不正确的问题
- ✅ 修复旧版本用户升级后可能遇到的字段缺失问题
- ✅ 优化数据库初始化和迁移流程

## 📦 下载

### GUI 版本（图形界面）

适合需要可视化界面的用户：

| 平台 | 文件 | 说明 |
|------|------|------|
| 🪟 **Windows** | `claude-code-config-manager_1.4.0_x64_zh-CN.msi` | Windows 安装包（推荐） |
| 🪟 **Windows** | `claude-code-config-manager_1.4.0_x64-setup.exe` | NSIS 安装程序 |
| 🐧 **Linux** | `claude-code-config-manager_1.4.0_amd64.deb` | Debian/Ubuntu 包 |
| 🐧 **Linux** | `claude-code-config-manager_1.4.0_amd64.AppImage` | 通用 AppImage |
| 🍎 **macOS** | `claude-code-config-manager_1.4.0_aarch64.dmg` | macOS 安装镜像 (Apple Silicon) |

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
| 🪟 **Windows** | `claude-config-cli-1.4.0-Windows-x86_64.zip` | Windows 可执行文件 |
| 🐧 **Linux** | `claude-config-cli-1.4.0-Linux-x86_64.tar.gz` | Linux 可执行文件 |
| 🍎 **macOS** | `claude-config-cli-1.4.0-Darwin-x86_64.tar.gz` | macOS 可执行文件 |

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
sudo dpkg -i claude-code-config-manager_1.4.0_amd64.deb

# 如有依赖问题，运行
sudo apt-get install -f
```

**Linux (AppImage):**
```bash
# 下载 .AppImage 文件后
chmod +x claude-code-config-manager_1.4.0_amd64.AppImage
./claude-code-config-manager_1.4.0_amd64.AppImage
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
# 将 claude-config-windows.exe 复制到 PATH 中的任意目录
# 或直接运行
.\claude-config-windows.exe
```

**Linux/macOS:**
```bash
# 解压 tar.gz 文件
tar -xzf claude-config-cli-1.4.0-*.tar.gz

# 安装到系统（可选）
sudo cp claude-config-* /usr/local/bin/claude-config
sudo chmod +x /usr/local/bin/claude-config

# 运行
claude-config
```

## 🔄 升级说明

### 从 v1.3.0 升级

**好消息！** 本版本支持自动迁移，无需任何手动操作：

1. 直接安装 v1.4.0 版本（会覆盖旧版本）
2. 首次启动时会自动检测并迁移数据库
3. 所有数据完全保留，无需备份
4. 迁移完成后即可正常使用

**迁移内容**:
- ✅ 自动添加 model 字段到 accounts 表
- ✅ 保留所有现有账号和配置数据
- ✅ 更新数据库默认值设置

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
- 🎯 **模型配置** - 为每个账号指定特定的 Claude 模型
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
