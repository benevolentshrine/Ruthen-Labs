package main

import (
	"errors"
	"os"
	"os/exec"
	"syscall"
	"time"
)

// ─── Daemon Manager ───────────────────────────────────────────────────────────

type DaemonStatus int

const (
	StatusNotFound DaemonStatus = iota // binary not on PATH
	StatusOffline                    // binary found, socket missing
	StatusReady                      // socket available
)

func (s DaemonStatus) Icon() string {
	switch s {
	case StatusReady:
		return "🟢"
	case StatusOffline:
		return "🔴"
	default:
		return "⚪"
	}
}

type DaemonManager struct {
	processes map[string]*os.Process
}

func NewDaemonManager() *DaemonManager {
	return &DaemonManager{
		processes: make(map[string]*os.Process),
	}
}

// SpawnIfMissing checks for the binary and socket. If binary exists but socket
// is missing, it spawns the daemon in the background.
func (d *DaemonManager) SpawnIfMissing(name, socketPath string) DaemonStatus {
	// Map common names to their absolute release paths in SumiLabs
	binPath := ""
	args := []string{}

	switch name {
	case "yomi":
		binPath = "/Users/lichi/SumiLabs/target/release/yomi"
		args = []string{"daemon", "start"}
	case "boru":
		binPath = "/Users/lichi/SumiLabs/target/release/boru"
		args = []string{"daemon"}
	default:
		p, err := exec.LookPath(name)
		if errors.Is(err, os.ErrNotExist) {
			return StatusNotFound
		}
		binPath = p
		args = []string{"--daemon"}
	}

	if _, err := os.Stat(binPath); err != nil {
		return StatusNotFound
	}

	// Check if socket already exists.
	if _, err := os.Stat(socketPath); err == nil {
		return StatusReady
	}

	// Ensure parent directory exists for the socket
	if err := os.MkdirAll("/tmp/sumi", 0755); err != nil {
		return StatusOffline
	}

	// Spawn daemon.
	cmd := exec.Command(binPath, args...)
	
	// Detach stdout/stderr.
	cmd.Stdout = nil
	cmd.Stderr = nil
	
	if err := cmd.Start(); err != nil {
		return StatusOffline
	}

	// Track process for cleanup.
	d.processes[name] = cmd.Process

	// Give it up to 5 seconds to create the socket.
	for i := 0; i < 10; i++ {
		time.Sleep(500 * time.Millisecond)
		if _, err := os.Stat(socketPath); err == nil {
			return StatusReady
		}
	}
	return StatusOffline
}

// Shutdown sends SIGTERM to all spawned daemons.
func (d *DaemonManager) Shutdown() {
	for _, p := range d.processes {
		_ = p.Signal(syscall.SIGTERM)
	}
}
