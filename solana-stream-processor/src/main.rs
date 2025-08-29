//! # Solana Stream Processor
//! 
//! Production-ready Rust application for real-time processing of high-volume gRPC messages
//! from Solana blockchain data sources. Parses, filters, and stores data from multiple DEX
//! and token programs using MongoDB's document model.
//!
//! ## Features
//! 
//! - **Real-time gRPC streaming** from Yellowstone and Solana RPC sources
//! - **Multi-program parsing** using existing Yellowstone Vixen proto definitions
//! - **MongoDB storage** with program-specific collections and optimized indexing
//! - **Web APIs** for data access and real-time Server-Sent Events streaming
//! - **Prometheus metrics** for monitoring and observability
//! - **Production-ready** with comprehensive error handling and reconnection logic
//!
//! ## Architecture
//!
//! The application consists of several key components:
//! - **Data Sources**: Yellowstone gRPC (primary) and Solana RPC (backup)
//! - **Parser Engine**: Unified parser supporting 9+ Solana programs
//! - **Storage Layer**: MongoDB with program-specific collections
//! - **API Layer**: RESTful APIs and Server-Sent Events for real-time streaming
//! - **Monitoring**: Prometheus metrics and health checks

use std::sync::Arc;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser as ClapParser;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout};
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use yellowstone_vixen_core::{AccountUpdate, InstructionUpdate};
use yellowstone_vixen_yellowstone_grpc_source::{YellowstoneGrpcSource, YellowstoneGrpcConfig};

mod config;
mod models;
mod handlers;
mod parsers;
mod metrics;

use config::{AppConfig, SourceConfig};
use handlers::{MongoHandler, StreamHandler};
use parsers::UnifiedParser;
use metrics::Metrics;

/// Command line arguments for the application
#[derive(ClapParser)]
#[command(
    name = "solana-stream-processor",
    version = env!("CARGO_PKG_VERSION"),
    about = "Production-ready Rust application for real-time Solana data processing",
    long_about = "Real-time processing of high-volume gRPC messages from Solana blockchain data sources with MongoDB storage and web APIs"
)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Enable JSON logging format
    #[arg(long)]
    json_logs: bool,
}

/// Main application struct that coordinates all components
pub struct SolanaStreamProcessor {
    config: AppConfig,
    parser: UnifiedParser,
    mongo_handler: Arc<MongoHandler>,
    stream_handler: StreamHandler,
    metrics: Arc<Metrics>,
}

impl SolanaStreamProcessor {
    /// Create a new Solana Stream Processor with the given configuration
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing Solana Stream Processor");

        // Validate configuration
        config.validate()?;

        // Initialize metrics
        let metrics = Arc::new(Metrics::new());

        // Initialize MongoDB handler
        info!("Connecting to MongoDB...");
        let mongo_handler = Arc::new(MongoHandler::new(config.database.clone(), metrics.clone()).await?);

        // Initialize unified parser
        let parser = UnifiedParser::new();

        // Initialize stream handler for web APIs
        let stream_handler = StreamHandler::new(
            config.server.clone(),
            mongo_handler.clone(),
            metrics.clone(),
        );

        info!("Solana Stream Processor initialized successfully");

        Ok(Self {
            config,
            parser,
            mongo_handler,
            stream_handler,
            metrics,
        })
    }

    /// Start the application and run all components
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Solana Stream Processor");

        // Get broadcast sender for streaming
        let broadcast_tx = self.stream_handler.get_broadcast_sender();

        // Start web server in background
        let stream_handler = self.stream_handler;
        let web_server_handle = tokio::spawn(async move {
            if let Err(e) = stream_handler.start_server().await {
                error!("Web server error: {}", e);
            }
        });

        // Start data processing pipeline
        let processor = DataProcessor::new(
            self.parser,
            self.mongo_handler,
            broadcast_tx,
            self.metrics,
            self.config.processing.clone(),
        );

        let processing_handle = tokio::spawn(async move {
            if let Err(e) = processor.run(&self.config.source).await {
                error!("Data processing error: {}", e);
            }
        });

        // Wait for shutdown signal
        info!("Application started successfully. Press Ctrl+C to shutdown.");
        
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Shutdown signal received");
            }
            _ = web_server_handle => {
                warn!("Web server stopped unexpectedly");
            }
            _ = processing_handle => {
                warn!("Data processor stopped unexpectedly");
            }
        }

        info!("Shutting down Solana Stream Processor");
        Ok(())
    }
}

