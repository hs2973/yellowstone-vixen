//! Configuration module for Chainstack gRPC source
//!
//! This module provides comprehensive configuration management for connecting to
//! Chainstack's Yellowstone gRPC service, including authentication, connection pooling,
//! circuit breaker patterns, and Redis streaming integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yellowstone_vixen::config::YellowstoneConfig;

/// Environment-specific configuration for different Chainstack deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name (dev, staging, prod)
    pub name: String,
    /// Chainstack endpoint for this environment
    pub endpoint: String,
    /// API key for this environment
    pub api_key: String,
    /// Environment-specific custom headers
    pub custom_headers: HashMap<String, String>,
}

/// Complete configuration for Chainstack Yellowstone gRPC integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainstackVixenConfig {
    /// Chainstack-specific configuration
    pub chainstack: ChainstackEnvironmentConfig,
    /// Redis configuration for streaming
    pub redis: Option<RedisStreamConfig>,
    /// Buffer configuration for high-throughput processing
    pub buffer: BufferConfig,
    /// Monitoring and metrics configuration
    pub monitoring: MonitoringConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

/// Multi-environment Chainstack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainstackEnvironmentConfig {
    /// Current active environment
    pub active_environment: String,
    /// Available environments
    pub environments: HashMap<String, EnvironmentConfig>,
    /// Global connection settings
    pub connection_pool: ConnectionPoolConfig,
    /// Circuit breaker settings
    pub circuit_breaker: CircuitBreakerConfig,
    /// Retry policy configuration
    pub retry_policy: RetryPolicyConfig,
}

/// Redis streaming configuration for pipeline integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisStreamConfig {
    /// Redis cluster configuration
    pub cluster: RedisClusterConfig,
    /// Stream configuration
    pub streams: HashMap<String, StreamConfig>,
    /// Consumer group configuration
    pub consumer_groups: HashMap<String, ConsumerGroupConfig>,
}

/// Redis cluster configuration for high availability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisClusterConfig {
    /// Redis nodes (for cluster mode)
    pub nodes: Vec<String>,
    /// Single Redis URL (for non-cluster mode)
    pub url: Option<String>,
    /// Connection pool size
    pub pool_size: usize,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Command timeout in milliseconds
    pub command_timeout_ms: u64,
}

/// Individual stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Stream name
    pub name: String,
    /// Maximum stream length
    pub max_length: Option<u64>,
    /// Data retention period in seconds
    pub retention_seconds: Option<u64>,
    /// Partitioning strategy
    pub partition_strategy: PartitionStrategy,
}

/// Consumer group configuration for parallel processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerGroupConfig {
    /// Consumer group name
    pub group_name: String,
    /// Number of consumers in the group
    pub consumer_count: usize,
    /// Block time for XREADGROUP in milliseconds
    pub block_ms: u64,
    /// Maximum pending messages per consumer
    pub max_pending: usize,
}

/// Stream partitioning strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStrategy {
    /// Single stream for all data
    Single,
    /// Partition by account
    ByAccount,
    /// Partition by program ID
    ByProgram,
    /// Partition by transaction signature
    BySignature,
    /// Custom partitioning function
    Custom(String),
}

/// Buffer configuration for high-throughput data processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// Channel buffer sizes
    pub channel_sizes: ChannelSizeConfig,
    /// Worker pool configuration
    pub worker_pools: WorkerPoolConfig,
    /// Batch processing configuration
    pub batching: BatchConfig,
}

/// Channel buffer size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSizeConfig {
    /// Source to parser channel size
    pub source_to_parser: usize,
    /// Parser to handler channel size
    pub parser_to_handler: usize,
    /// Handler to Redis channel size
    pub handler_to_redis: usize,
    /// Error handling channel size
    pub error_channel: usize,
}

/// Worker pool configuration for different processing stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolConfig {
    /// Number of parser workers
    pub parser_workers: usize,
    /// Number of handler workers
    pub handler_workers: usize,
    /// Number of Redis writer workers
    pub redis_workers: usize,
    /// Worker task timeout in seconds
    pub worker_timeout_secs: u64,
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum batch size for Redis writes
    pub redis_batch_size: usize,
    /// Maximum batch wait time in milliseconds
    pub batch_timeout_ms: u64,
    /// Memory pressure threshold for immediate flush
    pub memory_pressure_threshold: usize,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of concurrent connections per filter
    pub max_connections_per_filter: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Keep-alive interval in seconds
    pub keep_alive_interval_secs: u64,
    /// Maximum idle time before connection cleanup
    pub max_idle_secs: u64,
}

