export type OpenAIModel = 'xael-nano' | 'xael-mini' | 'xael-think' | 'xael-vision';

export const MODEL_LABELS: Record<OpenAIModel, string> = {
  'xael-nano': 'Nano',
  'xael-mini': 'Mini',
  'xael-think': 'Think',
  'xael-vision': 'Vision',
};

export const MODEL_ICONS: Record<OpenAIModel, string> = {
  'xael-nano': 'MdFlashOn',
  'xael-mini': 'MdAutoAwesome',
  'xael-think': 'MdPsychology',
  'xael-vision': 'MdVisibility',
};

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface ChatBody {
  inputCode: string;
  model: OpenAIModel;
  apiKey?: string | undefined;
  messages?: ChatMessage[];
  web_search?: boolean;
}
