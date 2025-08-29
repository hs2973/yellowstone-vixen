//! Unified handler that combines filtering, SSE streaming, and MongoDB storage

use async_trait::async_trait;
use mongodb::{Client, Collection, Database};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::error::{ProcessorError, ProcessorResult};
use crate::metrics::{Metrics, Timer};
use crate::models::{EssentialData, SseEvent};

/// Unified handler that processes all parsed data through a single pipeline
#[derive(Debug)]
pub struct UnifiedHandler {
    /// SSE broadcaster sender
    sse_sender: broadcast::Sender<SseEvent>,
    
    /// MongoDB client
    mongo_client: Client,
    
    /// MongoDB database
    database: Database,
    
    /// Metrics collector
    metrics: Metrics,
}

impl UnifiedHandler {
    /// Create a new unified handler
    pub async fn new(
        sse_sender: broadcast::Sender<SseEvent>,
        mongodb_uri: &str,
        metrics: Metrics,
    ) -> ProcessorResult<Self> {
        info!("Initializing MongoDB connection: {}", mongodb_uri);
        
        let mongo_client = Client::with_uri_str(mongodb_uri)
            .await
            .map_err(ProcessorError::Database)?;
        
        // Test the connection
        mongo_client
            .database("admin")
            .run_command(mongodb::bson::doc! { "ping": 1 }, None)
            .await
            .map_err(ProcessorError::Database)?;
        
        let database = mongo_client.database("solana_stream_processor");
        
        info!("MongoDB connection established successfully");
        
        Ok(Self {
            sse_sender,
            mongo_client,
            database,
            metrics,
        })
    }
    
    /// Process instruction data - main entry point for handling parsed instructions
    pub async fn handle_instruction_data(
        &self,
        program_id: &str,
        transaction_signature: &str,
        instruction_type: &str,
        instruction_data: serde_json::Value,
        slot: u64,
        blockchain_timestamp: i64,
    ) -> ProcessorResult<()> {
        let timer = Timer::new();
        
        debug!(
            "Processing instruction: program_id={}, type={}, slot={}, sig={}",
            program_id, instruction_type, slot, transaction_signature
        );
        
        // Extract token mint if available
        let token_mint = self.extract_token_mint(&instruction_data);
        
        // Create essential data structure
        let essential_data = EssentialData::new(
            program_id.to_string(),
            token_mint,
            transaction_signature.to_string(),
            instruction_type.to_string(),
            instruction_data.clone(),
            blockchain_timestamp,
            slot,
        );
        
        // Filter data (for now, we'll process all data, but this can be extended)
        if self.should_process_data(&essential_data) {
            self.metrics.record_message_filtered();
            
            // Send to SSE (non-blocking)
            self.send_to_sse(&essential_data).await;
            
            // Send to MongoDB (async, non-blocking)
            self.send_to_mongodb(&essential_data).await;
            
            // Update metrics
            self.metrics.set_last_processed_slot(slot);
        }
        
        self.metrics.record_message_processed();
        self.metrics.record_processing_duration(timer.elapsed());
        
        Ok(())
    }
    
