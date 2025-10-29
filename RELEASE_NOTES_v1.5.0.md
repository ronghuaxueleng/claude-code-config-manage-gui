# Claude Code 配置管理器 v1.5.0

## 🎉 新版本发布

支持为不同的 Base URL 配置不同的 API Key 环境变量名，提供更灵活的 API 配置方案。

---

## ✨ 主要更新

### 🔑 自定义 API Key 环境变量名
- **核心功能**: 为每个 Base URL 配置独立的环境变量名（如 `ANTHROPIC_API_KEY`、`CLAUDE_API_KEY` 等）
- **GUI 支持**: 完整的添加、编辑、显示功能
- **CLI 支持**: 命令行版本同步支持 API Key 管理
- **WebDAV 同步**: 云端备份和恢复时完整保留 API Key 配置

### 🐛 Bug 修复
- 修复 GUI 版本 URL 更新时 `api_key` 不生效的问题
- 修复前端 Tauri 命令参数命名不匹配（蛇形命名 vs 驼峰命名）

### 🌍 国际化增强
- 添加 API Key 字段完整的中英文翻译
- 改进用户界面的多语言体验

### 📦 数据库优化
- `base_urls` 表新增 `api_key` 字段，默认值为 `ANTHROPIC_API_KEY`
- 自动数据库迁移，无需手动操作

### 📝 文档完善
- 新增 `CHANGELOG.md` 详细记录版本变更
- 更新 `README.md` 反映最新功能

---

## 📥 下载安装

### Windows
- **安装包**: `claude-code-config-manager_1.5.0_x64_zh-CN.msi` (推荐)
- **便携版**: `claude-code-config-manager_1.5.0_x64.exe`

### macOS
- **Intel**: `claude-code-config-manager_1.5.0_x64.dmg`
- **Apple Silicon**: `claude-code-config-manager_1.5.0_aarch64.dmg`

### Linux
- **Debian/Ubuntu**: `claude-code-config-manager_1.5.0_amd64.deb`
- **AppImage**: `claude-code-config-manager_1.5.0_amd64.AppImage`

### CLI 版本
- **Windows**: `claude-config-cli-windows-x64.exe`
- **macOS**: `claude-config-cli-macos-x64` / `claude-config-cli-macos-arm64`
- **Linux**: `claude-config-cli-linux-x64`

---

## 🚀 使用指南

### 配置自定义 API Key 环境变量

1. **在 URL 管理页面**，添加或编辑 Base URL
2. 在 **"API Key 环境变量名"** 字段中输入自定义变量名
   - 官方 API: `ANTHROPIC_API_KEY` (默认)
   - 第三方 API: `CLAUDE_API_KEY` 或其他自定义名称
3. 保存后，切换到该 URL 的账号时会自动使用对应的环境变量名

### 示例场景

```javascript
// 官方 API
{
  "name": "Anthropic Official",
  "url": "https://api.anthropic.com",
  "api_key": "ANTHROPIC_API_KEY"
}

// 第三方 API
{
  "name": "Third Party",
  "url": "https://api.example.com",
  "api_key": "CLAUDE_API_KEY"
}
```

切换账号时，会自动生成对应的配置：
```json
{
  "env": {
    "ANTHROPIC_API_KEY": "sk-ant-xxx...",  // 或 CLAUDE_API_KEY
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com"
  }
}
```

---

## 🔄 升级说明

### 从 v1.4.0 升级

1. **自动数据库迁移**: 启动时自动为 `base_urls` 表添加 `api_key` 字段
2. **现有数据保留**: 所有现有 URL 自动设置 `api_key` 为 `ANTHROPIC_API_KEY`
3. **配置兼容性**: 完全向后兼容，无需手动修改配置

### 注意事项

- **首次启动**: 可能需要几秒钟完成数据库迁移
- **备份建议**: 升级前建议使用 WebDAV 同步备份配置
- **CLI 版本**: 需要重新下载最新的 CLI 可执行文件

---

## 📊 完整变更日志

详见 [CHANGELOG.md](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/blob/main/CHANGELOG.md)

---

## 🐛 已知问题

暂无

---

## 💬 反馈与支持

- **Bug 报告**: [GitHub Issues](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/issues)
- **功能建议**: [GitHub Discussions](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/discussions)
- **文档**: [README.md](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/blob/main/README.md)

---

## 🙏 致谢

感谢所有使用和支持本项目的开发者！

如果这个项目对你有帮助，请给我们一个 ⭐ Star！

---

**完整提交记录**: [v1.4.0...v1.5.0](https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.4.0...v1.5.0)
