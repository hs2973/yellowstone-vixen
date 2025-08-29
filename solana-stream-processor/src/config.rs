use std::{net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};
use yellowstone_vixen_yellowstone_grpc_source::YellowstoneGrpcConfig;

/// Application configuration structure
/// Loads from config.toml and supports environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Data source configuration
    pub source: SourceConfig,
    /// MongoDB database configuration
    pub database: DatabaseConfig,
    /// Web server configuration for streaming APIs
    pub server: ServerConfig,
    /// Prometheus metrics configuration
    pub metrics: Option<MetricsConfig>,
    /// Processing configuration
    pub processing: ProcessingConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    /// Primary Yellowstone gRPC source
    #[serde(rename = "yellowstone")]
    Yellowstone(YellowstoneSourceConfig),
    /// Secondary Solana RPC source
    #[serde(rename = "solana_rpc")]
    SolanaRpc(SolanaRpcSourceConfig),
}

/// Yellowstone gRPC source configuration
#[derive(Debug, Clone, Deserialize)]
pub struct YellowstoneSourceConfig {
    /// Yellowstone gRPC configuration
    #[serde(flatten)]
    pub grpc: YellowstoneGrpcConfig,
    /// Auto-reconnection settings
    pub reconnect: ReconnectConfig,
}

/// Solana RPC source configuration for backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaRpcSourceConfig {
    /// RPC endpoint URL
    pub endpoint: String,
    /// Polling interval in seconds
    pub poll_interval: u64,
    /// Request timeout in seconds
    pub timeout: u64,
}

/// Reconnection configuration for resilient connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// Initial retry delay in seconds
    pub initial_delay: u64,
    /// Maximum retry delay in seconds
    pub max_delay: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum number of retry attempts (0 = infinite)
    pub max_attempts: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: 1,
            max_delay: 60,
            backoff_multiplier: 2.0,
            max_attempts: 0, // Infinite retries
        }
    }
}

/// MongoDB database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// MongoDB connection URI
    pub uri: String,
    /// Database name
    pub database_name: String,
    /// Connection pool configuration
    pub pool: PoolConfig,
    /// Index configuration
    pub indexes: IndexConfig,
}

/// MongoDB connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of connections in pool
    pub min_connections: u32,
    /// Maximum number of connections in pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Server selection timeout in seconds
    pub server_selection_timeout: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 100,
            connect_timeout: 10,
            server_selection_timeout: 30,
        }
    }
}

/// Database indexing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Create indexes on startup
    pub create_on_startup: bool,
    /// Index on token_mint field
    pub token_mint_index: bool,
    /// Index on slot field
    pub slot_index: bool,
    /// Index on timestamp field
    pub timestamp_index: bool,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            create_on_startup: true,
            token_mint_index: true,
            slot_index: true,
            timestamp_index: true,
        }
    }
}

/// Web server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// CORS configuration
    pub cors: CorsConfig,
}

/// CORS configuration for web APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allow all origins
    pub allow_all_origins: bool,
    /// Allowed origins list
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_all_origins: true,
            allowed_origins: vec![],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
            allowed_headers: vec!["*".to_string()],
        }
    }
}

/// Prometheus metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Metrics endpoint path
    pub path: String,
    /// Metrics export interval in seconds
    pub export_interval: u64,
    /// Job name for Prometheus
    pub job_name: String,
    /// Instance name for Prometheus
    pub instance_name: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            path: "/metrics".to_string(),
            export_interval: 15,
            job_name: "solana-stream-processor".to_string(),
            instance_name: "localhost:8080".to_string(),
        }
    }
}

/// Processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Buffer size for incoming messages
    pub buffer_size: usize,
    /// Number of worker threads for processing
    pub worker_threads: usize,
    /// Batch size for MongoDB inserts
    pub batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Filter configuration
    pub filters: FilterConfig,
}

/// Filter configuration for processing only relevant data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Only process trading-related instructions
    pub trading_instructions_only: bool,
    /// Minimum slot to process from
    pub min_slot: Option<u64>,
    /// Supported program IDs
    pub program_ids: Vec<String>,
    /// Supported instruction types per program
    pub instruction_types: std::collections::HashMap<String, Vec<String>>,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            worker_threads: num_cpus::get(),
            batch_size: 100,
            batch_timeout_ms: 1000,
            filters: FilterConfig {
                trading_instructions_only: true,
                min_slot: None,
                program_ids: vec![],
                instruction_types: std::collections::HashMap::new(),
            },
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", path.display(), e))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file '{}': {}", path.display(), e))?;
        
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        // Validate database URI
        if self.database.uri.is_empty() {
            return Err(anyhow::anyhow!("Database URI cannot be empty"));
        }

        // Validate database name
        if self.database.database_name.is_empty() {
            return Err(anyhow::anyhow!("Database name cannot be empty"));
        }

        // Validate processing configuration
        if self.processing.buffer_size == 0 {
            return Err(anyhow::anyhow!("Buffer size must be greater than 0"));
        }

        if self.processing.worker_threads == 0 {
            return Err(anyhow::anyhow!("Worker threads must be greater than 0"));
        }

        if self.processing.batch_size == 0 {
            return Err(anyhow::anyhow!("Batch size must be greater than 0"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_default_configs() {
        let reconnect = ReconnectConfig::default();
        assert_eq!(reconnect.initial_delay, 1);
        assert_eq!(reconnect.max_delay, 60);
        assert_eq!(reconnect.backoff_multiplier, 2.0);
        assert_eq!(reconnect.max_attempts, 0);

        let pool = PoolConfig::default();
        assert_eq!(pool.min_connections, 5);
        assert_eq!(pool.max_connections, 100);

        let processing = ProcessingConfig::default();
        assert!(processing.buffer_size > 0);
        assert!(processing.worker_threads > 0);
        assert!(processing.batch_size > 0);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig {
            source: SourceConfig::Yellowstone(YellowstoneSourceConfig {
                grpc: YellowstoneGrpcConfig {
                    endpoint: "http://localhost:8080".to_string(),
                    x_token: None,
                    timeout: 60,
                    commitment_level: None,
                    from_slot: None,
                },
                reconnect: ReconnectConfig::default(),
            }),
            database: DatabaseConfig {
                uri: "mongodb://localhost:27017".to_string(),
                database_name: "solana_stream".to_string(),
                pool: PoolConfig::default(),
                indexes: IndexConfig::default(),
            },
            server: ServerConfig {
                bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                max_connections: 1000,
                request_timeout: 30,
                cors: CorsConfig::default(),
            },
            metrics: Some(MetricsConfig::default()),
            processing: ProcessingConfig::default(),
        };

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid database URI should fail
        config.database.uri = "".to_string();
        assert!(config.validate().is_err());
    }
}