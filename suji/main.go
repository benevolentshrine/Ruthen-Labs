package main

import (
	"bufio"
	"fmt"
	"io"
	"math/rand"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/charmbracelet/glamour"
	"github.com/charmbracelet/lipgloss"
)

var (
	brandColor  = lipgloss.Color("#00E5FF")
	grey        = lipgloss.Color("#767676")
	lightGrey   = lipgloss.Color("#cccccc")
	promptStyle = lipgloss.NewStyle().Foreground(lightGrey).Background(lipgloss.Color("#333333")).Padding(0, 1)
	userStyle   = lipgloss.NewStyle().Foreground(lightGrey).Background(lipgloss.Color("#333333"))
	toolStyle   = lipgloss.NewStyle().Foreground(grey)
)

func getRandomQuote() string {
	quotes := []string{
		"Code is like humor. When you have to explain it, it's bad.",
		"Simplicity is the soul of efficiency.",
		"First, solve the problem. Then, write the code.",
		"Talk is cheap. Show me the code.",
		"The best error message is the one that never shows up.",
	}
	rand.Seed(time.Now().UnixNano())
	return quotes[rand.Intn(len(quotes))]
}

func renderWelcomeBanner(cwd string, activeModel string, yomiStatus, boruStatus string) string {
	droplet := `      ▄      
     ███     
    █████    
   ███████   
   ███████   
    ▀███▀    `

	dropletStyle := lipgloss.NewStyle().Foreground(brandColor).Padding(1, 4)
	leftBlock := lipgloss.JoinVertical(lipgloss.Center,
		lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#ffffff")).Render(getRandomQuote()),
		dropletStyle.Render(droplet),
		lipgloss.NewStyle().Foreground(grey).Render(fmt.Sprintf("%s · Suji Sovereign", activeModel)),
		lipgloss.NewStyle().Foreground(grey).Render(cwd),
	)

	rightBlock := lipgloss.JoinVertical(lipgloss.Left,
		lipgloss.NewStyle().Foreground(brandColor).Render("Tips for getting started"),
		lipgloss.NewStyle().Foreground(lightGrey).Render("• Use @file:path to attach context"),
		lipgloss.NewStyle().Foreground(lightGrey).Render("• Run /style to toggle execution modes"),
		"",
		lipgloss.NewStyle().Foreground(brandColor).Render("Recent activity"),
		lipgloss.NewStyle().Foreground(lightGrey).Render(fmt.Sprintf("[%s] Yomi daemon active", yomiStatus)),
		lipgloss.NewStyle().Foreground(lightGrey).Render(fmt.Sprintf("[%s] Boru sandbox initialized", boruStatus)),
	)

	leftBox := lipgloss.NewStyle().PaddingRight(4).Render(leftBlock)
	rightBox := lipgloss.NewStyle().PaddingLeft(4).BorderStyle(lipgloss.NormalBorder()).BorderLeft(true).BorderForeground(brandColor).Render(rightBlock)

	content := lipgloss.JoinHorizontal(lipgloss.Top, leftBox, rightBox)

	return lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(brandColor).
		Padding(1, 2).
		MarginBottom(1).
		Render(content)
}

func renderResponse(text string) {
	// Colors
	mutedGrey := lipgloss.Color("#666666")

	// 1. Extract Thinking
	thinking := ""
	thinkStartTag := "<thinking>"
	thinkEndTag := "</thinking>"
	if start := strings.Index(text, thinkStartTag); start != -1 {
		if end := strings.Index(text, thinkEndTag); end != -1 {
			thinking = strings.TrimSpace(text[start+len(thinkStartTag) : end])
		}
	}

	// 2. Extract and Clean Main Content
	// We want to remove the specific mode tags but keep the meat
	content := text
	tagsToRemove := []string{
		"<thinking>", "</thinking>",
		"<response>", "</response>",
		"<analysis>", "</analysis>",
		"<context_used>", "</context_used>",
		"<tool_calls>", "</tool_calls>",
		"<task_complete />",
	}
	
	// If it's wrapped in a specific main tag, try to extract just that part first
	// to avoid printing thinking content twice if it was inside a tag (unlikely but safe)
	if thinking != "" {
		content = strings.Replace(content, thinkStartTag+thinking+thinkEndTag, "", 1)
	}

	for _, tag := range tagsToRemove {
		content = strings.ReplaceAll(content, tag, "")
	}
	content = strings.TrimSpace(content)

	// 3. Render Thinking (The "L" Tab)
	if thinking != "" {
		thinkStyle := lipgloss.NewStyle().
			Foreground(mutedGrey).
			Italic(true).
			PaddingLeft(2)
		
		fmt.Printf("\n%s\n", thinkStyle.Render("└ " + thinking))
	}

	// 4. Render Main Content
	if content != "" {
		fmt.Println() // Gap before response
		r, _ := glamour.NewTermRenderer(
			glamour.WithStandardStyle("dark"),
			glamour.WithWordWrap(100),
		)
		out, _ := r.Render(content)
		fmt.Print(out)
	}
	fmt.Println() // Gap at the end
}

