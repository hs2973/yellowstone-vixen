//! Redis streaming module for high-throughput data pipeline integration
//!
//! This module provides Redis stream integration for the first stage of the trading data
//! architecture pipeline, enabling high-performance data ingestion and streaming to the
//! second stage Go pipeline.

use crate::config::{RedisStreamConfig, StreamConfig, ConsumerGroupConfig, PartitionStrategy};
use redis::{aio::Connection, AsyncCommands, Client as RedisClient, RedisResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info, warn, debug};
use yellowstone_grpc_proto::geyser::SubscribeUpdate;

/// Redis stream manager for high-throughput data ingestion
#[derive(Debug)]
pub struct RedisStreamManager {
    client: RedisClient,
    config: RedisStreamConfig,
    stream_writers: HashMap<String, StreamWriter>,
}

/// Individual stream writer for specific data types
#[derive(Debug)]
pub struct StreamWriter {
    stream_name: String,
    partition_strategy: PartitionStrategy,
    max_length: Option<u64>,
    batch_buffer: Vec<StreamEntry>,
    batch_size: usize,
}

/// Stream entry for Redis streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEntry {
    pub id: String,
    pub data: StreamData,
    pub timestamp: i64,
    pub metadata: StreamMetadata,
}

/// Structured data for stream entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamData {
    /// Yellowstone update type
    pub update_type: String,
    /// Serialized update data
    pub payload: String,
    /// Account information (if applicable)
    pub account: Option<AccountInfo>,
    /// Transaction information (if applicable)
    pub transaction: Option<TransactionInfo>,
    /// Block information (if applicable)
    pub block: Option<BlockInfo>,
}

/// Account information extracted from updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub pubkey: String,
    pub owner: String,
    pub lamports: u64,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data_size: usize,
}

/// Transaction information extracted from updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub fee: u64,
    pub accounts: Vec<String>,
    pub program_ids: Vec<String>,
    pub success: bool,
    pub error: Option<String>,
}

/// Block information extracted from updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub slot: u64,
    pub block_hash: String,
    pub parent_slot: u64,
    pub parent_hash: String,
    pub block_time: Option<i64>,
    pub transaction_count: u32,
}

/// Stream metadata for processing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub source: String,
    pub filter_id: String,
    pub processing_time_ns: u64,
    pub partition_key: Option<String>,
    pub tags: HashMap<String, String>,
}

/// Redis stream consumer for the Go pipeline
#[derive(Debug)]
pub struct RedisStreamConsumer {
    client: RedisClient,
    consumer_group: String,
    consumer_name: String,
    stream_names: Vec<String>,
    batch_size: usize,
    block_timeout_ms: u64,
}

/// Batch of stream entries for efficient processing
#[derive(Debug)]
pub struct StreamBatch {
    pub entries: Vec<StreamEntry>,
    pub stream_name: String,
    pub processing_start: std::time::Instant,
}

impl RedisStreamManager {
    /// Create a new Redis stream manager
    pub async fn new(config: RedisStreamConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = if let Some(ref url) = config.cluster.url {
            RedisClient::open(url.as_str())?
        } else {
            // TODO: Implement cluster support
            return Err("Redis cluster mode not yet implemented".into());
        };

        let mut stream_writers = HashMap::new();
        for (name, stream_config) in &config.streams {
            let writer = StreamWriter::new(stream_config.clone());
            stream_writers.insert(name.clone(), writer);
        }

        Ok(Self {
            client,
            config,
            stream_writers,
        })
    }

    /// Write a Yellowstone update to the appropriate stream
    pub async fn write_update(
        &mut self,
        update: &SubscribeUpdate,
        filter_id: &str,
        processing_time_ns: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let stream_data = self.extract_stream_data(update)?;
        let metadata = StreamMetadata {
            source: "chainstack-grpc".to_string(),
            filter_id: filter_id.to_string(),
            processing_time_ns,
            partition_key: self.calculate_partition_key(&stream_data),
            tags: HashMap::new(),
        };

        let entry = StreamEntry {
            id: format!("{}:{}", uuid::Uuid::new_v4(), chrono::Utc::now().timestamp_millis()),
            data: stream_data.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            metadata,
        };

        // Determine which stream to write to based on update type
        let stream_name = self.determine_stream_name(&stream_data);
        
        if let Some(writer) = self.stream_writers.get_mut(&stream_name) {
            writer.add_entry(entry).await?;
            
            // Flush if batch is full
            if writer.should_flush() {
                self.flush_stream(&stream_name).await?;
            }
        } else {
            warn!(stream_name = %stream_name, "No writer configured for stream");
        }

        Ok(())
    }

    /// Flush all streams
    pub async fn flush_all(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for stream_name in self.stream_writers.keys().cloned().collect::<Vec<_>>() {
            self.flush_stream(&stream_name).await?;
        }
        Ok(())
    }