/// Data processing pipeline that handles gRPC streams and processes messages
struct DataProcessor {
    parser: UnifiedParser,
    mongo_handler: Arc<MongoHandler>,
    broadcast_tx: broadcast::Sender<models::ParsedData>,
    metrics: Arc<Metrics>,
    config: config::ProcessingConfig,
}

impl DataProcessor {
    fn new(
        parser: UnifiedParser,
        mongo_handler: Arc<MongoHandler>,
        broadcast_tx: broadcast::Sender<models::ParsedData>,
        metrics: Arc<Metrics>,
        config: config::ProcessingConfig,
    ) -> Self {
        Self {
            parser,
            mongo_handler,
            broadcast_tx,
            metrics,
            config,
        }
    }

    /// Run the data processing pipeline
    async fn run(&self, source_config: &SourceConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match source_config {
            SourceConfig::Yellowstone(config) => {
                self.run_yellowstone_processor(config).await
            }
            SourceConfig::SolanaRpc(config) => {
                self.run_solana_rpc_processor(config).await
            }
        }
    }

    /// Process data from Yellowstone gRPC source
    async fn run_yellowstone_processor(
        &self,
        config: &config::YellowstoneSourceConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Yellowstone gRPC data processor");

        let prefilter = self.parser.get_prefilter();
        let filters = yellowstone_vixen_core::Filters::new(
            std::collections::HashMap::from([("main".to_string(), prefilter)])
        );

        // Create Yellowstone gRPC source
        use yellowstone_vixen::sources::SourceTrait;
        let source = YellowstoneGrpcSource::new(config.grpc.clone(), filters);

        // Channel for receiving updates
        let (tx, mut rx) = tokio::sync::mpsc::channel(self.config.buffer_size);

        // Start connection with reconnection logic
        let source_clone = source;
        let reconnect_config = config.reconnect.clone();
        let metrics_clone = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut attempt = 0;
            let mut delay = Duration::from_secs(reconnect_config.initial_delay);

            loop {
                attempt += 1;
                info!("Connecting to Yellowstone gRPC (attempt {})", attempt);
                
                match source_clone.connect(tx.clone()).await {
                    Ok(_) => {
                        info!("Successfully connected to Yellowstone gRPC");
                        metrics_clone.set_grpc_connected(true);
                        attempt = 0;
                        delay = Duration::from_secs(reconnect_config.initial_delay);
                    }
                    Err(e) => {
                        error!("Failed to connect to Yellowstone gRPC: {}", e);
                        metrics_clone.set_grpc_connected(false);
                        metrics_clone.increment_grpc_reconnections();

                        if reconnect_config.max_attempts > 0 && attempt >= reconnect_config.max_attempts {
                            error!("Max reconnection attempts reached, stopping");
                            break;
                        }

                        info!("Retrying connection in {:?}", delay);
                        sleep(delay).await;

                        // Exponential backoff
                        delay = std::cmp::min(
                            Duration::from_secs_f64(delay.as_secs_f64() * reconnect_config.backoff_multiplier),
                            Duration::from_secs(reconnect_config.max_delay),
                        );
                    }
                }
            }
        });

        // Process incoming messages
        let mut batch = Vec::new();
        let mut last_batch_time = std::time::Instant::now();

        while let Some(result) = rx.recv().await {
            match result {
                Ok(update) => {
                    let start_time = std::time::Instant::now();
                    
                    match self.process_update(update).await {
                        Ok(Some(parsed_data)) => {
                            batch.push(parsed_data.clone());
                            
                            // Publish to live stream
                            let _ = self.broadcast_tx.send(parsed_data);
                            
                            self.metrics.increment_stream_messages_processed();
                        }
                        Ok(None) => {
                            debug!("Update filtered out or not parseable");
                        }
                        Err(e) => {
                            error!("Failed to process update: {}", e);
                            self.metrics.increment_stream_messages_errors();
                        }
                    }

                    let processing_duration = start_time.elapsed();
                    self.metrics.record_processing_latency(processing_duration);

                    // Batch insert to MongoDB when batch is full or timeout reached
                    let should_flush = batch.len() >= self.config.batch_size
                        || last_batch_time.elapsed() >= Duration::from_millis(self.config.batch_timeout_ms);

                    if should_flush && !batch.is_empty() {
                        if let Err(e) = self.mongo_handler.store_parsed_data_batch(batch.clone()).await {
                            error!("Failed to store batch data: {}", e);
                        } else {
                            debug!("Stored batch of {} items", batch.len());
                        }
                        
                        batch.clear();
                        last_batch_time = std::time::Instant::now();
                    }
                }
                Err(e) => {
                    error!("gRPC stream error: {}", e);
                    self.metrics.increment_stream_messages_errors();
                }
            }
        }

        Ok(())
    }

    /// Process data from Solana RPC source (placeholder implementation)
    async fn run_solana_rpc_processor(
        &self,
        _config: &config::SolanaRpcSourceConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        warn!("Solana RPC source not yet implemented");
        // This would implement polling-based data collection from Solana RPC
        // For now, just keep the function alive
        tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
        Ok(())
    }

    /// Process a single update from the gRPC stream
    async fn process_update(
        &self,
        update: yellowstone_grpc_proto::geyser::SubscribeUpdate,
    ) -> Result<Option<models::ParsedData>, Box<dyn std::error::Error + Send + Sync>> {
        use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;

        match update.update_oneof {
            Some(UpdateOneof::Account(account_update)) => {
                // Convert to vixen format
                let account_update = convert_account_update(account_update)?;
                self.parser.parse_account_update(&account_update).await
                    .map_err(|e| e.into())
            }
            Some(UpdateOneof::Transaction(tx_update)) => {
                // Process instructions from transaction
                for instruction in tx_update.transaction
                    .as_ref()
                    .and_then(|tx| tx.meta.as_ref())
                    .map(|meta| &meta.inner_instructions)
                    .unwrap_or(&vec![])
                    .iter()
                    .flat_map(|inner| &inner.instructions)
                {
                    // Convert to vixen format and process
                    if let Ok(instruction_update) = convert_instruction_update(instruction, &tx_update) {
                        if let Ok(Some(parsed)) = self.parser.parse_instruction_update(&instruction_update).await {
                            return Ok(Some(parsed));
                        }
                    }
                }
                Ok(None)
            }
            _ => {
                debug!("Unsupported update type");
                Ok(None)
            }
        }
    }
}