/// Circuit breaker configuration for resilient connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Half-open timeout in seconds
    pub half_open_timeout_secs: u64,
    /// Success threshold to close circuit
    pub success_threshold: u32,
    /// Maximum wait time in open state
    pub max_open_wait_secs: u64,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff delay in milliseconds
    pub max_backoff_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 to 1.0)
    pub jitter_factor: f64,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics export configuration
    pub metrics: MetricsConfig,
    /// Tracing configuration
    pub tracing: TracingConfig,
    /// Health check configuration
    pub health_checks: HealthCheckConfig,
}

/// Metrics collection and export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Prometheus metrics configuration
    pub prometheus: Option<PrometheusConfig>,
    /// Custom metrics backends
    pub custom_backends: Vec<CustomMetricsBackend>,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
}

/// Prometheus metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    /// Prometheus endpoint for metrics export
    pub endpoint: String,
    /// Metrics path
    pub path: String,
    /// Export interval in seconds
    pub export_interval_secs: u64,
    /// Custom labels for all metrics
    pub labels: HashMap<String, String>,
}

/// Custom metrics backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricsBackend {
    /// Backend name
    pub name: String,
    /// Backend endpoint
    pub endpoint: String,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Custom configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Tracing level
    pub level: String,
    /// OpenTelemetry configuration
    pub opentelemetry: Option<OpenTelemetryConfig>,
    /// Custom tracing backends
    pub custom_backends: Vec<CustomTracingBackend>,
}

/// OpenTelemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTelemetryConfig {
    /// OTLP endpoint
    pub endpoint: String,
    /// Service name
    pub service_name: String,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Custom tracing backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTracingBackend {
    /// Backend name
    pub name: String,
    /// Backend configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check endpoint port
    pub port: u16,
    /// Health check interval in seconds
    pub interval_secs: u64,
    /// Health check timeout in seconds
    pub timeout_secs: u64,
    /// Custom health checks
    pub custom_checks: Vec<CustomHealthCheck>,
}

/// Custom health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHealthCheck {
    /// Health check name
    pub name: String,
    /// Check configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// API key management
    pub api_keys: ApiKeyConfig,
    /// TLS configuration
    pub tls: TlsConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
}

/// API key management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Key rotation enabled
    pub rotation_enabled: bool,
    /// Key rotation interval in hours
    pub rotation_interval_hours: u32,
    /// Key storage backend
    pub storage_backend: KeyStorageBackend,
}

/// Key storage backend options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStorageBackend {
    /// Environment variables
    Environment,
    /// External secrets manager
    SecretsManager(SecretsManagerConfig),
    /// Local encrypted file
    EncryptedFile(String),
}

/// Secrets manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsManagerConfig {
    /// Provider (aws, gcp, azure, vault)
    pub provider: String,
    /// Provider-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    /// Certificate path
    pub cert_path: Option<String>,
    /// Private key path
    pub key_path: Option<String>,
    /// CA certificate path
    pub ca_path: Option<String>,
    /// Verify server certificates
    pub verify_server: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second limit
    pub requests_per_second: u32,
    /// Burst capacity
    pub burst_capacity: u32,
    /// Rate limit strategy
    pub strategy: RateLimitStrategy,
}

/// Rate limiting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    /// Token bucket algorithm
    TokenBucket,
    /// Fixed window
    FixedWindow,
    /// Sliding window
    SlidingWindow,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type
    pub auth_type: AuthType,
    /// Credentials
    pub credentials: HashMap<String, String>,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// Bearer token
    Bearer,
    /// Basic authentication
    Basic,
    /// API key
    ApiKey,
    /// Custom authentication
    Custom(String),
}

// Default implementations
impl Default for ChannelSizeConfig {
    fn default() -> Self {
        Self {
            source_to_parser: 10_000,
            parser_to_handler: 10_000,
            handler_to_redis: 10_000,
            error_channel: 1_000,
        }
    }
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            parser_workers: num_cpus::get(),
            handler_workers: num_cpus::get(),
            redis_workers: 4,
            worker_timeout_secs: 30,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            redis_batch_size: 1000,
            batch_timeout_ms: 100,
            memory_pressure_threshold: 50_000,
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_filter: 5,
            connection_timeout_secs: 30,
            keep_alive_interval_secs: 30,
            max_idle_secs: 300,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            half_open_timeout_secs: 60,
            success_threshold: 3,
            max_open_wait_secs: 300,
        }
    }
}

impl Default for RetryPolicyConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30_000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}