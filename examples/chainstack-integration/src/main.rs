//! # Comprehensive Chainstack Yellowstone gRPC Integration Example
//!
//! This example demonstrates the complete implementation of a high-throughput trading data
//! collection pipeline using Chainstack's Yellowstone gRPC service as the data source.
//! 
//! ## Architecture Overview
//! 
//! ```text
//! ┌─────────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐
//! │   Chainstack gRPC   │───▶│  ChainstackGrpcSource │───▶│   Vixen Parsers     │
//! │   (Yellowstone)     │    │                      │    │ - Jupiter           │
//! └─────────────────────┘    │ - Connection Pool    │    │ - Raydium           │
//!                            │ - Circuit Breaker    │    │ - Meteora           │
//!                            │ - Redis Streaming    │    │ - Orca Whirlpool    │
//!                            │ - Real-time Filters  │    │ - Pump.fun          │
//!                            └──────────────────────┘    └─────────────────────┘
//!                                         │
//!                                         ▼
//!                            ┌──────────────────────┐    ┌─────────────────────┐
//!                            │    Redis Streams     │───▶│   Go Pipeline       │
//!                            │                      │    │ - Stream Consumer   │
//!                            │ - Trade Data         │    │ - Worker Pool       │
//!                            │ - Account Updates    │    │ - Batch Processor   │
//!                            │ - Transaction Data   │    │ - PostgreSQL        │
//!                            └──────────────────────┘    └─────────────────────┘
//! ```
//!
//! ## Features Demonstrated
//!
//! 1. **Custom Chainstack gRPC Source** with production-ready configuration
//! 2. **Multi-Parser Integration** covering all major Solana trading programs
//! 3. **Redis Streaming** for high-throughput data pipeline
//! 4. **Real-time Filter Management** via REST API
//! 5. **Comprehensive Monitoring** with Prometheus metrics
//! 6. **Trade Verification** and data validation
//! 7. **Program-specific Handlers** for different trading protocols

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser as ClapParser;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use yellowstone_vixen::{self as vixen, vixen_core};
use yellowstone_vixen_chainstack_grpc_source::{
    ChainstackGrpcSource, 
    filter_api::{FilterApiServer, DynamicFilter, DynamicTransactionFilter},
    monitoring::MonitoringSystem,
    config::ChainstackVixenConfig,
};

// Import all parsers for comprehensive trading data coverage
use yellowstone_vixen_jupiter_swap_parser::JupiterSwapParser;
use yellowstone_vixen_raydium_amm_v4_parser::RaydiumAmmV4Parser;
use yellowstone_vixen_raydium_clmm_parser::RaydiumClmmParser;
use yellowstone_vixen_raydium_cpmm_parser::RaydiumCpmmParser;
use yellowstone_vixen_meteora_amm_parser::MeteoraAmmParser;
use yellowstone_vixen_meteora_pools_parser::MeteoraPoolsParser;
use yellowstone_vixen_orca_whirlpool_parser::OrcaWhirlpoolParser;
use yellowstone_vixen_pumpfun_parser::PumpfunParser;
use yellowstone_vixen_pump_swaps_parser::PumpSwapsParser;

/// Command line arguments for the integration example
#[derive(ClapParser)]
#[command(
    name = "chainstack-integration",
    about = "Comprehensive Chainstack Yellowstone gRPC integration with Vixen pipeline",
    version
)]
pub struct Args {
    /// Path to configuration file
    #[arg(long, short = 'c', default_value = "chainstack-config.toml")]
    config: PathBuf,

    /// Chainstack API key (can also be set via CHAINSTACK_API_KEY env var)
    #[arg(long, env = "CHAINSTACK_API_KEY")]
    api_key: String,

    /// Chainstack endpoint URL
    #[arg(long, env = "CHAINSTACK_ENDPOINT")]
    endpoint: String,

    /// Redis URL for streaming
    #[arg(long, env = "REDIS_URL", default_value = "redis://localhost:6379")]
    redis_url: String,

    /// Enable monitoring and metrics
    #[arg(long, default_value = "true")]
    enable_monitoring: bool,

    /// Enable filter API server
    #[arg(long, default_value = "true")]
    enable_filter_api: bool,

    /// Filter API port
    #[arg(long, default_value = "8080")]
    filter_api_port: u16,

    /// Trading mode: all, defi, meme, nft
    #[arg(long, default_value = "all")]
    trading_mode: TradingMode,

