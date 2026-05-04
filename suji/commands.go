package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"strings"

	"github.com/charmbracelet/huh"
	"github.com/charmbracelet/lipgloss"
)

// HandleCommandCLI intercepts user commands starting with "/"
func HandleCommandCLI(input string, cfg *Config, history *History, llm *LLMClient, currentSessionID *string, currentSessionName *string) {
	parts := strings.Fields(input)
	if len(parts) == 0 {
		return
	}
	cmd := parts[0]
	args := parts[1:]

	switch cmd {
	case "/help":
		fmt.Println("Suji Commands:")
		fmt.Println("  /search <q>   - Query the Yomi indexer")
		fmt.Println("  /context      - Manage active files and tokens")
		fmt.Println("  /sessions     - View and resume past sessions")
		fmt.Println("  /ask <q>      - Ask a quick question without saving to history")
		fmt.Println("  /undo         - Rollback the last Boru execution")
		fmt.Println("  /models       - List Ollama models")
		fmt.Println("  /models <name>- Switch to a specific model")
		fmt.Println("  /style        - Toggle behavior mode")
		fmt.Println("  /compact      - Manually summarize history")
		fmt.Println("  /mcp          - Manage tool servers")
		fmt.Println("  /quit         - Exit the application")

	case "/quit", "/exit":
		fmt.Println("Shutting down...")
		os.Exit(0)

	case "/models":
		if len(args) > 0 {
			llm.model = args[0]
			fmt.Printf("✅ Switched active model to: %s\n", llm.model)
			return
		}
		
		fmt.Println("Fetching available local models...")
		resp, err := http.Get("http://127.0.0.1:11434/api/tags")
		if err != nil {
			fmt.Printf("❌ Failed to contact Ollama: %v\n", err)
			return
		}
		defer resp.Body.Close()

		var result struct {
			Models []struct {
				Name string `json:"name"`
			} `json:"models"`
		}
		if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
			fmt.Printf("❌ Failed to parse models: %v\n", err)
			return
		}

		var options []huh.Option[string]
		for _, m := range result.Models {
			options = append(options, huh.NewOption(m.Name, m.Name))
		}

		var selectedModel string
		formErr := huh.NewSelect[string]().
			Title("Select an Ollama Model").
			Options(options...).
			Value(&selectedModel).
			Run()

		if formErr != nil {
			fmt.Println("Selection cancelled.")
			return
		}

		llm.model = selectedModel
		fmt.Printf("✅ Switched active model to: %s\n", llm.model)

	case "/style":
		if cfg.DefaultStyle == "casual" {
			cfg.DefaultStyle = "context"
		} else if cfg.DefaultStyle == "context" {
			cfg.DefaultStyle = "build"
		} else {
			cfg.DefaultStyle = "casual"
		}
		fmt.Printf("✅ Style mode toggled to: %s\n", cfg.DefaultStyle)

	case "/compact":
		fmt.Println("🧹 Compacting history...")
		history.Compact("Manually compacted by user.", 10)
		fmt.Println("✅ History compacted successfully.")

	case "/sessions":
		selectedID := HandleSessionsMenu(history)
		if selectedID == "new" {
			history.messages = nil
			*currentSessionID = ""
			*currentSessionName = "New Session"
			fmt.Println("✅ Started a fresh session.")
			// In main.go we will handle generating a new session ID
		} else if selectedID != "" {
			name, err := LoadSession(selectedID, history)
			if err != nil {
				fmt.Printf("❌ Failed to load session: %v\n", err)
			} else {
				*currentSessionID = selectedID
				*currentSessionName = name
				fmt.Printf("✅ Resumed session: %s\n", name)
			}
		}

	case "/ask":
		if len(args) == 0 {
			fmt.Println("Usage: /ask <question>")
			return
		}
		question := strings.Join(args, " ")
		fmt.Printf(lipgloss.NewStyle().Foreground(brandColor).Render("└ Quick Ask: "))
		
		tempMessages := []ollamaMessage{
			{Role: "system", Content: getSystemPrompt("casual")},
			{Role: "user", Content: question},
		}
		
		stopSpinner := func() { fmt.Print("\n") }
		fullResponse, _, _, _, err := llm.StreamCLI(tempMessages, os.Stdout, stopSpinner)
		if err != nil {
			fmt.Printf("\n[Error: %v]\n", err)
		} else {
			renderResponse(fullResponse)
		}

	case "/undo", "/rollback":
		fmt.Println("Rolling back the last Boru execution...")
		out := ExecuteTool("boru_rollback", map[string]any{"session_id": "latest"}, cfg)
		fmt.Println(out)

	case "/context":
		fmt.Printf("Current Context Window:\n")
		fmt.Printf("- Active Model: %s\n", llm.model)
		fmt.Printf("- Active Style: %s\n", cfg.DefaultStyle)
		fmt.Printf("- Messages in Memory: %d\n", len(history.messages))

	case "/search":
		if len(args) == 0 {
			fmt.Println("Usage: /search <query>")
			return
		}
		query := strings.Join(args, " ")
		fmt.Printf("🔍 Searching Yomi index for: %s\n", query)
		out := ExecuteTool("yomi_search", map[string]any{"query": query}, cfg)
		fmt.Println(out)

	case "/read":
		if len(args) == 0 {
			fmt.Println("Usage: /read <file_path>")
			return
		}
		path := args[0]
		fmt.Printf("📖 Reading file: %s\n", path)
		out := ExecuteTool("yomi_read", map[string]any{"path": path}, cfg)
		fmt.Println(out)

	default:
		fmt.Printf("Unknown command: %s. Type /help for a list of commands.\n", cmd)
	}
}
