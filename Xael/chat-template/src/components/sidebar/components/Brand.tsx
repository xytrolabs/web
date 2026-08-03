'use client';
// Chakra imports
import { Flex, Text, useColorModeValue } from '@chakra-ui/react';

import { HSeparator } from '@/components/separator/Separator';

export function SidebarBrand() {
  let logoColor = useColorModeValue('navy.700', 'white');
  let purpleColor = useColorModeValue('brand.500', 'brand.400');

  return (
    <Flex alignItems="center" flexDirection="column">
      <Text
        fontSize="28px"
        fontWeight="800"
        letterSpacing="-0.5px"
        my="24px"
        color={logoColor}
      >
        Xael{' '}
        <Text as="span" color={purpleColor}>
          AI
        </Text>
      </Text>
      <HSeparator mb="20px" w="284px" />
    </Flex>
  );
}

export default SidebarBrand;
