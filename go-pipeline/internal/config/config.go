// Package config provides configuration management for the Go pipeline.
package config

import (
	"fmt"
	"time"

	"github.com/spf13/viper"
)

// Config represents the complete pipeline configuration
type Config struct {
	Pipeline PipelineConfig `mapstructure:"pipeline"`
	Redis    RedisConfig    `mapstructure:"redis"`
	Database DatabaseConfig `mapstructure:"database"`
	Metrics  MetricsConfig  `mapstructure:"metrics"`
	Logging  LoggingConfig  `mapstructure:"logging"`
}

// PipelineConfig contains pipeline processing configuration
type PipelineConfig struct {
	WorkerPoolSize    int           `mapstructure:"worker_pool_size"`
	BatchSize         int           `mapstructure:"batch_size"`
	BatchTimeout      time.Duration `mapstructure:"batch_timeout"`
	ConsumerGroupName string        `mapstructure:"consumer_group_name"`
	ConsumerName      string        `mapstructure:"consumer_name"`
	BufferSizes       BufferSizes   `mapstructure:"buffer_sizes"`
	Processing        ProcessingConfig `mapstructure:"processing"`
}

// BufferSizes contains channel buffer size configuration
type BufferSizes struct {
	StreamConsumer int `mapstructure:"stream_consumer"`
	WorkerPool     int `mapstructure:"worker_pool"`
	BatchProcessor int `mapstructure:"batch_processor"`
	ErrorChannel   int `mapstructure:"error_channel"`
}

// ProcessingConfig contains data processing configuration
type ProcessingConfig struct {
	MaxRetries          int           `mapstructure:"max_retries"`
	RetryBackoff        time.Duration `mapstructure:"retry_backoff"`
	TransformTimeout    time.Duration `mapstructure:"transform_timeout"`
	ValidationEnabled   bool          `mapstructure:"validation_enabled"`
	CompressionEnabled  bool          `mapstructure:"compression_enabled"`
	DeduplicationWindow time.Duration `mapstructure:"deduplication_window"`
}

// RedisConfig contains Redis connection and streaming configuration
type RedisConfig struct {
	URL               string        `mapstructure:"url"`
	StreamNames       []string      `mapstructure:"stream_names"`
	ReadCount         int           `mapstructure:"read_count"`
	BlockTimeout      time.Duration `mapstructure:"block_timeout"`
	ConnectionPool    PoolConfig    `mapstructure:"connection_pool"`
	ClusterMode       bool          `mapstructure:"cluster_mode"`
	ClusterNodes      []string      `mapstructure:"cluster_nodes"`
	TLS               TLSConfig     `mapstructure:"tls"`
	Authentication    AuthConfig    `mapstructure:"authentication"`
}

// PoolConfig contains connection pool configuration
type PoolConfig struct {
	MaxIdle     int           `mapstructure:"max_idle"`
	MaxActive   int           `mapstructure:"max_active"`
	IdleTimeout time.Duration `mapstructure:"idle_timeout"`
	Wait        bool          `mapstructure:"wait"`
}

// TLSConfig contains TLS configuration
type TLSConfig struct {
	Enabled            bool   `mapstructure:"enabled"`
	CertFile           string `mapstructure:"cert_file"`
	KeyFile            string `mapstructure:"key_file"`
	CAFile             string `mapstructure:"ca_file"`
	InsecureSkipVerify bool   `mapstructure:"insecure_skip_verify"`
}

// AuthConfig contains authentication configuration
type AuthConfig struct {
	Username string `mapstructure:"username"`
	Password string `mapstructure:"password"`
}

// DatabaseConfig contains PostgreSQL database configuration
type DatabaseConfig struct {
	URL              string        `mapstructure:"url"`
	MaxOpenConns     int           `mapstructure:"max_open_conns"`
	MaxIdleConns     int           `mapstructure:"max_idle_conns"`
	ConnMaxLifetime  time.Duration `mapstructure:"conn_max_lifetime"`
	ConnMaxIdleTime  time.Duration `mapstructure:"conn_max_idle_time"`
	Tables           TablesConfig  `mapstructure:"tables"`
	Migrations       MigrationsConfig `mapstructure:"migrations"`
	Partitioning     PartitioningConfig `mapstructure:"partitioning"`
}

