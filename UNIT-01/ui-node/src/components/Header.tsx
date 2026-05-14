import React from 'react';
import { Box, Text } from 'ink';
import { Mascot } from './Mascot';

interface HeaderProps {
    version: string;
    mode: string;
    ram: string;
    indexerStatus: 'online' | 'offline';
    sandboxStatus: 'online' | 'offline';
}

export const Header: React.FC<HeaderProps> = ({ 
    version, 
    mode, 
    ram, 
    indexerStatus, 
    sandboxStatus 
}) => {
    const amber = '#ffb000';
    const grey = '#767676';
    const dim = '#444444';

    return (
        <Box width="100%" marginBottom={1} borderStyle="round" borderColor={dim} paddingX={1} paddingY={1}>
            <Mascot />
            <Box flexDirection="column" flexGrow={1}>
                <Box justifyContent="space-between">
                    <Box>
                        <Text backgroundColor={amber} color="black" bold> UNIT-01 </Text>
                        <Text color={dim}> v{version} </Text>
                    </Box>
                    <Box>
                        <Text color={grey}>HOST: </Text>
                        <Text color={amber}>LOCAL-OS</Text>
                    </Box>
                </Box>
                
                <Box marginTop={1}>
                    <Box paddingX={1} backgroundColor={dim}>
                        <Text color={amber}>●</Text>
                        <Text color="white" bold> {mode} </Text>
                    </Box>
                    <Box marginLeft={2}>
                        <Text color={grey}>RAM: </Text>
                        <Text color="white">{ram}</Text>
                    </Box>
                    <Box marginLeft={2}>
                        <Text color={grey}>INDEX: </Text>
                        <Text color={indexerStatus === 'online' ? amber : 'red'}>●</Text>
                    </Box>
                    <Box marginLeft={2}>
                        <Text color={grey}>SANDBOX: </Text>
                        <Text color={sandboxStatus === 'online' ? amber : 'red'}>●</Text>
                    </Box>
                </Box>
            </Box>
        </Box>
    );
};

