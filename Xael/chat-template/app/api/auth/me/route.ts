import { NextRequest, NextResponse } from 'next/server';

export const runtime = 'edge';

export async function GET(req: NextRequest): Promise<Response> {
  try {
    // Forward cookies to Xael API for session validation
    const cookieHeader = req.headers.get('cookie') || '';
    const res = await fetch('http://localhost:4005/v1/auth/me', {
      headers: { Cookie: cookieHeader, Host: 'ai.xytro.site' },
    });
    const data = await res.json();
    return NextResponse.json(data);
  } catch (e) {
    return NextResponse.json({ authenticated: false, error: 'Failed to check auth' });
  }
}
