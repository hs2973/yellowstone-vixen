#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    clippy::clone_on_ref_ptr
)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Solana Stream Processor
//! 
//! A real-time Solana data processing application built on yellowstone-vixen.
//! Processes blockchain data through custom filters and outputs to both SSE streams
//! and MongoDB for immediate access and historical analysis.

use std::path::PathBuf;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod models;
mod metrics;
mod web;

use config::Config;
use handlers::unified_handler::UnifiedHandler;
use web::server::WebServer;

#[derive(clap::Parser)]
#[command(name = "solana-stream-processor")]
#[command(version, author, about = "Real-time Solana stream processor")]
pub struct Opts {
    /// Path to configuration file
    #[arg(long, short, default_value = "config.toml")]
    config: PathBuf,
    
    /// Web server port
    #[arg(long, default_value = "8080")]
    port: u16,
    
    /// MongoDB connection string
    #[arg(long, env = "MONGODB_URI")]
    mongodb_uri: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Solana Stream Processor");

    let opts = Opts::parse();
    
    // Load configuration
    let config_content = std::fs::read_to_string(&opts.config)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", opts.config.display(), e))?;
    
    let mut config: Config = toml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;
    
    // Override with command line arguments
    if let Some(mongodb_uri) = opts.mongodb_uri {
        config.mongodb_uri = mongodb_uri;
    }
    config.web_server_port = opts.port;

    tracing::info!("Configuration loaded successfully");
    tracing::info!("Web server will start on port: {}", config.web_server_port);

    // Initialize metrics
    let metrics_registry = metrics::initialize_metrics();
    
    // Start web server in background
    let web_server = WebServer::new(config.web_server_port, metrics_registry.clone());
    let sse_sender = web_server.get_sse_sender();
    
    // Start web server
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web_server.start().await {
            tracing::error!("Web server error: {}", e);
        }
    });

    // Create unified handler
    let unified_handler = UnifiedHandler::new(
        sse_sender,
        &config.mongodb_uri,
        metrics_registry
    ).await?;

    tracing::info!("Unified handler initialized successfully");

    // For this initial implementation, we'll run the web server and demonstrate
    // the SSE and MongoDB functionality without the full Vixen pipeline
    // The Vixen integration will be added once the core dependencies are resolved
    
    tracing::info!("Solana Stream Processor started successfully");
    tracing::info!("SSE endpoint available at: http://localhost:{}/events/stream", config.web_server_port);
    tracing::info!("Health check available at: http://localhost:{}/health", config.web_server_port);
    tracing::info!("Metrics available at: http://localhost:{}/metrics", config.web_server_port);
    
    // Demonstrate the system by sending some test data
    let test_handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // Send test instruction data every 10 seconds
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        let mut counter = 0;
        
        loop {
            interval.tick().await;
            counter += 1;
            
            if let Err(e) = unified_handler.handle_instruction_data(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                &format!("test_signature_{}", counter),
                "transfer",
                serde_json::json!({
                    "accounts": {
                        "mint": "So11111111111111111111111111111111111111112"
                    },
                    "amount": 1000000
                }),
                123456789 + counter,
                chrono::Utc::now().timestamp(),
            ).await {
                tracing::error!("Failed to process test data: {}", e);
            } else {
                tracing::info!("Sent test instruction #{}", counter);
            }
        }
    });
    
    // Wait for either the web server or test handler to complete
    tokio::select! {
        result = web_handle => {
            if let Err(e) = result {
                tracing::error!("Web server task failed: {}", e);
            }
        }
        result = test_handle => {
            if let Err(e) = result {
                tracing::error!("Test handler task failed: {}", e);
            }
        }
    }

    Ok(())
}