//! Chainstack Yellowstone gRPC Source for Yellowstone Vixen
//!
//! This crate provides a Source implementation for connecting to Chainstack's Yellowstone gRPC
//! service, following the same patterns as YellowstoneGrpcSource.

use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::{sync::mpsc::{self, Sender}, task::JoinSet};
use tracing::{error, warn};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::{
    geyser::SubscribeUpdate,
    tonic::{transport::ClientTlsConfig, Status},
};
use yellowstone_vixen::{config::YellowstoneConfig, sources::Source, Error as VixenError};
use yellowstone_vixen_core::Filters;

/// A `Source` implementation for Chainstack's Yellowstone gRPC API.
/// 
/// This follows the same pattern as YellowstoneGrpcSource but includes optional
/// Redis streaming for pipeline integration.
#[derive(Debug, Default)]
pub struct ChainstackGrpcSource {
    config: Option<YellowstoneConfig>,
    filters: Option<Filters>,
    redis_writer_tx: Option<mpsc::UnboundedSender<SubscribeUpdate>>,
}

impl ChainstackGrpcSource {
    /// Create a new `ChainstackGrpcSource` with default values.
    #[must_use]
    pub fn new() -> Self { 
        Self::default() 
    }

    /// Enable Redis streaming for pipeline integration.
    /// This starts a background writer task that batches Redis writes.
    #[must_use]
    pub fn with_redis_streaming<S: Into<String>>(mut self, redis_url: S, stream_name: S) -> Self {
        let redis_url = redis_url.into();
        let stream_name = stream_name.into();
        
        // Create unbounded channel for Redis writes
        let (tx, rx) = mpsc::unbounded_channel();
        self.redis_writer_tx = Some(tx);

        // Spawn background Redis writer task
        tokio::spawn(async move {
            if let Err(e) = redis_writer_task(redis_url, stream_name, rx).await {
                error!(error = ?e, "Redis writer task failed");
            }
        });

        self
    }
}

#[async_trait]
impl Source for ChainstackGrpcSource {
    fn name(&self) -> String {
        "chainstack-grpc".to_string()
    }

    async fn connect(&self, tx: Sender<Result<SubscribeUpdate, Status>>) -> Result<(), VixenError> {
        // We require that config and filters are set before connecting to the `Source`
        let filters = self.filters.clone().ok_or(VixenError::ConfigError)?;
        let config = self.config.clone().ok_or(VixenError::ConfigError)?;

        let timeout = Duration::from_secs(config.timeout);

        // TODO: add tasks pool concurrency limit through config
        let mut tasks_set = JoinSet::new();

        for (filter_id, prefilter) in filters.parsers_filters {
            let mut filter = Filters::new(HashMap::from([(filter_id, prefilter)]));
            filter.global_filters = filters.global_filters;
            let config = config.clone();
            let tx = tx.clone();
            let redis_writer_tx = self.redis_writer_tx.clone();

            let mut client = GeyserGrpcClient::build_from_shared(config.endpoint)?
                .x_token(config.x_token)?
                .connect_timeout(timeout)
                .timeout(timeout)
                .tls_config(ClientTlsConfig::new().with_native_roots())?
                .connect()
                .await?;

            let (_sub_tx, stream) = client.subscribe_with_request(Some(filter.into())).await?;

            tasks_set.spawn(async move {
                let mut stream = std::pin::pin!(stream);

                while let Some(update) = stream.next().await {
                    match update {
                        Ok(update_data) => {
                            // Send to Redis writer if configured
                            if let Some(ref redis_tx) = redis_writer_tx {
                                if let Err(e) = redis_tx.send(update_data.clone()) {
                                    warn!(error = ?e, "Failed to send update to Redis writer");
                                }
                            }

                            // Send to Vixen pipeline
                            let res = tx.send(Ok(update_data)).await;
                            if res.is_err() {
                                error!("Failed to send update to buffer");
                                break;
                            }
                        }
                        Err(e) => {
                            let res = tx.send(Err(e)).await;
                            if res.is_err() {
                                error!("Failed to send error to buffer");
                                break;
                            }
                        }
                    }
                }
            });
        }

        tasks_set.join_all().await;

        Ok(())
    }

    fn set_filters_unchecked(&mut self, filters: Filters) { 
        self.filters = Some(filters); 
    }

    fn set_config_unchecked(&mut self, config: YellowstoneConfig) { 
        self.config = Some(config); 
    }

    fn get_filters(&self) -> &Option<Filters> { 
        &self.filters 
    }

    fn get_config(&self) -> Option<YellowstoneConfig> { 
        self.config.clone() 
    }
}

/// Background Redis writer task that batches writes for performance
async fn redis_writer_task(
    redis_url: String,
    stream_name: String,
    mut rx: mpsc::UnboundedReceiver<SubscribeUpdate>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_async_connection().await?;
    
    const BATCH_SIZE: usize = 100;
    const BATCH_TIMEOUT: Duration = Duration::from_millis(100);
    
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut batch_timer = tokio::time::interval(BATCH_TIMEOUT);
    
    loop {
        tokio::select! {
            update = rx.recv() => {
                match update {
                    Some(update) => {
                        batch.push(update);
                        if batch.len() >= BATCH_SIZE {
                            flush_batch(&mut conn, &stream_name, &mut batch).await?;
                        }
                    }
                    None => {
                        // Channel closed, flush remaining items and exit
                        if !batch.is_empty() {
                            flush_batch(&mut conn, &stream_name, &mut batch).await?;
                        }
                        break;
                    }
                }
            }
            _ = batch_timer.tick() => {
                if !batch.is_empty() {
                    flush_batch(&mut conn, &stream_name, &mut batch).await?;
                }
            }
        }
    }
    
    Ok(())
}

/// Flush a batch of updates to Redis using pipeline for performance
async fn flush_batch(
    conn: &mut redis::aio::Connection,
    stream_name: &str,
    batch: &mut Vec<SubscribeUpdate>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if batch.is_empty() {
        return Ok(());
    }
    
    let mut pipe = redis::pipe();
    let timestamp = chrono::Utc::now().timestamp_millis();
    
    for (i, _update) in batch.iter().enumerate() {
        let stream_id = format!("{}:{}", timestamp, i);
        
        pipe.cmd("XADD")
            .arg(stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(1_000_000) // Keep last 1M entries
            .arg(&stream_id)
            .arg("type")
            .arg("yellowstone_update")
            .arg("timestamp")
            .arg(timestamp);
    }
    
    let _: () = pipe.query_async(conn).await?;
    batch.clear();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chainstack_source_creation() {
        let source = ChainstackGrpcSource::new();
        assert_eq!(source.name(), "chainstack-grpc");
    }
}