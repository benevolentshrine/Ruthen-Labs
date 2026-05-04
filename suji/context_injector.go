package main

import (
	"fmt"
	"os"

	"github.com/pkoukk/tiktoken-go"
)

// InjectContext reads a file from disk, counts its tokens, and returns a formatted string.
func InjectContext(path string) (string, int, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return "", 0, fmt.Errorf("Failed to read %s: %w", path, err)
	}

	text := string(content)
	tokens := CountTokens(text)

	// 80% of 32k context window = 25,600 tokens.
	if tokens > 25600 {
		return text, tokens, fmt.Errorf("Warning: Context window at 80%% capacity (%d tokens).", tokens)
	}

	return text, tokens, nil
}

// CountTokens uses tiktoken (cl100k_base for gpt-4/qwen) to estimate token usage.
func CountTokens(text string) int {
	tkm, err := tiktoken.GetEncoding("cl100k_base")
	if err != nil {
		return len(text) / 4 // crude fallback
	}
	tokenIDs := tkm.Encode(text, nil, nil)
	return len(tokenIDs)
}
