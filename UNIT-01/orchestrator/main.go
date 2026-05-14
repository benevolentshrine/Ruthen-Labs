package main

import (
	"bufio"
	"fmt"
	"io"
	"math/rand"
	"os"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/charmbracelet/glamour"
	"github.com/charmbracelet/huh"
	"github.com/charmbracelet/lipgloss"
	"golang.org/x/term"
)

var (
	BrandColor  = lipgloss.Color("#00E5FF")
	Grey        = lipgloss.Color("#767676")
	LightGrey   = lipgloss.Color("#cccccc")
	PromptStyle = lipgloss.NewStyle().Foreground(BrandColor).Bold(true)
	ToolStyle   = lipgloss.NewStyle().Foreground(Grey).Italic(true)
)

func renderWelcomeBanner(cwd, activeModel, indexerIcon, sandboxIcon string, tier HardwareTier, ramGB int) string {
	width := getTerminalWidth()
	if width > 120 {
		width = 120
	}

	title := lipgloss.NewStyle().Width(width - 4).Align(lipgloss.Center).Bold(true).Foreground(BrandColor).Padding(0, 1).Render("UNIT-01 v1.5.0")
	leftStyle := lipgloss.NewStyle().Width((width - 10) / 2).Align(lipgloss.Left).PaddingLeft(2)
	rightStyle := lipgloss.NewStyle().Width((width - 10) / 2).Align(lipgloss.Left).PaddingLeft(4).BorderStyle(lipgloss.NormalBorder()).BorderLeft(true).BorderForeground(BrandColor)

	leftContent := fmt.Sprintf("\n%s\n\n%s\n%s\n%s", 
		lipgloss.NewStyle().Italic(true).Foreground(LightGrey).Render(getRandomQuote()), 
		PromptStyle.Render("● ACTIVE ENGINE"), 
		lipgloss.NewStyle().Foreground(Grey).Render(activeModel), 
		lipgloss.NewStyle().Foreground(Grey).Render(cwd))
	
	rightContent := fmt.Sprintf("\n%s\n%s\n\n%s\n%s\n\n%s\n%s %s\n%s %s", 
		PromptStyle.Render("SYSTEM STATUS"), 
		lipgloss.NewStyle().Foreground(LightGrey).Render("Sovereign Build Mode Active"), 
		PromptStyle.Render("HARDWARE PROFILE"),
		lipgloss.NewStyle().Foreground(lipgloss.Color("#F1FA8C")).Render(fmt.Sprintf("Detected %dGB RAM", ramGB)),
		PromptStyle.Render("DAEMON HEARTBEAT"), 
		indexerIcon, lipgloss.NewStyle().Foreground(LightGrey).Render("Indexer"), 
		sandboxIcon, lipgloss.NewStyle().Foreground(LightGrey).Render("Sandbox"))

	// Model recommendations block based on RAM
	var recs string
	switch tier {
	case Tier8GB:
		recs = lipgloss.NewStyle().Foreground(Grey).Render(" Recommended Models: 0.5B to 8B (Parser Mode)")
	case Tier16GB:
		recs = lipgloss.NewStyle().Foreground(Grey).Render(" Recommended Models: 0.5B to 14B (Mechanic Mode)")
	case Tier32GB:
		recs = lipgloss.NewStyle().Foreground(Grey).Render(" Recommended Models: 0.5B to 32B+ (Architect Mode)")
	}

	grid := lipgloss.JoinHorizontal(lipgloss.Top, leftStyle.Render(leftContent), rightStyle.Render(rightContent))
	
	banner := lipgloss.NewStyle().Width(width - 2).BorderStyle(lipgloss.RoundedBorder()).BorderForeground(BrandColor).Padding(1, 1).Render(title + "\n" + grid)
	return "\n" + lipgloss.NewStyle().Width(getTerminalWidth()).Align(lipgloss.Center).Render(banner) + "\n" + lipgloss.NewStyle().Width(getTerminalWidth()).Align(lipgloss.Center).Render(recs) + "\n"
}

