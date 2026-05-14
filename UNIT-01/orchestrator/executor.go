package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"unit01/clients"

	"github.com/charmbracelet/lipgloss"
)

// ─── Tool Execution ───────────────────────────────────────────────────────────

// ExecuteTool synchronously executes a tool call using the configured sidecar daemons.
func ExecuteTool(name string, args map[string]any, cfg *Config, ws *Workspace) string {
	// 3. Dispatch to Daemons
	var stdout string
	var err error

	// Pre-process workspace path for relative paths
	if ws.Active {
		for k, v := range args {
			if s, ok := v.(string); ok && !filepath.IsAbs(s) && (k == "path" || k == "cwd") {
				args[k] = filepath.Join(ws.Path, s)
			}
		}
	}

	switch name {
	case "set_workspace":
		path, _ := args["path"].(string)
		if path == "" {
			return "❌ ERROR: Missing 'path' parameter for set_workspace."
		}

		// Ask for permission to set workspace
		fmt.Printf("\n"+lipgloss.NewStyle().Foreground(BrandColor).Bold(true).Render("◆ SET WORKSPACE")+"(%s)\n", path)
		if !ShowDeleteConfirm(path, "SET WORKSPACE") {
			return "❌ Workspace change denied by user."
		}

		sessionID, err := ws.Set(path)
		if err != nil {
			return fmt.Sprintf("❌ ERROR: Failed to set workspace: %v", err)
		}
		return fmt.Sprintf("✅ Workspace set to %s. Session ID: %s. You now have permission to work in this directory.", path, sessionID)
	case "indexer_ls", "list_files":
		path, ok := args["path"].(string)
		if !ok {
			path, _ = args["param"].(string)
		}
		client := clients.NewIndexerClient()
		result, callErr := client.List(path)
		if callErr != nil {
			// Fallback: read directory directly if Indexer is offline
			entries, osErr := os.ReadDir(path)
			if osErr != nil {
				err = osErr
			} else {
				stdout = fmt.Sprintf("Contents of %s:\n", path)
				for _, e := range entries {
					prefix := "  f "
					if e.IsDir() {
						prefix = "  d "
					}
					stdout += fmt.Sprintf("%s %s\n", prefix, e.Name())
				}
			}
		} else {
			stdout = fmt.Sprintf("Contents of %s:\n", path)
			for _, e := range result.Entries {
				prefix := "  f "
				if e.Type == "dir" {
					prefix = "  d "
				}
				stdout += fmt.Sprintf("%s %s\n", prefix, e.Name)
			}
		}
	case "indexer_read":
		path, _ := args["path"].(string)
		client := clients.NewIndexerClient()
		stdout, err = client.Read(path)
		if err != nil {
			// Fallback: read file directly if Indexer is offline
			data, osErr := os.ReadFile(filepath.Clean(path))
			if osErr != nil {
				err = osErr
			} else {
				stdout = string(data)
				err = nil
			}
		}
	case "sandbox_exec", "execute":
		cmd, _ := args["command"].(string)
		client := clients.NewSandboxClient()
		stdout, err = client.Execute(cmd)
	case "sandbox_write":
		path, _ := args["path"].(string)
		content, _ := args["content"].(string)
		content = strings.ReplaceAll(content, "\\n", "\n")

		lang := DetectLanguage(path)
		review := ShowCodePreview(path, content, lang)

		switch review.Action {
		case ReviewApprove:
			client := clients.NewSandboxClient()
			_, err = client.Write(path, content)
			if err == nil {
				if _, statErr := os.Stat(path); statErr == nil {
					return fmt.Sprintf("✅ SUCCESS: File written and verified at %s", path)
				}
				return fmt.Sprintf("❌ ERROR: Sandbox reported success, but the file is missing from %s.", path)
			}
		case ReviewSuggest:
			return fmt.Sprintf("❌ USER SUGGESTED CHANGES: %s\n\nPlease revise the code according to this feedback.", review.Suggestion)
		case ReviewReject:
			return "❌ Write Denied by User. The code was not written to disk."
		}
	case "sandbox_patch":
		path, _ := args["path"].(string)
		target, _ := args["target"].(string)
		replacement, _ := args["replacement"].(string)
		target = strings.ReplaceAll(target, "\\n", "\n")
		replacement = strings.ReplaceAll(replacement, "\\n", "\n")

		// For patch, we show what's being replaced and what it's replaced with
		patchPreview := fmt.Sprintf("--- OLD TEXT ---\n%s\n\n+++ NEW TEXT +++\n%s", target, replacement)
		review := ShowCodePreview(path, patchPreview, DetectLanguage(path))

		switch review.Action {
		case ReviewApprove:
			client := clients.NewSandboxClient()
			stdout, err = client.Patch(path, target, replacement)
			if err == nil {
				return fmt.Sprintf("✅ SUCCESS: Patch applied to %s", path)
			}
		case ReviewSuggest:
			return fmt.Sprintf("❌ USER SUGGESTED CHANGES: %s\n\nPlease revise the patch according to this feedback.", review.Suggestion)
		case ReviewReject:
			return "❌ Patch Denied by User."
		}
	case "sandbox_delete":
		path, _ := args["path"].(string)
		if !ShowDeleteConfirm(path, "DELETE FILE") {
			return "❌ Delete Denied by User."
		}
		client := clients.NewSandboxClient()
		stdout, err = client.Delete(path)
		if err == nil {
			return fmt.Sprintf("✅ SUCCESS: %s", stdout)
		}
	case "sandbox_rollback":
		client := clients.NewSandboxClient()
		stdout, err = client.Rollback("latest")
		if err == nil {
			return fmt.Sprintf("✅ SUCCESS: %s", stdout)
		}

	default:
		return fmt.Sprintf("⚠️ Tool [%s] not recognized by UNIT-01.", name)
	}

	if err != nil {
		return fmt.Sprintf("Tool [%s] failed: %v", name, err)
	}

	return stdout
}
