# Claude Code 配置管理器 v1.3.0

> 🎉 一款基于 Tauri 和 Rust 构建的现代化 Claude Code 配置管理工具，为开发者提供便捷的 API 配置管理解决方案。

## ✨ 本版本更新

### 新增功能
- ☁️ **WebDAV 云同步**: 支持配置数据云端备份和多设备同步
  - 兼容坚果云、NextCloud、ownCloud 等 WebDAV 服务
  - 支持完全覆盖模式，避免冲突
  - 自动同步功能（可配置间隔）
  - 详细的同步日志记录

- 🔓 **删除 Root 限制**（CLI 版本）: 一键移除 Claude Code 的 Root Check 限制
  - 自动查找 claude 命令位置
  - 创建包装脚本，无侵入式修改
  - 脚本内容编译时嵌入，无需外部文件
  - 自动处理跨平台换行符兼容性

- 🚀 **脚本自动执行**: 切换账号时自动执行环境配置脚本
  - 支持 WSL 环境（Windows）
  - Unix 系统原生支持
  - 静默失败，不影响核心功能

### 改进优化
- 📝 **日志系统增强**: 分级日志记录，便于问题排查
- 🗄️ **数据库优化**:
  - 自动创建 WebDAV 相关表结构
  - 改进数据库迁移机制
  - 支持多重回退策略
- 🔧 **错误处理改进**:
  - 优化 WSL 命令检测
  - 静默处理非关键错误
  - 更详细的错误信息

### Bug 修复
- ✅ 修复数据库初始化失败导致的退出问题
- ✅ 修复 WebDAV 配置提示过多的问题
- ✅ 修复 Root Check 脚本 CRLF 行尾问题
- ✅ 修复 Linux 下的权限检查问题

## 📦 下载

### GUI 版本（图形界面）

适合需要可视化界面的用户：

| 平台 | 文件 | 说明 |
|------|------|------|
| 🪟 **Windows** | `claude-config-manager_1.3.0_x64_zh-CN.msi` | Windows 安装包（推荐） |
| 🪟 **Windows** | `claude-config-manager_1.3.0_x64-setup.exe` | NSIS 安装程序 |
| 🐧 **Linux** | `claude-config-manager_1.3.0_amd64.deb` | Debian/Ubuntu 包 |
| 🐧 **Linux** | `claude-config-manager_1.3.0_amd64.AppImage` | 通用 AppImage |
| 🍎 **macOS** | `claude-config-manager_1.3.0_x64.dmg` | macOS 安装镜像 |

**特点**:
- 🎨 现代化的图形界面
- ⚡ 启动时间 ~0.5s
- 💾 内存占用 ~15MB
- 📊 实时数据可视化

### CLI 版本（命令行）

适合服务器环境和命令行爱好者：

| 平台 | 文件 | 说明 |
|------|------|------|
| 🪟 **Windows** | `claude-config-cli-1.3.0-Windows-x86_64.zip` | Windows 可执行文件 |
| 🐧 **Linux** | `claude-config-cli-1.3.0-Linux-x86_64.tar.gz` | Linux 可执行文件 |
| 🍎 **macOS** | `claude-config-cli-1.3.0-Darwin-x86_64.tar.gz` | macOS 可执行文件 |

**特点**:
- 🚀 启动时间 ~0.1s
- 💾 内存占用 ~5MB
- 🔧 完美适配 SSH 远程使用
- 🤖 易于自动化脚本集成
- 🔓 支持删除 Root 限制

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
sudo dpkg -i claude-config-manager_1.3.0_amd64.deb

# 如有依赖问题，运行
sudo apt-get install -f
```

**Linux (AppImage):**
```bash
# 下载 .AppImage 文件后
chmod +x claude-config-manager_1.3.0_amd64.AppImage
./claude-config-manager_1.3.0_amd64.AppImage
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
tar -xzf claude-config-cli-1.3.0-*.tar.gz

# 安装到系统（可选）
sudo cp claude-config-* /usr/local/bin/claude-config
sudo chmod +x /usr/local/bin/claude-config

# 运行
claude-config
```

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