// TablesConfig contains table-specific configuration
type TablesConfig struct {
	Accounts     TableConfig `mapstructure:"accounts"`
	Transactions TableConfig `mapstructure:"transactions"`
	Blocks       TableConfig `mapstructure:"blocks"`
	TradeEvents  TableConfig `mapstructure:"trade_events"`
}

// TableConfig contains individual table configuration
type TableConfig struct {
	Name            string   `mapstructure:"name"`
	PartitionColumn string   `mapstructure:"partition_column"`
	IndexColumns    []string `mapstructure:"index_columns"`
	CompressionType string   `mapstructure:"compression_type"`
	RetentionDays   int      `mapstructure:"retention_days"`
}

// MigrationsConfig contains database migration configuration
type MigrationsConfig struct {
	Enabled       bool   `mapstructure:"enabled"`
	MigrationsDir string `mapstructure:"migrations_dir"`
	AutoMigrate   bool   `mapstructure:"auto_migrate"`
}

// PartitioningConfig contains table partitioning configuration
type PartitioningConfig struct {
	Enabled        bool          `mapstructure:"enabled"`
	Strategy       string        `mapstructure:"strategy"` // daily, weekly, monthly
	RetentionDays  int           `mapstructure:"retention_days"`
	MaintenanceSchedule string   `mapstructure:"maintenance_schedule"`
}

// MetricsConfig contains metrics and monitoring configuration
type MetricsConfig struct {
	Enabled    bool          `mapstructure:"enabled"`
	Port       int           `mapstructure:"port"`
	Path       string        `mapstructure:"path"`
	Namespace  string        `mapstructure:"namespace"`
	Prometheus PrometheusConfig `mapstructure:"prometheus"`
	Custom     CustomMetricsConfig `mapstructure:"custom"`
}

// PrometheusConfig contains Prometheus-specific configuration
type PrometheusConfig struct {
	PushGateway PushGatewayConfig `mapstructure:"push_gateway"`
	Labels      map[string]string `mapstructure:"labels"`
}

// PushGatewayConfig contains Prometheus Push Gateway configuration
type PushGatewayConfig struct {
	Enabled  bool          `mapstructure:"enabled"`
	URL      string        `mapstructure:"url"`
	Job      string        `mapstructure:"job"`
	Interval time.Duration `mapstructure:"interval"`
}

// CustomMetricsConfig contains custom metrics configuration
type CustomMetricsConfig struct {
	Collectors []MetricCollectorConfig `mapstructure:"collectors"`
}

// MetricCollectorConfig contains individual metric collector configuration
type MetricCollectorConfig struct {
	Name        string                 `mapstructure:"name"`
	Type        string                 `mapstructure:"type"`
	Description string                 `mapstructure:"description"`
	Labels      []string               `mapstructure:"labels"`
	Config      map[string]interface{} `mapstructure:"config"`
}

// LoggingConfig contains logging configuration
type LoggingConfig struct {
	Level      string          `mapstructure:"level"`
	Format     string          `mapstructure:"format"` // json, console
	Output     []string        `mapstructure:"output"` // stdout, stderr, file
	File       FileLogConfig   `mapstructure:"file"`
	Structured bool            `mapstructure:"structured"`
	Sampling   SamplingConfig  `mapstructure:"sampling"`
}

// FileLogConfig contains file logging configuration
type FileLogConfig struct {
	Path       string `mapstructure:"path"`
	MaxSize    int    `mapstructure:"max_size"`    // megabytes
	MaxBackups int    `mapstructure:"max_backups"`
	MaxAge     int    `mapstructure:"max_age"`     // days
	Compress   bool   `mapstructure:"compress"`
}

// SamplingConfig contains log sampling configuration
type SamplingConfig struct {
	Enabled bool `mapstructure:"enabled"`
	Initial int  `mapstructure:"initial"`
	Thereafter int `mapstructure:"thereafter"`
}

// Load loads configuration from file
func Load(configPath string) (*Config, error) {
	viper.SetConfigFile(configPath)
	
	// Set defaults
	setDefaults()
	
	// Environment variable support
	viper.AutomaticEnv()
	viper.SetEnvPrefix("PIPELINE")
	
	if err := viper.ReadInConfig(); err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}
	
	var config Config
	if err := viper.Unmarshal(&config); err != nil {
		return nil, fmt.Errorf("failed to unmarshal config: %w", err)
	}
	
	// Validate configuration
	if err := validateConfig(&config); err != nil {
		return nil, fmt.Errorf("invalid configuration: %w", err)
	}
	
	return &config, nil
}

