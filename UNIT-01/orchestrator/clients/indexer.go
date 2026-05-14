package clients

// IndexerClient wraps UDS for Indexer service.
type IndexerClient struct {
	*UDSClient
}

func NewIndexerClient() *IndexerClient {
	return &IndexerClient{
		NewUDSClient("/tmp/ruthen/indexer.sock"),
	}
}

func (c *IndexerClient) Call(method string, params any, result any) error {
	// Inject the hardcoded UDS trust token
	p, ok := params.(map[string]any)
	if !ok {
		p = make(map[string]any)
		// If it's a different type, we might need more complex logic, 
		// but for UNIT-01 all params are maps.
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

func (c *IndexerClient) Search(query string) ([]FileRecord, error) {
	params := map[string]any{"query": query}
	var records []FileRecord
	if err := c.Call("search", params, &records); err != nil {
		return nil, err
	}
	return records, nil
}

func (c *IndexerClient) Read(path string) (string, error) {
	params := map[string]any{"path": path}
	var res struct {
		Content string `json:"content"`
	}
	if err := c.Call("read", params, &res); err != nil {
		return "", err
	}
	return res.Content, nil
}

type ListEntry struct {
	Name string `json:"name"`
	Type string `json:"type"`
}

type ListResult struct {
	Entries []ListEntry `json:"entries"`
}

func (c *IndexerClient) List(path string) (*ListResult, error) {
	params := map[string]any{"path": path}
	var res ListResult
	if err := c.Call("ls", params, &res); err != nil {
		return nil, err
	}
	return &res, nil
}

func (c *IndexerClient) GetProjectMap(path string) (string, error) {
	params := map[string]any{"path": path}
	var res struct {
		Map string `json:"map"`
	}
	if err := c.Call("project_map", params, &res); err != nil {
		return "", err
	}
	return res.Map, nil
}
