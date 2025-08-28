# Quick Start

This guide will get you up and running with Yellowstone Vixen in minutes. We'll build a simple pipeline that parses Solana Token Program events and logs them.

## Prerequisites

- Rust (nightly-2024-02-01 or later)
- A Yellowstone Dragon's Mouth endpoint (gRPC)

## Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/rpcpool/yellowstone-vixen.git
   cd yellowstone-vixen
   ```

2. **Set the Rust toolchain:**
   ```bash
   rustup toolchain install nightly-2024-02-01
   rustup override set nightly-2024-02-01
   ```

3. **Build the project:**
   ```bash
   cargo build --release
   ```

## Your First Pipeline

Create a new Rust project or add the following to an existing one:

```rust
use std::path::PathBuf;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use yellowstone_vixen::Pipeline;
use yellowstone_vixen_parser::token_program::{AccountParser, InstructionParser};

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
        tracing::info!(?value);
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
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .metrics(yellowstone_vixen::metrics::Prometheus)
        .commitment_level(yellowstone_vixen::CommitmentLevel::Confirmed)
        .build(config)
        .run();
}
```

## Configuration

Create a `Vixen.toml` configuration file:

```toml
[grpc]
endpoint = "http://localhost:10000"
token = "your-auth-token"

[[programs]]
name = "Token Program"
address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

[[programs]]
name = "Associated Token Program"
address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
```

## Running Your Pipeline

1. **Start your Dragon's Mouth endpoint** (see [Dragon's Mouth setup](../operations/deployment.md))

2. **Run the pipeline:**
   ```bash
   RUST_LOG=info cargo run -- --config "./Vixen.toml"
   ```

3. **View metrics** at `http://localhost:9090/metrics`

## What's Next?

- Learn about [parsers and handlers](../architecture/core-concepts.md)
- Explore [configuration options](../getting-started/configuration.md)
- Check out [examples](../examples/) for more complex pipelines
- See [supported programs](../programs/supported-programs.md) for available parsers

## Troubleshooting

- **Connection issues**: Ensure your Dragon's Mouth endpoint is running and accessible
- **No events**: Check that the program addresses in your config are correct
- **Performance**: Adjust batch sizes and buffer configurations for your use case

For more help, see the [troubleshooting guide](../operations/troubleshooting.md) or open an issue on GitHub.
