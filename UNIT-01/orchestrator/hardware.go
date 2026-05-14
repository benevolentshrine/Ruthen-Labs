package main

import (
	"os/exec"
	"runtime"
	"strconv"
	"strings"
)

// HardwareTier represents the detected system RAM capability
type HardwareTier string

const (
	Tier8GB  HardwareTier = "8GB"
	Tier16GB HardwareTier = "16GB"
	Tier32GB HardwareTier = "32GB+"
)

func GetHardwareTier() (HardwareTier, int) {
	ramGB := getSystemRAMGB()
	
	if ramGB >= 32 {
		return Tier32GB, ramGB
	} else if ramGB >= 16 {
		return Tier16GB, ramGB
	}
	return Tier8GB, ramGB
}

func getSystemRAMGB() int {
	switch runtime.GOOS {
	case "darwin":
		out, err := exec.Command("sysctl", "-n", "hw.memsize").Output()
		if err == nil {
			bytes, err := strconv.ParseUint(strings.TrimSpace(string(out)), 10, 64)
			if err == nil {
				return int(bytes / (1024 * 1024 * 1024))
			}
		}
	case "linux":
		// Read /proc/meminfo
		out, err := exec.Command("grep", "MemTotal", "/proc/meminfo").Output()
		if err == nil {
			// output looks like: MemTotal:       16393452 kB
			fields := strings.Fields(string(out))
			if len(fields) >= 2 {
				kb, err := strconv.ParseUint(fields[1], 10, 64)
				if err == nil {
					return int(kb / (1024 * 1024))
				}
			}
		}
	}
	
	// Fallback if detection fails
	return 16 
}
