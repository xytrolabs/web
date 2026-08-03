import {
  createParser,
  ParsedEvent,
  ReconnectInterval,
} from 'eventsource-parser';
import type { ChatMessage } from '@/types/types';

// Xael API base URL — auto-detect from current origin
const getApiBase = () => {
  if (typeof window !== 'undefined') {
    const host = window.location.hostname;
    if (host.includes('xytro.site') || host.includes('valis')) {
      return window.location.origin;
    }
  }
  return 'http://localhost:4005';
};

export const OpenAIStream = async (
  inputCode: string,
  model: string,
  key: string | undefined,
  messages?: ChatMessage[],
  webSearch?: boolean,
) => {
  const apiBase = typeof window !== 'undefined'
    ? (window as any).__XAEL_API_BASE__ || getApiBase()
    : (process.env.NEXT_PUBLIC_XAEL_API_BASE || 'http://localhost:4005');

  // Build messages array: use provided history or just the single message
  const apiMessages: ChatMessage[] = messages && messages.length > 0
    ? messages
    : [{ role: 'user', content: inputCode }];

  const res = await fetch(`${apiBase}/v1/chat/completions`, {
    headers: {
      'Content-Type': 'application/json',
      ...(key ? { Authorization: `Bearer ${key}` } : {}),
    },
    method: 'POST',
    body: JSON.stringify({
      model,
      messages: apiMessages,
      temperature: 0.7,
      stream: true,
      ...(webSearch ? { web_search: true } : {}),
    }),
  });

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  if (res.status !== 200) {
    let errorBody = '';
    try {
      const result = await res.body?.getReader().read();
      errorBody = decoder.decode(result?.value) || res.statusText;
    } catch (e) {
      errorBody = res.statusText;
    }
    throw new Error(
      `Xael API returned an error (${res.status}): ${errorBody}`,
    );
  }

  const stream = new ReadableStream({
    async start(controller) {
      const onParse = (event: ParsedEvent | ReconnectInterval) => {
        if (event.type === 'event') {
          const data = event.data;

          if (data === '[DONE]') {
            controller.close();
            return;
          }

          try {
            const json = JSON.parse(data);
            const text = json.choices?.[0]?.delta?.content || '';
            if (text) {
              controller.enqueue(encoder.encode(text));
            }
          } catch (e) {
            controller.error(e);
          }
        }
      };

      const parser = createParser(onParse);

      const reader = res.body?.getReader();
      if (!reader) {
        controller.close();
        return;
      }

      let done = false;
      while (!done) {
        const { value, done: doneReading } = await reader.read();
        done = doneReading;
        if (value) {
          parser.feed(decoder.decode(value));
        }
      }
      controller.close();
    },
  });

  return stream;
};
