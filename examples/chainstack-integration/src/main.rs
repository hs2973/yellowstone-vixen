#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    clippy::clone_on_ref_ptr
)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{net::SocketAddr, path::PathBuf};

use clap::Parser as _;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use yellowstone_vixen::{self as vixen, vixen_core::proto::Proto};
use yellowstone_vixen_chainstack_grpc_source::{
    ChainstackGrpcSource, RedisWriterConfig,
};

// Import parser dependencies like stream-parser example
use yellowstone_vixen_jupiter_swap_parser::{
    accounts_parser::AccountParser as JupiterSwapAccParser,
    instructions_parser::InstructionParser as JupiterSwapIxParser,
    proto_def::DESCRIPTOR_SET as JUPITER_SWAP_DESCRIPTOR_SET,
};
use yellowstone_vixen_meteora_parser::{
    accounts_parser::AccountParser as MeteoraAccParser,
    instructions_parser::InstructionParser as MeteoraIxParser,
    proto_def::DESCRIPTOR_SET as METEORA_DESCRIPTOR_SET,
};
use yellowstone_vixen_pumpfun_parser::{
    accounts_parser::AccountParser as PumpfunAccParser,
    instructions_parser::InstructionParser as PumpfunIxParser,
    proto_def::DESCRIPTOR_SET as PUMP_DESCRIPTOR_SET,
};
use yellowstone_vixen_raydium_amm_v4_parser::{
    accounts_parser::AccountParser as RaydiumAmmV4AccParser,
    proto_def::DESCRIPTOR_SET as RAYDIUM_AMM_V4_DESCRIPTOR_SET,
};
use yellowstone_vixen_raydium_clmm_parser::{
    accounts_parser::AccountParser as RaydiumClmmAccParser,
    instructions_parser::InstructionParser as RaydiumClmmIxParser,
    proto_def::DESCRIPTOR_SET as RAYDIUM_CLMM_DESCRIPTOR_SET,
};
use yellowstone_vixen_parser::{
    token_program::{
        AccountParser as TokenProgramAccParser, InstructionParser as TokenProgramIxParser,
    },
};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
    
    #[arg(long, env = "CHAINSTACK_API_KEY")]
    api_key: Option<String>,
    
    #[arg(long)]
    redis_url: Option<String>,
    
    #[arg(long, default_value = "127.0.0.1:8080")]
    filter_api_addr: SocketAddr,
    
    #[arg(long, default_value = "100")]
    redis_batch_size: usize,
    
    #[arg(long, default_value = "1000000")]
    redis_max_entries: usize,
}

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { 
        config, 
        api_key, 
        redis_url, 
        filter_api_addr, 
        redis_batch_size, 
        redis_max_entries 
    } = Opts::parse();
    
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let mut yellowstone_config = toml::from_str(&config).expect("Error parsing config");
    
    // Set API key via x_token if provided (Phase 1 pattern)
    if let Some(api_key) = api_key {
        yellowstone_config.x_token = Some(api_key);
    }

    // Create source with Phase 2 & 3 optimizations
    let mut source = ChainstackGrpcSource::new();
    
    // Phase 3: Essential Redis streaming with optimized configuration
    if let Some(redis_url) = redis_url {
        let redis_config = RedisWriterConfig {
            redis_url,
            stream_name: "essential_transactions".to_string(),
            batch_size: redis_batch_size,
            max_stream_entries: redis_max_entries,
            ..Default::default()
        };
        
        source = source.with_essential_redis_streaming(redis_config)
            .expect("Failed to configure essential Redis streaming");
    }
    
    // Phase 2: Enable Filter API for real-time filter management
    source = source.with_filter_api(filter_api_addr);

    println!("🚀 Starting Chainstack Yellowstone gRPC integration");
    println!("📡 Filter API available at: http://{}", filter_api_addr);
    println!("🔧 Redis streaming: {}", if redis_url.is_some() { "enabled (essential transactions only)" } else { "disabled" });
    println!();
    println!("Filter API endpoints:");
    println!("  GET  /filters         - List current filters");
    println!("  POST /filters/update  - Update a filter");
    println!("  POST /filters/remove  - Remove a filter");
    println!();

    vixen::Runtime::builder()
        .source(source)
        .account(Proto::new(MeteoraAccParser))
        .account(Proto::new(PumpfunAccParser))
        .account(Proto::new(TokenProgramAccParser))
        .account(Proto::new(JupiterSwapAccParser))
        .account(Proto::new(RaydiumAmmV4AccParser))
        .account(Proto::new(RaydiumClmmAccParser))
        .instruction(Proto::new(MeteoraIxParser))
        .instruction(Proto::new(PumpfunIxParser))
        .instruction(Proto::new(TokenProgramIxParser))
        .instruction(Proto::new(JupiterSwapIxParser))
        .instruction(Proto::new(RaydiumClmmIxParser))
        .build(yellowstone_config)
        .run();
}