func main() {
	cfg, err := LoadConfig()
	if err != nil {
		fmt.Printf("Warning: Failed to load config: %v\n", err)
	}
	daemonMgr := NewDaemonManager()

	yomiStatus := daemonMgr.SpawnIfMissing("yomi", "/tmp/sumi/yomi.sock")
	boruStatus := daemonMgr.SpawnIfMissing("boru", "/tmp/sumi/boru.sock")

	llm := NewLLMClient(cfg.ModelEndpoint, "qwen2.5-coder:0.5b")
	history := newHistory()

	scanner := bufio.NewScanner(os.Stdin)

	cwd, _ := os.Getwd()
	fmt.Println()
	fmt.Println(lipgloss.NewStyle().Foreground(brandColor).Render(" Suji Orchestrator v1.0.0 "))
	fmt.Println(renderWelcomeBanner(cwd, llm.model, yomiStatus.Icon(), boruStatus.Icon()))

	currentSessionID := ""
	currentSessionName := "New Session"

	for {
		fmt.Print(promptStyle.Render(">") + " ")
		if !scanner.Scan() {
			break
		}

		input := strings.TrimSpace(scanner.Text())
		if input == "" {
			continue
		}

		// Handle Slash Commands
		if strings.HasPrefix(input, "/") {
			HandleCommandCLI(input, cfg, &history, llm, &currentSessionID, &currentSessionName)
			continue
		}

		// Handle @file context injection
		input = ParseInputContext(input, &history)

		if input == "" {
			continue
		}

		history.Append(Message{Role: "user", Content: input})

		// Go up 1 line (back to prompt line) and replace with the styled user message
		userMsgBox := lipgloss.NewStyle().
			Foreground(lightGrey).
			Background(lipgloss.Color("#2a2a2a")).
			Padding(0, 1).
			Bold(true).
			Render("> " + input)
		fmt.Printf("\033[1A\033[K%s\n", userMsgBox)

		// Spinner with a SYNC done channel
		doneThinking := make(chan struct{}, 1)
		spinnerDone := make(chan struct{})
		go func() {
			defer close(spinnerDone)
			
			// Mode-aware phrases
			var phrases []string
			switch cfg.DefaultStyle {
			case "build":
				phrases = []string{"Analyzing context...", "Planning execution...", "Making it happen...", "Verifying logic...", "Almost there..."}
			case "context":
				phrases = []string{"Mapping codebase...", "Tracing logic...", "Searching index..."}
			default:
				// Casual mode is silent/clean
				phrases = []string{"Thinking..."}
			}

			frames := []string{"⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"}
			i := 0
			phraseIdx := 0
			for {
				select {
				case <-doneThinking:
					fmt.Print("\r\033[K")
					return
				default:
					if i%20 == 0 {
						phraseIdx = rand.Intn(len(phrases))
					}
					spin := lipgloss.NewStyle().Foreground(brandColor).Render(frames[i%len(frames)] + " " + phrases[phraseIdx])
					fmt.Printf("\r\033[K%s", spin)
					time.Sleep(80 * time.Millisecond)
					i++
				}
			}
		}()

		// Inject Active System Prompt
		sysPrompt := getSystemPrompt(cfg.DefaultStyle)
		activeMessages := append([]ollamaMessage{{Role: "system", Content: sysPrompt}}, BuildMessages(history.messages)...)

		// Ensure we only stop the spinner once
		stopSpinnerOnce := &sync.Once{}
		stopSpinner := func() {
			stopSpinnerOnce.Do(func() {
				doneThinking <- struct{}{}
				<-spinnerDone
			})
		}

		// Execute LLM Stream
		start := time.Now()
		fullResponse, toolCalls, promptTokens, completionTokens, err := llm.StreamCLI(activeMessages, os.Stdout, stopSpinner)

		// Make sure spinner is stopped if stream ended before emitting tokens
		stopSpinner()

		if err != nil {
			fmt.Printf("\n[Error: %v]\n", err)
			continue
		}

		// Render the parsed and styled response (handles <thinking> and mode tags)
		renderResponse(fullResponse)

		history.Append(Message{Role: "assistant", Content: fullResponse})

		// Handle tool execution loop (Boru/Yomi)
		if len(toolCalls) > 0 {
			for _, tc := range toolCalls {
				fmt.Printf(lipgloss.NewStyle().Foreground(brandColor).Bold(true).Render("◆ %s")+"(%v)\n", tc.Function.Name, tc.Function.Arguments)
				stdout := ExecuteTool(tc.Function.Name, tc.Function.Arguments, cfg)
				fmt.Printf(toolStyle.Render("  └ Done")+"\n\n")
				history.Append(Message{Role: "system", Content: stdout})
			}
		}

		if promptTokens > 0 || completionTokens > 0 {
			elapsed := time.Since(start).Seconds()
			stats := fmt.Sprintf("(%.1fs · %d prompt tokens · %d completion tokens)", elapsed, promptTokens, completionTokens)
			fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("#555555")).Render(stats) + "\n")
		}

		// Session Persistence & Auto-Naming
		if currentSessionID == "" {
			currentSessionID = fmt.Sprintf("%d", time.Now().Unix())
		}
		
		// Auto-name on the 2nd interaction (4 messages: 2 user, 2 assistant)
		if currentSessionName == "New Session" && len(history.messages) >= 4 {
			go func(sid string, hist *History) {
				nameLLM := NewLLMClient(cfg.ModelEndpoint, "qwen2.5-coder:0.5b")
				prompt := "Summarize this conversation in 3 to 5 words. Do not use quotes or punctuation. Just the title."
				tempMsg := []ollamaMessage{
					{Role: "system", Content: "You are a title generator."},
					{Role: "user", Content: hist.messages[0].Content + "\n\n" + prompt},
				}
				// discard stdout
				title, _, _, _, _ := nameLLM.StreamCLI(tempMsg, io.Discard, nil)
				if title != "" {
					currentSessionName = strings.TrimSpace(title)
					SaveSession(sid, currentSessionName, hist)
				}
			}(currentSessionID, &history)
		}

		SaveSession(currentSessionID, currentSessionName, &history)
	}

	daemonMgr.Shutdown()
}

