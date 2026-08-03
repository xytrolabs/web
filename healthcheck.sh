#!/bin/bash
# healthcheck.sh — Monitor all Xytro services
# Run via cron: */5 * * * * /run/media/raf/Z/PrismTechnologies/healthcheck.sh

set -euo pipefail
LOG="/tmp/xytro-health.log"
ALERT_CMD="${ALERT_CMD:-}"  # Set to a webhook or ntfy URL for alerts
DOWN=()

check() {
  local name="$1" url="$2"
  if curl -sf --max-time 10 "$url" > /dev/null 2>&1; then
    return 0
  else
    DOWN+=("$name ($url)")
    return 1
  fi
}

echo "=== Healthcheck $(date -Iseconds) ===" >> "$LOG"

check "XytroMailing"    "http://127.0.0.1:4000/health" || true
check "Xael AI"          "http://127.0.0.1:4005/health" || true
check "Bulwark Proxy"    "http://127.0.0.1:3000/" || true
check "Bulwark Webmail"  "http://127.0.0.1:3001/" || true
check "Ollama"           "http://127.0.0.1:11434/api/tags" || true
check "Cache Engine"     "http://127.0.0.1:4006/health" || true
check "Stalwart"         "http://127.0.0.1:8080/" || true
check "XytroAdmin"       "http://127.0.0.1:4030/" || true
check "XytroCloud"       "http://127.0.0.1:4035/" || true
check "Horizon Chat"     "http://127.0.0.1:4024/" || true
check "Static Sites"     "http://127.0.0.1:4020/" || true

if [ ${#DOWN[@]} -gt 0 ]; then
  MSG="🔴 Xytro services DOWN: ${DOWN[*]}"
  echo "$MSG" >> "$LOG"
  echo "$MSG"

  # Try auto-restart
  if [ -f /run/media/raf/Z/PrismTechnologies/start_all.sh ]; then
    echo "Attempting auto-restart..." >> "$LOG"
    cd /run/media/raf/Z/PrismTechnologies && bash start_all.sh >> "$LOG" 2>&1 &
  fi

  # Send alert if configured
  if [ -n "$ALERT_CMD" ]; then
    echo "$MSG" | $ALERT_CMD
  fi
else
  echo "✅ All services healthy" >> "$LOG"
fi

# Rotate log if >1MB
if [ -f "$LOG" ] && [ "$(stat -c%s "$LOG" 2>/dev/null || echo 0)" -gt 1048576 ]; then
  mv "$LOG" "$LOG.old"
fi
