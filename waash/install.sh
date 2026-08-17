#!/usr/bin/env bash
# WAASH — What An Amazing SHell
# Universal installer for Linux (and macOS).
#
# Works on Debian/Ubuntu, Fedora/RHEL, Arch, openSUSE, Alpine, and any distro
# with a C toolchain + curl. Installs to ~/.local/bin (no root needed).
#
# Usage:
#   bash install.sh                # detect, build, install
#   bash install.sh --no-build     # just copy an existing target/release/waash
#   bash install.sh --prefix ~/.local
#   bash install.sh --version      # print version and exit
#
# One-liner:
#   curl -fsSL https://xytro.site/waash/install.sh | bash

set -euo pipefail

# ── Config ──────────────────────────────────────────────────────────────
PREFIX="${WAASH_PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
LIB_DIR="$PREFIX/share/waash/lib"
DOCS_DIR="$PREFIX/share/waash/docs"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NO_BUILD=0

# ── Pretty printing ─────────────────────────────────────────────────────
c_green=$'\033[32m'; c_yellow=$'\033[33m'; c_cyan=$'\033[36m'; c_reset=$'\033[0m'
info()  { printf "%s•%s %s\n" "$c_cyan" "$c_reset" "$*"; }
ok()    { printf "%s✓%s %s\n" "$c_green" "$c_reset" "$*"; }
warn()  { printf "%s!%s %s\n" "$c_yellow" "$c_reset" "$*"; }
die()   { printf "%s✗%s %s\n" "\033[31m" "$c_reset" "$*" >&2; exit 1; }

# ── Detect OS / arch ────────────────────────────────────────────────────
detect_os() {
  case "$(uname -s)" in
    Linux)  OS="linux" ;;
    Darwin) OS="macos" ;;
    *)      die "Unsupported OS: $(uname -s)" ;;
  esac
  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64|amd64)   ARCH="x86_64" ;;
    aarch64|arm64)  ARCH="aarch64" ;;
    armv7l)         ARCH="armv7" ;;
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
  if command -v cargo >/dev/null 2>&1; then
    ok "Rust installed: $(cargo --version 2>/dev/null | head -n1)"
  else
    die "cargo still not on PATH. Restart your shell and re-run install.sh"
  fi
}

# ── Build ───────────────────────────────────────────────────────────────
build() {
  info "Building WAASH (release)..."
  if ! command -v cargo >/dev/null 2>&1; then
    warn "cargo not found in this shell; trying ~/.cargo/bin"
    export PATH="$HOME/.cargo/bin:$PATH"
  fi
  (cd "$REPO_DIR" && cargo build --release) \
    || die "Build failed. See errors above."
  [ -f "$REPO_DIR/target/release/waash" ] || die "Build produced no binary"
  ok "Build complete"
}

# ── Install ─────────────────────────────────────────────────────────────
install() {
  mkdir -p "$BIN_DIR" "$LIB_DIR" "$DOCS_DIR"

  # Binary
  cp "$REPO_DIR/target/release/waash" "$BIN_DIR/waash"
  chmod +x "$BIN_DIR/waash"
  ok "Installed waash -> $BIN_DIR/waash"

  # Helper library for scripts
  if [ -f "$REPO_DIR/share/waash/waash.ind" ]; then
    cp "$REPO_DIR/share/waash/waash.ind" "$LIB_DIR/waash.ind"
    ok "Installed helper library -> $LIB_DIR/waash.ind"
  fi

  # Examples
  if [ -d "$REPO_DIR/share/waash/examples" ]; then
    mkdir -p "$PREFIX/share/waash/examples"
    cp "$REPO_DIR"/share/waash/examples/*.waash "$PREFIX/share/waash/examples/" 2>/dev/null || true
    ok "Installed example scripts"
  fi

  # Docs
  if [ -d "$REPO_DIR/docs" ]; then
    cp -r "$REPO_DIR"/docs/. "$DOCS_DIR/"
    ok "Installed docs -> $DOCS_DIR"
  fi

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
          grep -q "$BIN_DIR" "$HOME/.zshrc" 2>/dev/null || echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$HOME/.zshrc"
          ;;
        *)
          grep -q "$BIN_DIR" "$HOME/.bashrc" 2>/dev/null || echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$HOME/.bashrc"
          ;;
      esac
      ;;
  esac
}

# ── Check for Indent runtime (optional) ─────────────────────────────────
check_indent() {
  if command -v indent >/dev/null 2>&1 || [ -x "$HOME/.local/bin/indent" ]; then
    ok "Indent runtime found (scripts will work)"
  else
    warn "Indent runtime not found. WAASH scripts need it."
    warn "Install from https://indent.xytro.site or set WAASH_INDENT_BINARY."
  fi
}

# ── Version ─────────────────────────────────────────────────────────────
print_version() {
  if [ -f "$REPO_DIR/Cargo.toml" ]; then
    grep -m1 '^version' "$REPO_DIR/Cargo.toml" | sed 's/version *= *//; s/"//g' | awk '{print "waash "$1}'
  else
    echo "waash (unknown version)"
  fi
}

# ── Main ────────────────────────────────────────────────────────────────
main() {
  for arg in "$@"; do
    case "$arg" in
      --no-build) NO_BUILD=1 ;;
      --prefix) ;;
      --prefix=*) PREFIX="${arg#--prefix=}" ;;
      --version) print_version; exit 0 ;;
      -h|--help) sed -n '2,16p' "${BASH_SOURCE[0]}"; exit 0 ;;
      *) die "Unknown option: $arg (see --help)" ;;
    esac
  done

  echo
  echo "  ${c_yellow}WAASH — What An Amazing SHell${c_reset}"
  echo "  $(print_version) | ${OS:-linux}-${ARCH:-$(uname -m)}"
  echo

  detect_os
  if [ "$NO_BUILD" -eq 0 ]; then
    ensure_cargo
    build
  else
    [ -f "$REPO_DIR/target/release/waash" ] || die "--no-build needs target/release/waash (build first)"
    ok "Using existing build"
  fi
  install
  check_indent

  echo
  ok "WAASH installed! Start it with:  waash"
  info "Docs: $DOCS_DIR"
  info "If 'waash' isn't found, restart your shell or run: export PATH=\"$BIN_DIR:\$PATH\""
  echo
}

main "$@"
