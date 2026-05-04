package main

import (
	"errors"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

// ─── Config ───────────────────────────────────────────────────────────────────

// Config holds all user-persisted preferences for Suji.
type Config struct {
	// ModelEndpoint is the base URL of the Ollama (or compatible) API.
	ModelEndpoint string `toml:"model_endpoint"`
	// DefaultStyle controls conversation tone: "casual" | "context" | "build".
	DefaultStyle string `toml:"default_style"`
	// Onboarded is set to true once the user completes (or skips) setup.
	Onboarded bool `toml:"onboarded"`
}

// defaultConfig returns a Config populated with factory defaults.
func defaultConfig() *Config {
	return &Config{
		ModelEndpoint: "http://localhost:11434",
		DefaultStyle:  "casual",
		Onboarded:     false,
	}
}

// configPath returns the canonical path to the TOML config file.
func configPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, ".config", "suji", "config.toml"), nil
}

// LoadConfig reads ~/.config/suji/config.toml.
// If the file does not exist it returns defaultConfig() with no error, so
// callers can treat a missing file as "first run".
func LoadConfig() (*Config, error) {
	path, err := configPath()
	if err != nil {
		return defaultConfig(), err
	}

	cfg := defaultConfig()

	_, err = toml.DecodeFile(path, cfg)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			// First run — config not written yet.
			return cfg, nil
		}
		return cfg, err
	}
	return cfg, nil
}

// SaveConfig writes cfg to ~/.config/suji/config.toml, creating intermediate
// directories with mode 0700 if needed.
func SaveConfig(cfg *Config) error {
	path, err := configPath()
	if err != nil {
		return err
	}

	// Ensure the directory exists.
	if err := os.MkdirAll(filepath.Dir(path), 0700); err != nil {
		return err
	}

	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()

	enc := toml.NewEncoder(f)
	return enc.Encode(cfg)
}
