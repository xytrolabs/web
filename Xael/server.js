import express from "express";
import session from "express-session";
import cors from "cors";
import helmet from "helmet";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, appendFileSync, mkdirSync, existsSync, readdirSync, statSync, unlinkSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import Database from "better-sqlite3";
import bcrypt from "bcryptjs";
import { initBackups, runBackup } from "./backup.js";

const __dirname = dirname(fileURLToPath(import.meta.url));
const app = express();
const PORT = process.env.XAEL_PORT || 4005;
const OLLAMA_BASE = "http://127.0.0.1:11434";
const DATA_DIR = join(__dirname, "data");
const XYTROMAILING_DB = "/run/media/raf/Z/PrismTechnologies/XytroMailing/data/xytromailing.db";
const SESSION_SECRET = process.env.SESSION_SECRET || "local-session-secret-change-me-now";

let userDb;
try { userDb = new Database(XYTROMAILING_DB, { readonly: true }); } catch (e) { userDb = null; }

const sessionDb = new Database(join(DATA_DIR, "xael-sessions.db"));
sessionDb.exec("CREATE TABLE IF NOT EXISTS sessions (sid TEXT PRIMARY KEY, expires INTEGER, data TEXT)");

class SQLiteSessionStore extends session.Store {
  get(sid, cb) { try { const row = sessionDb.prepare("SELECT data FROM sessions WHERE sid = ? AND expires > ?").get(sid, Date.now()); cb(null, row ? JSON.parse(row.data) : null); } catch (e) { cb(e); } }
  set(sid, sess, cb) { try { sessionDb.prepare("INSERT OR REPLACE INTO sessions (sid, expires, data) VALUES (?, ?, ?)").run(sid, Date.now() + 7*24*60*60*1000, JSON.stringify(sess)); cb(null); } catch (e) { cb(e); } }
  destroy(sid, cb) { try { sessionDb.prepare("DELETE FROM sessions WHERE sid = ?").run(sid); cb(null); } catch (e) { cb(e); } }
  touch(sid, sess, cb) { try { sessionDb.prepare("UPDATE sessions SET expires = ? WHERE sid = ?").run(Date.now() + 7*24*60*60*1000, sid); cb(null); } catch (e) { cb(e); } }
}

if (!existsSync(DATA_DIR)) mkdirSync(DATA_DIR, { recursive: true });

app.set("trust proxy", 1);
app.use(helmet({ crossOriginResourcePolicy: false, crossOriginEmbedderPolicy: false, contentSecurityPolicy: false }));
app.use(cors({ origin: true, credentials: true }));
app.use((_req, res, next) => { res.setTimeout(120000, () => { if (!res.headersSent) res.status(504).json({error:{message:"Request timeout"}}); }); next(); });
app.use(express.json({ limit: "25mb" }));
app.use(session({ name: "xael.sid", store: new SQLiteSessionStore(), secret: SESSION_SECRET, resave: false, saveUninitialized: false, proxy: true, cookie: { httpOnly: true, sameSite: "lax", maxAge: 7*24*60*60*1000, secure: "auto", domain: ".xytro.site" } }));
setInterval(() => { sessionDb.prepare("DELETE FROM sessions WHERE expires < ?").run(Date.now()); }, 300000);

function getUserById(id) { if (!userDb) return null; try { return userDb.prepare("SELECT id, username, email FROM users WHERE id = ?").get(id); } catch { return null; } }
function getUserByLogin(login) { if (!userDb) return null; try { return userDb.prepare("SELECT id, username, password_hash, email FROM users WHERE username = ? OR email = ?").get(login, login); } catch { return null; } }

