import { NextRequest, NextResponse } from 'next/server';

export const runtime = 'edge';

export async function POST(req: NextRequest): Promise<Response> {
  try {
    const cookieHeader = req.headers.get('cookie') || '';
    const res = await fetch('http://localhost:4000/auth/logout', {
      method: 'POST',
      headers: { Cookie: cookieHeader, Host: 'mail.xytro.site' },
    });
    const setCookie = res.headers.get('set-cookie');
    const response = NextResponse.json({ ok: true });
    // Clear the session cookie
    response.headers.set('Set-Cookie', setCookie || 'connect.sid=; Path=/; Max-Age=0; Domain=.xytro.site');
    return response;
  } catch (e: any) {
    return NextResponse.json({ ok: true }); // Always succeed - clear cookie locally
  }
}
