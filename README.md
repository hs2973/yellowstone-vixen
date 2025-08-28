# Yellowstone Vixen

Yellowstone Vixen is a powerful framework for building program-aware, real-time Solana data pipelines. It provides the core components—runtime, parser definitions, and handler interfaces—needed to transform raw on-chain events into structured, actionable data with enterprise-grade reliability and performance.

Vixen consumes Dragon's Mouth gRPC streams and routes program-specific change events through pluggable parsers, enabling developers to log, store, or stream enriched data for indexing, analytics, and downstream consumption.

## Table of Contents

- [Yellowstone Vixen](#yellowstone-vixen)
  - [Table of Contents](#table-of-contents)
  - [Problem Solving](#problem-solving)
  - [Core Features](#core-features)
  - [Architecture Overview](#architecture-overview)
  - [Quick Start](#quick-start)
  - [Advanced Usage](#advanced-usage)
  - [Supported Programs](#supported-programs)
  - [Configuration](#configuration)
  - [Data Sources](#data-sources)
  - [gRPC Streaming API](#grpc-streaming-api)
  - [Metrics & Observability](#metrics--observability)
  - [Testing & Development](#testing--development)
  - [Examples](#examples)
  - [Developer Resources](#developer-resources)
  - [Maintainers](#maintainers)

## Problem Solving

Yellowstone Vixen solves core challenges for Solana dApp developers:

- **Cost Efficiency**: Share Dragon's Mouth subscriptions and filter only the data you care about, reducing infrastructure costs
- **Operational Simplicity**: Lightweight setup with minimal external dependencies and straightforward configuration
- **High Performance**: Built for enterprise workloads with efficient memory usage and high throughput processing
- **Real-time Processing**: Stream processing with sub-second latency for time-critical applications
- **Observability**: Built-in Prometheus and OpenTelemetry metrics for lag, throughput, and error tracking
- **Composability**: Independent, reusable parser crates that can deserialize complex cross-program interactions (CPI)
- **Developer Experience**: Rich tooling for testing, debugging, and developing custom parsers

## Core Features

### 🛠 Parser + Handler Architecture
Build modular pipelines that transform raw Solana events into structured models and trigger custom business logic. The architecture supports:
- **Account Parsers**: Decode on-chain account data into typed structures
- **Instruction Parsers**: Parse transaction instructions with program-specific logic
- **Transaction Parsers**: Process complete transaction metadata and context
- **Block Metadata Parsers**: Extract block-level information and timing data
- **Custom Handlers**: Implement custom logic for data processing, storage, and notifications

### 🔥 Multiple Data Source Support
Connect to various Solana data sources with unified interfaces:
- **Dragon's Mouth Integration**: Subscribe to Solana Geyser streams via gRPC with minimal configuration
- **Yellowstone gRPC**: Direct integration with Yellowstone infrastructure
- **Solana RPC**: Fallback to standard RPC endpoints for development and testing
- **Snapshot Sources**: Process historical data from Solana snapshots
- **Mock Sources**: Use fixtures for testing and development

### 📈 Enterprise-Grade Monitoring
Comprehensive observability with multiple metrics backends:
- **Prometheus**: Out-of-the-box metrics endpoint with custom dashboards
- **OpenTelemetry**: Industry-standard telemetry for distributed tracing
- **Custom Metrics**: Implement your own metrics collection and reporting

### 🧪 Advanced Testing Framework
Robust testing capabilities for reliable development:
- **Offline Testing with Fixtures**: Test parsers without connecting to live Solana nodes
- **Account Replay**: Replay real devnet account updates for comprehensive testing
- **Transaction Replay**: Test instruction parsing with real transaction data
- **Mock Data Generation**: Create synthetic test data for edge case testing

### 🔄 High-Performance gRPC Streaming
Serve parsed program events directly to external systems:
- **Real-time Streaming**: Sub-second latency for time-critical applications
- **Protocol Buffers**: Efficient serialization with type safety
- **Multiple Streams**: Serve different program types on separate streams
- **Client Libraries**: Generated client libraries for multiple languages

### 🎯 Advanced Filtering & Customization
Powerful filtering and customization capabilities:
- **FilterPipeline**: Apply custom transaction filters with account inclusion/exclusion
- **Prefilters**: Optimize performance by filtering at the source level
- **Custom Parsers**: Build program-specific parsers with shared components
- **Shared Data Features**: Access transaction-wide context like signatures and slot numbers

## Architecture Overview

Yellowstone Vixen follows a modular, pipeline-based architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Data Source   │───▶│     Runtime     │───▶│     Parsers     │───▶│    Handlers     │
│ (Dragon's Mouth)│    │   (Buffering)   │    │ (Program Logic) │    │ (Custom Logic)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                                              │
                                ▼                                              ▼
                       ┌─────────────────┐                          ┌─────────────────┐
                       │     Metrics     │                          │   gRPC Stream   │
                       │   (Prometheus)  │                          │   (Optional)    │
                       └─────────────────┘                          └─────────────────┘
```

### Core Components

1. **Runtime (`yellowstone-vixen`)**: Main orchestration engine that manages data flow, buffering, and pipeline execution
2. **Core (`yellowstone-vixen-core`)**: Foundational types, traits, and parsing utilities shared across the ecosystem
3. **Stream (`yellowstone-vixen-stream`)**: gRPC server implementation for real-time data streaming
4. **Sources**: Pluggable data source implementations (Yellowstone gRPC, Solana RPC, etc.)
5. **Parsers**: Program-specific parsing logic for 30+ Solana programs
6. **Proto**: Protocol buffer definitions for type-safe gRPC communication

## Quick Start

### Basic Pipeline Example

```rust
use std::path::PathBuf;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use yellowstone_vixen::Pipeline;
use yellowstone_vixen_parser::token_program::{AccountParser, InstructionParser};
use yellowstone_vixen_yellowstone_grpc_source::YellowstoneGrpcSource;

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

#[derive(Debug)]
pub struct Logger;

impl<V: std::fmt::Debug + Sync> yellowstone_vixen::Handler<V> for Logger {
    async fn handle(&self, value: &V) -> yellowstone_vixen::HandlerResult<()> {
        tracing::info!(?value, "Parsed data");
        Ok(())
    }
}

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config).expect("Error parsing config");

    yellowstone_vixen::Runtime::builder()
        .source(YellowstoneGrpcSource::new())
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .metrics(yellowstone_vixen::metrics::Prometheus)
        .commitment_level(yellowstone_vixen::CommitmentLevel::Confirmed)
        .build(config)
        .run();
}
```

### Configuration File (`Vixen.toml`)

```toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "your-api-token-here"
timeout = 60

[metrics]
endpoint = "0.0.0.0:9090"
```

### Running the Pipeline

```bash
# Set log level and run
RUST_LOG=info cargo run -- --config "./Vixen.toml"

# Access Prometheus metrics
curl http://localhost:9090/metrics
```

## Advanced Usage

### Custom Transaction Filtering

Use `FilterPipeline` for advanced transaction filtering based on account inclusion:

```rust
use yellowstone_vixen::filter_pipeline::FilterPipeline;
use yellowstone_vixen_core::{Prefilter, Pubkey};
use std::str::FromStr;

// Create a filtered pipeline for Raydium AMM v4
let filtered_pipeline = FilterPipeline::new(
    RaydiumAmmV4IxParser,
    [CustomHandler],
    Prefilter::builder()
        .transaction_accounts_include([
            Pubkey::from_str("CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW").unwrap(),
        ])
        .transaction_accounts_required([
            Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(),
        ])
        .build()
);

yellowstone_vixen::Runtime::builder()
    .instruction(filtered_pipeline)
    .build(config)
    .run();
```

### Multiple Program Parsers

Parse multiple programs simultaneously with shared transaction context:

```rust
use yellowstone_vixen_raydium_amm_v4_parser::{
    accounts_parser::AccountParser as RaydiumAccParser,
    instructions_parser::InstructionParser as RaydiumIxParser,
};
use yellowstone_vixen_jupiter_swap_parser::{
    accounts_parser::AccountParser as JupiterAccParser,
    instructions_parser::InstructionParser as JupiterIxParser,
};

yellowstone_vixen::Runtime::builder()
    // Raydium parsers
    .account(Pipeline::new(RaydiumAccParser, [DatabaseHandler, MetricsHandler]))
    .instruction(Pipeline::new(RaydiumIxParser, [SwapHandler, NotificationHandler]))
    
    // Jupiter parsers
    .account(Pipeline::new(JupiterAccParser, [DatabaseHandler]))
    .instruction(Pipeline::new(JupiterIxParser, [ArbitrageHandler]))
    
    // Shared components
    .metrics(yellowstone_vixen::metrics::Prometheus)
    .build(config)
    .run();
```

### Custom Handler Implementation

Implement custom business logic with the Handler trait:

```rust
use yellowstone_vixen::{Handler, HandlerResult};
use yellowstone_vixen_raydium_amm_v4_parser::RaydiumAmmV4ProgramIx;

#[derive(Debug)]
pub struct SwapAnalyzer {
    min_volume: u64,
}

impl Handler<RaydiumAmmV4ProgramIx> for SwapAnalyzer {
    async fn handle(&self, ix: &RaydiumAmmV4ProgramIx) -> HandlerResult<()> {
        match ix {
            RaydiumAmmV4ProgramIx::SwapBaseIn(accounts, data) => {
                if data.amount_in > self.min_volume {
                    // Process high-volume swap
                    self.process_large_swap(accounts, data).await?;
                }
            }
            RaydiumAmmV4ProgramIx::SwapBaseOut(accounts, data) => {
                // Handle different swap type
                self.process_swap_out(accounts, data).await?;
            }
            _ => {} // Ignore other instruction types
        }
        Ok(())
    }
}
```

### Stream Server with gRPC API

Set up a streaming server to serve parsed data via gRPC:

```rust
use yellowstone_vixen_stream::Server;
use yellowstone_vixen_yellowstone_grpc_source::YellowstoneGrpcSource;

fn main() {
    let config = load_config();
    
    Server::<_, YellowstoneGrpcSource>::builder()
        // Add protocol buffer descriptor sets
        .descriptor_set(parser::token::DESCRIPTOR_SET)
        .descriptor_set(RAYDIUM_AMM_V4_DESCRIPTOR_SET)
        
        // Configure parsers
        .account(Proto::new(TokenProgramAccParser))
        .instruction(Proto::new(RaydiumAmmV4IxParser))
        
        // Build and run
        .build(config)
        .run();
}
```

## Supported Programs

Yellowstone Vixen includes parsers for 30+ popular Solana programs:

| Address | Program Name | Parser Crate |
|---------|--------------|--------------|
| `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | **SPL Token Program** | `yellowstone-vixen-parser` |
| `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | **SPL Token Extensions** | `yellowstone-vixen-parser` |
| `boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4` | **Boop.fun** | `yellowstone-vixen-boop-parser` |
| `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | **Jupiter Aggregator v6** | `yellowstone-vixen-jupiter-swap-parser` |
| `LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF` | **Kamino Limit Order** | `yellowstone-vixen-kamino-limit-orders-parser` |
| `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG` | **Meteora DAMM v2** | `yellowstone-vixen-meteora-amm-parser` |
| `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN` | **Meteora Dynamic Bonding Curve** | `yellowstone-vixen-meteora-dbc-parser` |
| `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | **Meteora DLMM** | `yellowstone-vixen-meteora-parser` |
| `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB` | **Meteora Pools** | `yellowstone-vixen-meteora-pools-parser` |
| `24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi` | **Meteora Vault** | `yellowstone-vixen-meteora-vault-parser` |
| `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG` | **Moonshot** | `yellowstone-vixen-moonshot-parser` |
| `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` | **Whirlpools** | `yellowstone-vixen-orca-whirlpool-parser` |
| `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` | **Pump.fun AMM** | `yellowstone-vixen-pump-swaps-parser` |
| `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P` | **Pump.fun** | `yellowstone-vixen-pumpfun-parser` |
| `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` | **Raydium Liquidity Pool V4** | `yellowstone-vixen-raydium-amm-v4-parser` |
| `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` | **Raydium Concentrated Liquidity** | `yellowstone-vixen-raydium-clmm-parser` |
| `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C` | **Raydium CPMM** | `yellowstone-vixen-raydium-cpmm-parser` |
| `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj` | **Raydium Launchpad** | `yellowstone-vixen-raydium-launchpad-parser` |
| `5U3EU2ubXtK84QcRjWVmYt9RaDyA8gKxdUrPFXmZyaki` | **Virtuals** | `yellowstone-vixen-virtuals-parser` |

Each parser provides both account and instruction parsing capabilities, along with protocol buffer definitions for type-safe gRPC streaming.

## Configuration

### Complete Configuration Reference

```toml
# Data source configuration
[source]
# Yellowstone gRPC endpoint
endpoint = "https://yellowstone-api.triton.one"
# Authentication token
x-token = "your-api-token-here"  
# Connection timeout in seconds
timeout = 60
# Optional: Custom gRPC keepalive settings
keepalive-time = 30
keepalive-timeout = 5

# Alternative: Fumarole source configuration
# [source]
# endpoint = "https://fumarole-api.triton.one"
# x-token = "your-token"
# subscriber-name = "my_subscriber"

# Metrics configuration
[metrics]
# Prometheus endpoint (optional)
endpoint = "0.0.0.0:9090"
# Export interval in seconds
export-interval = 60
# Basic auth (optional)
username = "admin"
password = "secret"

# Buffer configuration
[buffer]
# Buffer size for processing pipeline
buffer-size = 1000
# Number of worker threads
num-workers = 4
# Batch processing size
batch-size = 100

# gRPC server configuration (for stream server)
[grpc]
# gRPC server address
address = "[::]:3030"
```

### Environment Variables

You can override configuration values using environment variables:

```bash
# Source configuration
export ENDPOINT="https://your-yellowstone-endpoint.com"
export X_TOKEN="your-auth-token"
export TIMEOUT="120"

# Metrics configuration  
export METRICS_ENDPOINT="0.0.0.0:9090"
export METRICS_EXPORT_INTERVAL="30"

# gRPC configuration
export GRPC_ADDRESS="0.0.0.0:3030"
```

## Data Sources

Yellowstone Vixen supports multiple data sources through a pluggable architecture:

### Yellowstone gRPC Source
The primary data source for production deployments:

```rust
use yellowstone_vixen_yellowstone_grpc_source::YellowstoneGrpcSource;

yellowstone_vixen::Runtime::builder()
    .source(YellowstoneGrpcSource::new())
    .build(config)
    .run();
```

### Solana RPC Source
For development and testing:

```rust
use yellowstone_vixen_solana_rpc_source::SolanaRpcSource;

yellowstone_vixen::Runtime::builder()
    .source(SolanaRpcSource::new())
    .build(config)
    .run();
```

### Snapshot Source
For historical data processing:

```rust
use yellowstone_vixen_solana_snapshot_source::SnapshotSource;

yellowstone_vixen::Runtime::builder()
    .source(SnapshotSource::new("path/to/snapshot"))
    .build(config)
    .run();
```

### Mock Source
For testing and development:

```rust
use yellowstone_vixen_mock::MockSource;

yellowstone_vixen::Runtime::builder()
    .source(MockSource::from_fixtures("fixtures/"))
    .build(config)
    .run();
```

## gRPC Streaming API

The stream server provides real-time access to parsed program data via gRPC:

### Starting the Stream Server

```rust
use yellowstone_vixen_stream::Server;
use yellowstone_vixen_yellowstone_grpc_source::YellowstoneGrpcSource;

Server::<_, YellowstoneGrpcSource>::builder()
    .descriptor_set(RAYDIUM_AMM_V4_DESCRIPTOR_SET)
    .descriptor_set(JUPITER_SWAP_DESCRIPTOR_SET)
    .account(Proto::new(RaydiumAmmV4AccParser))
    .instruction(Proto::new(RaydiumAmmV4IxParser))
    .instruction(Proto::new(JupiterSwapIxParser))
    .build(config)
    .run();
```

### Client Usage Examples

#### Using grpcurl

```bash
# List available services
grpcurl -plaintext localhost:3030 list

# Describe the streaming interface
grpcurl -plaintext localhost:3030 describe vixen.stream.ProgramStreams

# Subscribe to Raydium AMM v4 events
grpcurl -plaintext -d '{"program": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"}' \
    localhost:3030 vixen.stream.ProgramStreams/Subscribe
```

#### Generated Client Libraries

Clients can be generated for multiple languages using the protocol buffer definitions:

```bash
# Generate Python client
python -m grpc_tools.protoc --python_out=. --grpc_python_out=. \
    --proto_path=proto stream.proto

# Generate JavaScript client  
npx grpc_tools_node_protoc --js_out=import_style=commonjs,binary:. \
    --grpc_out=grpc_js:. --proto_path=proto stream.proto
```

## Metrics & Observability

### Prometheus Integration

Yellowstone Vixen provides comprehensive Prometheus metrics out of the box:

```rust
yellowstone_vixen::Runtime::builder()
    .metrics(yellowstone_vixen::metrics::Prometheus)
    .build(config)
    .run();
```

#### Available Metrics

- `vixen_transactions_processed_total`: Total number of transactions processed
- `vixen_accounts_processed_total`: Total number of accounts processed  
- `vixen_instructions_processed_total`: Total number of instructions processed
- `vixen_parse_errors_total`: Total number of parsing errors
- `vixen_handler_errors_total`: Total number of handler errors
- `vixen_processing_duration_seconds`: Time spent processing events
- `vixen_pipeline_lag_seconds`: Lag between event time and processing time

### OpenTelemetry Integration

For distributed tracing and advanced telemetry:

```rust
use yellowstone_vixen::metrics::OpenTelemetry;
use opentelemetry::global;

let meter_provider = /* your OpenTelemetry setup */;

yellowstone_vixen::Runtime::builder()
    .metrics(OpenTelemetry::new(meter_provider))
    .build(config)
    .run();
```

### Custom Metrics

Implement custom metrics collection:

```rust
use yellowstone_vixen::metrics::{Counter, Instrumenter};

#[derive(Debug)]
pub struct CustomMetrics {
    swap_counter: Box<dyn Counter>,
}

impl Handler<SwapInstruction> for CustomMetrics {
    async fn handle(&self, swap: &SwapInstruction) -> HandlerResult<()> {
        self.swap_counter.inc();
        Ok(())
    }
}
```

## Testing & Development

### Offline Testing with Mock Data

Use the mock framework for comprehensive testing:

```rust
#[cfg(test)]
mod tests {
    use yellowstone_vixen_mock::{account_fixture, tx_fixture};
    use yellowstone_vixen_raydium_amm_v4_parser::AccountParser;

    #[tokio::test]
    async fn test_raydium_pool_parsing() {
        let parser = AccountParser;
        let account = account_fixture!("pool_address", &parser);
        
        // Verify parsing results
        assert_eq!(account.token_a_mint, expected_mint_a);
        assert_eq!(account.token_b_mint, expected_mint_b);
    }

    #[tokio::test]
    async fn test_swap_instruction_parsing() {
        let parser = InstructionParser;
        let instructions = tx_fixture!("swap_transaction_signature", &parser);
        
        // Verify instruction parsing
        assert!(instructions.len() > 0);
        match &instructions[0] {
            SwapInstruction::SwapBaseIn(accounts, data) => {
                assert_eq!(data.amount_in, expected_amount);
            }
            _ => panic!("Unexpected instruction type"),
        }
    }
}
```

### Integration Testing

Test complete pipelines with mock sources:

```rust
#[tokio::test]
async fn test_complete_pipeline() {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    
    let handler = TestHandler::new(tx);
    let mock_source = MockSource::from_fixtures("test_fixtures/");
    
    let runtime = yellowstone_vixen::Runtime::builder()
        .source(mock_source)
        .account(Pipeline::new(AccountParser, [handler]))
        .build(test_config())
        .spawn();
        
    // Verify received events
    let events: Vec<_> = rx.collect().await;
    assert_eq!(events.len(), expected_count);
}
```

### Development Workflow

1. **Create Fixtures**: Capture real devnet data for testing
2. **Write Tests**: Use the mock framework for unit and integration tests
3. **Local Development**: Use Solana RPC source for local testing
4. **Staging**: Deploy with Yellowstone gRPC in staging environment
5. **Production**: Scale with enterprise Yellowstone infrastructure

## Examples

### [Stream Parser Example](./examples/stream-parser/)
Comprehensive example showing:
- Multiple program parsers (Raydium, Jupiter, Meteora, etc.)
- gRPC streaming server setup
- Advanced filtering with FilterPipeline
- Real-time data processing

```bash
cd examples/stream-parser
RUST_LOG=info cargo run -- --config ../../Vixen.toml
```

### [Prometheus Metrics Example](./examples/prometheus/)
Demonstrates metrics integration:
- Prometheus metrics collection
- Custom metrics implementation
- Dashboard setup with Docker Compose

```bash
cd examples/prometheus
cargo run -- --config config.toml
```

### [OpenTelemetry Example](./examples/opentelemetry/)
Shows distributed tracing setup:
- OpenTelemetry integration
- Jaeger tracing
- Performance monitoring

```bash
cd examples/opentelemetry  
cargo run -- --config config.toml
```

### [Streams Tracing Example](./examples/streams-tracing/)
Advanced tracing and debugging:
- Detailed execution tracing
- Performance profiling
- Debug logging strategies

## Developer Resources

### Creating Custom Parsers

1. **Define Account Structures**: Create Rust structs for account data
2. **Implement Parser Trait**: Parse raw account data into structured format
3. **Add Protocol Buffers**: Define protobuf messages for gRPC streaming
4. **Generate Descriptor Sets**: Create binary descriptor sets for the stream server
5. **Write Tests**: Use mock framework for comprehensive testing

Example parser structure:
```rust
// Define account structure
#[derive(Debug, Clone)]
pub struct MyProgramAccount {
    pub field1: u64,
    pub field2: Pubkey,
}

// Implement parser
pub struct AccountParser;

impl Parser for AccountParser {
    type Input = AccountUpdate;
    type Output = MyProgramAccount;
    
    async fn parse(&self, account: &AccountUpdate) -> Result<Self::Output, ParseError> {
        // Custom parsing logic
        parse_account_data(&account.account.data)
    }
}
```

### Best Practices

1. **Error Handling**: Use `ParseError::Filtered` to skip irrelevant updates
2. **Performance**: Implement efficient parsing with minimal allocations
3. **Testing**: Create comprehensive test suites with real devnet data
4. **Monitoring**: Add custom metrics for business-specific KPIs
5. **Documentation**: Document parser behavior and expected account formats

### Contributing

- [Contributing Guide](./CONTRIBUTING.md)
- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Parser Generation Guide](./codama-parser-generation.md)

## Maintainers

This project is developed by [ABK Labs](https://abklabs.com/) and [Triton One](https://triton.one/).

For enterprise support and custom development, contact the maintainers.