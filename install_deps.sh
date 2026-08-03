#!/bin/bash
set -euo pipefail

# install_deps.sh
# Installs the dependencies needed by start_all.sh on Linux.
#
# cloudflared is installed by downloading the binary from GitHub since
# it is not available in most distro package repositories.

PACKAGES=(nodejs npm)

echo "This script installs the required tools for start_all.sh."
echo "Supported package managers: pacman, apt, dnf, zypper."

default_pm=""
if command -v pacman >/dev/null 2>&1; then
  default_pm="pacman"
elif command -v apt >/dev/null 2>&1; then
  default_pm="apt"
elif command -v dnf >/dev/null 2>&1; then
  default_pm="dnf"
elif command -v zypper >/dev/null 2>&1; then
  default_pm="zypper"
fi

if [[ -n "$default_pm" ]]; then
  read -rp "Detected package manager '$default_pm'. Use it? [Y/n]: " use_default
  use_default=${use_default:-Y}
  if [[ "$use_default" =~ ^[Yy]$ ]]; then
    pkg_manager="$default_pm"
  else
    read -rp "Enter package manager to use (pacman/apt/dnf/zypper): " pkg_manager
  fi
else
  read -rp "No supported package manager detected. Enter package manager to use (pacman/apt/dnf/zypper): " pkg_manager
fi

# ─── 1. Install system packages (nodejs, npm) ───
echo "=== Installing system packages ==="
case "$pkg_manager" in
  pacman)
    sudo pacman -Syu --noconfirm nodejs npm
    ;;
  apt)
    sudo apt update
    sudo apt install -y nodejs npm
    ;;
  dnf)
    sudo dnf install -y nodejs npm
    ;;
  zypper)
    sudo zypper install -y nodejs npm
    ;;
  *)
    echo "Unsupported package manager: $pkg_manager"
    echo "Please install nodejs and npm manually."
    exit 1
    ;;
 esac

# ─── 2. Install cloudflared (via GitHub binary) ───
echo ""
echo "=== Installing cloudflared ==="
if command -v cloudflared >/dev/null 2>&1; then
  echo "cloudflared already installed: $(cloudflared version)"
else
  CLOUDFLARED_URL="https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
  echo "Downloading cloudflared from $CLOUDFLARED_URL ..."
  curl -fsSL "$CLOUDFLARED_URL" -o /tmp/cloudflared
  sudo install -m 755 /tmp/cloudflared /usr/local/bin/cloudflared
  echo "cloudflared installed: $(cloudflared version)"
fi

# ─── 3. Verify everything ───
echo ""
echo "=== Checking binaries ==="
for bin in npm npx cloudflared; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    echo "WARNING: '$bin' is not available after install."
  else
    echo "  Found $bin: $(command -v "$bin")"
  fi
 done

# ─── 4. Ollama is optional ───
echo ""
if ! command -v ollama >/dev/null 2>&1; then
  echo "NOTE: 'ollama' was not installed automatically."
  echo "      Install it from https://ollama.com if you need AI features."
  echo "      Otherwise XytroMailing will work fine without it."
else
  echo "Found ollama: $(command -v ollama)"
fi

echo ""
echo "Done! Run ./start_all.sh to start all services."
