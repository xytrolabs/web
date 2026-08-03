'use client';
import { useEffect, useState } from 'react';
import {
  Box,
  Button,
  Flex,
  Heading,
  Text,
  Input,
  Icon,
  useColorModeValue,
  useColorMode,
  Divider,
  Switch,
  FormLabel,
  FormControl,
  useToast,
} from '@chakra-ui/react';
import { MdArrowBack, MdPerson, MdVpnKey, MdLogout, MdDarkMode } from 'react-icons/md';
import Link from 'next/link';
import { checkAuth, AuthStatus } from '@/utils/chatStorage';

export default function SettingsPage() {
  const [auth, setAuth] = useState<AuthStatus>({ authenticated: false });
  const [apiKeyInput, setApiKeyInput] = useState('');
  const toast = useToast();
  const { colorMode, toggleColorMode } = useColorMode();

  const textColor = useColorModeValue('navy.700', 'white');
  const grayColor = useColorModeValue('gray.500', 'gray.400');
  const borderColor = useColorModeValue('gray.200', 'whiteAlpha.200');
  const cardBg = useColorModeValue('white', 'navy.800');
  const brandColor = useColorModeValue('brand.500', 'brand.400');
  const inputBg = useColorModeValue('gray.50', 'navy.900');

  useEffect(() => {
    checkAuth().then(setAuth);
    const saved = localStorage.getItem('apiKey');
    if (saved) setApiKeyInput(saved);
  }, []);

  const handleSaveApiKey = () => {
    if (apiKeyInput.trim().length >= 8) {
      localStorage.setItem('apiKey', apiKeyInput.trim());
      toast({ title: 'API key saved!', status: 'success', duration: 2000, isClosable: true });
    } else if (apiKeyInput.trim().length === 0) {
      localStorage.removeItem('apiKey');
      toast({ title: 'API key removed (free tier active)', status: 'info', duration: 2000, isClosable: true });
    } else {
      toast({ title: 'API key must be at least 8 characters', status: 'error', duration: 2000, isClosable: true });
    }
  };

  const handleLogout = async () => {
    await fetch('/api/auth/logout', { method: 'POST', credentials: 'include' });
    // Also try XytroMailing logout
    try { await fetch('https://mail.xytro.site/auth/logout', { method: 'POST', credentials: 'include' }); } catch {}
    setAuth({ authenticated: false });
    toast({ title: 'Logged out', status: 'info', duration: 2000, isClosable: true });
  };

  return (
    <Flex w="100%" minH="100vh" direction="column" px={{ base: '16px', md: '24px' }} py="20px" maxW="700px" mx="auto">
      {/* Header */}
      <Flex align="center" mb="30px" gap="12px">
        <Link href="/">
          <Button variant="ghost" size="sm" leftIcon={<Icon as={MdArrowBack} />} color={grayColor}>
            Back
          </Button>
        </Link>
        <Heading fontSize="24px" fontWeight="800" color={textColor}>Settings</Heading>
      </Flex>

      {/* Account Section */}
      <Box bg={cardBg} border="1px solid" borderColor={borderColor} borderRadius="16px" p="20px" mb="16px">
        <Heading fontSize="16px" fontWeight="700" color={textColor} mb="16px">
          <Icon as={MdPerson} mr="8px" /> Account
        </Heading>
        {auth.authenticated ? (
          <Flex direction="column" gap="10px">
            <Flex justify="space-between" align="center">
              <Text color={grayColor} fontSize="14px">Username</Text>
              <Text color={textColor} fontSize="14px" fontWeight="600">{auth.username}</Text>
            </Flex>
            <Flex justify="space-between" align="center">
              <Text color={grayColor} fontSize="14px">Email</Text>
              <Text color={textColor} fontSize="14px" fontWeight="600">{auth.email || '—'}</Text>
            </Flex>
            <Flex justify="space-between" align="center">
              <Text color={grayColor} fontSize="14px">Tier</Text>
              <Text color={brandColor} fontSize="14px" fontWeight="600">{auth.tier || 'free'}</Text>
            </Flex>
            <Divider borderColor={borderColor} my="8px" />
            <Button
              variant="outline"
              color="red.400"
              borderColor="red.400"
              _hover={{ bg: 'red.400', color: 'white' }}
              leftIcon={<Icon as={MdLogout} />}
              onClick={handleLogout}
              size="sm"
              borderRadius="10px"
              alignSelf="flex-start"
            >
              Logout
            </Button>
          </Flex>
        ) : (
          <Flex direction="column" gap="10px" align="center" py="10px">
            <Text color={grayColor} fontSize="14px">Not logged in</Text>
            <Link href="/">
              <Button variant="solid" bg={brandColor} color="white" size="sm" borderRadius="10px">
                Go to Chat to Login
              </Button>
            </Link>
          </Flex>
        )}
      </Box>

      {/* API Key Section */}
      <Box bg={cardBg} border="1px solid" borderColor={borderColor} borderRadius="16px" p="20px" mb="16px">
        <Heading fontSize="16px" fontWeight="700" color={textColor} mb="16px">
          <Icon as={MdVpnKey} mr="8px" /> API Key
        </Heading>
        <Text color={grayColor} fontSize="13px" mb="12px">
          Enter your Valis API key for higher rate limits and tier access.
          Leave empty to use the free tier.
        </Text>
        <Flex gap="10px">
          <Input
            placeholder="xa-xxxxxxxxxxxxxxxx"
            value={apiKeyInput}
            onChange={(e) => setApiKeyInput(e.target.value)}
            bg={inputBg}
            border="1px solid"
            borderColor={borderColor}
            color={textColor}
            borderRadius="12px"
            h="44px"
            fontSize="14px"
            _focus={{ borderColor: brandColor }}
            _placeholder={{ color: 'whiteAlpha.400' }}
            type="password"
          />
          <Button
            variant="solid"
            bg={brandColor}
            color="white"
            _hover={{ bg: 'brand.600' }}
            onClick={handleSaveApiKey}
            h="44px"
            borderRadius="12px"
            fontSize="14px"
            fontWeight="600"
            px="20px"
          >
            Save
          </Button>
        </Flex>
      </Box>

      {/* Appearance Section */}
      <Box bg={cardBg} border="1px solid" borderColor={borderColor} borderRadius="16px" p="20px" mb="16px">
        <Heading fontSize="16px" fontWeight="700" color={textColor} mb="16px">
          <Icon as={MdDarkMode} mr="8px" /> Appearance
        </Heading>
        <FormControl display="flex" alignItems="center" justifyContent="space-between">
          <FormLabel htmlFor="dark-mode" mb="0" color={textColor} fontSize="14px">
            Dark Mode
          </FormLabel>
          <Switch
            id="dark-mode"
            isChecked={colorMode === 'dark'}
            onChange={toggleColorMode}
            colorScheme="purple"
            size="md"
          />
        </FormControl>
      </Box>

      {/* Info */}
      <Text color={grayColor} fontSize="11px" textAlign="center" mt="auto" pt="20px">
        Valis AI · Xytro Labs · valischat.xytro.site
      </Text>
    </Flex>
  );
}
