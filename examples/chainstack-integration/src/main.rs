#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    clippy::clone_on_ref_ptr
)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::path::PathBuf;

use clap::Parser as _;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use yellowstone_vixen::{self as vixen, vixen_core::proto::Proto};
use yellowstone_vixen_chainstack_grpc_source::ChainstackGrpcSource;

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
}

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config, api_key, redis_url } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let mut yellowstone_config = toml::from_str(&config).expect("Error parsing config");
    
    // Set API key via x_token if provided
    if let Some(api_key) = api_key {
        yellowstone_config.x_token = Some(api_key);
    }

    // Create source with optional Redis streaming
    let mut source = ChainstackGrpcSource::new();
    if let Some(redis_url) = redis_url {
        source = source.with_redis_streaming(redis_url, "chainstack_data");
    }

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