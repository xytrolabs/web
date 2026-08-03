#!/usr/bin/env python3
"""Valis API Test Script — tests your API key end-to-end."""

import requests
import json
import sys

API_KEY = "valis_89ca7c65a92e881518f41b7f"
BASE = "https://ai.xytro.site/v1"
# For local testing: BASE = "http://localhost:4005/v1"

HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {API_KEY}"
}

def test(path, method="GET", body=None, label=""):
    url = BASE + path
    try:
        if method == "GET":
            r = requests.get(url, headers=HEADERS, timeout=30)
        elif method == "POST":
            r = requests.post(url, headers=HEADERS, json=body, timeout=30)
        else:
            r = requests.delete(url, headers=HEADERS, timeout=30)
        print(f"  {label or path}: HTTP {r.status_code}")
        if r.headers.get("content-type", "").startswith("application/json"):
            data = r.json()
            if "error" in data:
                print(f"    Error: {data['error'].get('message', data['error'])}")
            elif method != "DELETE":
                print(f"    {json.dumps(data, indent=4)[:300]}")
        return r
    except Exception as e:
        print(f"  {label or path}: FAILED - {e}")
        return None

if __name__ == "__main__":
    print(f"Valis API Test — Key: {API_KEY[:20]}...\n")

    # 1. Health
    test("/..", label="Health check (use /health on localhost)")

    # 2. List models
    test("/models", label="Available models")

    # 3. Pricing
    test("/pricing", label="Token pricing")

    # 4. Chat completion
    print("\n--- Chat Completion ---")
    test("/chat/completions", method="POST",
         body={"model": "xael-nano", "messages": [{"role": "user", "content": "What is 2+2?"}], "stream": False},
         label="Nano (free)")

    test("/chat/completions", method="POST",
         body={"model": "xael-mini", "messages": [{"role": "user", "content": "Explain gravity briefly"}], "stream": False},
         label="Mini (free)")

    test("/chat/completions", method="POST",
         body={"model": "xael-think", "messages": [{"role": "user", "content": "Write a haiku about AI"}], "stream": False},
         label="Think (gold, uses balance)")

    # 5. Usage stats
    print("\n--- Usage ---")
    test("/usage", label="Your usage stats")

    # 6. Cache stats
    print("\n--- Cache ---")
    test("/cache-stats", label="Cache stats")
