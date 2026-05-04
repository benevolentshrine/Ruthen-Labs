package main

import (
	"fmt"
	"regexp"
	"strings"
)

var fileTagRegex = regexp.MustCompile(`@file:([^\s]+)`)

// ParseInputContext scans the user input for @file tags, reads the file content,
// and injects it directly into the chat history as a system message.
// It returns the cleaned input string.
func ParseInputContext(input string, history *History) string {
	// Find all matches: e.g., @file:path/to/file.go
	matches := fileTagRegex.FindAllStringSubmatch(input, -1)

	cleanInput := input
	for _, match := range matches {
		fullTag := match[0]
		path := match[1]

		// Remove the tag from the message sent to the LLM.
		cleanInput = strings.ReplaceAll(cleanInput, fullTag, "")

		// Read and inject the file
		content, tokens, err := InjectContext(path)
		if err != nil {
			fmt.Printf("⚠️ Context warning: %v\n", err)
		}

		if content != "" {
			history.Append(Message{
				Role:    "system",
				Content: fmt.Sprintf("Context file %s attached (%d tokens):\n```\n%s\n```", path, tokens, content),
			})
			fmt.Printf("📎 Attached file: %s (%d tokens)\n", path, tokens)
		}
	}

	return strings.TrimSpace(cleanInput)
}
