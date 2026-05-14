import React, { useState, useEffect } from 'react';
import { render, Box, Text, useInput, useApp } from 'ink';
import TextInput from 'ink-text-input';
import Spinner from 'ink-spinner';
import { Header } from './components/Header';

const App = () => {
    const { exit } = useApp();
    const [query, setQuery] = useState('');
    const [messages, setMessages] = useState<{ role: string; content: string }[]>([]);
    const [isThinking, setIsThinking] = useState(false);
    const [statusMessage, setStatusMessage] = useState('STANDBY');

    useEffect(() => {
        setMessages([
            { role: 'system', content: 'UNIT-01 Online. Ready for commands.' }
        ]);
    }, []);

    useInput((input, key) => {
        if (key.escape || (key.ctrl && input === 'c')) {
            exit();
        }
    });

    const handleSubmit = async (value: string) => {
        if (!value.trim()) return;
        
        const userMsg = { role: 'user', content: value };
        setMessages(prev => [...prev, userMsg]);
        setQuery('');
        setIsThinking(true);
        setStatusMessage('WORKING');

        // Simulate AI reasoning
        setTimeout(() => {
            const aiMsg = { role: 'assistant', content: `Processed "${value}". Everything looks good.` };
            setMessages(prev => [...prev, aiMsg]);
            setIsThinking(false);
            setStatusMessage('READY');
        }, 2000);
    };

    return (
        <Box flexDirection="column" paddingX={2} paddingY={1}>
            <Header 
                version="1.5.0" 
                mode="ARCHITECT" 
                ram="32GB" 
                indexerStatus="online" 
                sandboxStatus="online" 
            />

            <Box flexDirection="column" flexGrow={1} minHeight={10} marginBottom={1}>
                {messages.map((msg, i) => (
                    <Box key={i} marginBottom={1}>
                        <Box marginRight={1}>
                            <Text color={msg.role === 'user' ? '#ffb000' : '#767676'} bold>
                                {msg.role === 'user' ? 'USR»' : 'SYS●'}
                            </Text>
                        </Box>
                        <Box flexShrink={1}>
                            <Text color={msg.role === 'system' ? '#444444' : 'white'}>
                                {msg.content}
                            </Text>
                        </Box>
                    </Box>
                ))}
                
                {isThinking && (
                    <Box marginBottom={1}>
                        <Text color="#ffb000">
                            <Spinner type="dots" />
                        </Text>
                        <Text color="#767676" italic>  Thinking...</Text>
                    </Box>
                )}
            </Box>

            <Box justifyContent="space-between" borderStyle="single" borderColor="#444444" paddingX={1}>
                <Box>
                    <Text color="#ffb000" bold>COMMAND » </Text>
                    <TextInput 
                        value={query} 
                        onChange={setQuery} 
                        onSubmit={handleSubmit}
                        placeholder="Type something..."
                    />
                </Box>
                <Box>
                    <Text color="#767676">[ STATUS: </Text>
                    <Text color="#ffb000" bold>{statusMessage}</Text>
                    <Text color="#767676"> ]</Text>
                </Box>
            </Box>
        </Box>
    );
};

render(<App />);