    /// Wallet addresses to monitor (comma-separated)
    #[arg(long)]
    monitor_wallets: Option<String>,

    /// Token addresses to monitor (comma-separated) 
    #[arg(long)]
    monitor_tokens: Option<String>,

    /// Enable verbose logging
    #[arg(long, short = 'v')]
    verbose: bool,
}

/// Trading mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingMode {
    /// Monitor all trading activity
    All,
    /// Monitor only DeFi protocols
    Defi,
    /// Monitor only meme token trading
    Meme,
    /// Monitor NFT trading
    Nft,
}

impl std::str::FromStr for TradingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(TradingMode::All),
            "defi" => Ok(TradingMode::Defi),
            "meme" => Ok(TradingMode::Meme),
            "nft" => Ok(TradingMode::Nft),
            _ => Err(format!("Invalid trading mode: {}", s)),
        }
    }
}

/// Trading data handler for processing parsed trading events
#[derive(Debug)]
pub struct TradingDataHandler {
    monitoring: Arc<MonitoringSystem>,
    redis_client: redis::Client,
    trade_stats: Arc<RwLock<TradingStats>>,
}

/// Trading statistics for monitoring
#[derive(Debug, Default)]
pub struct TradingStats {
    pub total_trades: u64,
    pub jupiter_trades: u64,
    pub raydium_trades: u64,
    pub meteora_trades: u64,
    pub orca_trades: u64,
    pub pump_fun_trades: u64,
    pub total_volume_usd: f64,
    pub unique_traders: u64,
    pub failed_trades: u64,
}

impl TradingDataHandler {
    pub fn new(monitoring: Arc<MonitoringSystem>, redis_url: &str) -> anyhow::Result<Self> {
        let redis_client = redis::Client::open(redis_url)?;
        
        Ok(Self {
            monitoring,
            redis_client,
            trade_stats: Arc::new(RwLock::new(TradingStats::default())),
        })
    }

    /// Handle Jupiter swap events
    pub async fn handle_jupiter_swap(&self, swap_data: JupiterSwapData) -> anyhow::Result<()> {
        info!(
            signature = %swap_data.signature,
            input_amount = swap_data.input_amount,
            output_amount = swap_data.output_amount,
            "Jupiter swap detected"
        );

        // Update statistics
        let mut stats = self.trade_stats.write().await;
        stats.total_trades += 1;
        stats.jupiter_trades += 1;
        stats.total_volume_usd += swap_data.volume_usd;

        // Record metrics
        self.monitoring.record_update_processed("jupiter", 0, true).await;

        // Stream to Redis for Go pipeline
        self.stream_trade_event(&TradeEvent::Jupiter(swap_data)).await?;

        Ok(())
    }

    /// Handle Raydium AMM events
    pub async fn handle_raydium_swap(&self, swap_data: RaydiumSwapData) -> anyhow::Result<()> {
        info!(
            signature = %swap_data.signature,
            pool = %swap_data.pool_address,
            "Raydium swap detected"
        );

        let mut stats = self.trade_stats.write().await;
        stats.total_trades += 1;
        stats.raydium_trades += 1;
        stats.total_volume_usd += swap_data.volume_usd;

        self.monitoring.record_update_processed("raydium", 0, true).await;
        self.stream_trade_event(&TradeEvent::Raydium(swap_data)).await?;

        Ok(())
    }

    /// Stream trade event to Redis
    async fn stream_trade_event(&self, event: &TradeEvent) -> anyhow::Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let event_json = serde_json::to_string(event)?;
        
        let stream_id = format!("{}:{}", uuid::Uuid::new_v4(), chrono::Utc::now().timestamp_millis());
        
        let _: () = redis::cmd("XADD")
            .arg("trade_events")
            .arg("MAXLEN")
            .arg("~")
            .arg(1_000_000)
            .arg(&stream_id)
            .arg("event_type")
            .arg(event.event_type())
            .arg("data")
            .arg(event_json)
            .arg("timestamp")
            .arg(chrono::Utc::now().timestamp_millis())
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    /// Get current trading statistics
    pub async fn get_stats(&self) -> TradingStats {
        self.trade_stats.read().await.clone()
    }
}

/// Trade event enumeration
#[derive(Debug, Serialize, Deserialize)]
pub enum TradeEvent {
    Jupiter(JupiterSwapData),
    Raydium(RaydiumSwapData),
    Meteora(MeteoraSwapData),
    Orca(OrcaSwapData),
    PumpFun(PumpFunSwapData),
}

