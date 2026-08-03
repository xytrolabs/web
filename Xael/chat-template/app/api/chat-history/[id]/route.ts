import { NextRequest, NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

const CLOUD_USERS = '/run/media/raf/C/xytrocloud/users';
const CHATS_SUBDIR = '.xael-chats';

async function getAuthUser(req: NextRequest): Promise<string | null> {
  const cookie = req.headers.get('cookie') || '';
  if (cookie) {
    try {
      const authRes = await fetch('http://localhost:4000/auth/me', { headers: { Cookie: cookie } });
      if (authRes.ok) { const user = await authRes.json(); if (user.username) return user.username; }
    } catch {}
  }
  return null;
}

export async function GET(req: NextRequest, { params }: { params: Promise<{ id: string }> }): Promise<Response> {
  const user = await getAuthUser(req);
  if (!user) return NextResponse.json(null);
  const { id } = await params;
  try {
    const safeName = (user + '@xytro.site').replace('@', '_');
    const dir = path.join(CLOUD_USERS, safeName, CHATS_SUBDIR);
    const file = path.join(dir, id + '.json');
    if (!fs.existsSync(file)) return NextResponse.json(null);
    return NextResponse.json(JSON.parse(fs.readFileSync(file, 'utf-8')));
  } catch { return NextResponse.json(null); }
}

export async function DELETE(req: NextRequest, { params }: { params: Promise<{ id: string }> }): Promise<Response> {
  const user = await getAuthUser(req);
  if (!user) return NextResponse.json({ error: 'Login required' }, { status: 401 });
  const { id } = await params;
  try {
    const safeName = (user + '@xytro.site').replace('@', '_');
    const dir = path.join(CLOUD_USERS, safeName, CHATS_SUBDIR);
    const file = path.join(dir, id + '.json');
    if (fs.existsSync(file)) fs.unlinkSync(file);
    return NextResponse.json({ ok: true });
  } catch { return NextResponse.json({ ok: false }, { status: 500 }); }
}
