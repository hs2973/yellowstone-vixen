//! Redis streaming for pipeline integration with essential transaction filtering
//!
//! This module provides optimized Redis streaming that stores only filtered
//! parsed transactions and implements cleanup mechanisms for processed data.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use yellowstone_grpc_proto::geyser::SubscribeUpdate;

/// Essential transaction data that gets stored in Redis
#[derive(Debug, Serialize, Deserialize)]
pub struct EssentialTransaction {
    /// Transaction signature
    pub signature: String,
    /// Parsed transaction data
    pub parsed_data: serde_json::Value,
    /// Transaction timestamp
    pub timestamp: i64,
    /// Parser ID that processed this transaction
    pub parser_id: String,
    /// Transaction status (verified/unverified)
    pub verification_status: String,
    /// Associated accounts
    pub accounts: Vec<String>,
    /// Program IDs involved
    pub programs: Vec<String>,
}

/// Transaction processing state
#[derive(Debug)]
struct TransactionState {
    /// Transaction data
    essential_tx: EssentialTransaction,
    /// Time when transaction was added
    added_at: Instant,
    /// Whether this transaction has been written to disk
    written_to_disk: bool,
}

/// Redis writer configuration
#[derive(Debug, Clone)]
pub struct RedisWriterConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Redis stream name
    pub stream_name: String,
    /// Batch size for Redis writes
    pub batch_size: usize,
    /// Batch timeout
    pub batch_timeout: Duration,
    /// Maximum entries to keep in Redis stream
    pub max_stream_entries: usize,
    /// Cleanup interval for processed transactions
    pub cleanup_interval: Duration,
}

impl Default for RedisWriterConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            stream_name: "chainstack_transactions".to_string(),
            batch_size: 100,
            batch_timeout: Duration::from_millis(100),
            max_stream_entries: 1_000_000,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Message types for the Redis writer
#[derive(Debug)]
pub enum RedisMessage {
    /// Store an essential transaction
    StoreTransaction(EssentialTransaction),
    /// Mark transactions as written to disk
    MarkWritten(Vec<String>), // transaction signatures
    /// Force flush current batch
    Flush,
}

/// Optimized Redis writer that handles only essential filtered transactions
pub struct OptimizedRedisWriter {
    config: RedisWriterConfig,
    client: redis::Client,
    transaction_states: Arc<RwLock<HashMap<String, TransactionState>>>,
    pending_writes: VecDeque<EssentialTransaction>,
    last_cleanup: Instant,
}

impl OptimizedRedisWriter {
    /// Create a new optimized Redis writer
    pub fn new(config: RedisWriterConfig) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(config.redis_url.clone())?;
        