const SERVER_BOOT = Date.now();
app.get("/v1/auth/me", async (req, res) => {
  if (req.session?.userId && (!req.session._created || req.session._created > SERVER_BOOT - 300000)) { const u = getUserById(req.session.userId); if (u) return res.json({ authenticated: true, id: u.id, username: u.username, email: u.email }); }
  try { const xm = await fetch("http://127.0.0.1:4000/auth/me", { headers: { cookie: req.headers.cookie || "" } }); if (xm.ok) { const d = await xm.json(); if (d.authenticated) return res.json(d); } } catch {}
  const key = (req.headers.authorization || "").replace(/^Bearer\s+/i, "").trim();
  if (key && key.length >= 8) return res.json({ authenticated: true, keyAuth: true });
  return res.json({ authenticated: false });
});
app.post("/v1/auth/login", async (req, res) => {
  try {
    const { username, password } = req.body;
    if (!username || !password) return res.status(400).json({ error: "Username and password required" });
    const user = getUserByLogin(username);
    if (!user || !await bcrypt.compare(password, user.password_hash)) return res.status(401).json({ error: "Invalid username or password" });
    req.session.userId = user.id; req.session.username = user.username; req.session.email = user.email; req.session._created = Date.now();
    req.session.save(err => err ? res.status(500).json({ error: "Session error" }) : res.json({ ok: true }));
  } catch (e) { res.status(502).json({ error: "Login error" }); }
});

app.post("/v1/auth/logout", (req, res) => { req.session.destroy(() => { res.clearCookie("xael.sid", { domain: ".xytro.site", path: "/" }); res.json({ ok: true }); }); });

async function authenticate(req, res, next) {
  const key = (req.headers.authorization || "").replace(/^Bearer\s+/i, "").trim();
  if (key && key.length >= 10) { const kd = keyStore[key]; if (kd) { if ((userBalances[kd.userId]||0) <= 0) return res.status(402).json({ error: { message: "Insufficient balance" } }); req.apiKey = key; req.freeMode = false; req.keyData = kd; req.userId = kd.userId; return next(); } return res.status(401).json({ error: { message: "Invalid API key" } }); }
  if (req.session?.userId) { const u = getUserById(req.session.userId); if (u) { req.user = u; req.userId = u.email || u.id; req.freeMode = false; return next(); } }
  const host = (req.headers.host||"").toLowerCase(); if (host.includes("valischat")||host.includes("localhost")) { req.freeMode = true; req.isChatSite = host.includes("valischat"); return next(); }
  return res.status(401).json({ error: { message: "API key required" } });
}

const MODELS = { "xael-nano": { ollama: "qwen2.5:0.5b", label: "Valis Nano", pricePer1M: 0.05, contextLimit: 8192, draftModel: true }, "xael-mini": { ollama: "xael-nano-q3:latest", label: "Valis Mini", pricePer1M: 0.15, contextLimit: 16384 }, "xael-turbo": { ollama: "qwen2.5:3b", label: "Valis Turbo", pricePer1M: 0.35, contextLimit: 32768 }, "xael-think": { ollama: "hf.co/LiquidAI/LFM2.5-1.2B-Thinking-GGUF:Q4_K_M", label: "Valis Think", pricePer1M: 0.50, contextLimit: 16384 }, "xael-thinkXL": { ollama: "xael-r1-fast:latest", label: "Valis Think XL", pricePer1M: 0.75, contextLimit: 32768 }, "xael-vision": { ollama: "moondream:latest", label: "Valis Vision", pricePer1M: 0, contextLimit: 3072, free: true } };
const TOKEN_PRICES = { "xael-nano": { input: 0.02, output: 0.03 }, "xael-mini": { input: 0.05, output: 0.10 }, "xael-turbo": { input: 0.10, output: 0.25 }, "xael-think": { input: 0.20, output: 0.30 }, "xael-thinkXL": { input: 0.30, output: 0.45 }, "xael-vision": { input: 0, output: 0 } };

