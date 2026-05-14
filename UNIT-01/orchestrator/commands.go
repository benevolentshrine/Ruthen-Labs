package main

import (
	"fmt"
	"os"
	"strings"
	"unit01/clients"

	"github.com/charmbracelet/huh"
	"github.com/charmbracelet/lipgloss"
)

// HandleCommandCLI processes slash commands in the Orchestrator terminal.
func HandleCommandCLI(input string, cfg *Config, history *History, llm *LLMClient, ws *Workspace) {
	parts := strings.Fields(input)
	if len(parts) == 0 {
		return
	}

	cmd := parts[0]

	switch cmd {
	case "/help":
		showHelp()
	case "/doctor":
		showDoctor(llm, ws)
	case "/models", "/model":
		interactiveModelSwitch(llm)
	case "/context":
		showContext(ws)
	case "/reindex":
		forceReindex(ws)
	case "/clear":
		history.messages = nil
		fmt.Println("◆ Conversation history cleared.")
	case "/undo", "/rollback":
		fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("#FF5555")).Render("◆ Initiating Rollback..."))
		stdout := ExecuteTool("sandbox_rollback", nil, cfg, ws)
		fmt.Println(stdout)
	case "/exit", "/quit":
		fmt.Println("◆ Shutting down UNIT-01...")
		os.Exit(0)
	default:
		fmt.Printf("Unknown command: %s. Type /help for a list of commands.\n", cmd)
	}
}

func showHelp() {
	helpStyle := lipgloss.NewStyle().Foreground(BrandColor)
	fmt.Println("\n" + helpStyle.Render("─── UNIT-01 COMMANDS ───"))
	fmt.Println(" /context  - See exactly what the AI knows right now")
	fmt.Println(" /doctor   - Run health check on Engine")
	fmt.Println(" /undo     - Revert the last file change (via Sandbox)")
	fmt.Println(" /model    - Switch active Ollama engine/tier")
	fmt.Println(" /reindex  - Force Indexer to re-scan your workspace")
	fmt.Println(" /clear    - Wipe conversation history")
	fmt.Println(" /exit     - Terminate session")
	fmt.Println()
}

func showDoctor(llm *LLMClient, ws *Workspace) {
	titleStyle := lipgloss.NewStyle().Bold(true).Foreground(BrandColor).Padding(0, 1).BorderStyle(lipgloss.RoundedBorder()).BorderForeground(BrandColor)
	fmt.Println("\n" + titleStyle.Render("UNIT-01 DIAGNOSTIC REPORT"))

	// Check Model
	tier := llm.Tier()
	fmt.Printf(" ENGINE:  %s (%s Tier)\n", llm.model, tier)

	// Check Workspace
	if ws.Active {
		fmt.Printf(" CONTEXT: Active in %s\n", ws.Path)
	} else {
		fmt.Printf(" CONTEXT: No workspace set\n")
	}

	// Check Sandbox
	sandboxClient := clients.NewSandboxClient()
	// Simple heartbeat check
	if _, err := sandboxClient.SetWorkspace(ws.Path); err == nil {
		fmt.Printf(" SANDBOX: %s [CONNECTED]\n", lipgloss.NewStyle().Foreground(lipgloss.Color("#50FA7B")).Render("ACTIVE"))
	} else {
		fmt.Printf(" SANDBOX: %s [UNREACHABLE]\n", lipgloss.NewStyle().Foreground(lipgloss.Color("#FF5555")).Render("OFFLINE"))
	}

	// Check Indexer
	indexerClient := clients.NewIndexerClient()
	if _, err := indexerClient.List("."); err == nil {
		fmt.Printf(" INDEXER: %s [CONNECTED]\n", lipgloss.NewStyle().Foreground(lipgloss.Color("#50FA7B")).Render("ACTIVE"))
	} else {
		fmt.Printf(" INDEXER: %s [UNREACHABLE]\n", lipgloss.NewStyle().Foreground(lipgloss.Color("#FF5555")).Render("OFFLINE"))
	}
	fmt.Println()
}

func showContext(ws *Workspace) {
	if !ws.Active {
		fmt.Println("❌ No active workspace. AI is operating in blind mode.")
		return
	}
	titleStyle := lipgloss.NewStyle().Bold(true).Background(BrandColor).Foreground(lipgloss.Color("#000000")).Padding(0, 1)
	fmt.Println("\n" + titleStyle.Render("AI VISIBILITY"))
	fmt.Println(ws.ContextLine())
}

func forceReindex(ws *Workspace) {
	if !ws.Active {
		fmt.Println("❌ Set a workspace first to reindex.")
		return
	}
	fmt.Print("◆ Refreshing project map... ")
	_, err := ws.Set(ws.Path)
	if err != nil {
		fmt.Printf("failed: %v\n", err)
	} else {
		fmt.Println("✅ Done.")
	}
}

func interactiveModelSwitch(llm *LLMClient) {
	models, err := llm.GetLocalModels()
	if err != nil {
		fmt.Printf("❌ Failed to fetch models from Ollama: %v\n", err)
		return
	}

	if len(models) == 0 {
		fmt.Println("❌ No models found in Ollama. Please pull a model first (e.g., 'ollama pull qwen2.5-coder').")
		return
	}

	var options []huh.Option[string]
	for _, m := range models {
		// Clean up common long names for the UI
		label := strings.ReplaceAll(m, ":latest", "")
		options = append(options, huh.NewOption(label, m))
	}

	var selectedModel string
	form := huh.NewForm(
		huh.NewGroup(
			huh.NewSelect[string]().
				Title("SELECT ACTIVE ENGINE").
				Description("Choose from your locally installed Ollama models").
				Options(options...).
				Value(&selectedModel),
		),
	)

	if err := form.Run(); err != nil {
		return
	}

	if selectedModel != "" {
		llm.model = selectedModel
		llm.GetModelInfo() // Update tier info immediately
		fmt.Printf("◆ Switched active engine to: %s\n", lipgloss.NewStyle().Foreground(BrandColor).Bold(true).Render(selectedModel))
	}
}
