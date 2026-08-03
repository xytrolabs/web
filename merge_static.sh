#!/bin/bash
# merge-static — Single Python server for all static sites
# Replaces 4 separate http.server processes on ports 4020-4023
# Usage: bash merge-static.sh

set -euo pipefail
PORT="${1:-4020}"
ROOT="$(dirname "$0")"

# Map virtual hosts via request path, but serve from different dirs
# Simple approach: single server with route mapping
cat > /tmp/merge_static_server.py << 'PYEOF'
import http.server
import os
import sys

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 4020
ROOT = sys.argv[2] if len(sys.argv) > 2 else "."

SITES = {
    "": os.path.join(ROOT, "site-root"),
    "indent": os.path.join(ROOT, "indent-site"),
    "balance": os.path.join(ROOT, "balance-site"),
    "zor": os.path.join(ROOT, "Zor"),
    "reports": os.path.join(ROOT, "..", "BACKUPS", "LEGACY CLOUDPRISM", "FSO Code", "AXIS - FSO Bot!", "site"),
}

class MultiSiteHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.directory = SITES.get("", ROOT)
        super().__init__(*args, **kwargs)

    def translate_path(self, path):
        # Cloudflare routes by Host header, but for local dev: serve all from root
        path = path.split("?", 1)[0].split("#", 1)[0]
        path = path.rstrip("/") or "/index.html"

        # Try to serve from appropriate site directory
        for prefix, site_dir in SITES.items():
            if path.startswith(f"/{prefix}") or prefix == "":
                full = os.path.join(site_dir, path.lstrip("/"))
                if os.path.exists(full) and not os.path.isdir(full):
                    return full
                # Try index.html
                idx = os.path.join(site_dir, path.lstrip("/"), "index.html")
                if os.path.exists(idx):
                    return idx
                # Fall back to site-root index
                return os.path.join(SITES[""], "index.html")

        return os.path.join(SITES[""], "index.html")

    def log_message(self, format, *args):
        sys.stderr.write("[%s] %s\n" % (self.log_date_time_string(), format % args))

if __name__ == "__main__":
    os.chdir(ROOT)
    server = http.server.HTTPServer(("0.0.0.0", PORT), MultiSiteHandler)
    print(f"Merged static server on :{PORT} — serving {len(SITES)} sites")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.shutdown()
PYEOF
nohup python3 /tmp/merge_static_server.py "$PORT" "$ROOT" > /tmp/merge_static.log 2>&1 &
echo "Merged static server on :$PORT (replaces :4020-4023 + :4010)"
