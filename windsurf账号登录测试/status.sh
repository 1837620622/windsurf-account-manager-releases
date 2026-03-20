#!/bin/bash
# ============================================================
# 状态查看脚本
# 显示运行状态、成功账号数、日志预览
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LOG_FILE="$SCRIPT_DIR/output.log"
SUCCESS_FILE="$SCRIPT_DIR/success_accounts.json"
PID_FILE="$SCRIPT_DIR/test.pid"

clear
echo "============================================================"
echo "Windsurf 测试状态面板"
echo "时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "============================================================"
echo ""

# 检查运行状态
echo "【运行状态】"
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p $PID > /dev/null 2>&1; then
        echo "✅ 脚本运行中 (PID: $PID)"
    else
        echo "❌ 脚本已停止"
    fi
else
    PIDS=$(pgrep -f "node.*test_login.js")
    if [ -n "$PIDS" ]; then
        echo "✅ 脚本运行中 (PID: $PIDS)"
    else
        echo "❌ 脚本未运行"
    fi
fi
echo ""

# 日志文件大小
echo "【日志信息】"
if [ -f "$LOG_FILE" ]; then
    SIZE=$(du -h "$LOG_FILE" | cut -f1)
    LINES=$(wc -l < "$LOG_FILE")
    echo "文件大小: $SIZE"
    echo "总行数: $LINES"
else
    echo "日志文件不存在"
fi
echo ""

# 成功账号数
echo "【成功账号】"
if [ -f "$SUCCESS_FILE" ]; then
    COUNT=$(grep -c '"email"' "$SUCCESS_FILE" 2>/dev/null || echo "0")
    echo "累计成功: $COUNT 个"
    echo ""
    echo "最近成功的账号:"
    tail -n 20 "$SUCCESS_FILE" | grep '"email"' | tail -5
else
    echo "暂无成功账号"
fi
echo ""

# 最近日志
echo "============================================================"
echo "【最近日志】(最后 20 行)"
echo "============================================================"
if [ -f "$LOG_FILE" ]; then
    tail -n 20 "$LOG_FILE"
fi
echo ""
echo "============================================================"
echo "常用命令:"
echo "  ./start_test.sh  - 启动测试"
echo "  ./stop_test.sh   - 停止测试"
echo "  ./log.sh         - 查看日志"
echo "  ./status.sh      - 刷新状态"
echo "  tail -f output.log - 实时日志"
echo "============================================================"
