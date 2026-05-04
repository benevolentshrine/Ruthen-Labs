package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"regexp"
	"strings"
)

type ollamaRequest struct {
	Model    string          `json:"model"`
	Messages []ollamaMessage `json:"messages"`
	Stream   bool            `json:"stream"`
}

type ollamaMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type ollamaChunk struct {
	Model   string `json:"model"`
	Message struct {
		Role      string     `json:"role"`
		Content   string     `json:"content"`
		ToolCalls []ToolCall `json:"tool_calls,omitempty"`
	} `json:"message"`
	Done            bool   `json:"done"`
	Error           string `json:"error,omitempty"`
	DoneReason      string `json:"done_reason,omitempty"`
	PromptEvalCount int    `json:"prompt_eval_count,omitempty"`
	EvalCount       int    `json:"eval_count,omitempty"`
}

type ToolCall struct {
	Function Function `json:"function"`
}

type Function struct {
	Name      string         `json:"name"`
	Arguments map[string]any `json:"arguments"`
}

func ParseStreamCLI(body io.Reader, w io.Writer, stopSpinner func()) (string, []ToolCall, int, int, error) {
	var toolCalls []ToolCall
	var fullResponse strings.Builder

	scanner := bufio.NewScanner(body)
	scanner.Buffer(make([]byte, 1<<20), 1<<20)

	var inThink bool
	var hasFinishedThink bool
	spinnerStopped := false

	for scanner.Scan() {
		line := scanner.Bytes()
		if len(line) == 0 {
			continue
		}

		var chunk ollamaChunk
		if err := json.Unmarshal(line, &chunk); err != nil {
			return fullResponse.String(), nil, 0, 0, fmt.Errorf("invalid chunk: %w", err)
		}

		if chunk.Error != "" {
			return fullResponse.String(), nil, 0, 0, fmt.Errorf("ollama API error: %s", chunk.Error)
		}

		if len(chunk.Message.ToolCalls) > 0 {
			toolCalls = append(toolCalls, chunk.Message.ToolCalls...)
		}

		if tok := chunk.Message.Content; tok != "" {
			fullResponse.WriteString(tok)
			currentStr := fullResponse.String()

			if !spinnerStopped {
				if stopSpinner != nil {
					stopSpinner()
				}
				spinnerStopped = true
			}

			// Real-time Thinking UI logic
			if !hasFinishedThink {
				thinkStart := strings.Index(currentStr, "<thinking>")
				if thinkStart == -1 {
					thinkStart = strings.Index(currentStr, "<think>")
				}

				if thinkStart != -1 {
					inThink = true
					thinkEnd := strings.Index(currentStr, "</thinking>")
					if thinkEnd == -1 {
						thinkEnd = strings.Index(currentStr, "</think>")
					}

					if thinkEnd != -1 {
						// Thinking just finished!
						inThink = false
						hasFinishedThink = true
						// Print the collapsed L-tab
						fmt.Printf("\r\033[K\033[38;5;242m└ ▾ Thought process collapsed\033[0m\n")
					} else {
						// We are currently thinking. Print the dynamic ticker.
						thinkText := currentStr[thinkStart:]
						// Clean up for single-line display
						thinkText = strings.ReplaceAll(thinkText, "<thinking>", "")
						thinkText = strings.ReplaceAll(thinkText, "<think>", "")
						thinkText = strings.ReplaceAll(thinkText, "\n", " ")
						
						display := thinkText
						if len(display) > 60 {
							display = "..." + display[len(display)-57:]
						}
						// Print updating line (carriage return, clear line)
						fmt.Printf("\r\033[K\033[38;5;242m└ \033[3m%s\033[0m", display)
					}
				}
			} else if hasFinishedThink || !inThink {
				// Only print the non-thinking content to the writer
				w.Write([]byte(tok))
			}
		}

		if chunk.Done {
			content := fullResponse.String()
			xmlTools := parseXMLTools(content)
			toolCalls = append(toolCalls, xmlTools...)
			return content, toolCalls, chunk.PromptEvalCount, chunk.EvalCount, nil
		}
	}

	if err := scanner.Err(); err != nil {
		return fullResponse.String(), nil, 0, 0, err
	}
	
	content := fullResponse.String()
	xmlTools := parseXMLTools(content)
	toolCalls = append(toolCalls, xmlTools...)

	return content, toolCalls, 0, 0, nil
}

func parseXMLTools(text string) []ToolCall {
	var calls []ToolCall
	
	// Handle <tool_call name="..." ... />
	toolRe := regexp.MustCompile(`<tool_call name="([^"]+)"\s*([^>]*)\s*/?>`)
	attrRe := regexp.MustCompile(`(\w+)="([^"]*)"`)

	toolMatches := toolRe.FindAllStringSubmatch(text, -1)
	for _, match := range toolMatches {
		name := match[1]
		attrs := match[2]
		args := make(map[string]any)
		attrMatches := attrRe.FindAllStringSubmatch(attrs, -1)
		for _, am := range attrMatches {
			args[am[1]] = am[2]
		}
		calls = append(calls, ToolCall{Function: Function{Name: name, Arguments: args}})
	}

	// Handle <request_context type="..." >...</request_context>
	ctxRe := regexp.MustCompile(`<request_context type="([^"]+)">([^<]+)</request_context>`)
	ctxMatches := ctxRe.FindAllStringSubmatch(text, -1)
	for _, match := range ctxMatches {
		ctxType := match[1]
		val := match[2]
		args := make(map[string]any)
		if ctxType == "file" {
			args["path"] = val
			calls = append(calls, ToolCall{Function: Function{Name: "yomi_read", Arguments: args}})
		} else if ctxType == "search" {
			args["query"] = val
			calls = append(calls, ToolCall{Function: Function{Name: "yomi_search", Arguments: args}})
		}
	}

	return calls
}

// stripThinkingTokens removes <think>...</think> blocks generated by reasoning models like qwen3.
func stripThinkingTokens(s string) string {
	for {
		start := strings.Index(s, "<think>")
		if start == -1 {
			break
		}
		end := strings.Index(s, "</think>")
		if end == -1 {
			s = s[:start]
			break
		}
		s = s[:start] + s[end+len("</think>"):]
	}
	return strings.TrimSpace(s)
}

func BuildMessages(history []Message) []ollamaMessage {
	out := make([]ollamaMessage, 0, len(history))
	for _, m := range history {
		out = append(out, ollamaMessage{Role: m.Role, Content: m.Content})
	}
	return out
}
