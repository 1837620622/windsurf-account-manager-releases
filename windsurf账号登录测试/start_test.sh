#!/bin/bash
# ============================================================
# Windsurf 账号测试脚本 - 启动
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

LOG_FILE="$SCRIPT_DIR/output.log"
PID_FILE="$SCRIPT_DIR/test.pid"

echo "============================================================"
echo "Windsurf 账号测试启动器"
echo "目录: $SCRIPT_DIR"
echo "============================================================"

# 检查是否已在运行
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p $OLD_PID > /dev/null 2>&1; then
        echo "⚠️ 脚本已在运行中 (PID: $OLD_PID)"
        echo "如需重启，请先运行 ./stop_test.sh"
        exit 1
    else
        rm -f "$PID_FILE"
    fi
fi

# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js 未安装"
    exit 1
fi

echo "Node.js 版本: $(node -v)"
echo ""

# 启动脚本
echo "启动测试脚本..."
nohup node test_login.js > "$LOG_FILE" 2>&1 &
NEW_PID=$!
echo $NEW_PID > "$PID_FILE"

sleep 1

# 检查是否成功启动
if ps -p $NEW_PID > /dev/null 2>&1; then
    echo "✅ 启动成功!"
    echo "   PID: $NEW_PID"
    echo "   日志: $LOG_FILE"
    echo ""
    echo "查看日志: tail -f $LOG_FILE"
    echo "停止脚本: ./stop_test.sh"
else
    echo "❌ 启动失败，请检查日志"
    cat "$LOG_FILE"
    rm -f "$PID_FILE"
    exit 1
fi
