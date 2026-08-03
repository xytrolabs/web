import express from "express";
import session from "express-session";
import cors from "cors";
import helmet from "helmet";
import rateLimit from "express-rate-limit";
import { createHash, randomBytes } from "node:crypto";
import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync, unlinkSync, rmdirSync, statSync, createReadStream } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import Database from "better-sqlite3";
import bcrypt from "bcryptjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const app = express();
const PORT = process.env.XAEL_PORT || 4005;
const OLLAMA_BASE = process.env.OLLAMA_BASE || "http://127.0.0.1:11434";
const CACHE_ENGINE_BASE = process.env.CACHE_ENGINE_BASE || "http://127.0.0.1:4006";
const DATA_DIR = join(__dirname, "data");
const CHATS_DIR = join(DATA_DIR, "chats");
const PROFILES_DIR = join(DATA_DIR, "profiles");
const CLOUD_USERS_BASE = "/run/media/raf/C/xytrocloud/users";
const XYTROMAILING_DB = process.env.XYTROMAILING_DB || "/run/media/raf/Z/PrismTechnologies/XytroMailing/data/xytromailing.db";
const SESSION_SECRET = process.env.SESSION_SECRET || "local-session-secret-change-me-now";

let userDb;
try { userDb = new Database(XYTROMAILING_DB, { readonly: true }); }
catch (e) { console.error("[Xael] Cannot open user DB:", e.message); userDb = null; }

const sessionDb = new Database(join(DATA_DIR, "xael-sessions.db"));
sessionDb.exec("CREATE TABLE IF NOT EXISTS sessions (sid TEXT PRIMARY KEY, expires INTEGER, data TEXT)");
sessionDb.pragma("journal_mode=WAL");

class SQLiteSessionStore extends session.Store {
  get(sid, cb) { try { const row = sessionDb.prepare("SELECT data FROM sessions WHERE sid = ? AND expires > ?").get(sid, Date.now()); cb(null, row ? JSON.parse(row.data) : null); } catch (e) { cb(e); } }
  set(sid, sess, cb) { try { const maxAge = sess.cookie?.maxAge || 7*24*60*60*1000; sessionDb.prepare("INSERT OR REPLACE INTO sessions (sid, expires, data) VALUES (?, ?, ?)").run(sid, Date.now() + maxAge, JSON.stringify(sess)); cb(null); } catch (e) { cb(e); } }
  destroy(sid, cb) { try { sessionDb.prepare("DELETE FROM sessions WHERE sid = ?").run(sid); cb(null); } catch (e) { cb(e); } }
  touch(sid, sess, cb) { try { const maxAge = sess.cookie?.maxAge || 7*24*60*60*1000; sessionDb.prepare("UPDATE sessions SET expires = ? WHERE sid = ?").run(Date.now() + maxAge, sid); cb(null); } catch (e) { cb(e); } }
}

[DATA_DIR, CHATS_DIR, PROFILES_DIR].forEach(d => { if (!existsSync(d)) mkdirSync(d, { recursive: true }); });