const USAGE_FILE = join(DATA_DIR, "usage.json"), KEYS_FILE = join(DATA_DIR, "api-keys.json");
let usageStore = {}, keyStore = {}, userBalances = {}, paymentsStore = [];
function getUsageKey(uid) { return "user_" + String(uid||"anon").replace(/[^a-zA-Z0-9@._-]/g, "_").slice(0, 64); }
try { if (existsSync(USAGE_FILE)) usageStore = JSON.parse(readFileSync(USAGE_FILE, "utf-8")); } catch {}
try { if (existsSync(KEYS_FILE)) { const l = JSON.parse(readFileSync(KEYS_FILE, "utf-8")); for (const [k,v] of Object.entries(l)) { keyStore[k] = { userId: v.userId, created: v.created, totalTokens: v.totalTokens||0 }; if (v.userId && v._balance !== undefined) userBalances[v.userId] = Math.min(userBalances[v.userId]||999, v._balance); } } } catch {}
function saveUsageStore() { try { const prev = existsSync(USAGE_FILE) ? JSON.parse(readFileSync(USAGE_FILE, "utf-8")) : null; const prevKeys = prev ? Object.keys(prev).length : 0; const curKeys = Object.keys(usageStore).length; if (curKeys === 0 && prevKeys > 0) { console.error("[Valis] Refusing to overwrite usage file — memory store is empty but file has " + prevKeys + " entries. Restoring from file."); usageStore = prev; return; } writeFileSync(USAGE_FILE, JSON.stringify(usageStore)); } catch(e) { console.error("[Valis] Failed to save usage:", e.message); } }
function saveKeyStore() { try { const prev = existsSync(KEYS_FILE) ? JSON.parse(readFileSync(KEYS_FILE, "utf-8")) : null; const prevKeys = prev ? Object.keys(prev).length : 0; const curKeys = Object.keys(keyStore).length; if (curKeys === 0 && prevKeys > 0) { console.error("[Valis] Refusing to overwrite keys file — memory store is empty but file has " + prevKeys + " entries. Restoring from file."); for (const [k,v] of Object.entries(prev)) { keyStore[k] = { userId: v.userId, created: v.created, totalTokens: v.totalTokens||0 }; if (v.userId && v._balance !== undefined) userBalances[v.userId] = Math.min(userBalances[v.userId]||999, v._balance); } return; } const out = {}; for (const [k,v] of Object.entries(keyStore)) { out[k] = { userId: v.userId, created: v.created, totalTokens: v.totalTokens||0, _balance: userBalances[v.userId] }; } writeFileSync(KEYS_FILE, JSON.stringify(out)); } catch(e) { console.error("[Valis] Failed to save keys:", e.message); } }
setInterval(() => { saveUsageStore(); saveKeyStore(); }, 10000);
// ── Automated backups every 5 minutes ──
const BACKUP_DIR = initBackups(DATA_DIR);
backupData(); // initial backup on startup
setInterval(backupData, 5 * 60 * 1000);
function backupData() { runBackup(BACKUP_DIR, [["usage", usageStore], ["api-keys", keyStore], ["payments", paymentsStore]]); }


function trackUsage(userId, model, pt, ct, wasCached) {
  const key = getUsageKey(userId||"anon"); if (!usageStore[key]) usageStore[key] = { requests: 0, tokens: 0, cost: 0, models: {}, daily: {} };
  const u = usageStore[key]; u._ownerId = userId; u.requests++; const tokens = (pt||0)+(ct||0); u.tokens += tokens; u.promptTokens = (u.promptTokens||0)+(pt||0); u.cacheHits = (u.cacheHits||0)+(wasCached?1:0);
  const p = TOKEN_PRICES[model]||{input:0,output:0}; const cost = ((pt||0)*p.input+(ct||0)*p.output)/1_000_000; u.cost += cost;
  if (!u.models[model]) u.models[model] = { requests: 0, tokens: 0, cost: 0, cacheHits: 0 }; u.models[model].requests++; u.models[model].tokens += tokens; u.models[model].cost += cost; if (wasCached) u.models[model].cacheHits++;
  const today = new Date().toISOString().slice(0,10); if (!u.daily[today]) u.daily[today] = { requests: 0, tokens: 0, cost: 0 }; u.daily[today].requests++; u.daily[today].tokens += tokens; u.daily[today].cost += cost;
  saveUsageStore();
}

