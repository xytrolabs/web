#!/bin/bash
# keep-alive-all.sh — Auto-restart all critical Xytro services
# Each service runs in a subshell with crash recovery

LOG="/tmp/xytro-keepalive.log"
echo "[$(date -Iseconds)] === Keep-alive started ===" > "$LOG"

keep_alive() {
    local name="$1"
    local dir="$2"
    local cmd="$3"
    local log="$4"
    
    (
        cd "$dir" || exit 1
        while true; do
            echo "[$(date -Iseconds)] [$name] Starting..." >> "$LOG"
            bash -c "$cmd" 2>&1 | while read line; do
                echo "[$(date -Iseconds)] [$name] $line" >> "$log"
            done
            echo "[$(date -Iseconds)] [$name] CRASHED — restarting in 2s" >> "$LOG"
            sleep 2
        done
    ) &
}

# ── XytroMailing (critical — mail backend) ──
keep_alive "XytroMailing" \
    "/run/media/raf/Z/PrismTechnologies/XytroMailing" \
    "node dist/server.js" \
    "/tmp/xytromailing.log"

# ── Xael AI API ──
keep_alive "Xael" \
    "/run/media/raf/Z/PrismTechnologies/Xael" \
    "node server.js" \
    "/tmp/xael.log"

# ── Valis Chat UI (with API rewrite to Xael) ──
keep_alive "ChatUI" \
    "/run/media/raf/Z/PrismTechnologies/Xael/chat-template" \
    "npx next start -p 4024" \
    "/tmp/xael_chat.log"

# ── XytroAdmin ──
keep_alive "Admin" \
    "/run/media/raf/Z/PrismTechnologies/XytroAdmin" \
    "npx vite preview --port 4030 --host 0.0.0.0" \
    "/tmp/xytro_admin.log"

echo "[$(date -Iseconds)] All keep-alives launched" >> "$LOG"
echo "All services running with auto-restart. Monitor: tail -f $LOG"

# Keep script alive
wait
