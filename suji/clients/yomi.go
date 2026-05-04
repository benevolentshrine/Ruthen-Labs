package clients

// YomiClient wraps UDS for Yomi service.
type YomiClient struct {
	*UDSClient
}

func NewYomiClient() *YomiClient {
	return &YomiClient{
		NewUDSClient("/tmp/sumi/yomi.sock"),
	}
}

func (c *YomiClient) Call(method string, params any, result any) error {
	// Inject the hardcoded UDS trust token
	p, ok := params.(map[string]any)
	if !ok {
		p = make(map[string]any)
		// If it's a different type, we might need more complex logic, 
		// but for Suji all params are maps.
	}
	p["token"] = "uds-internal-trust"
	return c.UDSClient.Call(method, p, result)
}

type FileRecord struct {
	Path         string `json:"path"`
	Language     string `json:"language"`
	LastModified string `json:"last_modified"`
	Size         uint64 `json:"size"`
}

func (c *YomiClient) Search(query string) ([]FileRecord, error) {
	params := map[string]any{"query": query}
	var records []FileRecord
	if err := c.Call("search", params, &records); err != nil {
		return nil, err
	}
	return records, nil
}

func (c *YomiClient) Read(path string) (string, error) {
	params := map[string]any{"path": path}
	var res struct {
		Content string `json:"content"`
	}
	if err := c.Call("read", params, &res); err != nil {
		return "", err
	}
	return res.Content, nil
}