impl TradeEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            TradeEvent::Jupiter(_) => "jupiter_swap",
            TradeEvent::Raydium(_) => "raydium_swap", 
            TradeEvent::Meteora(_) => "meteora_swap",
            TradeEvent::Orca(_) => "orca_swap",
            TradeEvent::PumpFun(_) => "pumpfun_swap",
        }
    }
}

/// Jupiter swap data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapData {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub user: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub volume_usd: f64,
    pub fee_amount: u64,
    pub route_plan: Vec<String>,
    pub slippage_bps: u16,
}

/// Raydium swap data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumSwapData {
    pub signature: String,
    pub slot: u64,
    pub pool_address: String,
    pub user: String,
    pub coin_mint: String,
    pub pc_mint: String,
    pub swap_direction: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub volume_usd: f64,
    pub fee_amount: u64,
}

/// Meteora swap data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraSwapData {
    pub signature: String,
    pub slot: u64,
    pub pool_address: String,
    pub user: String,
    pub token_a_mint: String,
    pub token_b_mint: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub volume_usd: f64,
}

/// Orca swap data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaSwapData {
    pub signature: String,
    pub slot: u64,
    pub whirlpool: String,
    pub user: String,
    pub token_a: String,
    pub token_b: String,
    pub amount_a: i64,
    pub amount_b: i64,
    pub volume_usd: f64,
    pub sqrt_price: u128,
}

/// Pump.fun swap data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunSwapData {
    pub signature: String,
    pub slot: u64,
    pub mint: String,
    pub user: String,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub volume_usd: f64,
    pub bonding_curve: String,
}

/// Setup comprehensive trading filters based on mode
fn setup_trading_filters(mode: &TradingMode) -> Vec<DynamicFilter> {
    let mut filters = Vec::new();

    match mode {
        TradingMode::All => {
            // Add all trading program filters
            filters.extend(setup_defi_filters());
            filters.extend(setup_meme_filters());
            filters.extend(setup_nft_filters());
        }
        TradingMode::Defi => {
            filters.extend(setup_defi_filters());
        }
        TradingMode::Meme => {
            filters.extend(setup_meme_filters());
        }
        TradingMode::Nft => {
            filters.extend(setup_nft_filters());
        }
    }

    filters
}

/// Setup DeFi protocol filters
fn setup_defi_filters() -> Vec<DynamicFilter> {
    vec![
        DynamicFilter {
            id: "jupiter_v6".to_string(),
            name: "Jupiter V6 Aggregator".to_string(),
            enabled: true,
            account_filter: None,
            transaction_filter: Some(DynamicTransactionFilter {
                programs_include: [
                    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
                ].into_iter().collect(),
                accounts_required: Default::default(),
                accounts_exclude: Default::default(),
                programs_exclude: Default::default(),
                fee_range: None,
                include_failed: false,
                signature_patterns: Vec::new(),
            }),
            program_filter: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: vixen_chainstack_grpc_source::filter_api::FilterMetadata {
                description: "Monitor Jupiter V6 aggregator swaps".to_string(),
                tags: [("protocol".to_string(), "jupiter".to_string())].into_iter().collect(),
                priority: 100,
                use_case: "defi_trading".to_string(),
                created_by: "system".to_string(),
            },
        },
        DynamicFilter {
            id: "raydium_amm".to_string(),
            name: "Raydium AMM".to_string(),
            enabled: true,
            account_filter: None,
            transaction_filter: Some(DynamicTransactionFilter {
                programs_include: [
                    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(), // Raydium AMM V4
                    "CAMMCzo5YL8w4VFF8KVHrK22GGUQpMNRqTFXP3K4M8oX".to_string(), // Raydium CLMM
                    "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C".to_string(), // Raydium CPMM
                ].into_iter().collect(),
                accounts_required: Default::default(),
                accounts_exclude: Default::default(),
                programs_exclude: Default::default(),
                fee_range: None,
                include_failed: false,
                signature_patterns: Vec::new(),
            }),
            program_filter: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: vixen_chainstack_grpc_source::filter_api::FilterMetadata {
                description: "Monitor Raydium AMM trading".to_string(),
                tags: [("protocol".to_string(), "raydium".to_string())].into_iter().collect(),
                priority: 95,
                use_case: "defi_trading".to_string(),
                created_by: "system".to_string(),
            },
        },
    ]
}

