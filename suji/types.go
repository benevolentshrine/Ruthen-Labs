package main

import "time"

// ─── Domain Types ─────────────────────────────────────────────────────────────

// Message represents a single chat turn stored in History.
// Role is "user" or "assistant".
type Message struct {
	Role      string
	Content   string
	Timestamp time.Time
}
