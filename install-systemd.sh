#!/bin/bash
# install-systemd.sh — Install systemd services for Xytro Labs
# Run: sudo bash install-systemd.sh

set -euo pipefail
SERVICE_DIR="$(dirname "$0")/systemd"
TARGET="/etc/systemd/system"

echo "Installing Xytro Labs systemd services..."

for svc in "$SERVICE_DIR"/*.service; do
  name="$(basename "$svc")"
  cp "$svc" "$TARGET/$name"
  chmod 644 "$TARGET/$name"
  echo "  ✓ $name"
done

systemctl daemon-reload

echo ""
echo "Services installed. Enable with:"
echo "  sudo systemctl enable xytromailing xael bulwark bulwark-proxy cache-engine"
echo "  sudo systemctl start xytromailing xael bulwark bulwark-proxy cache-engine"
echo ""
echo "Or use start_all.sh which still works for manual starting."
