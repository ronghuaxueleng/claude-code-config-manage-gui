# 快速构建指南

## 使用 npm 脚本（推荐）

```bash
# 进入 CLI 目录
cd claude-config-cli

# 查看所有可用命令
npm run

# 构建 release 版本
npm run build

# 构建 debug 版本
npm run build:debug

# 清理后构建
npm run build:clean

# 构建并安装
npm run build:install

# 构建并剥离调试符号
npm run build:strip

# 完整构建（推荐用于发布）
npm run build:full

# 查看帮助
npm run build:help
```

## 直接使用 Node.js

```bash
# 基本构建
node build.mjs

# 带选项构建
node build.mjs --debug
node build.mjs --clean --strip --install

# 查看所有选项
node build.mjs --help
```

## 传统 Cargo 命令

```bash
# 如果你熟悉 Cargo，也可以直接使用
cargo build --release
cargo install --path .
```

## 快速入门

第一次使用？按以下步骤操作：

```bash
# 1. 进入目录
cd claude-config-cli

# 2. 构建并安装
npm run build:full

# 3. 运行
claude-config
```

更多详细信息请查看 BUILD_GUIDE.md
