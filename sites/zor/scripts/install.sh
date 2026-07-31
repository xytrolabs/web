#!/usr/bin/env bash
set -euo pipefail

# Zor Language — Universal Installer for Linux & macOS
# curl -fsSL https://raw.githubusercontent.com/xytrolabs/zor/main/scripts/install.sh | bash

BOLD="\033[1m"
GREEN="\033[32m"
RED="\033[31m"
NC="\033[0m"

REPO="xytrolabs/zor"
ZOR_HOME="$HOME/.local/share/zor"
BIN_DIR="$ZOR_HOME/bin"
LAUNCHER_DIR="$HOME/.local/bin"
LAUNCHER="$LAUNCHER_DIR/zor"

echo ""
echo -e "${BOLD}⚡ Zor Installer${NC}"
echo ""

mkdir -p "$BIN_DIR" "$LAUNCHER_DIR"

# Clone and build from source
BUILD_DIR="$(mktemp -d)"
trap 'rm -rf "$BUILD_DIR"' EXIT

echo "→ Cloning zor..."
git clone --depth 1 "https://github.com/${REPO}.git" "$BUILD_DIR" 2>/dev/null || {
    echo -e "${RED}Failed to clone ${REPO}${NC}"
    exit 1
}

cd "$BUILD_DIR/zor-native"
echo "→ Building zor (release)..."
cargo build --release 2>&1 | tail -3

if [[ ! -f target/release/zor ]]; then
    echo -e "${RED}Build failed. Install Rust first: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

cp target/release/zor "$BIN_DIR/zor"
chmod +x "$BIN_DIR/zor"
echo -e "${GREEN}✓ Built and installed zor${NC}"

# Install stdlib
echo "→ Installing standard library..."
STD_DIR="$ZOR_HOME/std"
mkdir -p "$STD_DIR"
if [[ -d "$BUILD_DIR/std" ]]; then
    cp -r "$BUILD_DIR/std"/* "$STD_DIR"/
    echo -e "${GREEN}✓ Installed std/${NC}"
fi

# Create launcher
cat > "$LAUNCHER" <<'EOF'
#!/usr/bin/env bash
exec "$HOME/.local/share/zor/bin/zor" "$@"
EOF
chmod +x "$LAUNCHER"

# PATH setup
if [[ ":$PATH:" != *":$LAUNCHER_DIR:"* ]]; then
    echo ""
    echo "Add to your PATH:"
    echo "  export PATH=\"$LAUNCHER_DIR:\$PATH\""
    # Auto-add to .bashrc if it exists
    if [[ -f "$HOME/.bashrc" ]] && ! grep -q "$LAUNCHER_DIR" "$HOME/.bashrc" 2>/dev/null; then
        printf '\n# Zor language\nexport PATH="%s:$PATH"\n' "$LAUNCHER_DIR" >> "$HOME/.bashrc"
        echo -e "${GREEN}✓ Added to ~/.bashrc${NC}"
    fi
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}✅ Zor installed!${NC}"
echo ""
echo "  Try: zor --help"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
