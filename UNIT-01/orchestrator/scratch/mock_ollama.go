package scratch

import (
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type chunk struct {
	Model   string `json:"model"`
	Message struct {
		Role    string `json:"role"`
		Content string `json:"content"`
	} `json:"message"`
	Done bool `json:"done"`
}

func main() {
	http.HandleFunc("/api/chat", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/x-ndjson")
		flusher, _ := w.(http.Flusher)

		tokens := []string{"Hello", "!", " How", " can", " I", " help", "?"}
		for _, t := range tokens {
			c := chunk{Model: "mock"}
			c.Message.Role = "assistant"
			c.Message.Content = t
			json.NewEncoder(w).Encode(c)
			fmt.Fprint(w, "\n")
			flusher.Flush()
			time.Sleep(100 * time.Millisecond)
		}
		
		// Send done chunk
		c := chunk{Done: true}
		json.NewEncoder(w).Encode(c)
		fmt.Fprint(w, "\n")
		flusher.Flush()
	})

	http.HandleFunc("/api/tags", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintln(w, `{"models":[{"name":"mock"}]}`)
	})

	fmt.Println("Mock Ollama on :11434")
	http.ListenAndServe(":11434", nil)
}