    /// Flush a specific stream
    async fn flush_stream(&mut self, stream_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(writer) = self.stream_writers.get_mut(stream_name) {
            if !writer.batch_buffer.is_empty() {
                let mut conn = self.client.get_async_connection().await?;
                writer.flush_to_redis(&mut conn).await?;
                debug!(stream_name = %stream_name, "Flushed stream");
            }
        }
        Ok(())
    }

    /// Extract structured data from Yellowstone update
    fn extract_stream_data(&self, update: &SubscribeUpdate) -> Result<StreamData, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::to_string(update)?;
        
        let (update_type, account, transaction, block) = match &update.update_oneof {
            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Account(acc)) => {
                let account_info = AccountInfo {
                    pubkey: acc.account.as_ref().map(|a| a.pubkey.clone()).unwrap_or_default(),
                    owner: acc.account.as_ref().map(|a| a.owner.clone()).unwrap_or_default(),
                    lamports: acc.account.as_ref().map(|a| a.lamports).unwrap_or_default(),
                    executable: acc.account.as_ref().map(|a| a.executable).unwrap_or_default(),
                    rent_epoch: acc.account.as_ref().map(|a| a.rent_epoch).unwrap_or_default(),
                    data_size: acc.account.as_ref().map(|a| a.data.len()).unwrap_or_default(),
                };
                ("account".to_string(), Some(account_info), None, None)
            }
            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Transaction(tx)) => {
                let transaction_info = TransactionInfo {
                    signature: tx.transaction.as_ref()
                        .and_then(|t| t.signature.as_ref())
                        .map(|s| s.clone())
                        .unwrap_or_default(),
                    slot: tx.slot,
                    block_time: tx.block_time,
                    fee: tx.transaction.as_ref()
                        .and_then(|t| t.meta.as_ref())
                        .map(|m| m.fee)
                        .unwrap_or_default(),
                    accounts: tx.transaction.as_ref()
                        .map(|t| t.account_keys.clone())
                        .unwrap_or_default(),
                    program_ids: Vec::new(), // TODO: Extract program IDs
                    success: tx.transaction.as_ref()
                        .and_then(|t| t.meta.as_ref())
                        .map(|m| m.err.is_none())
                        .unwrap_or_default(),
                    error: tx.transaction.as_ref()
                        .and_then(|t| t.meta.as_ref())
                        .and_then(|m| m.err.as_ref())
                        .map(|e| format!("{:?}", e)),
                };
                ("transaction".to_string(), None, Some(transaction_info), None)
            }
            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Block(blk)) => {
                let block_info = BlockInfo {
                    slot: blk.slot,
                    block_hash: blk.blockhash.clone(),
                    parent_slot: blk.parent_slot,
                    parent_hash: blk.parent_blockhash.clone(),
                    block_time: blk.block_time,
                    transaction_count: blk.executed_transaction_count,
                };
                ("block".to_string(), None, None, Some(block_info))
            }
            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::BlockMeta(meta)) => {
                let block_info = BlockInfo {
                    slot: meta.slot,
                    block_hash: meta.blockhash.clone(),
                    parent_slot: meta.parent_slot,
                    parent_hash: meta.parent_blockhash.clone(),
                    block_time: meta.block_time,
                    transaction_count: meta.executed_transaction_count,
                };
                ("block_meta".to_string(), None, None, Some(block_info))
            }
            _ => ("unknown".to_string(), None, None, None),
        };

        Ok(StreamData {
            update_type,
            payload,
            account,
            transaction,
            block,
        })
    }

    /// Calculate partition key for data distribution
    fn calculate_partition_key(&self, data: &StreamData) -> Option<String> {
        match &data.update_type as &str {
            "account" => data.account.as_ref().map(|a| a.pubkey.clone()),
            "transaction" => data.transaction.as_ref().map(|t| t.signature.clone()),
            "block" | "block_meta" => data.block.as_ref().map(|b| b.slot.to_string()),
            _ => None,
        }
    }

    /// Determine which stream to write to based on data type
    fn determine_stream_name(&self, data: &StreamData) -> String {
        match &data.update_type as &str {
            "account" => "accounts".to_string(),
            "transaction" => "transactions".to_string(),
            "block" | "block_meta" => "blocks".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

impl StreamWriter {
    fn new(config: StreamConfig) -> Self {
        Self {
            stream_name: config.name,
            partition_strategy: config.partition_strategy,
            max_length: config.max_length,
            batch_buffer: Vec::new(),
            batch_size: 1000, // TODO: Make configurable
        }
    }

    async fn add_entry(&mut self, entry: StreamEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.batch_buffer.push(entry);
        Ok(())
    }

    fn should_flush(&self) -> bool {
        self.batch_buffer.len() >= self.batch_size
    }

    async fn flush_to_redis(&mut self, conn: &mut Connection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.batch_buffer.is_empty() {
            return Ok(());
        }

        // Use Redis pipeline for batch writes
        let mut pipe = redis::pipe();
        
        for entry in &self.batch_buffer {
            let mut cmd = pipe.cmd("XADD");
            cmd.arg(&self.stream_name);

            // Add MAXLEN if configured
            if let Some(max_len) = self.max_length {
                cmd.arg("MAXLEN").arg("~").arg(max_len);
            }

            cmd.arg(&entry.id)
                .arg("data")
                .arg(serde_json::to_string(&entry.data)?)
                .arg("timestamp")
                .arg(entry.timestamp)
                .arg("metadata")
                .arg(serde_json::to_string(&entry.metadata)?);
        }

        let _: Vec<String> = pipe.query_async(conn).await?;
        
        info!(
            stream = %self.stream_name,
            entries = self.batch_buffer.len(),
            "Flushed batch to Redis"
        );

        self.batch_buffer.clear();
        Ok(())
    }
}

impl RedisStreamConsumer {
    /// Create a new Redis stream consumer for the Go pipeline
    pub async fn new(
        redis_url: &str,
        consumer_group: String,
        consumer_name: String,
        stream_names: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = RedisClient::open(redis_url)?;
        
        Ok(Self {
            client,
            consumer_group,
            consumer_name,
            stream_names,
            batch_size: 100,
            block_timeout_ms: 1000,
        })
    }

    /// Initialize consumer groups for the streams
    pub async fn initialize_consumer_groups(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        
        for stream_name in &self.stream_names {
            // Create consumer group if it doesn't exist
            let result: RedisResult<String> = redis::cmd("XGROUP")
                .arg("CREATE")
                .arg(stream_name)
                .arg(&self.consumer_group)
                .arg("0")
                .arg("MKSTREAM")
                .query_async(&mut conn)
                .await;

            match result {
                Ok(_) => info!(stream = %stream_name, group = %self.consumer_group, "Created consumer group"),
                Err(e) if e.to_string().contains("BUSYGROUP") => {
                    debug!(stream = %stream_name, group = %self.consumer_group, "Consumer group already exists");
                }
                Err(e) => {
                    error!(stream = %stream_name, group = %self.consumer_group, error = ?e, "Failed to create consumer group");
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    /// Consume messages from streams
    pub async fn consume_batch(&self) -> Result<Vec<StreamBatch>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.get_async_connection().await?;
        let mut batches = Vec::new();

        for stream_name in &self.stream_names {
            let result: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(&self.consumer_group)
                .arg(&self.consumer_name)
                .arg("COUNT")
                .arg(self.batch_size)
                .arg("BLOCK")
                .arg(self.block_timeout_ms)
                .arg("STREAMS")
                .arg(stream_name)
                .arg(">")
                .query_async(&mut conn)
                .await?;

            for (stream, messages) in result {
                let mut entries = Vec::new();
                
                for (id, fields) in messages {
                    if let Ok(entry) = self.parse_stream_entry(id, fields) {
                        entries.push(entry);
                    }
                }

                if !entries.is_empty() {
                    batches.push(StreamBatch {
                        entries,
                        stream_name: stream,
                        processing_start: std::time::Instant::now(),
                    });
                }
            }
        }

        Ok(batches)
    }

    /// Acknowledge processed messages
    pub async fn acknowledge_batch(
        &self,
        stream_name: &str,
        message_ids: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if message_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.client.get_async_connection().await?;
        
        let mut cmd = redis::cmd("XACK");
        cmd.arg(stream_name).arg(&self.consumer_group);
        for id in &message_ids {
            cmd.arg(id);
        }

        let _: u64 = cmd.query_async(&mut conn).await?;
        
        debug!(
            stream = %stream_name,
            count = message_ids.len(),
            "Acknowledged messages"
        );

        Ok(())
    }

    fn parse_stream_entry(&self, id: String, fields: Vec<(String, String)>) -> Result<StreamEntry, Box<dyn std::error::Error + Send + Sync>> {
        let mut data_json = None;
        let mut timestamp = None;
        let mut metadata_json = None;

        for (field, value) in fields {
            match field.as_str() {
                "data" => data_json = Some(value),
                "timestamp" => timestamp = Some(value.parse::<i64>()?),
                "metadata" => metadata_json = Some(value),
                _ => {}
            }
        }

        let data: StreamData = serde_json::from_str(
            &data_json.ok_or("Missing data field")?
        )?;
        
        let metadata: StreamMetadata = serde_json::from_str(
            &metadata_json.ok_or("Missing metadata field")?
        )?;

        Ok(StreamEntry {
            id,
            data,
            timestamp: timestamp.ok_or("Missing timestamp field")?,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_entry_serialization() {
        let entry = StreamEntry {
            id: "test-id".to_string(),
            data: StreamData {
                update_type: "account".to_string(),
                payload: "{}".to_string(),
                account: None,
                transaction: None,
                block: None,
            },
            timestamp: 1234567890,
            metadata: StreamMetadata {
                source: "test".to_string(),
                filter_id: "test-filter".to_string(),
                processing_time_ns: 1000,
                partition_key: None,
                tags: HashMap::new(),
            },
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: StreamEntry = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(entry.id, deserialized.id);
        assert_eq!(entry.timestamp, deserialized.timestamp);
    }
}