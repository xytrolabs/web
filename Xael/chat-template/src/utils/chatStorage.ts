// ─── Auth: check if user is logged in via XytroMailing session ───
export interface AuthStatus {
  authenticated: boolean;
  id?: string;
  username?: string;
  email?: string;
  tier?: string;
  loginUrl?: string;
  signupUrl?: string;
}

export async function checkAuth(): Promise<AuthStatus> {
  try {
    const res = await fetch('/api/auth/me', { credentials: 'include' });
    if (!res.ok) return { authenticated: false, loginUrl: 'https://mail.xytro.site/login' };
    return await res.json();
  } catch (e) {
    return { authenticated: false, loginUrl: 'https://mail.xytro.site/login' };
  }
}

// ─── Device ID (fallback for non-logged-in users) ───
export function getDeviceId(): string {
  if (typeof window === 'undefined') return '';
  let id = localStorage.getItem('xael_device_id');
  if (!id) {
    id = 'dev_' + Math.random().toString(36).slice(2) + Date.now().toString(36);
    localStorage.setItem('xael_device_id', id);
  }
  return id;
}

// ─── API base ───
export function getApiBase(): string {
  if (typeof window === 'undefined') return 'http://localhost:4005';
  // Always use the API server (ai.xytro.site) for data endpoints
  return 'https://ai.xytro.site';
}

// ─── Types ───
export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

interface ChatHistory {
  id: string;
  title: string;
  created: number;
  model: string;
  messages: ChatMessage[];
}

export interface ChatListItem {
  id: string;
  title: string;
  created: number;
  model: string;
  messageCount: number;
}

// ─── Chat CRUD ───
export async function saveChat(id: string, title: string, messages: ChatMessage[], model: string): Promise<boolean> {
  try {
    // Only save if user is logged in
    const auth = await checkAuth();
    if (!auth.authenticated) return false;
    const res = await fetch(`/api/chat-history`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ id, title, messages, model }),
    });
    return res.ok;
  } catch (e) {
    return false;
  }
}

export async function listChats(): Promise<ChatListItem[]> {
  try {
    const base = getApiBase();
    const res = await fetch(`/api/chat-history`, {
      headers: { 'x-device-id': getDeviceId() },
      credentials: 'include',
    });
    if (!res.ok) return [];
    const data = await res.json();
    return data.data || [];
  } catch (e) {
    console.warn('[ChatStorage] Failed to list chats:', e);
    return [];
  }
}

export async function loadChat(id: string): Promise<ChatHistory | null> {
  try {
    const base = getApiBase();
    const res = await fetch(`/api/chat-history/${id}`, {
      headers: { 'x-device-id': getDeviceId() },
      credentials: 'include',
    });
    if (!res.ok) return null;
    return await res.json();
  } catch (e) {
    console.warn('[ChatStorage] Failed to load chat:', e);
    return null;
  }
}

export async function deleteChat(id: string): Promise<boolean> {
  try {
    const base = getApiBase();
    const res = await fetch(`/api/chat-history/${id}`, {
      method: 'DELETE',
      headers: { 'x-device-id': getDeviceId() },
      credentials: 'include',
    });
    return res.ok;
  } catch (e) {
    console.warn('[ChatStorage] Failed to delete chat:', e);
    return false;
  }
}

export async function updateProfile(messages: ChatMessage[]): Promise<string | null> {
  try {
    const base = getApiBase();
    const res = await fetch(`${base}/v1/profile`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-device-id': getDeviceId() },
      credentials: 'include',
      body: JSON.stringify({ messages: messages.slice(-6) }),
    });
    if (!res.ok) return null;
    const data = await res.json();
    return data.systemPrompt || null;
  } catch (e) {
    console.warn('[Profile] Failed to update:', e);
    return null;
  }
}
