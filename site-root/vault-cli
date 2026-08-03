#!/bin/bash
# Xytro Vault CLI — One-line installer
# Usage: curl -sSL https://xytro.site/vault-cli | bash
# Or:    wget -qO- https://xytro.site/vault-cli | bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
ORANGE='\033[0;33m'
NC='\033[0m'

echo -e "${ORANGE}"
echo "  ╔═══════════════════════════════════╗"
echo "  ║     Xytro Vault CLI Installer     ║"
echo "  ║   Mount your cloud like a drive   ║"
echo "  ╚═══════════════════════════════════╝"
echo -e "${NC}"

# Detect OS
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo -e "${RED}This installer is for Linux only.${NC}"
    echo "For other platforms, see: https://xytro.site/docs/vault"
    exit 1
fi

# Check Python
if ! command -v python3 &>/dev/null; then
    echo -e "${RED}Python 3 is required but not installed.${NC}"
    echo "Install with: sudo apt install python3  (or your distro's package manager)"
    exit 1
fi

# Check pip
if ! command -v pip3 &>/dev/null && ! python3 -m pip --version &>/dev/null; then
    echo -e "${RED}pip is required. Install with: sudo apt install python3-pip${NC}"
    exit 1
fi

PIP="python3 -m pip"

# Check FUSE (for mount tool)
HAVE_FUSE=0
if command -v fusermount &>/dev/null; then
    HAVE_FUSE=1
fi

# Install directory
INSTALL_DIR="$HOME/.xytro-vault"
BIN_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR" "$BIN_DIR"

# Add ~/.local/bin to PATH if not already
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "Adding $BIN_DIR to PATH..."
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.profile" 2>/dev/null || true
    # Fish shell
    if [ -f "$HOME/.config/fish/config.fish" ]; then
        echo 'set -gx PATH "$HOME/.local/bin" $PATH' >> "$HOME/.config/fish/config.fish"
    fi
    export PATH="$HOME/.local/bin:$PATH"
fi

# Install dependencies
echo -e "\n${BLUE}Installing dependencies...${NC}"
$PIP install --user --quiet requests 2>/dev/null || $PIP install --user requests

if [ $HAVE_FUSE -eq 1 ]; then
    echo -e "${GREEN}✓ FUSE detected — mount tool will be available${NC}"
    $PIP install --user --quiet fusepy 2>/dev/null || $PIP install --user fusepy
else
    echo -e "${ORANGE}⚠ FUSE not found. Install 'fuse' or 'libfuse2' for mount support.${NC}"
    echo "  Ubuntu/Debian: sudo apt install libfuse2"
    echo "  Fedora:        sudo dnf install fuse"
    echo "  Arch:          sudo pacman -S fuse2"
fi

# Determine download base URL
BASE_URL="${XYTRO_DOWNLOAD_BASE:-https://raw.githubusercontent.com/xytrolabs/xytro-vault/main}"

# Try to download the CLI tools
echo -e "\n${BLUE}Downloading Xytro Vault tools...${NC}"

# If running from local files, copy; otherwise download
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/xytro-vault" ]; then
    # Local install
    echo "Installing from local files..."
    cp "$SCRIPT_DIR/xytro-vault" "$BIN_DIR/xytro-vault"
    cp "$SCRIPT_DIR/xytro-vault-mount" "$BIN_DIR/xytro-vault-mount" 2>/dev/null || true
    cp "$SCRIPT_DIR/xytro-vault-desktop" "$BIN_DIR/xytro-vault-desktop" 2>/dev/null || true
else
    # Remote download
    echo "Downloading from xytro.site..."
    curl -sSL -o "$BIN_DIR/xytro-vault" "https://xytro.site/dl/xytro-vault" 2>/dev/null || {
        echo -e "${ORANGE}Download failed — tools are at /PrismTechnologies/VaultCLI/${NC}"
        echo "Please copy them manually to $BIN_DIR/"
        exit 1
    }
    curl -sSL -o "$BIN_DIR/xytro-vault-mount" "https://xytro.site/dl/xytro-vault-mount" 2>/dev/null || true
    curl -sSL -o "$BIN_DIR/xytro-vault-desktop" "https://xytro.site/dl/xytro-vault-desktop" 2>/dev/null || true
fi

chmod +x "$BIN_DIR/xytro-vault" "$BIN_DIR/xytro-vault-mount" "$BIN_DIR/xytro-vault-desktop" 2>/dev/null || true

# Install desktop entry
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_DIR/xytro-vault.desktop" << 'DESKTOPEOF'
[Desktop Entry]
Name=Xytro Vault
Comment=Mount your Xytro Vault cloud storage as a local drive
Exec=xytro-vault-desktop
Icon=folder-remote
Terminal=false
Type=Application
Categories=Network;FileTools;
StartupNotify=false
X-GNOME-Autostart-enabled=false
DESKTOPEOF

update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

echo ""
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Xytro Vault CLI installed!${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
echo "  📁 CLI:        xytro-vault"
echo "  📁 Mount:      xytro-vault-mount"
echo "  📁 Desktop:    xytro-vault-desktop (in app launcher)"
echo ""
echo "  Get started:"
echo "    xytro-vault login"
echo "    xytro-vault ls /"
echo ""
echo "  Mount as a drive:"
echo "    xytro-vault-desktop"
echo "    # Or search 'Xytro Vault' in your app launcher"
echo ""
