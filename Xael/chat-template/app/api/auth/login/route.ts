import { NextRequest, NextResponse } from 'next/server';

export const runtime = 'edge';

export async function POST(req: NextRequest): Promise<Response> {
  try {
    const body = await req.json();
    const { username, password } = body;
    if (!username || !password) {
      return NextResponse.json({ error: 'Username and password required' }, { status: 400 });
    }

    const res = await fetch('http://localhost:4000/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Host': 'mail.xytro.site',
        'X-Forwarded-For': req.headers.get('x-forwarded-for') || '',
        'User-Agent': req.headers.get('user-agent') || '',
      },
      body: JSON.stringify({ username, password }),
    });

    const data = await res.json();
    
    // Forward the session cookie from XytroMailing
    const setCookie = res.headers.get('set-cookie');
    const response = NextResponse.json(data, { status: res.status });
    if (setCookie) {
      response.headers.set('Set-Cookie', setCookie);
    }
    return response;
  } catch (e: any) {
    return NextResponse.json({ error: 'Login failed: ' + (e.message || 'Unknown error') }, { status: 500 });
  }
}