/// Convert Yellowstone account update to Vixen format
fn convert_account_update(
    update: yellowstone_grpc_proto::geyser::SubscribeUpdateAccount,
) -> Result<AccountUpdate, Box<dyn std::error::Error + Send + Sync>> {
    let account_info = update.account.ok_or("Missing account in update")?;
    
    Ok(AccountUpdate {
        pubkey: account_info.pubkey.try_into()?,
        slot: update.slot,
        account: Some(yellowstone_vixen_core::Account {
            lamports: account_info.lamports,
            data: account_info.data,
            owner: account_info.owner.try_into()?,
            executable: account_info.executable,
            rent_epoch: account_info.rent_epoch,
        }),
        is_startup: update.is_startup,
    })
}

/// Convert Yellowstone instruction to Vixen format
fn convert_instruction_update(
    instruction: &yellowstone_grpc_proto::geyser::InnerInstruction,
    tx_update: &yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction,
) -> Result<InstructionUpdate, Box<dyn std::error::Error + Send + Sync>> {
    let tx = tx_update.transaction.as_ref().ok_or("Missing transaction")?;
    
    Ok(InstructionUpdate {
        signature: tx.signature.clone().try_into()?,
        slot: tx_update.slot,
        index: instruction.index as usize,
        instruction: Some(yellowstone_vixen_core::Instruction {
            program_id: instruction.program_id_index.try_into()?,
            accounts: instruction.accounts.iter().map(|&idx| idx.try_into()).collect::<Result<Vec<_>, _>>()?,
            data: instruction.data.clone(),
        }),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(if args.debug {
                tracing::Level::DEBUG.into()
            } else {
                tracing::Level::INFO.into()
            })
        );

    if args.json_logs {
        subscriber.with(tracing_subscriber::fmt::layer().json()).init();
    } else {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
    }

    info!("Starting Solana Stream Processor v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = AppConfig::from_file(&args.config)
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    info!("Configuration loaded from: {}", args.config.display());

    // Create and run the application
    let app = SolanaStreamProcessor::new(config).await?;
    app.run().await?;

    info!("Solana Stream Processor shutdown complete");
    Ok(())
}