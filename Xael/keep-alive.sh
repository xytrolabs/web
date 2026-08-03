#!/bin/bash
# keep-alive.sh — Auto-restart Xael server on crash
# Usage: bash keep-alive.sh

SERVICE_NAME="Xael AI API"
CMD="node /run/media/raf/Z/PrismTechnologies/Xael/server.js"
LOGFILE="/tmp/xael-keepalive.log"
MAX_RESTARTS=10
RESTART_WINDOW=60  # seconds

restart_count=0
last_restart=0

echo "[$(date -Iseconds)] Keep-alive started for $SERVICE_NAME" >> "$LOGFILE"

while true; do
    echo "[$(date -Iseconds)] Starting $SERVICE_NAME..." >> "$LOGFILE"
    node /run/media/raf/Z/PrismTechnologies/Xael/server.js 2>&1 | while read line; do
        echo "[$(date -Iseconds)] $line" >> "$LOGFILE"
    done
    
    exit_code=$?
    now=$(date +%s)
    
    # Reset counter if window passed
    if [ $((now - last_restart)) -gt $RESTART_WINDOW ]; then
        restart_count=0
    fi
    
    restart_count=$((restart_count + 1))
    last_restart=$now
    
    echo "[$(date -Iseconds)] $SERVICE_NAME crashed (exit $exit_code). Restart #$restart_count" >> "$LOGFILE"
    
    if [ $restart_count -gt $MAX_RESTARTS ]; then
        echo "[$(date -Iseconds)] Too many crashes ($restart_count in ${RESTART_WINDOW}s). Giving up." >> "$LOGFILE"
        exit 1
    fi
    
    sleep 2
done
