import { NextRequest, NextResponse } from 'next/server';

export const runtime = 'edge';

export async function POST(req: NextRequest): Promise<Response> {
  try {
    const body = await req.json();
    const { username, password, email } = body;
    if (!username || !password) {
      return NextResponse.json({ error: 'Username and password required' }, { status: 400 });
    }
    if (username.length < 3 || username.length > 32) {
      return NextResponse.json({ error: 'Username must be 3-32 characters' }, { status: 400 });
    }
    if (password.length < 8) {
      return NextResponse.json({ error: 'Password must be at least 8 characters' }, { status: 400 });
    }

    const res = await fetch('http://localhost:4000/auth/signup', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Host': 'mail.xytro.site',
        'X-Forwarded-For': req.headers.get('x-forwarded-for') || '',
        'User-Agent': req.headers.get('user-agent') || '',
      },
      body: JSON.stringify({ username, password, email: email || '' }),
    });

    const data = await res.json();
    
    const setCookie = res.headers.get('set-cookie');
    const response = NextResponse.json(data, { status: res.status });
    if (setCookie) {
      response.headers.set('Set-Cookie', setCookie);
    }
    return response;
  } catch (e: any) {
    return NextResponse.json({ error: 'Signup failed: ' + (e.message || 'Unknown error') }, { status: 500 });
  }
}
