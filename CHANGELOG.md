# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.7.0] - 2025-12-19

### ✨ Added
- **账号批量导入导出**: 支持账号数据的批量导入和导出
  - 添加选择性导出功能，可选择特定账号导出
  - 批量导出时支持 URL 筛选功能
  - 支持自定义保存位置选择
  - 完善账号导入时的重复检测逻辑

- **宿主机 IP 功能**: 账号切换时支持使用宿主机 IP
  - GUI 版本添加宿主机 IP 切换选项
  - 优化 URL 显示效果

- **CLAUDE.local.md 文件保护**:
  - 切换账号时检测并询问是否保留现有 CLAUDE.local.md 文件
  - 覆盖前自动备份现有文件

- **环境变量增强**:
  - 创建配置文件时自动添加 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC
  - 拆分为独立环境变量：DISABLE_BUG_COMMAND、DISABLE_ERROR_REPORTING、DISABLE_TELEMETRY
  - 不再禁用自动更新功能

- **Commands 目录支持**: 自动打包和释放 commands 目录下的所有文件

- **Rust 安装脚本**: 添加国内镜像一键安装脚本，支持多镜像源选择

- **CLI 增强**:
  - 启动时自动设置 hasCompletedOnboarding
  - 构建脚本自动同步版本号
  - 配置切换时添加 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC

### 🐛 Fixed
- 修复批量导入账号时参数传递错误导致创建失败的问题
- 修复导出文件时的 Tauri API 错误
- 修复批量导出账号功能并优化 URL 显示
- 修复 token 匹配时因空格导致的匹配失败问题
- 修复账号切换时未添加 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC 配置的问题
- 修复 Cargo 源名称冲突问题
- 添加 Tauri Bundler 工具 GitHub 镜像配置解决 NSIS 下载超时
- CLI: 修复数据库迁移顺序和版本号显示
- CLI: 修复 base_urls 表 api_key 字段数据库迁移问题
- CLI: 添加 accounts 表 custom_env_vars 字段支持

### 🔧 Changed
- 将 Cargo 镜像源切换为官方源
- 移除未使用的 update_env_config_with_options 方法
- 提交代码时不再附带 Co-Authored-By: Claude 信息

---

## [1.6.0] - 2025-10-30

### ✨ Added
- **代理配置支持**: 新增 HTTP_PROXY 和 HTTPS_PROXY 环境变量配置
  - 在 Claude 配置页面添加代理输入框
  - 在账号关联页面添加"使用代理"复选框选项
  - 支持从 Claude 配置中加载代理设置并写入目标目录
  - CLI 版本同步支持代理配置功能
  - 完整的中英文国际化支持

- **跨平台构建系统**: 新增 build.mjs 现代化构建脚本
  - GUI 版本：支持多种构建选项（debug、target、clean 等）
  - GUI 版本：自动配置国内镜像源加速构建
  - GUI 版本：智能依赖管理（支持 pnpm/npm）
  - CLI 版本：纯 Rust 项目构建支持
  - CLI 版本：交叉编译和二进制优化（LTO、strip）
  - 详细的进度提示和错误处理
  - 替换旧的 shell 脚本（build.sh、build.bat、cleanup.bat）

- **环境变量配置系统重构**:
  - 将环境变量输入从键值对模式改为 JSON 文本输入
  - 支持更精确的数据类型控制（字符串、数字、布尔值）
  - 账号编辑时切换 URL 自动更新环境变量预览
  - 数据库层面支持分层环境变量配置
    - accounts 表新增 `custom_env_vars` 字段存储账号自定义环境变量
    - base_urls 表新增 `default_env_vars` 字段存储 URL 默认环境变量

- **构建文档完善**:
  - BUILD_GUIDE.md - GUI 构建指南
  - claude-config-cli/BUILD_GUIDE.md - CLI 构建详细文档
  - claude-config-cli/BUILD_README.md - CLI 快速入门

### 🐛 Fixed
- **环境变量配置修复**:
  - 修复智能标点符号修正功能导致的引号转换问题
  - 修复编辑账号时 URL 选择错误问题
  - 修复环境变量类型转换问题
  - 优化数据库更新逻辑，防止空对象覆盖已有配置
  - 完善 JSON 解析和错误处理

- **代码质量提升**:
  - 移除未使用的 migrated 变量赋值（修复 Rust 编译警告）
  - 统一环境变量处理逻辑
  - 增强错误处理和边界检查

### 🔧 Changed
- **配置精简**: 移除较少使用的配置项
  - 移除快速模型（ANTHROPIC_SMALL_FAST_MODEL）配置
  - 移除 Bash 命令超时时间配置
  - 移除 MCP 超时时间配置
  - 代理配置移至提示缓存配置项后面

- **构建流程优化**:
  - 更新 package.json 构建脚本
  - 优化系统管理环境变量列表
  - 完善数据库迁移逻辑

### 📝 Documentation
- 添加 RELEASE_NOTES_v1.5.0.md 发布说明
- 更新所有构建相关文档

### 🗄️ Database
- 自动迁移脚本：为 accounts 和 base_urls 表添加环境变量字段
- 向后兼容的数据库迁移逻辑

---

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

[1.7.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.6.0...v1.7.0
[1.6.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/ronghuaxueleng/claude-code-config-manage-gui/releases/tag/v1.1.0