func getRandomQuote() string {
	quotes := []string{
		"Simplicity is the soul of efficiency.",
		"Talk is cheap. Show me the code.",
		"The best error message is the one that never shows up.",
		"First, solve the problem. Then, write the code.",
		"Clean code always looks like it was written by someone who cares.",
		"Complexity is the enemy of reliability.",
		"One man's constant is another man's variable.",
		"Computers are good at following instructions, but not at reading your mind.",
		"Sovereign AI: Local, private, and yours.",
		"Refactoring is the process of cleaning up the past.",
	}
	return quotes[rand.Intn(len(quotes))]
}

func getExecutorPrompt(ws *Workspace, llm *LLMClient, tier HardwareTier) string {
	home, _ := os.UserHomeDir()
	
	base := fmt.Sprintf(`### UNIT-01 DIRECTIVE PROTOCOL (NON-NEGOTIABLE) ###
- OPERATING_SYSTEM: %s
- USER_HOME: %s
- CURRENT_WORKSPACE: %s
- IDENTITY: You are the UNIT-01 SOVEREIGN ENGINE. You are a native coding orchestrator by Ruthen Labs.

### GROUND TRUTH (PROJECT_MAP & CONTEXT):
%s

### CORE DIRECTIVE:
1. INTERNAL KNOWLEDGE IS DEPRECATED. Use the Ground Truth context provided. It is your only source of reality.
2. You DO have access to the file system. The PROJECT_MAP above IS the real file system.
3. Be concise. Talk is cheap. Show me the code.

### DIRECTIVE TAGS (USE THESE FOR ALL ACTIONS):
- To write a file: <sandbox_write path="path/to/file">CONTENT</sandbox_write>
- To execute a command: <sandbox_exec command="CMD" />
- To list a directory: <indexer_ls path="PATH" />
- To read a file: <indexer_read path="PATH" />
- To delete a file: <sandbox_delete path="PATH" />
- To rollback changes: <sandbox_rollback />

4. If writing code, use the <sandbox_write> tag.`, runtime.GOOS, home, ws.Path, ws.ProjectMap)

	switch tier {
	case Tier8GB:
		base += "\n\n### TIER 8GB RULES:\n- YOU MUST NOT use <thinking> tags.\n- YOU MUST ACT AS A PURE MECHANICAL TRANSLATOR.\n- NO CONVERSATION. NO EXPLANATIONS."
	case Tier16GB:
		base += "\n\n### TIER 16GB RULES:\n- YOU MAY use short <thinking> tags to plan logic.\n- NO CONVERSATION. OUTPUT ONLY CODE."
	case Tier32GB:
		base += "\n\n### TIER 32GB RULES:\n- YOU ARE THE ARCHITECT. Use extensive <thinking> tags to plan multi-file refactors.\n- NO CONVERSATION OUTSIDE OF THINKING TAGS."
	}

	return base
}

func getNarratorPrompt() string {
	return `You are the UNIT-01 NARRATOR. Summarize the tool results in premium English. Focus on completion status.`
}

func getTerminalWidth() int {
	width, _, err := term.GetSize(int(os.Stdout.Fd()))
	if err != nil || width <= 0 {
		return 80 // Fallback
	}
	return width
}

func renderResponse(text string) {
	if text == "" {
		return
	}
	width := getTerminalWidth()
	// Apply some padding for readability
	contentWidth := width - 10
	if contentWidth < 40 {
		contentWidth = 40
	}

	r, _ := glamour.NewTermRenderer(
		glamour.WithAutoStyle(),
		glamour.WithWordWrap(contentWidth),
	)
	out, _ := r.Render(text)
	
	// Center the output slightly with left padding
	padding := lipgloss.NewStyle().PaddingLeft(4).Render(out)
	fmt.Print(padding)
}