app.get("/health", (_req, res) => res.json({ status: "ok", engine: "Valis AI" }));
app.get("/v1/pricing", (_req, res) => res.json(TOKEN_PRICES));
app.get("/v1/models", (_req, res) => { res.json({ object: "list", data: Object.entries(MODELS).map(([id, m]) => ({ id, object: "model", owned_by: "xytro-labs", label: m.label, context_length: m.contextLimit, pricing: TOKEN_PRICES[id] || { input: 0, output: 0 } })) }); });
app.get("/v1/models", authenticate, (req, res) => { if (req.freeMode) return res.status(401).json({ error: "Auth required" }); res.json({ object: "list", data: Object.entries(MODELS).map(([id, m]) => ({ id, object: "model", owned_by: "xytro-labs", label: m.label, pricing: TOKEN_PRICES[id] })) }); });
app.get("/v1/usage", authenticate, (req, res) => { if (req.freeMode) return res.status(401).json({ error: "Login required" }); const uid = req.userId || (req.user && req.user.email) || "anon"; const key = getUsageKey(uid); const raw = JSON.parse(JSON.stringify(usageStore[key]||{requests:0,tokens:0,cost:0,models:{},daily:{}})); if (raw.tokens>0&&raw.requests===0) raw.requests=1; res.json({ requests: raw.requests, tokens: raw.tokens, cost: raw.cost, promptTokens: raw.promptTokens||0, cacheHits: raw.cacheHits||0, cacheRatio: raw.requests>0?Math.round((raw.cacheHits||0)/raw.requests*100):0, cacheSavings: +((raw.promptTokens||0)*0.0000185).toFixed(6), models: Object.entries(raw.models||{}).filter(([n])=>n!=="(all models)").map(([n,m])=>({name:n,...m})), daily: Object.entries(raw.daily||{}).map(([d,dd])=>({date:d,...dd})).sort((a,b)=>a.date.localeCompare(b.date)) }); });
app.get("/v1/keys", authenticate, (req, res) => { const userId = req.userId || (req.user && (req.user.email || req.user.id)); if (!userId) return res.status(401).json({ error: "Login required" }); const ids = [userId]; if (req.user) { ids.push(req.user.id, req.user.username, req.user.email); } const bal = userBalances[userId]||0; res.json(Object.entries(keyStore).filter(([,v])=>ids.includes(v.userId)).map(([k,v])=>({key:k.slice(0,14)+"...",fullKey:k,created:v.created,balance:bal,totalTokens:v.totalTokens||0}))); });
app.post("/v1/keys", authenticate, (req, res) => {
  const userId = (req.user && (req.user.email || req.user.id)) || (req.keyData && req.keyData.userId);
  if (!userId) return res.status(401).json({ error: "Login required — sign in to create API keys" }); if (!Object.values(keyStore).some(k=>k.userId===userId)) userBalances[userId]=1; const key = "valis_"+createHash("sha256").update(Date.now()+Math.random().toString()).digest("hex").slice(0,24); keyStore[key]={created:new Date().toISOString(),userId}; saveKeyStore(); res.json({key,balance:userBalances[userId]}); });
app.delete("/v1/keys/:key", authenticate, (req, res) => { if (!req.user) return res.status(401).json({ error: "Login required" }); const ids = [req.user.id,req.user.username,req.user.email].filter(Boolean); if (keyStore[req.params.key]&&ids.includes(keyStore[req.params.key].userId)) { delete keyStore[req.params.key]; saveKeyStore(); return res.json({ok:true}); } res.status(404).json({error:"Key not found"}); });
app.get("/v1/payments", authenticate, (req, res) => { if (!req.user) return res.json([]); res.json(paymentsStore.filter(p=>p.userId===(req.user.id||req.user.username))); });
app.post("/v1/payments", authenticate, (req, res) => { if (!req.user) return res.status(401).json({error:"Login required"}); paymentsStore.push({id:"pay_"+Date.now().toString(36)+Math.random().toString(36).slice(2,6),userId:req.user.id||req.user.username,username:req.user.username,email:req.user.email,amount:req.body?.amount||0,method:req.body?.method||"cashapp",note:req.body?.note||"",created:new Date().toISOString(),status:"pending"}); savePayments(); res.json({ok:true,id:"ok"}); });


