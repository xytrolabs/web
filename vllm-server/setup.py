#!/usr/bin/env python3
"""Setup the Valis Cache Engine — builds llama.cpp server with CUDA"""

import subprocess, os, sys

BASE = os.path.dirname(os.path.abspath(__file__))

def run(cmd):
    print(f"  $ {cmd}")
    subprocess.run(cmd, shell=True, check=True)

print("=== Valis Cache Engine Setup ===")

# 1. Check deps
print("\n1. Checking system dependencies...")
deps = ["cmake", "gcc", "g++", "git", "make"]
missing = []
for d in deps:
    r = subprocess.run(["which", d], capture_output=True)
    if r.returncode != 0:
        missing.append(d)
if missing:
    print(f"  Missing: {', '.join(missing)}")
    print(f"  Install with: sudo pacman -S {' '.join(missing)}")
    sys.exit(1)
print("  All present!")

# 2. Clone and build llama.cpp
print("\n2. Building llama.cpp server with CUDA...")
if not os.path.exists("llama.cpp"):
    run("git clone --depth 1 https://github.com/ggerganov/llama.cpp")
os.chdir("llama.cpp")
run("cmake -B build -DGGML_CUDA=ON")
nproc = os.cpu_count() or 4
run(f"cmake --build build --config Release -j{nproc} --target llama-server")

server_path = os.path.join(os.getcwd(), "build/bin/llama-server")
print(f"\n✅ Server built: {server_path}")

# 3. Done
print(f"""
Setup complete!

To start the cache engine:
  cd {BASE}
  ./start.sh

To test prefix caching:
  python3 test.py
""")
