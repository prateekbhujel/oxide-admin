#!/usr/bin/env sh
# ==============================================================================
# OxideAdmin Process Watchdog for cPanel & Shared Hosting Cron Jobs
# Run every minute: * * * * * /home/USERNAME/app/watchdog.sh > /dev/null 2>&1
# Or on server reboot: @reboot /home/USERNAME/app/watchdog.sh > /dev/null 2>&1
# ==============================================================================

APP_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_NAME="app"
BIN_PATH="$APP_DIR/$BIN_NAME"
LOG_PATH="$APP_DIR/oxide.log"
PORT=3000

# Ensure binary is executable
if [ ! -x "$BIN_PATH" ]; then
    chmod +x "$BIN_PATH" 2>/dev/null
fi

# 1. Health check via curl
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:$PORT/admin/login" 2>/dev/null)

if [ "$STATUS" = "200" ] || [ "$STATUS" = "302" ] || [ "$STATUS" = "303" ]; then
    # Application is active and healthy
    exit 0
fi

# 2. Check if process is running by port binding
if command -v lsof >/dev/null 2>&1; then
    if lsof -i :$PORT >/dev/null 2>&1; then
        exit 0
    fi
fi

# 3. Start binary in background with unbuffered logging
cd "$APP_DIR" || exit 1
PORT=$PORT nohup "$BIN_PATH" >> "$LOG_PATH" 2>&1 &
echo "[$(date -u '+%Y-%m-%d %H:%M:%S UTC')] [watchdog] Started $BIN_NAME on port $PORT (PID $!)" >> "$LOG_PATH"
