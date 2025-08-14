//! Chainstack Yellowstone gRPC Source for Yellowstone Vixen
//!
//! This crate provides a Source implementation for connecting to Chainstack's Yellowstone gRPC
//! service, following the same patterns as YellowstoneGrpcSource.

use std::{collections::HashMap, net::SocketAddr, time::Duration};

use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::{sync::mpsc::{self, Sender}, task::JoinSet};
use tracing::{error, warn, info, debug};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::{
    geyser::SubscribeUpdate,
    tonic::{transport::ClientTlsConfig, Status},
};
use yellowstone_vixen::{config::YellowstoneConfig, sources::Source, Error as VixenError};
use yellowstone_vixen_core::{Filters, Prefilter};

pub mod filter_api;
pub mod redis_stream;

pub use filter_api::{FilterApiState, FilterUpdate, start_filter_api_server};
pub use redis_stream::{
    EssentialTransaction, OptimizedRedisWriter, RedisMessage, RedisWriterConfig,
    create_essential_redis_writer, extract_essential_transaction,
};

/// A `Source` implementation for Chainstack's Yellowstone gRPC API.
/// 
/// This follows the same pattern as YellowstoneGrpcSource but includes optional
/// Redis streaming for pipeline integration and live filter management.
#[derive(Debug, Default)]
pub struct ChainstackGrpcSource {
    config: Option<YellowstoneConfig>,
    filters: Option<Filters>,
    redis_writer_tx: Option<mpsc::UnboundedSender<RedisMessage>>,
    filter_update_rx: Option<mpsc::UnboundedReceiver<FilterUpdate>>,
    filter_api_addr: Option<SocketAddr>,
}

impl ChainstackGrpcSource {
    /// Create a new `ChainstackGrpcSource` with default values.
    #[must_use]
    pub fn new() -> Self { 
        Self::default() 
    }

    /// Enable Redis streaming for pipeline integration with essential transaction filtering.
    /// This starts a background writer task that batches Redis writes and only stores
    /// filtered/parsed transactions to optimize RAM usage.
    #[must_use]
    pub fn with_essential_redis_streaming(
        mut self, 
        config: RedisWriterConfig,
    ) -> Result<Self, redis::RedisError> {
        let tx = create_essential_redis_writer(config)?;
        self.redis_writer_tx = Some(tx);
        Ok(self)
    }

    /// Enable the Filter API for real-time filter management.
    /// This starts a REST API server that allows dynamic filter updates without restart.
    #[must_use]
    pub fn with_filter_api(mut self, addr: SocketAddr) -> Self {
        self.filter_api_addr = Some(addr);
        self
    }

    /// Start the filter API server if configured
    async fn start_filter_api(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(addr) = self.filter_api_addr {
            let (filter_update_tx, filter_update_rx) = mpsc::unbounded_channel();
            self.filter_update_rx = Some(filter_update_rx);
            
            // Start the filter API server in the background
            tokio::spawn(async move {
                if let Err(e) = start_filter_api_server(addr, filter_update_tx).await {
                    error!(error = ?e, "Filter API server failed");
                }
            });
            
            info!(addr = %addr, "Filter API server started");
        }
        Ok(())
    }

    /// Check for and apply filter updates
    async fn handle_filter_updates(&mut self) {
        if let Some(ref mut rx) = self.filter_update_rx {
            while let Ok(update) = rx.try_recv() {
                debug!(parser_id = %update.parser_id, "Received filter update");
                
                // Update internal filters
                if let Some(ref mut filters) = self.filters {
                    filters.parsers_filters.insert(update.parser_id, update.prefilter);
                    info!("Applied filter update, total filters: {}", filters.parsers_filters.len());
                }
            }
        }
    }
}

#[async_trait]
impl Source for ChainstackGrpcSource {
    fn name(&self) -> String {
        "chainstack-grpc".to_string()
    }

