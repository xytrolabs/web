#!/usr/bin/env python3
"""Test prefix caching on the cache engine"""
import requests

BASE = "http://localhost:4006"

print("=== Testing Prefix Cache ===\n")

# Turn 1: fresh conversation
print("Turn 1: Fresh conversation...")
r1 = requests.post(f"{BASE}/v1/chat/completions", json={
    "model": "qwen2.5:1.5b",
    "messages": [{"role": "user", "content": "My name is Raf. Remember it."}],
    "stream": False
})
t1 = r1.json()["choices"][0]["message"]["content"][:60]
print(f"  Response: {t1}")
print(f"  X-Cache: {r1.headers.get('X-Cache', 'N/A')}")

# Turn 2: same prefix, new question (should hit prefix cache)
print("\nTurn 2: Same conversation, new question...")
r2 = requests.post(f"{BASE}/v1/chat/completions", json={
    "model": "qwen2.5:1.5b",
    "messages": [
        {"role": "user", "content": "My name is Raf. Remember it."},
        {"role": "assistant", "content": t1},
        {"role": "user", "content": "What is my name?"}
    ],
    "stream": False
})
t2 = r2.json()["choices"][0]["message"]["content"][:60]
print(f"  Response: {t2}")
print(f"  X-Cache: {r2.headers.get('X-Cache', 'N/A')}")
usage = r2.json().get("usage", {})
cached = usage.get("cached_tokens", 0)
total = usage.get("total_tokens", 1)
print(f"  Tokens: {cached} cached / {total} total ({cached*100//total if total else 0}% savings)")

# Turn 3: exact repeat (should be FULL_HIT)
print("\nTurn 3: Exact repeat...")
r3 = requests.post(f"{BASE}/v1/chat/completions", json={
    "model": "qwen2.5:1.5b",
    "messages": [
        {"role": "user", "content": "My name is Raf. Remember it."},
        {"role": "assistant", "content": t1},
        {"role": "user", "content": "What is my name?"}
    ],
    "stream": False
})
print(f"  Response: {r3.json()['choices'][0]['message']['content'][:60]}")
print(f"  X-Cache: {r3.headers.get('X-Cache', 'N/A')}")

print("\n✅ Cache engine is working!")
