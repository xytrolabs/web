'use client';
/*eslint-disable*/

import MessageBoxChat from '@/components/MessageBox';
import ChatHistoryPanel from '@/components/ChatHistoryPanel';
import LoginModal from '@/components/LoginModal';
import { ChatBody, OpenAIModel, MODEL_LABELS } from '@/types/types';
import { saveChat, loadChat as loadChatFromCloud, updateProfile, getDeviceId, checkAuth, AuthStatus } from '@/utils/chatStorage';
import {
  Accordion,
  AccordionButton,
  AccordionIcon,
  AccordionItem,
  AccordionPanel,
  Box,
  Button,
  Flex,
  Icon,
  Img,
  Input,
  Text,
  useColorModeValue,
  useColorMode,
  IconButton,
} from '@chakra-ui/react';
import { useEffect, useRef, useState, useCallback } from 'react';
import Link from 'next/link';
import { MdAutoAwesome, MdBolt, MdEdit, MdPerson, MdFlashOn, MdPsychology, MdVisibility, MdHistory, MdDarkMode, MdLightMode, MdSettings } from 'react-icons/md';
import APIModal from '@/components/apiModal';
import Bg from '../public/img/chat/bg-image.png';

const ICON_MAP: Record<string, any> = {
  'MdFlashOn': MdFlashOn,
  'MdAutoAwesome': MdAutoAwesome,
  'MdPsychology': MdPsychology,
  'MdVisibility': MdVisibility,
};

const MODELS: { id: OpenAIModel; icon: string; color: string }[] = [
  { id: 'xael-nano', icon: 'MdFlashOn', color: 'green.400' },
  { id: 'xael-mini', icon: 'MdAutoAwesome', color: 'brand.500' },
  { id: 'xael-think', icon: 'MdPsychology', color: 'purple.400' },
  { id: 'xael-vision', icon: 'MdVisibility', color: 'cyan.400' },
];

interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export default function Chat() {
  // Chat state
  const [inputCode, setInputCode] = useState<string>('');
  const [outputCode, setOutputCode] = useState<string>('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [model, setModel] = useState<OpenAIModel>('xael-mini');
  const [loading, setLoading] = useState<boolean>(false);
  const [webSearch, setWebSearch] = useState<boolean>(true); // Always search by default
  const [attachedImages, setAttachedImages] = useState<string[]>([]); // Multiple base64 images for vision
  const [chatId, setChatId] = useState<string>('');
  const [showHistory, setShowHistory] = useState<boolean>(false);
  const [historyRefresh, setHistoryRefresh] = useState<number>(0);
  const [profilePrompt, setProfilePrompt] = useState<string>('');
  const [auth, setAuth] = useState<AuthStatus>({ authenticated: false });
  const [showLoginModal, setShowLoginModal] = useState(false);
  const [apiKey, setApiKey] = useState('');
  const { colorMode, toggleColorMode } = useColorMode();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Check auth on mount
  useEffect(() => {
    checkAuth().then(setAuth);
  }, []);

  // UI colors
  const borderColor = useColorModeValue('gray.200', 'whiteAlpha.200');
  const inputColor = useColorModeValue('navy.700', 'white');
  const iconColor = useColorModeValue('brand.500', 'white');
  const bgIcon = useColorModeValue(
    'linear-gradient(180deg, #FBFBFF 0%, #CACAFF 100%)',
    'whiteAlpha.200',
  );
  const brandColor = useColorModeValue('brand.500', 'white');
  const buttonBg = useColorModeValue('white', 'whiteAlpha.100');
  const gray = useColorModeValue('gray.500', 'white');
  const buttonShadow = useColorModeValue(
    '14px 27px 45px rgba(112, 144, 176, 0.2)',
    'none',
  );
  const textColor = useColorModeValue('navy.700', 'white');

  // Initialize new chat ID
  useEffect(() => {
    if (!chatId) {
      setChatId('chat_' + Date.now());
    }
  }, []);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [outputCode, messages]);

  // Generate chat title from first user message
  const getChatTitle = useCallback(() => {
    const firstUserMsg = messages.find(m => m.role === 'user');
    if (firstUserMsg) {
      return firstUserMsg.content.slice(0, 80).replace(/\n/g, ' ');
    }
    return 'New Chat';
  }, [messages]);

  // Save current chat to cloud
  const saveCurrentChat = useCallback(async () => {
    if (messages.length === 0 || !chatId) return;
    await saveChat(chatId, getChatTitle(), messages, model);
  }, [messages, chatId, model, getChatTitle]);

  // Auto-save after each exchange
  useEffect(() => {
    if (messages.length >= 2 && !loading && chatId) {
      saveCurrentChat();
      // Update profile
      updateProfile(messages).then(prompt => {
        if (prompt) setProfilePrompt(prompt);
      });
      // Refresh history list
      setHistoryRefresh(Date.now());
    }
  }, [loading, messages.length]);

  // Load a previous chat
  const handleLoadChat = async (id: string) => {
    const chat = await loadChatFromCloud(id);
    if (chat) {
      setChatId(chat.id);
      setMessages(chat.messages || []);
      setModel((chat.model as OpenAIModel) || 'xael-mini');
      setOutputCode('');
      // Reconstruct output from last assistant message
      const lastAssistant = [...(chat.messages || [])].reverse().find(m => m.role === 'assistant');
      if (lastAssistant) {
        setOutputCode(lastAssistant.content);
      }
      setShowHistory(false);
    }
  };

  // New chat
  const handleNewChat = () => {
    setChatId('chat_' + Date.now());
    setMessages([]);
    setOutputCode('');
    setInputCode('');
    setAttachedImages([]); // Clear attached image after send
    setShowHistory(false);
  };

  const handleTranslate = async () => {
    const apiKey = localStorage.getItem('apiKey');

    if (!inputCode) {
      alert('Please enter your message.');
      return;
    }

    const userMsg: ChatMessage = { role: 'user', content: inputCode };
    const newMessages = [...messages, userMsg];
    setMessages(newMessages);
    setInputCode('');
    setAttachedImages([]); // Clear attached image after send
    setOutputCode(' ');
    setLoading(true);

    const controller = new AbortController();

    // Build the API messages array
    const apiMessages: ChatMessage[] = [];
    
    // Add profile context if available
    if (profilePrompt) {
      apiMessages.push({ role: 'system', content: 'You are a helpful AI assistant. ' + profilePrompt });
    }

    // Add conversation history (last 10 exchanges to save context)
    apiMessages.push(...newMessages.slice(-20));

    // If image attached, route through vision model
    const effectiveModel = attachedImages.length > 0 ? 'xael-vision' : model;
    const body: ChatBody = {
      inputCode: webSearch ? `[web search enabled] ${inputCode}` : inputCode,
      model: effectiveModel,
      apiKey: apiKey || undefined,
      messages: apiMessages,
      web_search: webSearch,
    };

    // Build multimodal content if image attached
    let requestBody: any = {
      model: effectiveModel,
      messages: apiMessages,
      temperature: 0.7,
      stream: true,
      ...(webSearch ? { web_search: true } : {}),
    };

    if (attachedImages.length > 0) {
      // Replace last user message with multimodal version
      const textContent = apiMessages[apiMessages.length - 1].content;
      requestBody.messages = [
        ...apiMessages.slice(0, -1),
        {
          role: 'user',
          content: [
            { type: 'text', text: textContent },
            ...attachedImages.map(img => ({ type: 'image_url' as const, image_url: { url: img } })),
          ],
        },
      ];
    }

    requestBody.stream = false;

    const response = await fetch('/v1/chat/completions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      signal: controller.signal,
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      setLoading(false);
      alert('Server error. Please try again.');
      return;
    }

    const data = await response.json();
    const reply = data.choices?.[0]?.message?.content || '(no response)';
    setMessages([...newMessages, { role: 'assistant', content: reply }]);
    setOutputCode(reply);

    setLoading(false);
  };

  const handleChange = (Event: any) => {
    setInputCode(Event.target.value);
  };

  // Handle image file selection
  const handleImageAttach = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    const maxImages = 5;
    let loaded = 0;
    const results: string[] = [];
    const count = Math.min(files.length, maxImages);
    for (let i = 0; i < count; i++) {
      const file = files[i];
      if (!file.type.startsWith('image/')) continue;
      const reader = new FileReader();
      reader.onload = () => {
        results.push(reader.result as string);
        loaded++;
        if (loaded === count || loaded === results.length) {
          setAttachedImages(prev => [...prev, ...results].slice(0, maxImages));
        }
      };
      reader.readAsDataURL(file);
    }
    if (files.length > maxImages) alert('Max ' + maxImages + ' images per message.');
  };

  return (
    <Flex
      w="100%"
      minH="100vh"
      direction="column"
      position="relative"
      px={{ base: '16px', md: '24px' }}
      py="20px"
    >
      <Img
        src={Bg.src}
        position={'absolute'}
        w="350px"
        left="50%"
        top="50%"
        transform={'translate(-50%, -50%)'}
        opacity={outputCode ? 0 : 0.3}
        pointerEvents="none"
      />
      <Flex
        direction="column"
        mx="auto"
        w={{ base: '100%', md: '100%', xl: '100%' }}
        minH={{ base: '75vh', '2xl': '85vh' }}
        maxW="1000px"
      >
        {/* Top bar: Model Selector + History Toggle */}
        <Flex direction="column" w="100%" mb={outputCode ? '20px' : 'auto'}>
          <Flex justify="space-between" align="center" mb="10px" px="4px">
            {/* Compact model selector */}
            <Flex gap="6px" flexWrap="wrap" justify="center" flex="1">
              {MODELS.map((m) => {
                const IconComp = ICON_MAP[m.icon] || MdAutoAwesome;
                return (
                  <Button
                    key={m.id}
                    variant={model === m.id ? 'chakraLinear' : 'outline'}
                    size="xs"
                    borderRadius="45px"
                    fontSize="10px"
                    fontWeight="600"
                    px="10px"
                    h="28px"
                    onClick={() => setModel(m.id)}
                    leftIcon={<Icon as={IconComp} width="12px" height="12px" />}
                  >
                    {MODEL_LABELS[m.id]}
                  </Button>
                );
              })}
            </Flex>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowHistory(!showHistory)}
              leftIcon={<Icon as={MdHistory} />}
              color={gray}
              fontSize="12px"
              ml="8px"
            >
              {showHistory ? 'Hide' : 'History'}
            </Button>
            {/* Theme toggle */}
            <IconButton
              aria-label="Toggle theme"
              icon={colorMode === 'dark' ? <Icon as={MdLightMode} /> : <Icon as={MdDarkMode} />}
              variant="ghost"
              size="sm"
              onClick={toggleColorMode}
              color={gray}
              fontSize="16px"
              ml="2px"
            />
            {/* Settings */}
            <Link href="/settings" passHref>
              <IconButton
                aria-label="Settings"
                icon={<Icon as={MdSettings} />}
                variant="ghost"
                size="sm"
                color={gray}
                fontSize="16px"
                ml="2px"
              />
            </Link>
            {/* API Key button (rendered by APIModal) */}
            <APIModal setApiKey={setApiKey} sidebar={false} />
            {/* Login / User indicator */}
            {auth.authenticated ? (
              <Button
                variant="ghost"
                size="sm"
                leftIcon={<Icon as={MdPerson} color="green.400" />}
                color="green.400"
                fontSize="12px"
                ml="4px"
                title={`Logged in as ${auth.username || auth.email}`}
              >
                {auth.username || auth.email?.split('@')[0] || 'User'}
              </Button>
            ) : (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowLoginModal(true)}
                leftIcon={<Icon as={MdPerson} />}
                color={brandColor}
                borderColor={brandColor}
                fontSize="11px"
                ml="4px"
                borderRadius="45px"
              >
                Login
              </Button>
            )}
          </Flex>

          {/* Web Search Toggle */}
          <Flex justify="center" mb="10px" gap="8px">
            <Button
              variant={webSearch ? 'chakraLinear' : 'outline'}
              size="sm"
              borderRadius="45px"
              fontSize="11px"
              fontWeight="600"
              onClick={() => setWebSearch(!webSearch)}
              leftIcon={<Icon as={MdBolt} />}
            >
              {webSearch ? 'Web: ON' : 'Web: OFF'}
            </Button>
            {profilePrompt && (
              <Text color="green.400" fontSize="10px" alignSelf="center">
                Profile active
              </Text>
            )}
          </Flex>

          {/* Chat History Panel */}
          {showHistory && (
            <Box
              mb="16px"
              p="14px"
              border="1px solid"
              borderColor={borderColor}
              borderRadius="14px"
              bg={useColorModeValue('white', 'navy.800')}
            >
              <ChatHistoryPanel
                onLoadChat={handleLoadChat}
                onNewChat={handleNewChat}
                activeChatId={chatId}
                refreshTrigger={historyRefresh}
              />
            </Box>
          )}
        </Flex>

        {/* Chat Messages Display */}
        {messages.length > 0 && (
          <Flex direction="column" w="100%" mx="auto" mb="auto" gap="12px">
            {messages.map((msg, i) => (
              <Flex key={i} w="100%" align="flex-start" gap="12px">
                <Flex
                  borderRadius="full"
                  justify="center"
                  align="center"
                  bg={msg.role === 'user' ? 'transparent' : 'linear-gradient(15.46deg, #4A25E1 26.3%, #7B5AFF 86.4%)'}
                  border={msg.role === 'user' ? '1px solid' : 'none'}
                  borderColor={borderColor}
                  h="36px"
                  minH="36px"
                  minW="36px"
                  flexShrink={0}
                >
                  <Icon
                    as={msg.role === 'user' ? MdPerson : MdAutoAwesome}
                    width="16px"
                    height="16px"
                    color={msg.role === 'user' ? brandColor : 'white'}
                  />
                </Flex>
                <Flex
                  p="16px"
                  border="1px solid"
                  borderColor={borderColor}
                  borderRadius="14px"
                  w="fit-content"
                  bg={msg.role === 'user' ? 'transparent' : undefined}
                >
                  {msg.role === 'assistant' ? (
                    <MessageBoxChat output={msg.content} />
                  ) : (
                    <Text
                      color={textColor}
                      fontWeight="500"
                      fontSize={{ base: 'sm', md: 'md' }}
                      lineHeight="1.6"
                      whiteSpace="pre-wrap"
                    >
                      {msg.content}
                    </Text>
                  )}
                </Flex>
              </Flex>
            ))}
            <div ref={messagesEndRef} />
          </Flex>
        )}

        {/* Input Area */}
        <Flex ms={{ base: '0px', xl: '0px' }} mt="20px" justify="center" w="100%">
          <Flex align="center" justify="center" w="100%" maxW="920px">
                        {/* Hidden file input for image upload */}
            <input
              type="file"
              accept="image/*" multiple
              onChange={handleImageAttach}
              style={{ display: 'none' }}
              id="image-upload-input"
            />
            <Box
              as="label"
              htmlFor="image-upload-input"
              cursor="pointer"
              px="10px"
              py="8px"
              borderRadius="8px"
              _hover={{ bg: 'whiteAlpha.100' }}
              color={attachedImages.length > 0 ? 'green.400' : iconColor}
              fontSize="18px"
              title={attachedImages.length > 0 ? 'Image attached!' : 'Attach an image'}
            >
              {attachedImages.length > 0 ? '🖼' : '🖼'}
            </Box>
<Input
              minH="54px"
              h="100%"
              border="1px solid"
              borderColor={borderColor}
              borderRadius="45px"
              p="15px 20px"
              me="10px"
              fontSize="sm"
              fontWeight="500"
              _focus={{ borderColor: brandColor }}
              color={inputColor}
              placeholder="Type your message here..."
              onChange={handleChange}
              value={inputCode}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  handleTranslate();
                }
              }}
            />
            <Button
              variant="chakraLinear"
              py="20px"
              px="16px"
              fontSize="sm"
              borderRadius="45px"
              ms="auto"
              w={{ base: '160px', md: '210px' }}
              h="54px"
              onClick={handleTranslate}
              isLoading={loading}
              loadingText="Thinking..."
            >
              Send Message
            </Button>
          </Flex>
        </Flex>
      </Flex>
      {/* Login Modal */}
      <LoginModal
        isOpen={showLoginModal}
        onClose={() => setShowLoginModal(false)}
        onLogin={(newAuth) => setAuth(newAuth)}
      />
    </Flex>
  );
}
