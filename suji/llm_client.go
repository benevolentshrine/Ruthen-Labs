package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

type LLMClient struct {
	endpoint   string
	model      string
	httpClient *http.Client
}

func NewLLMClient(endpoint, defaultModel string) *LLMClient {
	if endpoint == "" {
		endpoint = "http://127.0.0.1:11434"
	}
	return &LLMClient{
		endpoint:   endpoint,
		model:      defaultModel,
		httpClient: &http.Client{},
	}
}

// StreamCLI sends the request to Ollama, streams the raw text to the provided writer (e.g. stdout),
// and returns the accumulated full text alongside any tool calls.
func (c *LLMClient) StreamCLI(messages []ollamaMessage, w io.Writer, stopSpinner func()) (string, []ToolCall, int, int, error) {
	reqBody := ollamaRequest{
		Model:    c.model,
		Messages: messages,
		Stream:   true,
	}

	b, err := json.Marshal(reqBody)
	if err != nil {
		return "", nil, 0, 0, fmt.Errorf("marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", c.endpoint+"/api/chat", bytes.NewReader(b))
	if err != nil {
		return "", nil, 0, 0, fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return "", nil, 0, 0, fmt.Errorf("http: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		b, _ := io.ReadAll(resp.Body)
		return "", nil, 0, 0, fmt.Errorf("status %d: %s", resp.StatusCode, string(b))
	}

	return ParseStreamCLI(resp.Body, w, stopSpinner)
}