// setDefaults sets default configuration values
func setDefaults() {
	// Pipeline defaults
	viper.SetDefault("pipeline.worker_pool_size", 16)
	viper.SetDefault("pipeline.batch_size", 1000)
	viper.SetDefault("pipeline.batch_timeout", "1s")
	viper.SetDefault("pipeline.consumer_group_name", "trading_pipeline")
	viper.SetDefault("pipeline.consumer_name", "consumer_1")
	
	// Buffer size defaults
	viper.SetDefault("pipeline.buffer_sizes.stream_consumer", 10000)
	viper.SetDefault("pipeline.buffer_sizes.worker_pool", 10000)
	viper.SetDefault("pipeline.buffer_sizes.batch_processor", 5000)
	viper.SetDefault("pipeline.buffer_sizes.error_channel", 1000)
	
	// Processing defaults
	viper.SetDefault("pipeline.processing.max_retries", 3)
	viper.SetDefault("pipeline.processing.retry_backoff", "1s")
	viper.SetDefault("pipeline.processing.transform_timeout", "5s")
	viper.SetDefault("pipeline.processing.validation_enabled", true)
	viper.SetDefault("pipeline.processing.compression_enabled", false)
	viper.SetDefault("pipeline.processing.deduplication_window", "5m")
	
	// Redis defaults
	viper.SetDefault("redis.url", "redis://localhost:6379")
	viper.SetDefault("redis.stream_names", []string{"solana_transactions", "solana_accounts"})
	viper.SetDefault("redis.read_count", 100)
	viper.SetDefault("redis.block_timeout", "1s")
	viper.SetDefault("redis.connection_pool.max_idle", 10)
	viper.SetDefault("redis.connection_pool.max_active", 100)
	viper.SetDefault("redis.connection_pool.idle_timeout", "300s")
	viper.SetDefault("redis.connection_pool.wait", true)
	
	// Database defaults
	viper.SetDefault("database.max_open_conns", 25)
	viper.SetDefault("database.max_idle_conns", 10)
	viper.SetDefault("database.conn_max_lifetime", "1h")
	viper.SetDefault("database.conn_max_idle_time", "15m")
	
	// Table defaults
	viper.SetDefault("database.tables.accounts.name", "solana_accounts")
	viper.SetDefault("database.tables.transactions.name", "solana_transactions")
	viper.SetDefault("database.tables.blocks.name", "solana_blocks")
	viper.SetDefault("database.tables.trade_events.name", "trade_events")
	
	// Partitioning defaults
	viper.SetDefault("database.partitioning.enabled", true)
	viper.SetDefault("database.partitioning.strategy", "daily")
	viper.SetDefault("database.partitioning.retention_days", 30)
	
	// Metrics defaults
	viper.SetDefault("metrics.enabled", true)
	viper.SetDefault("metrics.port", 8080)
	viper.SetDefault("metrics.path", "/metrics")
	viper.SetDefault("metrics.namespace", "pipeline")
	
	// Logging defaults
	viper.SetDefault("logging.level", "info")
	viper.SetDefault("logging.format", "json")
	viper.SetDefault("logging.output", []string{"stdout"})
	viper.SetDefault("logging.structured", true)
}

// validateConfig validates the configuration
func validateConfig(config *Config) error {
	// Validate pipeline configuration
	if config.Pipeline.WorkerPoolSize <= 0 {
		return fmt.Errorf("pipeline.worker_pool_size must be positive")
	}
	
	if config.Pipeline.BatchSize <= 0 {
		return fmt.Errorf("pipeline.batch_size must be positive")
	}
	
	if config.Pipeline.BatchTimeout <= 0 {
		return fmt.Errorf("pipeline.batch_timeout must be positive")
	}
	
	// Validate Redis configuration
	if len(config.Redis.StreamNames) == 0 {
		return fmt.Errorf("redis.stream_names cannot be empty")
	}
	
	// Validate database configuration
	if config.Database.URL == "" {
		return fmt.Errorf("database.url is required")
	}
	
	return nil
}