func getSystemPrompt(style string) string {
	switch style {
	case "context":
		return `# ROLE & IDENTITY
You are SUJI-CONTEXT, a read-only codebase analyst built by Sumi Labs. You have access to indexed project files via YOMI.

# COGNITIVE BUDGET (ANTI-OVERTHINKING)
- THINKING CAP: Maximum 5 lines. Only when analyzing complex interactions or resolving ambiguous queries.
- PRECISION OVER PROSE: Prioritize file paths, function names, and exact line references.
- FLUFF BAN: No greetings. Start analysis immediately.

# MODE CONSTRAINTS
- READ/SEARCH ONLY: You may request file content or semantic search. You CANNOT apply diffs or run commands.
- YOMI PROTOCOL: Use XML tags to fetch data. Suji will intercept these.
  <request_context type="file">path/to/file.ext</request_context>
  <request_context type="search">query terms</request_context>

# RESPONSE TEMPLATE
<thinking>[Max 5 lines]</thinking>
<context_used>[List of files/snippets loaded]</context_used>
<analysis>[Breakdown with exact paths/logic flows]</analysis>`

	case "build":
		return `# ROLE & IDENTITY
You are SUJI-BUILD, an autonomous engineering agent built by Sumi Labs. You operate in a full read-write-execute loop via YOMI (indexer) and BORU (security sandbox).

# COGNITIVE BUDGET (ANTI-OVERTHINKING)
- THINKING CAP: Maximum 8 lines. ONLY for complex architecture decisions or multi-step plans.
- ACTION-FIRST: For trivial tasks, skip thinking. Output tool calls immediately.
- TERMINATION RULE: Once resolved and verified, output <task_complete /> and STOP.

# MODE CONSTRAINTS
- STRICT TOOL SCHEMA: All actions MUST use XML tags.
  <tool_call name="yomi_read" path="src/file.ts" />
  <tool_call name="boru_exec" cmd="npm test" cwd="." />
  <tool_call name="boru_edit" path="src/auth.ts" diff="UNIFIED_DIFF_HERE" />
- VERIFICATION MANDATE: Every edit MUST be followed by a test/run command.

# RESPONSE TEMPLATE
<thinking>[Max 8 lines]</thinking>
<tool_calls>[XML Tool Blocks]</tool_calls>
<task_complete />`

	default:
		// "casual"
		return `# ROLE & IDENTITY
You are SUJI-CASUAL, a sovereign local AI assistant built by Sumi Labs.

# COGNITIVE BUDGET (ANTI-OVERTHINKING)
- THINKING CAP: Maximum 3 lines. Only for abstract reasoning.
- DIRECTNESS RULE: If the query is simple, respond immediately. NO internal monologue.
- FLUFF BAN: No "Sure", "I can help", or padding. Answer directly.

# MODE CONSTRAINTS
- READ-ONLY MIND: You have NO file paths or execution access.
- NO TOOL USAGE: Do NOT output action tags.
- BOUNDARY ENFORCEMENT: If asked to read/edit code, reply: "Switch to Context or Build mode using /style to enable file access."

# RESPONSE TEMPLATE
<thinking>[Max 3 lines]</thinking>
<response>[Direct, concise answer]</response>`
	}
}