const PAYMENTS_FILE = join(DATA_DIR, "payments.json");
function savePayments() { try { writeFileSync(PAYMENTS_FILE, JSON.stringify(paymentsStore)); } catch {} }
function reloadPaymentsAndKeys() {
  try {
    if (existsSync(PAYMENTS_FILE)) { var raw = JSON.parse(readFileSync(PAYMENTS_FILE, "utf-8")); paymentsStore = Array.isArray(raw) ? raw : Object.values(raw); }
    if (existsSync(KEYS_FILE)) { var l = JSON.parse(readFileSync(KEYS_FILE, "utf-8")); Object.assign(keyStore, l); for (var [k,v] of Object.entries(l)) { if (v.userId && v._balance !== undefined) userBalances[v.userId] = Math.max(userBalances[v.userId]||0, v._balance); } }
  } catch(e) { console.error("[Valis] Reload error:", e.message); }
}
try {
  if (existsSync(PAYMENTS_FILE)) { var raw = JSON.parse(readFileSync(PAYMENTS_FILE, "utf-8")); paymentsStore = Array.isArray(raw) ? raw : Object.values(raw); }
} catch { paymentsStore = []; }
app.post("/v1/admin/reload", (_req, res) => { reloadPaymentsAndKeys(); res.json({ok:true}); });
app.post("/v1/chat/completions", authenticate, async (req, res) => {
  const { model="xael-nano", messages: msgs=[], stream=false } = req.body;
  if (!msgs.length) return res.status(400).json({error:{message:"messages required"}});
  const mc = MODELS[model]||MODELS["xael-nano"];
  const messages = msgs.some(m=>m.role==="system") ? msgs : [{role:"system",content:"You are Valis AI by Xytro Labs. When web context is provided, ALWAYS cite your sources. Be concise but thorough."},...msgs];
  try {
    // Augment with web context (shared by both stream and non-stream)
    let augmentedMessages = messages;
    try { if (typeof augmentWithWebContext === "function") augmentedMessages = await augmentWithWebContext(messages); } catch {}
    if (stream) {
      const or = await fetch(OLLAMA_BASE+"/api/chat",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({model:mc.ollama,messages:augmentedMessages,stream:true,options:{num_predict:mc.contextLimit>8192?12288:6144,num_ctx:mc.contextLimit,num_gpu:99,use_mmap:true,f16_kv:false,low_vram:false,use_mlock:true}})});
      res.setHeader("Content-Type","text/event-stream"); res.setHeader("Cache-Control","no-cache");
      const reader = or.body.getReader(); const decoder = new TextDecoder(); let pt=0, ct=0;
      while (true) { const {done,value} = await reader.read(); if (done) break;
        for (const line of decoder.decode(value,{stream:true}).split("\\n").filter(l=>l.trim())) { try { const d=JSON.parse(line); if (d.prompt_eval_count) pt=d.prompt_eval_count; if (d.eval_count) ct=d.eval_count; if (d.message?.content) res.write("data: "+JSON.stringify({id:"c-"+Date.now(),object:"chat.completion.chunk",created:Math.floor(Date.now()/1000),model,choices:[{delta:{content:d.message.content},index:0}]})+"\\n\\n"); } catch {} } }
      res.write("data: [DONE]\\n\\n"); res.end();
      if (!mc.free&&!req.isChatSite&&req.keyData) { trackUsage(req.keyData.userId,model,pt,ct,false); const cost = ((pt*(TOKEN_PRICES[model]?.input||0)+ct*(TOKEN_PRICES[model]?.output||0))/1_000_000); userBalances[req.keyData.userId]=Math.max(0,(userBalances[req.keyData.userId]||0)-cost); req.keyData.totalTokens=(req.keyData.totalTokens||0)+pt+ct; saveKeyStore(); }
    } else {
      const cachedResp = getCachedResponse(req.keyData?.userId||req.userId, model, augmentedMessages); if (cachedResp && !stream) { trackUsage(req.keyData?.userId||"anon",model,cachedResp.usage.prompt_tokens,cachedResp.usage.completion_tokens,true); if (req.keyData && !mc.free) { const cc=((cachedResp.usage.prompt_tokens*(TOKEN_PRICES[model]?.input||0)+cachedResp.usage.completion_tokens*(TOKEN_PRICES[model]?.output||0))/1_000_000); userBalances[req.keyData.userId]=Math.max(0,(userBalances[req.keyData.userId]||0)-cc); req.keyData.totalTokens=(req.keyData.totalTokens||0)+cachedResp.usage.prompt_tokens+cachedResp.usage.completion_tokens; saveKeyStore(); } saveUsageStore(); return res.json({id:"c-"+Date.now(),object:"chat.completion",created:Math.floor(Date.now()/1000),model,choices:[{index:0,message:{role:"assistant",content:cachedResp.content},finish_reason:"stop"}],usage:cachedResp.usage, cached:true, source: cachedResp.source || null}); } const or = await fetch(OLLAMA_BASE+"/api/chat",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({model:mc.ollama,messages:augmentedMessages,stream:false,options:{num_predict:mc.contextLimit>8192?12288:6144,num_ctx:mc.contextLimit,num_gpu:99,use_mmap:true,f16_kv:false,low_vram:false,use_mlock:true}})});
      const r=await or.json(); const msg=r.message?.content||""; const pt=r.prompt_eval_count||0; const ct=r.eval_count||0; const peDur=r.prompt_eval_duration||0; const cached=pt>0&&peDur>0&&(peDur/pt)<5_000_000;
      const sourceUrl = messages !== augmentedMessages ? (augmentedMessages.find(m=>m.content?.startsWith("WEB SOURCE"))?.content?.match(/https?:\/\/[^\s\]]+/)?.[0] || null) : null; const resp={id:"c-"+Date.now(),object:"chat.completion",created:Math.floor(Date.now()/1000),model,choices:[{index:0,message:{role:"assistant",content:msg},finish_reason:"stop"}],usage:{prompt_tokens:pt,completion_tokens:ct,total_tokens:pt+ct}, source: sourceUrl}; saveCachedResponse(req.keyData?.userId||req.userId, model, augmentedMessages, msg, pt, ct);
      trackUsage(req.keyData?.userId||"anon",model,pt,ct,cached); 
      if (!mc.free&&req.keyData&&!req.isChatSite) { const cost=((pt*(TOKEN_PRICES[model]?.input||0)+ct*(TOKEN_PRICES[model]?.output||0))/1_000_000); userBalances[req.keyData.userId]=Math.max(0,(userBalances[req.keyData.userId]||0)-cost); req.keyData.totalTokens=(req.keyData.totalTokens||0)+pt+ct; saveKeyStore(); }
      saveUsageStore(); res.json(resp);
    }
  } catch(e) { console.error("[Valis]",e.message); res.status(503).json({error:{message:"AI engine unavailable"}}); }
});

