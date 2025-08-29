//! Configuration module for the Solana Stream Processor

use serde::Deserialize;

/// Main configuration structure for the application
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// MongoDB connection URI
    pub mongodb_uri: String,
    
    /// Web server port
    pub web_server_port: u16,
    
    /// Yellowstone Vixen configuration
    pub vixen: VixenConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// SSE configuration
    pub sse: SseConfig,
}

/// Vixen-specific configuration
#[derive(Debug, Clone, Deserialize)]
pub struct VixenConfig {
    /// gRPC endpoint for Yellowstone
    pub grpc_endpoint: Option<String>,
    
    /// RPC endpoint for Solana
    pub rpc_endpoint: Option<String>,
    
    /// Authentication token if required
    pub auth_token: Option<String>,
    
    /// Programs to monitor
    pub programs: Vec<String>,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Database name
    pub database_name: String,
    
    /// Connection pool size
    pub pool_size: Option<u32>,
    
    /// Connection timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// SSE configuration
#[derive(Debug, Clone, Deserialize)]
pub struct SseConfig {
    /// Maximum number of concurrent SSE connections
    pub max_connections: usize,
    
    /// Buffer size for SSE messages
    pub buffer_size: usize,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mongodb_uri: "mongodb://localhost:27017".to_string(),
            web_server_port: 8080,
            vixen: VixenConfig::default(),
            database: DatabaseConfig::default(),
            sse: SseConfig::default(),
        }
    }
}

impl Default for VixenConfig {
    fn default() -> Self {
        Self {
            grpc_endpoint: None,
            rpc_endpoint: Some("https://api.mainnet-beta.solana.com".to_string()),
            auth_token: None,
            programs: vec![
                "11111111111111111111111111111112".to_string(), // System Program
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(), // SPL Token
            ],
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_name: "solana_stream_processor".to_string(),
            pool_size: Some(10),
            timeout_seconds: Some(30),
        }
    }
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            buffer_size: 1000,
            heartbeat_interval: 30,
        }
    }
}