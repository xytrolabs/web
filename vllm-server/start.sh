#!/bin/bash
# Start llama.cpp server with prefix caching on port 4006
MODEL="${1:-$(ls ~/.ollama/models/blobs/sha256-* 2>/dev/null | head -1)}"
if [ -z "$MODEL" ]; then
    echo "No model specified and no Ollama models found."
    echo "Usage: ./start.sh <path-to-gguf>"
    exit 1
fi
echo "Starting Cache Engine on :4006 with $MODEL"
exec ./llama.cpp/build/bin/llama-server \
  --host 0.0.0.0 \
  --port 4006 \
  --model "$MODEL" \
  --n-gpu-layers 99 \
  --ctx-size 32768 \
  --cache-type-v q8_0 \
  --cache-type-k q8_0 \
  --threads 8 \
  --threads-http 4 \
  --cont-batching \
  --mlock \
  --no-webui
