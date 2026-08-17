#!/usr/bin/env bash
# Indent — the interpreting language
# Universal installer for Linux (and macOS). No root needed.
#
# Works on Debian/Ubuntu, Fedora/RHEL, Arch, openSUSE, Alpine, and any distro
# with a C toolchain + git. Installs to ~/.local/share/indent (binary) and
# ~/.local/bin (launcher).
#
# Usage:
#   bash install.sh            # detect, build, install
#   bash install.sh --prefix ~/.local
#   bash install.sh --version  # print version and exit
#
# One-liner:
#   curl -fsSL https://indent.xytro.site/install.sh | bash

set -euo pipefail

# ── Config ──────────────────────────────────────────────────────────────
PREFIX="${INDENT_PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
INDENT_HOME="$PREFIX/share/indent"
STD_DIR="$INDENT_HOME/std"
PKG_DIR="$INDENT_HOME/packages"
REPO="xytrolabs/indent"

# ── Pretty printing ─────────────────────────────────────────────────────
c_green=$'\033[32m'; c_yellow=$'\033[33m'; c_cyan=$'\033[36m'; c_bold=$'\033[1m'; c_reset=$'\033[0m'
info()  { printf "%s•%s %s\n" "$c_cyan" "$c_reset" "$*"; }
ok()    { printf "%s✓%s %s\n" "$c_green" "$c_reset" "$*"; }
warn()  { printf "%s!%s %s\n" "$c_yellow" "$c_reset" "$*"; }
die()   { printf "%s✗%s %s\n" "\033[31m" "$c_reset" "$*" >&2; exit 1; }

# ── Version flag ────────────────────────────────────────────────────────
if [[ "${1:-}" == "--version" || "${1:-}" == "-V" ]]; then
  echo "indent installer 1.3.0"
  exit 0
fi

# ── Detect OS / arch ────────────────────────────────────────────────────
detect_os() {
  case "$(uname -s)" in
    Linux)  OS="linux" ;;
    Darwin) OS="macos" ;;
    *)      die "Unsupported OS: $(uname -s)" ;;
  esac
  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64|amd64)  ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    armv7l)        ARCH="armv7" ;;
  esac
  info "Detected: ${OS}-${ARCH}"
}

# ── Ensure a Rust toolchain exists ──────────────────────────────────────
ensure_cargo() {
  if command -v cargo >/dev/null 2>&1; then
    ok "cargo found: $(cargo --version 2>/dev/null | head -n1)"
    return
  fi
  warn "cargo not found — installing Rust via rustup..."
  if command -v curl >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
      || die "Failed to install Rust via rustup"
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env" || true
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
      || die "Failed to install Rust via rustup"
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env" || true
  else
    die "Need a Rust toolchain. Install from https://rustup.rs first."
  fi
  command -v cargo >/dev/null 2>&1 || die "cargo still not on PATH. Restart your shell and re-run."
}

# ── Build ───────────────────────────────────────────────────────────────
build() {
  info "Building Indent (release)..."
  BUILD_DIR="$(mktemp -d)"
  trap 'rm -rf "$BUILD_DIR"' EXIT
  info "Cloning ${REPO}..."
  git clone --depth 1 "https://github.com/${REPO}.git" "$BUILD_DIR" 2>/dev/null \
    || die "Failed to clone ${REPO}. Check your connection."
  (cd "$BUILD_DIR/indent-native" && cargo build --release 2>&1 | tail -3) \
    || die "Build failed. See errors above."
  [ -f "$BUILD_DIR/indent-native/target/release/indent" ] || die "Build produced no binary"
  ok "Build complete"
  BUILD_BIN="$BUILD_DIR/indent-native/target/release/indent"
  REPO_ROOT="$BUILD_DIR"
}

# ── Install ─────────────────────────────────────────────────────────────
install() {
  mkdir -p "$BIN_DIR" "$INDENT_HOME/bin" "$STD_DIR" "$PKG_DIR"

  # Real binary (lives under ~/.local/share/indent/bin)
  cp "$BUILD_BIN" "$INDENT_HOME/bin/indent"
  chmod +x "$INDENT_HOME/bin/indent"
  ok "Installed indent -> $INDENT_HOME/bin/indent"

  # Standard library
  if [ -d "$REPO_ROOT/std" ]; then
    cp -r "$REPO_ROOT"/std/* "$STD_DIR/" 2>/dev/null || true
    ok "Installed standard library -> $STD_DIR"
  fi
  # Packages
  if [ -d "$REPO_ROOT/packages" ]; then
    cp -r "$REPO_ROOT"/packages/* "$PKG_DIR/" 2>/dev/null || true
    ok "Installed packages -> $PKG_DIR"
  fi

  # Launcher in ~/.local/bin that sets up INDENT_PATH
  cat > "$BIN_DIR/indent" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
INDENT_HOME="${HOME}/.local/share/indent"
if [[ -z "${INDENT_PATH:-}" ]]; then
  export INDENT_PATH="${INDENT_HOME}/packages:${INDENT_HOME}"
else
  export INDENT_PATH="${INDENT_HOME}/packages:${INDENT_HOME}:${INDENT_PATH}"
fi
exec "${INDENT_HOME}/bin/indent" "$@"
EOF
  chmod +x "$BIN_DIR/indent"
  ok "Installed launcher -> $BIN_DIR/indent"

  # PATH
  add_to_path
}

add_to_path() {
  case ":$PATH:" in
    *":$BIN_DIR:"*) : ;;
    *)
      warn "Adding $BIN_DIR to your PATH..."
      shell="${SHELL:-$(basename "$(command -v sh)")}"
      case "$shell" in
        *fish)
          fish -c "fish_add_path $BIN_DIR" 2>/dev/null || echo "fish_add_path $BIN_DIR" >> "$HOME/.config/fish/config.fish"
          ;;
        *zsh)
          if ! grep -q "$BIN_DIR" "$HOME/.zshrc" 2>/dev/null; then
            printf '\n# Indent language\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$HOME/.zshrc"
          fi
          ;;
        *)
          if ! grep -q "$BIN_DIR" "$HOME/.bashrc" 2>/dev/null; then
            printf '\n# Indent language\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$HOME/.bashrc"
          fi
          ;;
      esac
      ;;
  esac
}

# ── Run ─────────────────────────────────────────────────────────────────
detect_os
ensure_cargo
build
install

ok "Indent installed! Start it with:  indent repl"
echo "  • Binary: $BIN_DIR/indent"
echo "  • Docs: https://indent.xytro.site/docs/"
echo "  • If 'indent' isn't found, restart your shell."
