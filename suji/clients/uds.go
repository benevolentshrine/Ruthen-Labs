package clients

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"os"
	"time"
)

// ─── JSON-RPC 2.0 Types ───────────────────────────────────────────────────────

type Request struct {
	JSONRPC string `json:"jsonrpc"`
	Method  string `json:"method"`
	Params  any    `json:"params"`
	ID      int    `json:"id"`
}

type Response struct {
	JSONRPC string          `json:"jsonrpc"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *RPCError       `json:"error,omitempty"`
	ID      int             `json:"id"`
}

type RPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

func (e *RPCError) Error() string {
	return fmt.Sprintf("RPC Error %d: %s", e.Code, e.Message)
}

// ─── UDS Client ───────────────────────────────────────────────────────────────

type UDSClient struct {
	SocketPath string
	Timeout    time.Duration
}

func NewUDSClient(socketPath string) *UDSClient {
	return &UDSClient{
		SocketPath: socketPath,
		Timeout:    2 * time.Second,
	}
}

// IsAvailable checks if the socket file exists.
func (c *UDSClient) IsAvailable() bool {
	_, err := os.Stat(c.SocketPath)
	return err == nil
}

// Call executes a JSON-RPC 2.0 request over the UDS socket with retries.
func (c *UDSClient) Call(method string, params any, result any) error {
	var err error
	for i := 0; i < 2; i++ {
		err = c.callOnce(method, params, result)
		if err == nil {
			return nil
		}
		// If it's a connection error, we can retry.
		// Otherwise (like a timeout or logic error), we return immediately.
		if errors.Is(err, os.ErrNotExist) {
			return fmt.Errorf("socket not found at %s", c.SocketPath)
		}
	}
	return err
}

func (c *UDSClient) callOnce(method string, params any, result any) error {
	conn, err := net.DialTimeout("unix", c.SocketPath, c.Timeout)
	if err != nil {
		return err
	}
	defer conn.Close()

	if err := conn.SetDeadline(time.Now().Add(c.Timeout)); err != nil {
		return err
	}

	req := Request{
		JSONRPC: "2.0",
		Method:  method,
		Params:  params,
		ID:      1,
	}

	// Send request.
	data, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("encode failed: %w", err)
	}
	if _, err := conn.Write(append(data, '\n')); err != nil {
		return err
	}

	// Read response.
	scanner := bufio.NewScanner(conn)
	if !scanner.Scan() {
		if err := scanner.Err(); err != nil {
			return fmt.Errorf("read failed: %w", err)
		}
		return errors.New("empty response from socket")
	}

	var resp Response
	if err := json.Unmarshal(scanner.Bytes(), &resp); err != nil {
		return fmt.Errorf("decode failed: %w", err)
	}

	if resp.Error != nil {
		return resp.Error
	}

	if result != nil {
		if err := json.Unmarshal(resp.Result, result); err != nil {
			return fmt.Errorf("result unmarshal failed: %w", err)
		}
	}

	return nil
}
