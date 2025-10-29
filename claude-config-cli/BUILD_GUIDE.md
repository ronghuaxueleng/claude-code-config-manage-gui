# CLI 版本构建指南

这是 Claude Config CLI 的 Node.js 构建脚本，提供了更好的跨平台支持和更多构建选项。

## 功能特性

- ✅ 跨平台支持（Windows、Linux、macOS）
- ✅ 自动环境检查
- ✅ 国内镜像源配置（可选）
- ✅ 多种构建选项
- ✅ 自动安装到系统
- ✅ 二进制文件优化
- ✅ 交叉编译支持
- ✅ 彩色输出和进度提示

## 快速开始

### 基本用法

```bash
# 进入 CLI 目录
cd claude-config-cli

# 构建 release 版本
node build.mjs
```

### 构建选项

```bash
# 构建 debug 版本（快速，用于开发）
node build.mjs --debug

# 清理缓存后构建
node build.mjs --clean

# 构建并安装到系统
node build.mjs --install

# 构建并剥离调试符号（减小体积）
node build.mjs --strip

# 交叉编译到特定平台
node build.mjs --target=x86_64-unknown-linux-musl

# 完整构建（清理 + release + 剥离 + 安装）
node build.mjs --clean --strip --install
```

## 命令行选项

| 选项 | 说明 |
|------|------|
| `--debug` | 构建 debug 版本（默认构建 release） |
| `--target=<triple>` | 指定目标三元组（交叉编译） |
| `--no-mirror` | 不使用国内镜像源 |
| `--clean` | 清理构建缓存后再构建 |
| `--install` | 构建完成后安装到系统 |
| `--strip` | 剥离二进制文件的调试符号 |

## 环境要求

- **Node.js** >= 16.0.0（用于运行构建脚本）
- **Rust** >= 1.70.0
- **Cargo**（随 Rust 一起安装）

更多详细信息请参考完整文档。
