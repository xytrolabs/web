#!/bin/bash
# Zor Mini Kernel Build Script — works on stable Rust
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Zor Kernel Build ==="

# Step 1: Transpile Zor → Rust
echo "[1/2] Transpiling kernel.zor → Rust..."
cd "$SCRIPT_DIR/../.."
ZOR_NO_STD=1 ./target/release/zor build examples/kernel/kernel.zor 2>&1 | grep -v "^warning\|^  |\|^  =\|^$"
if [ -f kernel ]; then
    mv kernel "$SCRIPT_DIR/kernel.o"
    echo "  kernel.o ready"
fi

# Step 2: Compile Rust to bare-metal object (stable — no custom target)
if [ -f "$SCRIPT_DIR/kernel.rs" ]; then
    echo "[2/2] Compiling to bare-metal object..."
    rustc --edition 2021 -C panic=abort -C opt-level=2 \
        -C relocation-model=static --emit=obj \
        "$SCRIPT_DIR/kernel.rs" -o "$SCRIPT_DIR/kernel.o" 2>&1 | grep -v "^warning\|^  |\|^  ="
fi

echo ""
echo "=== Done ==="
ls -la "$SCRIPT_DIR/kernel.o" 2>/dev/null && echo "Ready for linking!"
echo "Run: nasm -f elf64 boot.asm -o boot.o && ld.lld -T linker.ld -o kernel.bin boot.o kernel.o"
