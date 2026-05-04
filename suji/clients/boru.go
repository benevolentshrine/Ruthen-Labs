package clients

// BoruClient wraps UDS for Boru service.
type BoruClient struct {
	*UDSClient
}

func NewBoruClient() *BoruClient {
	return &BoruClient{
		NewUDSClient("/tmp/sumi/boru.sock"),
	}
}

type SetPolicyParams struct {
	Mode string `json:"mode"`
}

func (c *BoruClient) SetPolicy(mode string) error {
	return c.Call("set_policy", SetPolicyParams{Mode: mode}, nil)
}
