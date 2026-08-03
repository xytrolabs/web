import { ChatBody } from '@/types/types';
import { OpenAIStream } from '@/utils/chatStream';

export async function GET(req: Request): Promise<Response> {
  try {
    const { inputCode, model, apiKey, messages, web_search } = (await req.json()) as ChatBody;
    const apiKeyFinal = apiKey || process.env.NEXT_PUBLIC_XAEL_API_KEY || undefined;
    const stream = await OpenAIStream(inputCode, model, apiKeyFinal, messages, web_search);
    return new Response(stream);
  } catch (error) {
    console.error(error);
    return new Response('Error', { status: 500 });
  }
}

export async function POST(req: Request): Promise<Response> {
  try {
    const { inputCode, model, apiKey, messages, web_search } = (await req.json()) as ChatBody;
    const apiKeyFinal = apiKey || process.env.NEXT_PUBLIC_XAEL_API_KEY || undefined;
    const stream = await OpenAIStream(inputCode, model, apiKeyFinal, messages, web_search);
    return new Response(stream);
  } catch (error) {
    console.error(error);
    return new Response('Error', { status: 500 });
  }
}
