import React from 'react';
import { Box, Text } from 'ink';

export const Mascot: React.FC = () => {
    const amber = '#ffb000';
    const grey = '#767676';

    return (
        <Box flexDirection="column" alignItems="center" marginRight={2}>
            <Text color={amber}>   [ UNIT-01 ]</Text>
            <Text color={grey}>      /───\</Text>
            <Box>
                <Text color={grey}>     │ </Text>
                <Text color={amber} bold>●</Text>
                <Text color={grey}> _ </Text>
                <Text color={amber} bold>●</Text>
                <Text color={grey}> │</Text>
            </Box>
            <Text color={grey}>      \───/</Text>
            <Text color={grey}>      /   \</Text>
        </Box>
    );
};
