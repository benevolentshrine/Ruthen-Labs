package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"unit01/clients"
)

// getAutoContext uses Yomi's semantic/fuzzy search to pre-fetch context based on the raw user input.
func getAutoContext(input string, ws *Workspace) string {
	if !ws.Active {
		return ""
	}

	indexer := clients.NewIndexerClient()
	records, err := indexer.Search(input)
	if err != nil || len(records) == 0 {
		// SMART FALLBACK: If user is asking for contents and search fails, fetch the directory list
		if strings.Contains(strings.ToLower(input), "content") || strings.Contains(strings.ToLower(input), "list") || strings.Contains(strings.ToLower(input), "folder") {
			list, err := indexer.List(ws.Path)
			if err == nil && list != nil && len(list.Entries) > 0 {
				var context strings.Builder
				context.WriteString("\n# DIRECTORY CONTENTS:\n")
				for _, item := range list.Entries {
					context.WriteString(fmt.Sprintf("- [%s] %s\n", item.Type, item.Name))
				}
				return context.String()
			}
		}
		return ""
	}

	var context strings.Builder
	context.WriteString("\n# IMPLICIT CONTEXT (Pre-fetched based on your request):\n")

	foundCount := 0
	for _, rec := range records {
		fullPath := rec.Path
		if !filepath.IsAbs(rec.Path) {
			fullPath = filepath.Join(ws.Path, rec.Path)
		}

		data, err := os.ReadFile(fullPath)
		if err == nil {
			foundCount++
			content := string(data)
			if len(content) > 2000 {
				content = content[:2000] + "\n... (truncated)"
			}
			context.WriteString(fmt.Sprintf("\n## File: %s\n```\n%s\n```\n", rec.Path, content))
		}
		
		if foundCount >= 3 {
			break
		}
	}

	if foundCount == 0 {
		return ""
	}

	return context.String()
}