    /// Extract token mint from instruction data if available
    fn extract_token_mint(&self, instruction_data: &serde_json::Value) -> Option<String> {
        // Try common field names for token mints
        let possible_fields = ["mint", "token_mint", "tokenMint", "mintAccount"];
        
        for field in &possible_fields {
            if let Some(mint) = instruction_data.get(field) {
                if let Some(mint_str) = mint.as_str() {
                    return Some(mint_str.to_string());
                }
            }
        }
        
        // Try to extract from nested objects
        if let Some(accounts) = instruction_data.get("accounts") {
            for field in &possible_fields {
                if let Some(mint) = accounts.get(field) {
                    if let Some(mint_str) = mint.as_str() {
                        return Some(mint_str.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Determine if data should be processed (filtering logic)
    fn should_process_data(&self, data: &EssentialData) -> bool {
        // For now, process all data
        // This can be extended with sophisticated filtering logic
        
        // Example filtering criteria:
        // - Skip system program transfers below a certain amount
        // - Only process certain instruction types
        // - Filter by token mint address
        
        match data.program_id.as_str() {
            // System program - might want to filter out small transfers
            "11111111111111111111111111111112" => {
                // Process all system program instructions for now
                true
            },
            // SPL Token program
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {
                // Process all token instructions
                true
            },
            // Other programs - process everything
            _ => true,
        }
    }
    
    /// Send data to SSE stream (non-blocking)
    async fn send_to_sse(&self, data: &EssentialData) {
        match data.to_sse_event() {
            Ok(json_data) => {
                let event = SseEvent::new("instruction".to_string(), json_data);
                
                match self.sse_sender.send(event) {
                    Ok(receivers) => {
                        debug!("Sent SSE event to {} receivers", receivers);
                        self.metrics.record_sse_message_sent();
                    },
                    Err(_) => {
                        // No receivers connected, this is normal
                        debug!("No SSE receivers connected");
                    }
                }
            },
            Err(e) => {
                warn!("Failed to serialize data for SSE: {}", e);
            }
        }
    }
    
    /// Send data to MongoDB (async, non-blocking)
    async fn send_to_mongodb(&self, data: &EssentialData) {
        let data_clone = data.clone();
        let database = self.database.clone();
        let metrics = self.metrics.clone();
        
        // Spawn a task for the database write so it doesn't block
        tokio::spawn(async move {
            let collection_name = data_clone.collection_name();
            let collection: Collection<EssentialData> = database.collection(&collection_name);
            
            match collection.insert_one(&data_clone, None).await {
                Ok(_) => {
                    debug!("Successfully wrote to MongoDB collection: {}", collection_name);
                    metrics.record_mongodb_write();
                },
                Err(e) => {
                    error!("Failed to write to MongoDB: {}", e);
                    metrics.record_mongodb_write_error();
                }
            }
        });
    }
}

// Note: The actual Handler trait implementation will depend on the yellowstone-vixen API
// For now, we'll create a placeholder trait that shows the intended structure

/// Placeholder trait for Vixen handler integration
/// This will be replaced with the actual yellowstone-vixen Handler trait once we verify the API
#[async_trait]
pub trait VixenHandler: Send + Sync {
    async fn handle(&self, message: &VixenMessage) -> ProcessorResult<()>;
}

/// Placeholder message type
/// This will be replaced with the actual yellowstone-vixen message types
#[derive(Debug)]
pub enum VixenMessage {
    Instruction {
        program_id: String,
        transaction_signature: String,
        instruction_data: serde_json::Value,
        slot: u64,
        timestamp: i64,
    },
    Account {
        program_id: String,
        account_address: String,
        account_data: serde_json::Value,
        slot: u64,
        write_version: u64,
    },
}

#[async_trait]
impl VixenHandler for UnifiedHandler {
    async fn handle(&self, message: &VixenMessage) -> ProcessorResult<()> {
        match message {
            VixenMessage::Instruction {
                program_id,
                transaction_signature,
                instruction_data,
                slot,
                timestamp,
            } => {
                // Determine instruction type from the data
                let instruction_type = self.determine_instruction_type(program_id, instruction_data);
                
                self.handle_instruction_data(
                    program_id,
                    transaction_signature,
                    &instruction_type,
                    instruction_data.clone(),
                    *slot,
                    *timestamp,
                ).await
            },
            VixenMessage::Account { .. } => {
                // Account handling can be implemented later if needed
                debug!("Account data received but not processed yet");
                Ok(())
            }
        }
    }
}

impl UnifiedHandler {
    /// Determine instruction type from program ID and instruction data
    fn determine_instruction_type(&self, program_id: &str, instruction_data: &serde_json::Value) -> String {
        match program_id {
            "11111111111111111111111111111112" => "system".to_string(),
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {
                // Try to determine specific token instruction type
                if let Some(instruction_type) = instruction_data.get("instruction_type") {
                    instruction_type.as_str().unwrap_or("token").to_string()
                } else {
                    "token".to_string()
                }
            },
            _ => "unknown".to_string(),
        }
    }
}