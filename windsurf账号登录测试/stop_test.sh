#!/bin/bash
# ============================================================
# Windsurf 账号测试脚本 - 停止
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PID_FILE="$SCRIPT_DIR/test.pid"

echo "============================================================"
echo "Windsurf 账号测试停止器"
echo "============================================================"

# 方式1：通过 PID 文件停止
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p $PID > /dev/null 2>&1; then
        echo "正在停止进程 (PID: $PID)..."
        kill $PID
        sleep 2
        
        # 检查是否已停止
        if ps -p $PID > /dev/null 2>&1; then
            echo "进程未响应，强制终止..."
            kill -9 $PID
        fi
        
        echo "✅ 已停止"
    else
        echo "进程已不存在"
    fi
    rm -f "$PID_FILE"
else
    # 方式2：通过进程名停止
    PIDS=$(pgrep -f "node.*test_login.js")
    if [ -n "$PIDS" ]; then
        echo "发现运行中的进程: $PIDS"
        echo "正在停止..."
        pkill -f "node.*test_login.js"
        sleep 2
        
        # 强制停止
        pkill -9 -f "node.*test_login.js" 2>/dev/null
        echo "✅ 已停止"
    else
        echo "没有找到运行中的测试脚本"
    fi
fi

echo ""
echo "查看成功账号: cat success_accounts.txt"
