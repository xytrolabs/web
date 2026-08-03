'use client';
import { useEffect, useState } from 'react';
import {
  Box,
  Flex,
  Text,
  Button,
  Icon,
  useColorModeValue,
  Spinner,
} from '@chakra-ui/react';
import { MdDelete, MdChatBubble, MdAdd } from 'react-icons/md';
import { listChats, deleteChat, ChatListItem, getDeviceId } from '@/utils/chatStorage';

interface Props {
  onLoadChat: (id: string) => void;
  onNewChat: () => void;
  activeChatId: string | null;
  refreshTrigger: number;
}

export default function ChatHistoryPanel({ onLoadChat, onNewChat, activeChatId, refreshTrigger }: Props) {
  const [chats, setChats] = useState<ChatListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [deviceId, setDeviceId] = useState('');

  const textColor = useColorModeValue('navy.700', 'white');
  const grayColor = useColorModeValue('gray.500', 'gray.500');
  const hoverBg = useColorModeValue('gray.100', 'whiteAlpha.100');
  const borderColor = useColorModeValue('gray.200', 'whiteAlpha.200');
  const brandColor = useColorModeValue('brand.500', 'brand.400');

  useEffect(() => {
    setDeviceId(getDeviceId());
  }, []);

  useEffect(() => {
    loadChatList();
  }, [refreshTrigger]);

  const loadChatList = async () => {
    setLoading(true);
    const list = await listChats();
    setChats(list);
    setLoading(false);
  };

  const handleDelete = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    await deleteChat(id);
    setChats(chats.filter(c => c.id !== id));
    if (activeChatId === id) {
      onNewChat();
    }
  };

  const formatDate = (ts: number) => {
    const d = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    if (diff < 86400000) return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (diff < 604800000) return d.toLocaleDateString([], { weekday: 'short' });
    return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
  };

  return (
    <Box w="100%">
      <Flex justify="space-between" align="center" mb="12px" px="4px">
        <Text color={textColor} fontWeight="700" fontSize="14px">
          Chat History
        </Text>
        <Button
          size="xs"
          variant="ghost"
          leftIcon={<Icon as={MdAdd} />}
          onClick={onNewChat}
          color={brandColor}
          fontSize="11px"
          fontWeight="600"
        >
          New
        </Button>
      </Flex>

      {loading ? (
        <Flex justify="center" py="20px">
          <Spinner size="sm" color={brandColor} />
        </Flex>
      ) : chats.length === 0 ? (
        <Text color={grayColor} fontSize="12px" textAlign="center" py="10px">
          No saved chats yet
        </Text>
      ) : (
        <Flex direction="column" gap="2px" maxH="250px" overflowY="auto">
          {chats.map((chat) => (
            <Flex
              key={chat.id}
              p="8px 10px"
              borderRadius="8px"
              cursor="pointer"
              align="center"
              justify="space-between"
              bg={activeChatId === chat.id ? hoverBg : 'transparent'}
              _hover={{ bg: hoverBg }}
              onClick={() => onLoadChat(chat.id)}
              transition="0.2s"
              border="1px solid"
              borderColor={activeChatId === chat.id ? brandColor : 'transparent'}
            >
              <Flex align="center" gap="8px" overflow="hidden">
                <Icon as={MdChatBubble} color={grayColor} width="14px" height="14px" flexShrink={0} />
                <Box overflow="hidden">
                  <Text
                    color={textColor}
                    fontSize="12px"
                    fontWeight="600"
                    whiteSpace="nowrap"
                    overflow="hidden"
                    textOverflow="ellipsis"
                    maxW="150px"
                  >
                    {chat.title || 'Untitled'}
                  </Text>
                  <Text color={grayColor} fontSize="10px">
                    {formatDate(chat.created)} · {chat.messageCount} msgs
                  </Text>
                </Box>
              </Flex>
              <Button
                size="xs"
                variant="ghost"
                p="2px"
                minW="20px"
                h="20px"
                onClick={(e) => handleDelete(e, chat.id)}
                _hover={{ color: 'red.400' }}
              >
                <Icon as={MdDelete} width="12px" height="12px" color={grayColor} />
              </Button>
            </Flex>
          ))}
        </Flex>
      )}

      <Text color={grayColor} fontSize="9px" textAlign="center" mt="8px" fontFamily="mono">
        Device: {deviceId.slice(0, 16)}...
      </Text>
    </Box>
  );
}
