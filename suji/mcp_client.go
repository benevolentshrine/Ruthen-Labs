package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os/exec"
)

// ─── MCP Stdio Client ─────────────────────────────────────────────────────────

type MCPClient struct {
	cmd    *exec.Cmd
	stdin  io.WriteCloser
	stdout *bufio.Scanner
}

func NewMCPClient(command string, args []string) (*MCPClient, error) {
	cmd := exec.Command(command, args...)
	
	stdin, err := cmd.StdinPipe()
	if err != nil {
		return nil, err
	}
	
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, err
	}

	if err := cmd.Start(); err != nil {
		return nil, err
	}

	return &MCPClient{
		cmd:    cmd,
		stdin:  stdin,
		stdout: bufio.NewScanner(stdout),
	}, nil
}

func (c *MCPClient) Call(method string, params any) (string, error) {
	req := map[string]any{
		"jsonrpc": "2.0",
		"method":  method,
		"params":  params,
		"id":      1,
	}
	
	data, _ := json.Marshal(req)
	_, err := fmt.Fprintln(c.stdin, string(data))
	if err != nil {
		return "", err
	}

	if !c.stdout.Scan() {
		return "", fmt.Errorf("mcp: connection closed")
	}

	return c.stdout.Text(), nil
}

func (c *MCPClient) Close() {
	if c.cmd != nil && c.cmd.Process != nil {
		_ = c.cmd.Process.Kill()
	}
}
