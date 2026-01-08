#!/bin/bash

echo "=========================================="
echo "Claude Code Root Check 移除工具"
echo "=========================================="
echo ""

# 通过 which 命令找到 claude 可执行文件
echo "正在查找 claude 命令..."
CLAUDE_PATH=$(which claude)

if [ -z "$CLAUDE_PATH" ]; then
    echo "❌ 错误: 未找到 claude 命令"
    exit 1
fi

echo "找到 claude 位置: $CLAUDE_PATH"

# 如果是软链接，获取实际文件路径
if [ -L "$CLAUDE_PATH" ]; then
    REAL_PATH=$(readlink -f "$CLAUDE_PATH")
    echo "这是一个软链接，实际路径: $REAL_PATH"
else
    REAL_PATH="$CLAUDE_PATH"
fi

# 获取 claude 所在的目录
CLAUDE_DIR=$(dirname "$CLAUDE_PATH")
echo "claude 目录: $CLAUDE_DIR"
echo ""

# 检查是否已经是包装脚本
if grep -q "Claude Code Wrapper" "$CLAUDE_PATH" 2>/dev/null; then
    echo "✓ 检测到已安装包装脚本"
    echo "正在更新包装脚本..."
else
    echo "正在创建包装脚本..."
fi

# 创建 claude-wrapper.sh
WRAPPER_PATH="$CLAUDE_DIR/claude-wrapper.sh"

cat > "$WRAPPER_PATH" << 'EOF'
#!/bin/bash

# Claude Code Wrapper - 自动删除 root check 限制
# 此脚本会在每次执行 claude 前绕过 root 用户限制
#
# 新版本 (2.1.x+) 支持通过环境变量绕过检查：
# - IS_SANDBOX=1
# - CLAUDE_CODE_BUBBLEWRAP=1
#
# 旧版本需要修改 cli.js 文件删除检查代码

# 获取当前脚本的真实路径
SCRIPT_PATH="$(readlink -f "$0")"
SCRIPT_DIR="$(dirname "$SCRIPT_PATH")"

# 查找同目录下的 claude.bak（原始软链接）
CLAUDE_BAK="$SCRIPT_DIR/claude.bak"

# 如果 claude.bak 不存在，尝试通过 which 和目录搜索找到真实路径
if [ ! -L "$CLAUDE_BAK" ] && [ ! -f "$CLAUDE_BAK" ]; then
    # 在当前目录查找指向 claude-code 的软链接或文件
    for file in "$SCRIPT_DIR"/*; do
        if [ -L "$file" ] || [ -f "$file" ]; then
            target=$(readlink -f "$file" 2>/dev/null)
            if [[ "$target" == *"@anthropic-ai/claude-code/cli.js" ]]; then
                CLAUDE_REAL_PATH="$target"
                break
            fi
        fi
    done

    # 如果还是没找到，尝试常见路径
    if [ -z "$CLAUDE_REAL_PATH" ]; then
        for path in \
            "$SCRIPT_DIR/../lib/node_modules/@anthropic-ai/claude-code/cli.js" \
            "/usr/local/lib/node_modules/@anthropic-ai/claude-code/cli.js" \
            "/usr/lib/node_modules/@anthropic-ai/claude-code/cli.js"; do
            if [ -f "$path" ]; then
                CLAUDE_REAL_PATH="$path"
                break
            fi
        done
    fi
else
    # 通过 claude.bak 获取真实的 cli.js 路径
    CLAUDE_REAL_PATH="$(readlink -f "$CLAUDE_BAK")"
fi

if [ -z "$CLAUDE_REAL_PATH" ] || [ ! -f "$CLAUDE_REAL_PATH" ]; then
    echo "错误: 未找到真实的 claude cli.js 文件" >&2
    echo "请确保 claude 已正确安装" >&2
    exit 1
fi

# 获取 claude 版本号（用于提示信息）
CLAUDE_VERSION=$(node "$CLAUDE_REAL_PATH" --version 2>/dev/null | head -1 || echo "unknown")

# 新版本 (2.1.x+) 直接使用环境变量绕过 root check
# 设置 IS_SANDBOX=1 或 CLAUDE_CODE_BUBBLEWRAP=1 即可
export IS_SANDBOX=1
export CLAUDE_CODE_BUBBLEWRAP=1

# 执行原始 claude 命令，传递所有参数
exec node "$CLAUDE_REAL_PATH" "$@"
EOF

# 给包装脚本添加执行权限
chmod +x "$WRAPPER_PATH"
echo "✓ 已创建包装脚本: $WRAPPER_PATH"
echo ""

# 备份原 claude 命令（如果尚未备份）
CLAUDE_BAK="$CLAUDE_DIR/claude.bak"
if [ ! -e "$CLAUDE_BAK" ]; then
    if [ -L "$CLAUDE_PATH" ]; then
        # 如果是软链接，复制软链接本身
        cp -P "$CLAUDE_PATH" "$CLAUDE_BAK"
        echo "✓ 已备份原 claude 软链接为: $CLAUDE_BAK"
    else
        # 如果是普通文件，复制文件
        cp "$CLAUDE_PATH" "$CLAUDE_BAK"
        echo "✓ 已备份原 claude 文件为: $CLAUDE_BAK"
    fi
else
    echo "✓ 检测到已存在备份: $CLAUDE_BAK"
fi

# 替换 claude 命令为包装脚本
echo ""
echo "正在替换 claude 命令..."

# 删除原有的 claude（如果是软链接或文件）
rm -f "$CLAUDE_PATH"

# 创建新的软链接指向包装脚本
ln -s "$WRAPPER_PATH" "$CLAUDE_PATH"

echo "✓ 已将 claude 命令替换为包装脚本"
echo ""

# 验证安装
echo "=========================================="
echo "验证安装..."
echo ""

if [ -L "$CLAUDE_PATH" ]; then
    TARGET_PATH=$(readlink "$CLAUDE_PATH")
    echo "✓ claude 现在指向: $TARGET_PATH"
fi

if [ -e "$CLAUDE_BAK" ]; then
    echo "✓ 原始 claude 已备份为: $CLAUDE_BAK"
fi

if [ -x "$WRAPPER_PATH" ]; then
    echo "✓ 包装脚本具有执行权限"
fi

echo ""
echo "=========================================="
echo "✓ 安装完成！"
echo ""
echo "现在你可以在 root 用户下使用："
echo "  claude --dangerously-skip-permissions"
echo ""
echo "如需恢复原始 claude 命令："
echo "  rm $CLAUDE_PATH"
echo "  mv $CLAUDE_BAK $CLAUDE_PATH"
echo "=========================================="