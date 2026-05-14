package main

import (
	"fmt"
	"os"
	"path/filepath"
	"unit01/clients"
)

// ─── Workspace Session ────────────────────────────────────────────────────────

// Workspace tracks the active working directory and session state.
// When a user says "set workspace to X", this gets populated and
// Sandbox is told to scope operations to that path.
type Workspace struct {
	Path         string // Absolute path to the workspace directory
	SessionID    string // Sandbox session ID for rollback tracking
	ProjectMap   string // Structural overview of files/dirs
	Instructions string // Content of UNIT-01.md
	Identity     string // Content of go.mod and README.md
	Active       bool   // Whether a workspace is currently set
}

// NewWorkspace returns an empty, inactive workspace.
func NewWorkspace() *Workspace {
	return &Workspace{}
}

// Set activates a workspace at the given path.
// Returns the session ID from Sandbox, or falls back to a local marker.
func (w *Workspace) Set(path string) (string, error) {
	client := clients.NewSandboxClient()
	sessionID, err := client.SetWorkspace(path)
	if err != nil {
		// Sandbox might be offline — still set the workspace locally
		w.Path = path
		w.SessionID = "local"
		w.Active = true
		return "local", nil
	}
	w.Path = path
	w.SessionID = sessionID
	w.Active = true

	// Fetch project map from Indexer
	indexer := clients.NewIndexerClient()
	if m, err := indexer.GetProjectMap(path); err == nil {
		w.ProjectMap = m
	}

	// Fetch UNIT-01.md if exists
	if data, err := os.ReadFile(filepath.Join(path, "UNIT-01.md")); err == nil {
		w.Instructions = string(data)
	}

	// Fetch Identity (go.mod and README.md)
	identity := ""
	if data, err := os.ReadFile(filepath.Join(path, "go.mod")); err == nil {
		identity += "--- GO.MOD ---\n" + string(data) + "\n"
	}
	if data, err := os.ReadFile(filepath.Join(path, "README.md")); err == nil {
		// Truncate README if too long
		content := string(data)
		if len(content) > 1000 {
			content = content[:1000] + "... (truncated)"
		}
		identity += "--- README.MD ---\n" + content + "\n"
	}
	w.Identity = identity

	return sessionID, nil
}

// Refresh dynamically updates the project map from Indexer.
func (w *Workspace) Refresh() {
	if !w.Active {
		return
	}
	indexer := clients.NewIndexerClient()
	m, err := indexer.GetProjectMap(w.Path)
	if err != nil {
		w.ProjectMap = fmt.Sprintf("[Indexer Error: failed to fetch project map: %v]", err)
	} else if m == "" {
		w.ProjectMap = "[Indexer Error: project map returned empty string]"
	} else {
		w.ProjectMap = m
	}
}

// Clear deactivates the workspace.
func (w *Workspace) Clear() {
	w.Path = ""
	w.SessionID = ""
	w.Active = false
}

// ContextLine returns a human-readable status for the LLM system prompt.
func (w *Workspace) ContextLine() string {
	if !w.Active {
		return "No workspace set. Ask the user where to work."
	}
	res := fmt.Sprintf("Active workspace: %s (session: %s)", w.Path, w.SessionID)
	if w.ProjectMap != "" {
		res += "\n\n# PROJECT STRUCTURE:\n" + w.ProjectMap
	}
	if w.Instructions != "" {
		res += "\n\n# PROJECT INSTRUCTIONS (UNIT-01.md):\n" + w.Instructions
	}
	if w.Identity != "" {
		res += "\n\n# PROJECT IDENTITY (DNA):\n" + w.Identity
	}
	return res
}