function requireMailLogin(req,res,next) { if (req.session?.userId) return next(); return res.redirect("/dashboard.html"); }
app.get("/api/mail/inbox",requireMailLogin,(req,res)=>{try{const db=new Database(XYTROMAILING_DB,{readonly:true});const rows=db.prepare("SELECT id,sender_email,subject,is_read,received_at FROM mailbox_messages WHERE recipient_email=? ORDER BY received_at DESC LIMIT 50").all(req.session.email||"");db.close();res.json(rows);}catch(e){res.status(500).json({error:e.message});}});
app.get("/api/mail/message/:id",requireMailLogin,(req,res)=>{try{const db=new Database(XYTROMAILING_DB,{readonly:true});const row=db.prepare("SELECT * FROM mailbox_messages WHERE id=?").get(req.params.id);db.close();if(!row)return res.status(404).json({error:"Not found"});res.json(row);}catch(e){res.status(500).json({error:e.message});}});

app.use(express.static(join(__dirname,"public"), { setHeaders: (res, path) => { if (path.endsWith(".html")) { res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate"); res.setHeader("Pragma", "no-cache"); res.setHeader("Expires", "0"); } } }));
process.on("uncaughtException", (e) => { console.error("[Valis] Uncaught:", e.message); });
process.on("unhandledRejection", (e) => { console.error("[Valis] Unhandled rejection:", e?.message || e); });
// Cache -> Cloud Storage (system drive /home)
const CACHE_STORAGE_DIR = join(process.env.HOME || "/home/raf", ".xael-cache", "tokens");
if (!existsSync(CACHE_STORAGE_DIR)) mkdirSync(CACHE_STORAGE_DIR, { recursive: true });

async function storeCacheTokens(userId, model, tokenCount) {
  const now = new Date().toISOString();
  const safeId = String(userId||"anon").replace(/[^a-zA-Z0-9@._-]/g, "_");
  const userDir = join(CACHE_STORAGE_DIR, safeId);
  if (!existsSync(userDir)) mkdirSync(userDir, { recursive: true });
  const dayFile = join(userDir, now.slice(0,10) + ".cache");
  appendFileSync(dayFile, JSON.stringify({ts:now, model, tokens:tokenCount}) + "\n");
}


// ── Response Cache (cloud-stored, per-user, instant replies) ──
// ── Response Cache (XytroCloud user storage, LRU eviction, never expires) ──
const CLOUD_USERS = "/run/media/raf/C/xytrocloud/users";
const CACHE_SUBDIR = "valis-cache";
function getUserStorageLimit(userId) {
  try {
    const db = new Database("/run/media/raf/Z/PrismTechnologies/XytroCloud/omnicloud.db", { readonly: true });
    const row = db.prepare("SELECT total_space, used_space FROM cloud_accounts WHERE user_id = ? AND status = 'active' LIMIT 1").get(userId);
    db.close();
    if (row) return { total: row.total_space, used: row.used_space, free: row.total_space - row.used_space };
  } catch {}
  return { total: 1073741824, used: 0, free: 1073741824 };
}
function cacheKey(model, messages) { return createHash("sha256").update(JSON.stringify({model, messages})).digest("hex").slice(0, 24); }

function cloudCachePath(userId) {
  const dir = String(userId||"anon").replace(/@/g, "_").replace(/[^a-zA-Z0-9._-]/g, "_");
  return join(CLOUD_USERS, dir, CACHE_SUBDIR);
}

function getCachedResponse(userId, model, messages) {
  const ck = cacheKey(model, messages);
  const file = join(cloudCachePath(userId), ck + ".json");
  try { return existsSync(file) ? JSON.parse(readFileSync(file, "utf-8")) : null; }
  catch { return null; }
}

function saveCachedResponse(userId, model, messages, content, pt, ct) {
  const dir = cloudCachePath(userId);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  const ck = cacheKey(model, messages);
  const entry = { ts: Date.now(), model, content, usage: { prompt_tokens: pt, completion_tokens: ct } };
  try { writeFileSync(join(dir, ck + ".json"), JSON.stringify(entry)); } catch {}
  // Update XytroCloud used_space directly
  try {
    const xdb = new Database("/run/media/raf/Z/PrismTechnologies/XytroCloud/omnicloud.db");
    xdb.prepare("UPDATE cloud_accounts SET used_space = used_space + ?, updated_at = CURRENT_TIMESTAMP WHERE user_id = ? AND status = 'active'").run(JSON.stringify(entry).length, userId);
    xdb.close();
  } catch {}
  // LRU eviction: if over 100MB, delete oldest entries
  try {
    const files = readdirSync(dir).map(f => {
      const s = statSync(join(dir, f));
      return { name: f, mtime: s.mtimeMs, size: s.size };
    }).sort((a,b) => a.mtime - b.mtime);
    let total = files.reduce((s,f) => s + f.size, 0);
    while (total > getUserStorageLimit(userId).free && files.length > 0) {
      const oldest = files.shift();
      try { unlinkSync(join(dir, oldest.name)); total -= oldest.size; try { const xdb3 = new Database("/run/media/raf/Z/PrismTechnologies/XytroCloud/omnicloud.db"); xdb3.prepare("UPDATE cloud_accounts SET used_space = MAX(0, used_space - ?), updated_at = CURRENT_TIMESTAMP WHERE user_id = ? AND status = 'active'").run(oldest.size, userId); xdb3.close(); } catch {} } catch {}
    }
  } catch {}
}

app.get("/v1/cache/responses", authenticate, (req, res) => {
  const uid = (req.userId || (req.user?.email||"anon")).replace(/@/g, "_").replace(/[^a-zA-Z0-9._-]/g, "_");
  const dir = join(CLOUD_USERS, uid, CACHE_SUBDIR);
  if (!existsSync(dir)) return res.json({ entries: 0, bytes: 0 });
  let count = 0, bytes = 0;
  try { for (const f of readdirSync(dir)) { count++; try { bytes += statSync(join(dir, f)).size; } catch {} } } catch {}
  const ul = getUserStorageLimit(uid); res.json({ entries: count, bytes, limit: ul.total, used: ul.used, free: ul.free });
});

// ── Web Context: fetch URLs and search for small-model augmentation ──
async function fetchWebContext(userMessage) {
  // 1. Check for explicit URLs
  const urlMatch = userMessage.match(/https?:\/\/[^\s]+/);
  if (urlMatch) {
    try {
      const res = await fetch(urlMatch[0], { signal: AbortSignal.timeout(5000) });
      const html = await res.text();
      const text = html.replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 3000);
      return "LIVE WEB PAGE (use this as your primary source):\n" + urlMatch[0] + "\n" + text;
    } catch { return null; }
  }
  // 2. Autonomous search via DuckDuckGo Instant Answer API (free, no key)
  try {
    const query = encodeURIComponent(userMessage.slice(0, 200));
    const ddg = await fetch("https://api.duckduckgo.com/?q=" + query + "&format=json&no_html=1&skip_disambig=1", { signal: AbortSignal.timeout(5000) });
    const data = await ddg.json();
    const parts = [];
    if (data.AbstractText) parts.push(data.AbstractText);
    if (data.Answer) parts.push("Answer: " + data.Answer);
    if (data.Definition) parts.push("Definition: " + data.Definition);
    if (data.Heading) parts.push("Heading: " + data.Heading);
    // Also include related topics
    if (data.RelatedTopics) {
      for (const t of data.RelatedTopics.slice(0, 3)) {
        if (t.Text) parts.push(t.Text);
      }
    }
    if (parts.length > 0) {
      return "DUCKDUCKGO RESULTS (cite these sources):\n" + parts.join("\n").slice(0, 2500) + "\n\n[When answering, mention that this information comes from web search and cite DuckDuckGo as the source.]";
    }
  } catch {}
  // 3. Wikipedia API fallback (free, no key)
  try {
    const searchQuery = encodeURIComponent(userMessage.slice(0, 200));
    const wiki = await fetch("https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch=" + searchQuery + "&format=json&srlimit=3", { signal: AbortSignal.timeout(5000) });
    const wdata = await wiki.json();
    const results = (wdata.query?.search || []).map(r => r.snippet.replace(/<[^>]+>/g, '')).filter(s => s.length > 30);
    if (results.length > 0) {
      return "WIKIPEDIA RESULTS (cite these sources):\n" + results.map((s,i) => (i+1) + ". " + s).join("\n").slice(0, 2500) + "\n\n[When answering, cite Wikipedia as your source. Mention the article titles if possible.]";
    }
  } catch {}
  return null;
}

// Augment messages with web context before Ollama
async function augmentWithWebContext(messages) {
  const lastUser = [...messages].reverse().find(m => m.role === 'user');
  if (!lastUser) return messages;
  const ctx = await fetchWebContext(lastUser.content);
  if (!ctx) return messages;
  const sysIdx = messages.findIndex(m => m.role === 'system');
  const augmented = [...messages];
  if (sysIdx >= 0) {
    augmented[sysIdx] = { ...augmented[sysIdx], content: augmented[sysIdx].content + "\n\n" + ctx };
  } else {
    augmented.unshift({ role: 'system', content: ctx });
  }
  return augmented;
}

app.listen(PORT,"127.0.0.1",()=>console.log("Xael Liquid API on port "+PORT));
