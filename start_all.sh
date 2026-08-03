#!/bin/bash
set -euo pipefail
# Start all Xytro Labs services and Cloudflare tunnel

# Load environment variables
ENV_FILE="$(dirname "$0")/.env"
if [[ -f "$ENV_FILE" ]]; then
  set -a; source "$ENV_FILE"; set +a
  echo "Loaded .env"
else
  echo "WARNING: .env not found at $ENV_FILE — using defaults"
fi

REQUIRED_CMDS=(curl python3 npx npm fuser)
for cmd in "${REQUIRED_CMDS[@]}"; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "ERROR: required command '$cmd' is missing. Install it before running start_all.sh."
    exit 1
  fi
 done

# Stop stale listeners so re-runs do not leave duplicate processes
fuser -k 3000/tcp 2>/dev/null || true
fuser -k 3001/tcp 2>/dev/null || true
fuser -k 4000/tcp 2>/dev/null || true
fuser -k 4005/tcp 2>/dev/null || true
fuser -k 4010/tcp 2>/dev/null || true
fuser -k 4020/tcp 2>/dev/null || true
fuser -k 4021/tcp 2>/dev/null || true
fuser -k 4022/tcp 2>/dev/null || true
fuser -k 4006/tcp 2>/dev/null || true
fuser -k 4023/tcp 2>/dev/null || true
fuser -k 4024/tcp 2>/dev/null || true
fuser -k 4030/tcp 2>/dev/null || true
fuser -k 4035/tcp 2>/dev/null || true
fuser -k 11435/tcp 2>/dev/null || true
fuser -k 8080/tcp 2>/dev/null || true

# Kill stale processes
kill $(pgrep stalwart) 2>/dev/null || true

# ── Stalwart (SMTP + JMAP on :8080) ──────────────────────────────────
echo "[stalwart] Starting..."
STALWART_PUBLIC_URL=https://mail.xytro.site \
  /run/media/raf/C/stalwart/stalwart \
  -c /run/media/raf/C/stalwart/config.json \
  > /tmp/stalwart.log 2>&1 &
echo "  ✓ Stalwart on :8080"

CLOUDFLARED_CONFIG="$HOME/.cloudflared/config.yml"
if [[ -f "$CLOUDFLARED_CONFIG" ]]; then
  SKIP_CLOUDFLARED=false
  REQUIRED_CMDS+=(cloudflared)
else
  echo "WARNING: Cloudflared config not found at $CLOUDFLARED_CONFIG. Tunnel will be skipped."
  SKIP_CLOUDFLARED=true
fi

for cmd in "${REQUIRED_CMDS[@]}"; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "ERROR: required command '$cmd' is missing. Install it before running start_all.sh."
    exit 1
  fi
 done

# Verify systemd Ollama is running with optimizations (flash attention, KV cache q8, etc.)
# The optimized override is at /etc/systemd/system/ollama.service.d/override.conf
if curl -sf http://127.0.0.1:11434/api/tags >/dev/null 2>&1; then
  echo "✓ Systemd Ollama running on :11434 (9+ models)"
else
  echo "WARNING: Systemd Ollama not responding on :11434 — starting fallback..."
  nohup env OLLAMA_FLASH_ATTENTION=1 OLLAMA_KV_CACHE_TYPE=q8_0 OLLAMA_NUM_PARALLEL=4 OLLAMA_MMAP=1 OLLAMA_NUMA=1 OLLAMA_HOST=127.0.0.1:11435 ollama serve > /tmp/ollama_fallback.log 2>&1 &
  sleep 3
fi

# Start reports static server
nohup python3 -m http.server 4010 --directory '/run/media/raf/Z/BACKUPS/LEGACY CLOUDPRISM/FSO Code/AXIS - FSO Bot!/site' > /tmp/reports_static_server.log 2>&1 &
echo "Started reports static server on port 4010."

# Start all static sites via merged server (replaces 4 separate http.server processes)
nohup python3 -m http.server "${PORT_LANDING:-4020}" --directory '/run/media/raf/Z/PrismTechnologies/site-root' > /tmp/xytro_root_site.log 2>&1 &
echo "Started xytro.site landing page on :${PORT_LANDING:-4020}."

