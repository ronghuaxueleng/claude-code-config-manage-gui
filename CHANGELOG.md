# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.5.0] - 2025-01-24

### ✨ Added
- **自定义 API Key 环境变量名**: 支持为不同的 Base URL 配置不同的 API Key 环境变量名
  - GUI 版本：在 URL 管理界面添加 API Key 字段输入和显示
  - CLI 版本：在列表、添加、编辑功能中添加 API Key 支持
  - 数据库：`base_urls` 表新增 `api_key` 字段，默认值为 `ANTHROPIC_API_KEY`
- **国际化支持**: API Key 字段添加完整的中英文翻译
  - 中文：API Key 环境变量名、例如: ANTHROPIC_API_KEY 或 CLAUDE_API_KEY
  - 英文：API Key Environment Variable、e.g., ANTHROPIC_API_KEY or CLAUDE_API_KEY
- **WebDAV 同步增强**: 完整支持 API Key 配置的导入和导出

### 🐛 Fixed
- 修复 GUI 版本 URL 更新时 `api_key` 不生效的问题
- 修复前端 `tauriUpdateBaseUrl` 函数参数命名不匹配问题（蛇形命名 vs 驼峰命名）

### 📝 Changed
- 更新所有相关的请求/响应模型以支持 `api_key` 字段
- 切换账号时根据 Base URL 的 `api_key` 配置使用不同的环境变量名

### 📦 Database
- 迁移脚本：为 `base_urls` 表添加 `api_key` 字段
- 默认值：`ANTHROPIC_API_KEY`

---

## [1.4.0] - 2025-01-XX

### ✨ Added
- 状态栏显示登录信息
- 改进的用户界面交互

### 🐛 Fixed
- 修复构建产物文件名问题
- 修复 macOS 构建架构问题

---

## [1.3.0] - 2024-XX-XX

### ✨ Added
- 基础功能实现
- 账号管理
- 目录管理
- 配置切换

---

## [1.2.0] - 2024-XX-XX

### ✨ Added
- ☁️ **新增 WebDAV 云同步**: 支持配置数据云端备份和多设备同步
- 🔄 **自动同步功能**: 可设置定时自动同步，实时备份配置
- 🚀 **脚本自动执行**: 切换账号时自动执行环境配置脚本

### 🔧 Improved
- 改进错误处理：优化 WSL 命令检测，静默处理非关键错误
- 日志系统增强：分级日志记录，便于问题排查
- 数据库迁移优化：支持自动创建 WebDAV 相关表结构

---

## [1.1.0] - 2024-XX-XX

### ✨ Added
- 初始版本发布
- 基础账号管理功能
- 基础目录管理功能
- 基础配置切换功能

---

[1.5.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/releases/tag/v1.1.0
