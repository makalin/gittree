package config

import (
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

// Config represents the application configuration
type Config struct {
	Style            string            `yaml:"style"`            // light | dark | auto
	Unicode          bool              `yaml:"unicode"`
	NoColor          bool              `yaml:"no_color"`
	DateFormat       string            `yaml:"dateFormat"`
	ConfirmDangerous bool              `yaml:"confirmDangerous"`
	Paging           string            `yaml:"paging"` // auto | always | never
	Colors           map[string]string `yaml:"colors"`
	Git              GitConfig         `yaml:"git"`
}

// GitConfig represents git-specific configuration
type GitConfig struct {
	DefaultRange string   `yaml:"defaultRange"`
	ExtraArgs    []string `yaml:"extraArgs"`
}

// Default returns the default configuration
func Default() *Config {
	return &Config{
		Style:            "auto",
		Unicode:          false,
		NoColor:          false,
		DateFormat:       "2006-01-02 15:04",
		ConfirmDangerous: true,
		Paging:           "auto",
		Colors: map[string]string{
			"graph1": "blue",
			"graph2": "magenta",
			"head":   "cyan",
		},
		Git: GitConfig{
			DefaultRange: "",
			ExtraArgs:    []string{},
		},
	}
}

// Load loads configuration from the default location
func Load() (*Config, error) {
	configPath, err := getConfigPath()
	if err != nil {
		return nil, err
	}

	// If config file doesn't exist, return default
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		return Default(), nil
	}

	data, err := os.ReadFile(configPath)
	if err != nil {
		return nil, err
	}

	var config Config
	if err := yaml.Unmarshal(data, &config); err != nil {
		return nil, err
	}

	// Merge with defaults for missing fields
	defaultConfig := Default()
	if config.Style == "" {
		config.Style = defaultConfig.Style
	}
	if config.DateFormat == "" {
		config.DateFormat = defaultConfig.DateFormat
	}
	if config.Paging == "" {
		config.Paging = defaultConfig.Paging
	}
	if config.Colors == nil {
		config.Colors = defaultConfig.Colors
	}

	return &config, nil
}

// Save saves the configuration to the default location
func (c *Config) Save() error {
	configPath, err := getConfigPath()
	if err != nil {
		return err
	}

	// Create config directory if it doesn't exist
	configDir := filepath.Dir(configPath)
	if err := os.MkdirAll(configDir, 0755); err != nil {
		return err
	}

	data, err := yaml.Marshal(c)
	if err != nil {
		return err
	}

	return os.WriteFile(configPath, data, 0644)
}

// getConfigPath returns the path to the configuration file
func getConfigPath() (string, error) {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(homeDir, ".config", "gittree", "config.yml"), nil
}