//! Data models for the Solana Stream Processor

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Simplified data structure containing essential fields for streaming and storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssentialData {
    /// Program identifier that generated this data
    pub program_id: String,
    
    /// Token mint address (if applicable)
    pub token_mint: Option<String>,
    
    /// Transaction signature
    pub transaction_signature: String,
    
    /// Type of instruction (e.g., "buy", "sell", "create", "transfer")
    pub instruction_type: String,
    
    /// Parsed instruction data as JSON
    pub instruction_data: serde_json::Value,
    
    /// Timestamp from the blockchain
    pub blockchain_timestamp: i64,
    
    /// Timestamp when data was ingested by our processor
    pub ingestion_timestamp: i64,
    
    /// Slot number from the blockchain
    pub slot: u64,
    
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl EssentialData {
    /// Create a new EssentialData instance with current ingestion timestamp
    pub fn new(
        program_id: String,
        token_mint: Option<String>,
        transaction_signature: String,
        instruction_type: String,
        instruction_data: serde_json::Value,
        blockchain_timestamp: i64,
        slot: u64,
    ) -> Self {
        let ingestion_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
            
        Self {
            program_id,
            token_mint,
            transaction_signature,
            instruction_type,
            instruction_data,
            blockchain_timestamp,
            ingestion_timestamp,
            slot,
            metadata: None,
        }
    }
    
    /// Get the collection name for MongoDB storage
    pub fn collection_name(&self) -> String {
        format!("{}.instructions", self.program_id)
    }
    
    /// Convert to JSON string for SSE streaming
    pub fn to_sse_event(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Account update data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    /// Program identifier
    pub program_id: String,
    
    /// Account address
    pub account_address: String,
    
    /// Account owner
    pub owner: String,
    
    /// Parsed account data
    pub account_data: serde_json::Value,
    
    /// Slot number
    pub slot: u64,
    
    /// Write version
    pub write_version: u64,
    
    /// Ingestion timestamp
    pub ingestion_timestamp: i64,
}

impl AccountData {
    /// Create a new AccountData instance
    pub fn new(
        program_id: String,
        account_address: String,
        owner: String,
        account_data: serde_json::Value,
        slot: u64,
        write_version: u64,
    ) -> Self {
        let ingestion_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
            
        Self {
            program_id,
            account_address,
            owner,
            account_data,
            slot,
            write_version,
            ingestion_timestamp,
        }
    }
    
    /// Get the collection name for MongoDB storage
    pub fn collection_name(&self) -> String {
        format!("{}.accounts", self.program_id)
    }
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: i64,
    pub uptime_seconds: u64,
    pub version: String,
}

impl HealthResponse {
    pub fn healthy(uptime_seconds: u64) -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            uptime_seconds,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// SSE event wrapper
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: String,
    pub data: String,
    pub id: Option<String>,
}

impl SseEvent {
    pub fn new(event_type: String, data: String) -> Self {
        Self {
            event_type,
            data,
            id: None,
        }
    }
    
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    
    /// Format as SSE string
    pub fn format(&self) -> String {
        let mut result = String::new();
        
        if let Some(id) = &self.id {
            result.push_str(&format!("id: {}\n", id));
        }
        
        result.push_str(&format!("event: {}\n", self.event_type));
        result.push_str(&format!("data: {}\n\n", self.data));
        
        result
    }
}