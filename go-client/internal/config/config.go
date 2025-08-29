// Package config handles configuration for the Go client
package config

import (
	"fmt"
	"os"
	"time"

	"gopkg.in/yaml.v3"
)

// Config represents the client configuration
type Config struct {
	// Stream server configuration
	Server ServerConfig `yaml:"server"`
	
	// Processing configuration
	Processing ProcessingConfig `yaml:"processing"`
	
	// Logging configuration
	Logging LoggingConfig `yaml:"logging"`
	
	// Filters for data processing
	Filters FilterConfig `yaml:"filters"`
}

// ServerConfig contains server connection settings
type ServerConfig struct {
	// Base URL of the stream processor
	BaseURL string `yaml:"base_url"`
	
	// Timeout for HTTP requests
	Timeout time.Duration `yaml:"timeout"`
	
	// Maximum number of retry attempts
	MaxRetries int `yaml:"max_retries"`
	
	// Reconnection settings for SSE
	Reconnect ReconnectConfig `yaml:"reconnect"`
}

// ReconnectConfig contains reconnection settings
type ReconnectConfig struct {
	// Initial delay before reconnecting
	InitialDelay time.Duration `yaml:"initial_delay"`
	
	// Maximum delay between reconnections
	MaxDelay time.Duration `yaml:"max_delay"`
	
	// Backoff multiplier
	BackoffMultiplier float64 `yaml:"backoff_multiplier"`
	
	// Maximum number of reconnection attempts (0 = infinite)
	MaxAttempts int `yaml:"max_attempts"`
}

// ProcessingConfig contains data processing settings
type ProcessingConfig struct {
	// Buffer size for incoming messages
	BufferSize int `yaml:"buffer_size"`
	
	// Number of worker goroutines
	Workers int `yaml:"workers"`
	
	// Batch size for processing
	BatchSize int `yaml:"batch_size"`
	
	// Batch timeout
	BatchTimeout time.Duration `yaml:"batch_timeout"`
}

// LoggingConfig contains logging settings
type LoggingConfig struct {
	// Log level (debug, info, warn, error)
	Level string `yaml:"level"`
	
	// Log format (text, json)
	Format string `yaml:"format"`
	
	// Enable timestamp in logs
	Timestamp bool `yaml:"timestamp"`
}

// FilterConfig contains data filtering settings
type FilterConfig struct {
	// Only process trading instructions
	TradingOnly bool `yaml:"trading_only"`
	
	// Specific programs to monitor
	Programs []string `yaml:"programs"`
	
	// Specific token mints to monitor
	TokenMints []string `yaml:"token_mints"`
	
	// Instruction types to process
	InstructionTypes []string `yaml:"instruction_types"`
}

// DefaultConfig returns a configuration with sensible defaults
func DefaultConfig() *Config {
	return &Config{
		Server: ServerConfig{
			BaseURL:    "http://localhost:8080",
			Timeout:    30 * time.Second,
			MaxRetries: 3,
			Reconnect: ReconnectConfig{
				InitialDelay:      1 * time.Second,
				MaxDelay:          60 * time.Second,
				BackoffMultiplier: 2.0,
				MaxAttempts:       0, // Infinite
			},
		},
		Processing: ProcessingConfig{
			BufferSize:   1000,
			Workers:      4,
			BatchSize:    50,
			BatchTimeout: 5 * time.Second,
		},
		Logging: LoggingConfig{
			Level:     "info",
			Format:    "text",
			Timestamp: true,
		},
		Filters: FilterConfig{
			TradingOnly: true,
			Programs:    []string{},
			TokenMints:  []string{},
			InstructionTypes: []string{},
		},
	}
}

// LoadFromFile loads configuration from a YAML file
func LoadFromFile(filename string) (*Config, error) {
	data, err := os.ReadFile(filename)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	config := DefaultConfig()
	if err := yaml.Unmarshal(data, config); err != nil {
		return nil, fmt.Errorf("failed to parse config file: %w", err)
	}

	return config, nil
}

// LoadFromEnv loads configuration from environment variables
func LoadFromEnv() *Config {
	config := DefaultConfig()

	if baseURL := os.Getenv("STREAM_BASE_URL"); baseURL != "" {
		config.Server.BaseURL = baseURL
	}

	if level := os.Getenv("LOG_LEVEL"); level != "" {
		config.Logging.Level = level
	}

	if format := os.Getenv("LOG_FORMAT"); format != "" {
		config.Logging.Format = format
	}

	return config
}

// Validate validates the configuration
func (c *Config) Validate() error {
	if c.Server.BaseURL == "" {
		return fmt.Errorf("server base URL cannot be empty")
	}

	if c.Server.Timeout <= 0 {
		return fmt.Errorf("server timeout must be positive")
	}

	if c.Processing.BufferSize <= 0 {
		return fmt.Errorf("buffer size must be positive")
	}

	if c.Processing.Workers <= 0 {
		return fmt.Errorf("number of workers must be positive")
	}

	validLogLevels := map[string]bool{
		"debug": true,
		"info":  true,
		"warn":  true,
		"error": true,
	}

	if !validLogLevels[c.Logging.Level] {
		return fmt.Errorf("invalid log level: %s", c.Logging.Level)
	}

	validLogFormats := map[string]bool{
		"text": true,
		"json": true,
	}

	if !validLogFormats[c.Logging.Format] {
		return fmt.Errorf("invalid log format: %s", c.Logging.Format)
	}

	return nil
}

// SaveToFile saves the configuration to a YAML file
func (c *Config) SaveToFile(filename string) error {
	data, err := yaml.Marshal(c)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	if err := os.WriteFile(filename, data, 0644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}