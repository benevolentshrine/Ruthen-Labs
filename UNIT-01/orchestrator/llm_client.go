package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

type LLMClient struct {
	endpoint   string
	model      string
	httpClient *http.Client
	info       *ModelInfo
}

type ModelInfo struct {
	Name      string `json:"name"`
	Details   struct {
		ParameterSize string `json:"parameter_size"`
	} `json:"details"`
}

type ModelTier string

const (
	TierTiny     ModelTier = "TINY"     // < 1B
	TierSmall    ModelTier = "SMALL"    // 1B - 4B
	TierStandard ModelTier = "STANDARD" // > 4B
)

func (c *LLMClient) Tier() ModelTier {
	if c.info == nil {
		c.GetModelInfo()
	}
	if c.info == nil {
		return TierSmall // Default
	}
	size := c.info.Details.ParameterSize
	if strings.Contains(size, "0.5") || strings.Contains(size, "500") {
		return TierTiny
	}
	if strings.Contains(size, "1") || strings.Contains(size, "3") {
		return TierSmall
	}
	return TierStandard
}

func (c *LLMClient) GetModelInfo() {
	resp, err := c.httpClient.Post(c.endpoint+"/api/show", "application/json", bytes.NewBufferString(fmt.Sprintf(`{"name":"%s"}`, c.model)))
	if err != nil {
		return
	}
	defer resp.Body.Close()
	var info ModelInfo
	if err := json.NewDecoder(resp.Body).Decode(&info); err == nil {
		c.info = &info
	}
}

type TagsResponse struct {
	Models []struct {
		Name string `json:"name"`
	} `json:"models"`
}

func (c *LLMClient) GetLocalModels() ([]string, error) {
	resp, err := c.httpClient.Get(c.endpoint + "/api/tags")
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var tags TagsResponse
	if err := json.NewDecoder(resp.Body).Decode(&tags); err != nil {
		return nil, err
	}

	var models []string
	for _, m := range tags.Models {
		models = append(models, m.Name)
	}
	return models, nil
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
func (c *LLMClient) StreamCLI(messages []ollamaMessage, w io.Writer, stopSpinner func(), tier HardwareTier) (string, []Directive, int, int, error) {
	var temp float32
	switch tier {
	case Tier8GB:
		temp = 0.0
	case Tier16GB:
		temp = 0.1
	case Tier32GB:
		temp = 0.3
	default:
		temp = 0.0
	}

	reqBody := ollamaRequest{
		Model:    c.model,
		Messages: messages,
		Stream:   true,
		Options: map[string]interface{}{
			"temperature": temp,
		},
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

	return ParseStreamCLI(resp.Body, w, stopSpinner, tier)
}

// Chat performs a synchronous request for the summary turn.
func (c *LLMClient) Chat(messages []ollamaMessage) (string, error) {
	reqBody := ollamaRequest{
		Model:    c.model,
		Messages: messages,
		Stream:   false,
	}

	b, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	resp, err := http.Post(c.endpoint+"/api/chat", "application/json", bytes.NewReader(b))
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	var chatResp struct {
		Message struct {
			Content string `json:"content"`
		} `json:"message"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&chatResp); err != nil {
		return "", err
	}
	return chatResp.Message.Content, nil
}