        Ok(Self {
            config,
            client,
            transaction_states: Arc::new(RwLock::new(HashMap::new())),
            pending_writes: VecDeque::new(),
            last_cleanup: Instant::now(),
        })
    }
    
    /// Start the Redis writer task
    pub async fn run(
        mut self,
        mut rx: mpsc::UnboundedReceiver<RedisMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        let mut batch_timer = tokio::time::interval(self.config.batch_timeout);
        
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Some(RedisMessage::StoreTransaction(tx)) => {
                            self.add_transaction(tx);
                            if self.pending_writes.len() >= self.config.batch_size {
                                self.flush_batch(&mut conn).await?;
                            }
                        }
                        Some(RedisMessage::MarkWritten(signatures)) => {
                            self.mark_written(signatures);
                        }
                        Some(RedisMessage::Flush) => {
                            self.flush_batch(&mut conn).await?;
                        }
                        None => {
                            // Channel closed, flush remaining and exit
                            if !self.pending_writes.is_empty() {
                                self.flush_batch(&mut conn).await?;
                            }
                            break;
                        }
                    }
                }
                _ = batch_timer.tick() => {
                    if !self.pending_writes.is_empty() {
                        self.flush_batch(&mut conn).await?;
                    }
                    
                    // Periodic cleanup
                    if self.last_cleanup.elapsed() >= self.config.cleanup_interval {
                        self.cleanup_processed_transactions().await;
                        self.last_cleanup = Instant::now();
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Add a transaction to pending writes
    fn add_transaction(&mut self, tx: EssentialTransaction) {
        let signature = tx.signature.clone();
        
        // Store transaction state
        {
            let mut states = self.transaction_states.write().unwrap();
            states.insert(signature, TransactionState {
                essential_tx: tx.clone(),
                added_at: Instant::now(),
                written_to_disk: false,
            });
        }
        
        // Add to pending writes
        self.pending_writes.push_back(tx);
        
        debug!("Added transaction to pending writes, queue size: {}", self.pending_writes.len());
    }
    
    /// Mark transactions as written to disk
    fn mark_written(&self, signatures: Vec<String>) {
        let mut states = self.transaction_states.write().unwrap();
        for signature in signatures {
            if let Some(state) = states.get_mut(&signature) {
                state.written_to_disk = true;
                debug!(signature = %signature, "Marked transaction as written to disk");
            }
        }
    }
    
    /// Flush pending writes to Redis
    async fn flush_batch(
        &mut self,
        conn: &mut redis::aio::Connection,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.pending_writes.is_empty() {
            return Ok(());
        }
        
        let mut pipe = redis::pipe();
        let timestamp = chrono::Utc::now().timestamp_millis();
        
        // Process each transaction in the batch
        for (i, tx) in self.pending_writes.iter().enumerate() {
            let stream_id = format!("{}:{}", timestamp, i);
            
            // Serialize transaction data
            let serialized = serde_json::to_string(tx)?;
            
            pipe.cmd("XADD")
                .arg(&self.config.stream_name)
                .arg("MAXLEN")
                .arg("~")
                .arg(self.config.max_stream_entries)
                .arg(&stream_id)
                .arg("signature")
                .arg(&tx.signature)
                .arg("parser_id")
                .arg(&tx.parser_id)
                .arg("timestamp")
                .arg(tx.timestamp)
                .arg("verification_status")
                .arg(&tx.verification_status)
                .arg("data")
                .arg(serialized);
        }
        
        // Execute pipeline
        let _: () = pipe.query_async(conn).await?;
        
        debug!("Flushed {} transactions to Redis", self.pending_writes.len());
        self.pending_writes.clear();
        
        Ok(())
    }
    
    /// Clean up processed transactions that have been written to disk
    async fn cleanup_processed_transactions(&self) {
        let signatures_to_remove: Vec<String> = {
            let states = self.transaction_states.read().unwrap();
            states
                .iter()
                .filter(|(_, state)| {
                    state.written_to_disk && 
                    state.added_at.elapsed() > Duration::from_secs(3600) // Keep for 1 hour after disk write
                })
                .map(|(sig, _)| sig.clone())
                .collect()
        };
        
        if !signatures_to_remove.is_empty() {
            let mut states = self.transaction_states.write().unwrap();
            for signature in &signatures_to_remove {
                states.remove(signature);
            }
            
            debug!("Cleaned up {} processed transactions from memory", signatures_to_remove.len());
        }
    }
}

/// Create a Redis writer for essential transactions only
pub fn create_essential_redis_writer(
    config: RedisWriterConfig,
) -> Result<mpsc::UnboundedSender<RedisMessage>, redis::RedisError> {
    let writer = OptimizedRedisWriter::new(config)?;
    let (tx, rx) = mpsc::unbounded_channel();
    
    // Spawn the writer task
    tokio::spawn(async move {
        if let Err(e) = writer.run(rx).await {
            error!(error = ?e, "Essential Redis writer task failed");
        }
    });
    
    Ok(tx)
}

/// Helper function to extract essential transaction data from a SubscribeUpdate
pub fn extract_essential_transaction(
    update: &SubscribeUpdate,
    parser_id: &str,
    parsed_data: serde_json::Value,
) -> Option<EssentialTransaction> {
    match &update.update_oneof {
        Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Transaction(tx)) => {
            let transaction = tx.transaction.as_ref()?;
            let meta = transaction.meta.as_ref()?;
            
            Some(EssentialTransaction {
                signature: bs58::encode(&tx.signature).into_string(),
                parsed_data,
                timestamp: chrono::Utc::now().timestamp_millis(),
                parser_id: parser_id.to_string(),
                verification_status: if meta.err.is_none() { "verified" } else { "failed" }.to_string(),
                accounts: transaction.message.as_ref()
                    .map(|msg| msg.account_keys.iter().map(|key| bs58::encode(key).into_string()).collect())
                    .unwrap_or_default(),
                programs: transaction.message.as_ref()
                    .and_then(|msg| msg.instructions.first())
                    .and_then(|ix| msg.account_keys.get(ix.program_id_index as usize))
                    .map(|key| vec![bs58::encode(key).into_string()])
                    .unwrap_or_default(),
            })
        }
        _ => None,
    }
}