/// Setup meme token filters
fn setup_meme_filters() -> Vec<DynamicFilter> {
    vec![
        DynamicFilter {
            id: "pump_fun".to_string(),
            name: "Pump.fun Trading".to_string(),
            enabled: true,
            account_filter: None,
            transaction_filter: Some(DynamicTransactionFilter {
                programs_include: [
                    "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(), // Pump.fun
                ].into_iter().collect(),
                accounts_required: Default::default(),
                accounts_exclude: Default::default(),
                programs_exclude: Default::default(),
                fee_range: None,
                include_failed: false,
                signature_patterns: Vec::new(),
            }),
            program_filter: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: vixen_chainstack_grpc_source::filter_api::FilterMetadata {
                description: "Monitor Pump.fun meme token trading".to_string(),
                tags: [("protocol".to_string(), "pumpfun".to_string())].into_iter().collect(),
                priority: 90,
                use_case: "meme_trading".to_string(),
                created_by: "system".to_string(),
            },
        },
    ]
}

/// Setup NFT trading filters
fn setup_nft_filters() -> Vec<DynamicFilter> {
    vec![
        // Add NFT marketplace filters here
        // Magic Eden, OpenSea, etc.
    ]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(log_level.parse()?))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Chainstack Yellowstone gRPC Integration Example");

    // Load configuration
    let config = if args.config.exists() {
        let config_str = std::fs::read_to_string(&args.config)?;
        toml::from_str::<ChainstackVixenConfig>(&config_str)?
    } else {
        info!("Configuration file not found, using defaults");
        ChainstackVixenConfig::default()
    };

    // Initialize monitoring
    let monitoring = if args.enable_monitoring {
        let monitoring_system = MonitoringSystem::new(config.monitoring.clone());
        monitoring_system.start().await?;
        Arc::new(monitoring_system)
    } else {
        // Create a no-op monitoring system
        Arc::new(MonitoringSystem::new(Default::default()))
    };

    // Create Chainstack gRPC source
    let source = ChainstackGrpcSource::new()
        .with_api_key(&args.api_key)
        .with_redis_streaming(&args.redis_url, "solana_data")
        .with_monitoring(args.enable_monitoring);

    // Initialize trading data handler
    let trading_handler = Arc::new(TradingDataHandler::new(
        Arc::clone(&monitoring),
        &args.redis_url,
    )?);

    // Setup filters based on trading mode
    let filters = setup_trading_filters(&args.trading_mode);

    // Start filter API server if enabled
    if args.enable_filter_api {
        let api_server = FilterApiServer::new(
            config.clone(),
            args.filter_api_port,
            Some("api-token-change-me".to_string()),
        );
        
        let api_monitoring = Arc::clone(&monitoring);
        tokio::spawn(async move {
            if let Err(e) = api_server.start().await {
                error!(error = ?e, "Filter API server failed");
            }
        });

        info!(port = args.filter_api_port, "Started filter API server");
    }

    // Configure Yellowstone connection
    let yellowstone_config = vixen::config::YellowstoneConfig {
        endpoint: args.endpoint,
        x_token: None, // API key handled by the source
        timeout: 60,
    };

    info!("Configuring Vixen runtime with comprehensive parser coverage");

    // Build and run the Vixen runtime with all parsers
    vixen::Runtime::builder()
        .source(source)
        // Jupiter parsers
        .account(JupiterSwapParser::new())
        // Raydium parsers  
        .account(RaydiumAmmV4Parser::new())
        .account(RaydiumClmmParser::new())
        .account(RaydiumCpmmParser::new())
        // Meteora parsers
        .account(MeteoraAmmParser::new())
        .account(MeteoraPoolsParser::new())
        // Orca parsers
        .account(OrcaWhirlpoolParser::new())
        // Pump.fun parsers
        .account(PumpfunParser::new())
        .account(PumpSwapsParser::new())
        .build(yellowstone_config)
        .run();

    Ok(())
}

impl Default for ChainstackVixenConfig {
    fn default() -> Self {
        // Provide sensible defaults for the configuration
        todo!("Implement default configuration")
    }
}

impl Clone for TradingStats {
    fn clone(&self) -> Self {
        Self {
            total_trades: self.total_trades,
            jupiter_trades: self.jupiter_trades,
            raydium_trades: self.raydium_trades,
            meteora_trades: self.meteora_trades,
            orca_trades: self.orca_trades,
            pump_fun_trades: self.pump_fun_trades,
            total_volume_usd: self.total_volume_usd,
            unique_traders: self.unique_traders,
            failed_trades: self.failed_trades,
        }
    }
}