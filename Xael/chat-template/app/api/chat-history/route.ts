import { NextRequest, NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';
const CLOUD_USERS = '/run/media/raf/C/xytrocloud/users';
const CHATS_SUBDIR = '.xael-chats';

async function getAuthUser(req: NextRequest): Promise<string | null> {
  // Check for session cookie
  const cookie = req.headers.get('cookie') || '';
  if (cookie) {
    try {
      // Verify by calling auth endpoint on same server
      const authRes = await fetch('http://localhost:4000/auth/me', {
        headers: { Cookie: cookie }
      });
      if (authRes.ok) {
        const user = await authRes.json();
        if (user.username) return user.username;
      }
    } catch {}
  }
  return null;
}

function getUserDir(username: string): string {
  const safeName = (username + '@xytro.site').replace('@', '_');
  return path.join(CLOUD_USERS, safeName, CHATS_SUBDIR);
}
function ensureDir(dir: string) { if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true }); }

export async function GET(req: NextRequest): Promise<Response> {
  const user = await getAuthUser(req);
  if (!user) return NextResponse.json({ data: [], authRequired: true });
  try {
    const dir = getUserDir(user);
    if (!fs.existsSync(dir)) return NextResponse.json({ data: [] });
    const files = fs.readdirSync(dir).filter(f => f.endsWith('.json'));
    const list = files.map(f => { try { const data = JSON.parse(fs.readFileSync(path.join(dir, f), 'utf-8')); return { id: data.id, title: data.title, created: data.created, model: data.model, messageCount: (data.messages || []).length }; } catch { return null; } }).filter(Boolean);
    return NextResponse.json({ data: list });
  } catch { return NextResponse.json({ data: [] }); }
}

export async function POST(req: NextRequest): Promise<Response> {
  const user = await getAuthUser(req);
  if (!user) return NextResponse.json({ error: 'Login required' }, { status: 401 });
  try {
    const body = await req.json(); const dir = getUserDir(user); ensureDir(dir);
    const file = path.join(dir, (body.id || 'chat') + '.json');
    fs.writeFileSync(file, JSON.stringify({ id: body.id, title: body.title, created: body.created || Date.now(), model: body.model || 'xael-nano', messages: body.messages || [] }));
    return NextResponse.json({ ok: true });
  } catch (e: any) { return NextResponse.json({ error: e.message }, { status: 500 }); }
}

export async function DELETE(req: NextRequest): Promise<Response> {
  const user = await getAuthUser(req);
  if (!user) return NextResponse.json({ error: 'Login required' }, { status: 401 });
  try {
    const url = new URL(req.url); const id = url.pathname.split('/').pop();
    if (!id) return NextResponse.json({ error: 'Missing id' }, { status: 400 });
    const dir = getUserDir(user); const file = path.join(dir, id + '.json');
    if (fs.existsSync(file)) fs.unlinkSync(file);
    return NextResponse.json({ ok: true });
  } catch (e: any) { return NextResponse.json({ error: e.message }, { status: 500 }); }
}
