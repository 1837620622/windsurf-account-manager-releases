#!/bin/bash
# ============================================================
# Node.js Firebase 代理服务器启动脚本
# ============================================================

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# 端口号（可通过环境变量修改）
PORT=${PORT:-3000}

echo "============================================================"
echo "Firebase Proxy Server 启动器"
echo "目录: $SCRIPT_DIR"
echo "端口: $PORT"
echo "============================================================"
echo ""

# 检查 Node.js 是否安装
if ! command -v node &> /dev/null; then
    echo "❌ Node.js 未安装，请先安装 Node.js"
    exit 1
fi

echo "Node.js 版本: $(node -v)"
echo ""

# 启动服务器
echo "启动服务器..."
PORT=$PORT node server.js
