'use client';
import { useState } from 'react';
import {
  Box,
  Button,
  Flex,
  Icon,
  Input,
  InputGroup,
  InputRightElement,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalHeader,
  ModalOverlay,
  Text,
  Tabs,
  TabList,
  TabPanels,
  Tab,
  TabPanel,
  useColorModeValue,
  useToast,
} from '@chakra-ui/react';
import { MdVisibility, MdVisibilityOff, MdPerson, MdEmail, MdLock } from 'react-icons/md';
import { checkAuth, AuthStatus } from '@/utils/chatStorage';

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onLogin: (auth: AuthStatus) => void;
}

export default function LoginModal({ isOpen, onClose, onLogin }: Props) {
  // Login fields
  const [loginUser, setLoginUser] = useState('');
  const [loginPass, setLoginPass] = useState('');
  const [showLoginPass, setShowLoginPass] = useState(false);
  // Signup fields
  const [signUser, setSignUser] = useState('');
  const [signPass, setSignPass] = useState('');
  const [signEmail, setSignEmail] = useState('');
  const [showSignPass, setShowSignPass] = useState(false);
  // State
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [tabIndex, setTabIndex] = useState(0);
  const toast = useToast();

  const textColor = useColorModeValue('navy.700', 'white');
  const grayColor = useColorModeValue('gray.500', 'gray.400');
  const inputBg = useColorModeValue('gray.50', 'navy.800');
  const inputBorder = useColorModeValue('gray.200', 'whiteAlpha.200');
  const brandColor = useColorModeValue('brand.500', 'brand.400');

  const handleLogin = async () => {
    setError('');
    if (!loginUser || !loginPass) { setError('Please fill in all fields'); return; }
    setLoading(true);
    try {
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ username: loginUser, password: loginPass }),
      });
      const data = await res.json();
      if (!res.ok) { setError(data.error || 'Login failed'); setLoading(false); return; }
      
      toast({ title: `Welcome, ${data.username || loginUser}!`, status: 'success', duration: 3000, isClosable: true });
      
      // Re-check auth to get user info
      const auth = await checkAuth();
      onLogin(auth);
      onClose();
      // Reset fields
      setLoginUser(''); setLoginPass('');
    } catch (e: any) {
      setError('Network error: ' + e.message);
    }
    setLoading(false);
  };

  const handleSignup = async () => {
    setError('');
    if (!signUser || !signPass) { setError('Please fill in all fields'); return; }
    if (signPass.length < 8) { setError('Password must be at least 8 characters'); return; }
    setLoading(true);
    try {
      const res = await fetch('/api/auth/signup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ username: signUser, password: signPass, email: signEmail }),
      });
      const data = await res.json();
      if (!res.ok) { setError(data.error || 'Signup failed'); setLoading(false); return; }
      
      toast({ title: `Account created! Welcome, ${signUser}!`, status: 'success', duration: 3000, isClosable: true });
      
      const auth = await checkAuth();
      onLogin(auth);
      onClose();
      setSignUser(''); setSignPass(''); setSignEmail('');
    } catch (e: any) {
      setError('Network error: ' + e.message);
    }
    setLoading(false);
  };

  return (
    <Modal blockScrollOnMount={false} isOpen={isOpen} onClose={onClose} size="md">
      <ModalOverlay backdropFilter="blur(8px)" />
      <ModalContent bg="navy.900" border="1px solid" borderColor="whiteAlpha.200" borderRadius="20px" boxShadow="2xl">
        <ModalHeader pt="30px" pb="0px" textAlign="center">
          <Text fontSize="24px" fontWeight="800" color="white">
            Welcome to <Text as="span" color={brandColor}>Valis AI</Text>
          </Text>
        </ModalHeader>
        <ModalCloseButton color="whiteAlpha.600" _focus={{ boxShadow: 'none' }} />
        <ModalBody px="30px" pb="30px">
          <Tabs isFitted variant="soft-rounded" colorScheme="purple" mb="6px" index={tabIndex} onChange={(i) => setTabIndex(i)}>
            <TabList mb="20px" bg="navy.800" borderRadius="12px" p="4px">
              <Tab _selected={{ bg: 'brand.500', color: 'white' }} color="whiteAlpha.600" fontSize="14px" fontWeight="600" borderRadius="10px">Login</Tab>
              <Tab _selected={{ bg: 'brand.500', color: 'white' }} color="whiteAlpha.600" fontSize="14px" fontWeight="600" borderRadius="10px">Sign Up</Tab>
            </TabList>

            <TabPanels>
              {/* ── Login Tab ── */}
              <TabPanel p="0">
                <Flex direction="column" gap="12px">
                  <Box>
                    <Text color={grayColor} fontSize="12px" mb="4px" fontWeight="500">Username</Text>
                    <InputGroup>
                      <Input
                        placeholder="your_username"
                        value={loginUser}
                        onChange={(e) => setLoginUser(e.target.value)}
                        bg={inputBg}
                        border="1px solid"
                        borderColor={inputBorder}
                        color="white"
                        borderRadius="12px"
                        h="48px"
                        fontSize="14px"
                        _focus={{ borderColor: brandColor }}
                        _placeholder={{ color: 'whiteAlpha.400' }}
                        onKeyDown={(e) => e.key === 'Enter' && handleLogin()}
                      />
                    </InputGroup>
                  </Box>
                  <Box>
                    <Text color={grayColor} fontSize="12px" mb="4px" fontWeight="500">Password</Text>
                    <InputGroup>
                      <Input
                        type={showLoginPass ? 'text' : 'password'}
                        placeholder="••••••••"
                        value={loginPass}
                        onChange={(e) => setLoginPass(e.target.value)}
                        bg={inputBg}
                        border="1px solid"
                        borderColor={inputBorder}
                        color="white"
                        borderRadius="12px"
                        h="48px"
                        fontSize="14px"
                        _focus={{ borderColor: brandColor }}
                        _placeholder={{ color: 'whiteAlpha.400' }}
                        onKeyDown={(e) => e.key === 'Enter' && handleLogin()}
                      />
                      <InputRightElement h="48px">
                        <Icon
                          as={showLoginPass ? MdVisibilityOff : MdVisibility}
                          color="whiteAlpha.500"
                          cursor="pointer"
                          onClick={() => setShowLoginPass(!showLoginPass)}
                        />
                      </InputRightElement>
                    </InputGroup>
                  </Box>
                </Flex>
              </TabPanel>

              {/* ── Signup Tab ── */}
              <TabPanel p="0">
                <Flex direction="column" gap="12px">
                  <Box>
                    <Text color={grayColor} fontSize="12px" mb="4px" fontWeight="500">Username</Text>
                    <Input
                      placeholder="new_user"
                      value={signUser}
                      onChange={(e) => setSignUser(e.target.value)}
                      bg={inputBg}
                      border="1px solid"
                      borderColor={inputBorder}
                      color="white"
                      borderRadius="12px"
                      h="48px"
                      fontSize="14px"
                      _focus={{ borderColor: brandColor }}
                      _placeholder={{ color: 'whiteAlpha.400' }}
                    />
                  </Box>
                  <Box>
                    <Text color={grayColor} fontSize="12px" mb="4px" fontWeight="500">Email (optional)</Text>
                    <Input
                      type="email"
                      placeholder="you@example.com"
                      value={signEmail}
                      onChange={(e) => setSignEmail(e.target.value)}
                      bg={inputBg}
                      border="1px solid"
                      borderColor={inputBorder}
                      color="white"
                      borderRadius="12px"
                      h="48px"
                      fontSize="14px"
                      _focus={{ borderColor: brandColor }}
                      _placeholder={{ color: 'whiteAlpha.400' }}
                    />
                  </Box>
                  <Box>
                    <Text color={grayColor} fontSize="12px" mb="4px" fontWeight="500">Password (min 8 chars)</Text>
                    <InputGroup>
                      <Input
                        type={showSignPass ? 'text' : 'password'}
                        placeholder="••••••••"
                        value={signPass}
                        onChange={(e) => setSignPass(e.target.value)}
                        bg={inputBg}
                        border="1px solid"
                        borderColor={inputBorder}
                        color="white"
                        borderRadius="12px"
                        h="48px"
                        fontSize="14px"
                        _focus={{ borderColor: brandColor }}
                        _placeholder={{ color: 'whiteAlpha.400' }}
                        onKeyDown={(e) => e.key === 'Enter' && handleSignup()}
                      />
                      <InputRightElement h="48px">
                        <Icon
                          as={showSignPass ? MdVisibilityOff : MdVisibility}
                          color="whiteAlpha.500"
                          cursor="pointer"
                          onClick={() => setShowSignPass(!showSignPass)}
                        />
                      </InputRightElement>
                    </InputGroup>
                  </Box>
                </Flex>
              </TabPanel>
            </TabPanels>
          </Tabs>

          {error && (
            <Text color="red.400" fontSize="13px" textAlign="center" mb="10px" fontWeight="500">
              {error}
            </Text>
          )}

          {/* Action button changes based on active tab */}
          <Flex gap="10px" mt="4px">
            <Button
              variant="solid"
              bg={brandColor}
              color="white"
              _hover={{ bg: 'brand.600' }}
              h="48px"
              borderRadius="12px"
              fontSize="14px"
              fontWeight="600"
              flex="1"
              isLoading={loading}
              loadingText="Please wait..."
              onClick={tabIndex === 0 ? handleLogin : handleSignup}
            >
              {tabIndex === 0 ? 'Login' : 'Create Account'}
            </Button>
            <Button
              variant="outline"
              borderColor="whiteAlpha.300"
              color="whiteAlpha.700"
              _hover={{ bg: 'whiteAlpha.100' }}
              h="48px"
              borderRadius="12px"
              fontSize="14px"
              fontWeight="600"
              onClick={onClose}
            >
              Cancel
            </Button>
          </Flex>
          <Text color="whiteAlpha.500" fontSize="11px" textAlign="center" mt="14px">
            Your account works across all Xytro sites — mail, cloud, AI, and more.
          </Text>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
}
