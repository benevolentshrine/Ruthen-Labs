package main

import (
	"fmt"
	"suji/clients"
)

// ─── Tool Execution ───────────────────────────────────────────────────────────

type ToolResult struct {
	Tool   string
	Stdout string
	Stderr string
	Error  error
}

// ExecuteTool synchronously executes a tool call using the configured sidecar daemons.
func ExecuteTool(name string, args map[string]any, cfg *Config) string {
	// Mode Enforcement
	if cfg.DefaultStyle == "casual" {
		return fmt.Sprintf("🚫 Tool [%s] Blocked: Switch to Context or Build mode to use tools.", name)
	}

	isRead := name == "yomi_read" || name == "yomi_search" || name == "search" || name == "read_file" || name == "get_file" || name == "ls"
	if cfg.DefaultStyle == "context" && !isRead {
		return fmt.Sprintf("🚫 Tool [%s] Blocked: Context mode only allows read/search tools.", name)
	}

	var stdout string
	var err error

	switch name {
	case "yomi_search", "search":
		query, _ := args["query"].(string)
		client := clients.NewYomiClient()
		var records []clients.FileRecord
		records, err = client.Search(query)
		if err == nil {
			if len(records) == 0 {
				stdout = "No matches found."
			} else {
				stdout = "Found matching files:\n"
				for _, r := range records {
					stdout += fmt.Sprintf("- %s (%s, %d bytes)\n", r.Path, r.Language, r.Size)
				}
			}
		}
	case "yomi_read", "read_file", "get_file", "read":
		path, _ := args["path"].(string)
		client := clients.NewYomiClient()
		stdout, err = client.Read(path)
	case "boru_exec", "execute":
		client := clients.NewBoruClient()
		err = client.Call("execute", args, &stdout)
	case "boru_edit", "edit":
		client := clients.NewBoruClient()
		err = client.Call("edit", args, &stdout)
	case "boru_rollback":
		client := clients.NewBoruClient()
		err = client.Call("rollback", args, &stdout)
	default:
		err = fmt.Errorf("unknown tool: %s", name)
	}

	if err != nil {
		return fmt.Sprintf("Tool [%s] failed: %v", name, err)
	}
	return stdout
}