func main() {
	rand.Seed(time.Now().UnixNano())
	history := &History{}
	llm := NewLLMClient("", "qwen2.5-coder:3b")

	daemonMgr := NewDaemonManager()
	ws := NewWorkspace()

	// --- BOOTSTRAP DAEMONS ---
	// We spawn them first so they are ready for the Trust Gate
	indexerStatus := daemonMgr.SpawnIfMissing("indexer", "/tmp/ruthen/indexer.sock")
	sandboxStatus := daemonMgr.SpawnIfMissing("sandbox", "/tmp/ruthen/sandbox.sock")

	cwd, _ := os.Getwd()

	// --- STARTUP TRUST GATE ---
	fmt.Printf("\n"+lipgloss.NewStyle().Foreground(BrandColor).Bold(true).Render("◆ UNIT-01 TRUST GATE")+"\n")
	var trusted bool
	form := huh.NewForm(
		huh.NewGroup(
			huh.NewConfirm().
				Title(fmt.Sprintf("Authorize UNIT-01 to manage this directory?\n[%s]", cwd)).
				Value(&trusted).
				Affirmative("Trust & Index").
				Negative("Stay Restricted"),
		),
	)
	if err := form.Run(); err == nil && trusted {
		fmt.Printf("◆ Activating workspace — syncing Indexer...\n")
		ws.Set(cwd)
	}

	tier, ramGB := GetHardwareTier()

	fmt.Println(renderWelcomeBanner(cwd, llm.model, indexerStatus.Icon(), sandboxStatus.Icon(), tier, ramGB))

	scanner := bufio.NewScanner(os.Stdin)
	for {
		// --- RESET: Clear raw technical memory before new user input ---
		history.PurgeSystemMessages()
		
		fmt.Printf("\n %s  ", PromptStyle.Render(">"))
		if !scanner.Scan() {
			break
		}
		input := strings.TrimSpace(scanner.Text())
		if input == "" {
			continue
		}

		if strings.HasPrefix(input, "/") {
			HandleCommandCLI(input, &Config{DefaultStyle: "sovereign"}, history, llm, ws)
			continue
		}

		history.Append(Message{Role: "user", Content: input, Timestamp: time.Now()})

		// --- SOVEREIGN PIPELINE ---
		doneThinking := make(chan struct{})
		var wg sync.WaitGroup
		wg.Add(1)
		go func() {
			defer wg.Done()
			frames := []string{"⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"}
			idx := 0
			quote := getRandomQuote()
			quoteTick := time.NewTicker(3 * time.Second)
			defer quoteTick.Stop()

			for {
				select {
				case <-doneThinking:
					fmt.Print("\r\033[K")
					return
				case <-quoteTick.C:
					quote = getRandomQuote()
				default:
					fmt.Printf("\r %s %s %s  ", 
						lipgloss.NewStyle().Foreground(BrandColor).Render(frames[idx%len(frames)]),
						lipgloss.NewStyle().Foreground(Grey).Render("Thinking..."),
						lipgloss.NewStyle().Foreground(Grey).Italic(true).Render("“"+quote+"”"),
					)
					idx++
					time.Sleep(80 * time.Millisecond)
				}
			}
		}()

		stopSpinner := func() {
			select {
			case <-doneThinking:
			default:
				close(doneThinking)
				wg.Wait()
			}
		}

		// 1. GATHER CONTEXT (YOMI)
		ws.Refresh() // Ensure ProjectMap is fresh
		autoCtx := getAutoContext(input, ws)
		
		// 2. QUERY ENGINE
		execPrompt := ollamaMessage{Role: "system", Content: getExecutorPrompt(ws, llm, tier) + autoCtx}
		activeMessages := append([]ollamaMessage{execPrompt}, history.OllamaMessages()...)
		
		fullResponse, directives, _, _, err := llm.StreamCLI(activeMessages, io.Discard, stopSpinner, tier)
		stopSpinner()

		if err != nil {
			fmt.Printf("\n[Error: %v]\n", err)
			continue
		}

		history.Append(Message{Role: "assistant", Content: fullResponse, Timestamp: time.Now()})
		renderResponse(fullResponse)

		// 3. EXECUTE DIRECTIVES (BORU)
		if len(directives) > 0 {
			for _, dir := range directives {
				result := ExecuteTool(dir.Name, dir.Args, &Config{}, ws)
				fmt.Printf("\n" + lipgloss.NewStyle().Foreground(lipgloss.Color("#FFB000")).Render("◆ DIRECTIVE RESULT: ") + "%s\n", result)
				history.Append(Message{Role: "system", Content: fmt.Sprintf("Tool [%s] result: %s", dir.Name, result), Timestamp: time.Now()})
			}
		}
	}
}