# Start Indent language site
nohup python3 -m http.server "${PORT_INDENT:-4021}" --directory '/run/media/raf/Z/PrismTechnologies/indent-site' > /tmp/indent_site.log 2>&1 &
echo "Started indent.xytro.site on :${PORT_INDENT:-4021}."

# Start Balance site
nohup python3 -m http.server "${PORT_BALANCE:-4022}" --directory '/run/media/raf/Z/PrismTechnologies/balance-site' > /tmp/balance_site.log 2>&1 &
echo "Started balance.xytro.site on :${PORT_BALANCE:-4022}."

# Start Zor transpiler site
nohup python3 -m http.server "${PORT_ZOR:-4023}" --directory '/run/media/raf/Z/PrismTechnologies/Zor' > /tmp/zor_site.log 2>&1 &
echo "Started zor.xytro.site on :${PORT_ZOR:-4023}."

# Start Cache Engine (llama.cpp KV cache — GPU accelerated)
nohup /run/media/raf/Z/PrismTechnologies/vllm-server/llama.cpp/build/bin/llama-server \
  --host 0.0.0.0 --port "${PORT_CACHE:-4006}" \
  --model /run/media/raf/Z/PrismTechnologies/vllm-server/qwen2.5-0.5b.gguf \
  --n-gpu-layers 99 --ctx-size 8192 \
  --threads 14 --threads-http 4 \
  --batch-size 2048 --ubatch-size 512 \
  --cache-type-k q8_0 --cache-type-v q8_0 \
  --cont-batching --load-mode mlock --no-webui --flash-attn on \
  > /tmp/llama-server.log 2>&1 &
echo "Started Cache Engine on :${PORT_CACHE:-4006} (GPU)."



# ── Bulwark Webmail + JMAP proxy ──────────────────────────────────────
echo "[bulwark] Starting Bulwark on :3001 + proxy on :3000..."
fuser -k 3001/tcp 2>/dev/null || true
cd /run/media/raf/Z/PrismTechnologies/XytroMail-NG
# Ensure build exists (cleanup may have removed .next)
if [ ! -d ".next/BUILD_ID" ] && [ ! -f ".next/BUILD_ID" ]; then
  echo "  Rebuilding Bulwark (no .next found)..."
  npx next build --turbopack > /tmp/bulwark_build.log 2>&1
fi
HOSTNAME=0.0.0.0 PORT=3001 nohup npx next start > /tmp/bulwark.log 2>&1 &
HOSTNAME=0.0.0.0 PORT=3000 nohup node proxy.js > /tmp/proxy.log 2>&1 &
cd /run/media/raf/Z/PrismTechnologies
echo "  ✓ Bulwark Webmail on :3000 (proxy → :3001 + Stalwart :8080)"

# Start XytroMailing using pre-compiled dist
npm --prefix /run/media/raf/Z/PrismTechnologies/XytroMailing rebuild better-sqlite3 > /tmp/xytromailing_rebuild.log 2>&1 || true
echo "Started XytroMailing on port ${PORT_MAIL:-4000} (Stalwart backend)."

# ── All Node services managed by keep-alive (auto-restart on crash) ──
nohup bash /run/media/raf/Z/PrismTechnologies/keep-alive-all.sh &>/dev/null &
echo "Started keep-alive for: XytroMailing, Xael AI, Chat UI, Admin"

# Start Xytro Vault (OmniCloud frontend)
cd /run/media/raf/Z/PrismTechnologies/XytroCloud/frontend && nohup npx vite preview --port 4035 --host 0.0.0.0 > /tmp/xytro_cloud.log 2>&1 &
cd /run/media/raf/Z/PrismTechnologies
echo "Started Xytro Vault (OmniCloud) on port 4035."

# Start Cloudflare tunnel if config is present
if [[ "$SKIP_CLOUDFLARED" == "false" ]]; then
  nohup cloudflared tunnel --config "$CLOUDFLARED_CONFIG" --no-autoupdate run > /tmp/cloudflared_tunnel.log 2>&1 &
  echo "Started Cloudflare tunnel."
else
  echo "Skipping Cloudflare tunnel because config is missing: $CLOUDFLARED_CONFIG"
fi

cd /run/media/raf/Z/PrismTechnologies
