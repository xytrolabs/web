import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const CACHE_DIR = join(process.env.HOME || "/home/raf", ".xael-cache");
const CACHE_FILE = join(CACHE_DIR, "kv-cache.json");
const CACHE_WARM_FILE = join(CACHE_DIR, "warm-prompts.json");

// Ensure cache dir exists on system drive
if (!existsSync(CACHE_DIR)) mkdirSync(CACHE_DIR, { recursive: true });

let cacheLog = {};
let warmPrompts = [];

// Load existing cache
try { if (existsSync(CACHE_FILE)) cacheLog = JSON.parse(readFileSync(CACHE_FILE, "utf-8")); } catch {}
try { if (existsSync(CACHE_WARM_FILE)) warmPrompts = JSON.parse(readFileSync(CACHE_WARM_FILE, "utf-8")); } catch {}

export function logCacheHit(userId, model, promptTokens) {
  const now = new Date().toISOString();
  if (!cacheLog[userId]) cacheLog[userId] = { hits: 0, tokens: 0, lastHit: now, prompts: [] };
  cacheLog[userId].hits++;
  cacheLog[userId].tokens += promptTokens;
  cacheLog[userId].lastHit = now;

  // Save every 10 cache hits to reduce I/O
  if (cacheLog[userId].hits % 10 === 0) {
    try { writeFileSync(CACHE_FILE, JSON.stringify(cacheLog)); } catch {}
  }
}

export function saveWarmPrompt(model, messages) {
  const entry = { model, messages: messages.slice(-2), timestamp: Date.now() };
  warmPrompts.push(entry);
  // Keep last 50 prompts
  if (warmPrompts.length > 50) warmPrompts = warmPrompts.slice(-50);
  try { writeFileSync(CACHE_WARM_FILE, JSON.stringify(warmPrompts)); } catch {}
}

export function getWarmPrompts() { return warmPrompts; }
export function getCacheStats(userId) { return cacheLog[userId] || { hits: 0, tokens: 0 }; }
export function flushCacheLog() { try { writeFileSync(CACHE_FILE, JSON.stringify(cacheLog)); } catch {} }

// Flush on exit
process.on("exit", flushCacheLog);
process.on("SIGTERM", () => { flushCacheLog(); process.exit(); });
process.on("SIGINT", () => { flushCacheLog(); process.exit(); });
