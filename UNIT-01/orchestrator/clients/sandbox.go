package clients

// SandboxClient wraps UDS for Sandbox service.
type SandboxClient struct {
	*UDSClient
}

func NewSandboxClient() *SandboxClient {
	return &SandboxClient{
		NewUDSClient("/tmp/ruthen/sandbox.sock"),
	}
}

type SetPolicyParams struct {
	Mode string `json:"mode"`
}

func (c *SandboxClient) SetPolicy(mode string) error {
	return c.Call("set_policy", SetPolicyParams{Mode: mode}, nil)
}

func (c *SandboxClient) Execute(cmd string) (string, error) {
	params := map[string]any{"cmd": cmd, "cwd": "."}
	var res struct {
		Verdict string `json:"verdict"`
		Stdout  string `json:"stdout"`
	}
	if err := c.Call("execute", params, &res); err != nil {
		return "", err
	}
	if res.Stdout != "" {
		return res.Stdout, nil
	}
	return res.Verdict, nil
}

func (c *SandboxClient) Write(path, content string) (string, error) {
	params := map[string]any{
		"path":    path,
		"content": content,
	}
	var res struct {
		Stdout string `json:"stdout"`
	}
	if err := c.Call("write", params, &res); err != nil {
		return "", err
	}
	return res.Stdout, nil
}

func (c *SandboxClient) Patch(path, target, replacement string) (string, error) {
	params := map[string]any{
		"path":        path,
		"target":      target,
		"replacement": replacement,
	}
	var res struct {
		Stdout string `json:"stdout"`
	}
	if err := c.Call("patch", params, &res); err != nil {
		return "", err
	}
	return res.Stdout, nil
}

// SetWorkspace tells Sandbox to scope all file operations to the given directory.
// This also creates a new rollback session for the workspace.
func (c *SandboxClient) SetWorkspace(path string) (string, error) {
	params := map[string]any{"path": path}
	var res struct {
		Verdict  string `json:"verdict"`
		AuditRef string `json:"audit_ref"`
	}
	if err := c.Call("set_workspace", params, &res); err != nil {
		return "", err
	}
	return res.AuditRef, nil // Returns session ID
}

// Rollback restores all files modified in the given session.
// Pass "latest" or "" to rollback the current session.
func (c *SandboxClient) Rollback(sessionID string) (string, error) {
	params := map[string]any{}
	if sessionID != "" {
		params["session_id"] = sessionID
	}
	var res struct {
		Verdict string `json:"verdict"`
	}
	if err := c.Call("rollback", params, &res); err != nil {
		return "", err
	}
	return res.Verdict, nil
}

func (c *SandboxClient) Delete(path string) (string, error) {
	params := map[string]any{"path": path}
	var res struct {
		Verdict string `json:"verdict"`
	}
	if err := c.Call("delete", params, &res); err != nil {
		return "", err
	}
	return res.Verdict, nil
}
