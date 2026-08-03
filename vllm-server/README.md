# Valis Cache Engine — True KV-Cache Prefix Caching

## Goal
Run alongside Ollama on port 4006 with inference-level prefix caching.
Ollama on :4005 handles all current traffic; this engine is the upgrade path.

## How It Works
- llama.cpp server loads the same GGUF model files Ollama uses
- Explicit cache slots track which prompt prefixes are "hot"
- Same conversation prefix → cached tokens skip recomputation
- API returns `cached_tokens` count → billed at **20%** of normal rate

## Architecture
```
Browser → Cloudflare → ai.xytro.site:4005 (Ollama, current production)
                     → ai.xytro.site:4006 (llama.cpp, cache engine — future)

Valis API (server.js) → /v1/chat/completions → Ollama (4005)
Cache Engine            → /v1/chat/completions → llama.cpp (4006)
```

## Setup
```bash
cd vllm-server
python3 setup.py     # clones & builds llama.cpp with CUDA
./start.sh           # launches cache engine on :4006
python3 test.py      # verifies prefix caching works
```

## Requirements
- cmake, gcc, g++, git, make (`sudo pacman -S cmake gcc git make`)
- CUDA toolkit (already installed with NVIDIA drivers)
- Ollama running with models pulled

## Models (reuses Ollama's GGUF files)
- qwen2.5:0.5b → ~480MB
- qwen2.5:1.5b → ~1GB  
- qwen2.5:3b → ~2GB
- deepseek-r1:7b → ~4.7GB
- moondream:latest → ~1.7GB

## Cache Tiers

| Tier | What | Hit Rate | Billing |
|------|------|----------|---------|
| FULL_HIT | Exact prompt match | ~10% | Free |
| PREFIX_HIT | Same conversation, new question | ~40% | 20% rate |
| MISS | New prompt | ~50% | Full rate |

## Integration Plan
1. Build & test llama.cpp server ✅ (files ready)
2. Add as secondary backend in server.js
3. Route API key users to cache engine for cost savings
4. Eventually make cache engine the primary backend