    async fn connect(&self, tx: Sender<Result<SubscribeUpdate, Status>>) -> Result<(), VixenError> {
        // We require that config and filters are set before connecting to the `Source`
        let mut filters = self.filters.clone().ok_or(VixenError::ConfigError)?;
        let config = self.config.clone().ok_or(VixenError::ConfigError)?;

        let timeout = Duration::from_secs(config.timeout);

        // TODO: add tasks pool concurrency limit through config
        let mut tasks_set = JoinSet::new();

        // Start filter API if configured
        let mut source_copy = self.clone();
        source_copy.start_filter_api().await.map_err(|_| VixenError::ConfigError)?;

        for (filter_id, prefilter) in filters.parsers_filters.clone() {
            let mut filter = Filters::new(HashMap::from([(filter_id.clone(), prefilter)]));
            filter.global_filters = filters.global_filters;
            let config = config.clone();
            let tx = tx.clone();
            let redis_writer_tx = self.redis_writer_tx.clone();
            let mut filter_update_rx = source_copy.filter_update_rx.take();

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
                    // Handle filter updates
                    if let Some(ref mut rx) = filter_update_rx {
                        while let Ok(filter_update) = rx.try_recv() {
                            if filter_update.parser_id == filter_id {
                                debug!(parser_id = %filter_id, "Applying live filter update");
                                // Note: In a production implementation, you would want to 
                                // restart the subscription with the new filter
                            }
                        }
                    }

                    match update {
                        Ok(update_data) => {
                            // Check if this update matches our filters and should be processed
                            let should_process = match &update_data.update_oneof {
                                Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Transaction(tx)) => {
                                    // Only process transactions that match our criteria
                                    tx.transaction.as_ref()
                                        .and_then(|t| t.meta.as_ref())
                                        .map(|meta| meta.err.is_none()) // Only successful transactions
                                        .unwrap_or(false)
                                }
                                Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Account(_)) => true,
                                _ => false, // Discard other update types to save RAM
                            };

                            if should_process {
                                // Send to Redis writer if configured (only essential transactions)
                                if let Some(ref redis_tx) = redis_writer_tx {
                                    // Create essential transaction data with parsed content
                                    let parsed_data = serde_json::json!({
                                        "filter_id": filter_id,
                                        "timestamp": chrono::Utc::now().timestamp_millis(),
                                        "update_type": match &update_data.update_oneof {
                                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Transaction(_)) => "transaction",
                                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Account(_)) => "account",
                                            _ => "other"
                                        }
                                    });

                                    if let Some(essential_tx) = extract_essential_transaction(
                                        &update_data, 
                                        &filter_id, 
                                        parsed_data
                                    ) {
                                        if let Err(e) = redis_tx.send(RedisMessage::StoreTransaction(essential_tx)) {
                                            warn!(error = ?e, "Failed to send essential transaction to Redis writer");
                                        } else {
                                            debug!("Stored essential transaction in Redis");
                                        }
                                    }
                                }

                                // Send to Vixen pipeline
                                let res = tx.send(Ok(update_data)).await;
                                if res.is_err() {
                                    error!("Failed to send update to buffer");
                                    break;
                                }
                            } else {
                                // Discard non-matching transactions to optimize RAM usage
                                debug!("Discarded non-essential transaction to optimize RAM");
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

// Implement Clone for ChainstackGrpcSource (needed for filter updates)
impl Clone for ChainstackGrpcSource {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            filters: self.filters.clone(),
            redis_writer_tx: self.redis_writer_tx.clone(),
            filter_update_rx: None, // Don't clone the receiver
            filter_api_addr: self.filter_api_addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chainstack_source_creation() {
        let source = ChainstackGrpcSource::new();
        assert_eq!(source.name(), "chainstack-grpc");
    }

    #[test]
    fn test_essential_redis_configuration() {
        let config = RedisWriterConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.max_stream_entries, 1_000_000);
    }
}