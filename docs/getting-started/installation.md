# Installation

This guide covers installing Yellowstone Vixen and its dependencies.

## System Requirements

- **Rust**: nightly-2024-02-01 or later
- **Operating System**: Linux, macOS, or Windows
- **Memory**: At least 4GB RAM recommended
- **Storage**: 2GB free space for build artifacts

## Installing Rust

If you don't have Rust installed:

1. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Set the correct toolchain:**
   ```bash
   rustup toolchain install nightly-2024-02-01
   rustup override set nightly-2024-02-01
   ```

3. **Verify installation:**
   ```bash
   rustc --version
   cargo --version
   ```

## Installing Yellowstone Vixen

### Option 1: From Source (Recommended)

1. **Clone the repository:**
   ```bash
   git clone https://github.com/rpcpool/yellowstone-vixen.git
   cd yellowstone-vixen
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Run tests (optional):**
   ```bash
   cargo test
   ```

### Option 2: As a Library Dependency

Add Yellowstone Vixen to your `Cargo.toml`:

```toml
[dependencies]
yellowstone-vixen = "0.4"
yellowstone-vixen-parser = "0.4"
# Add specific parsers as needed
yellowstone-vixen-jupiter-swap-parser = "0.4"
```

## Installing Dragon's Mouth

Yellowstone Vixen requires a Dragon's Mouth endpoint for Solana data. You have several options:

### Option 1: Self-Hosted (Recommended for Production)

1. **Install Yellowstone:**
   ```bash
   git clone https://github.com/rpcpool/yellowstone-grpc.git
   cd yellowstone-grpc
   cargo build --release
   ```

2. **Configure Yellowstone:**
   Create a `yellowstone-grpc.toml` config file with your Solana RPC endpoint.

3. **Run Yellowstone:**
   ```bash
   ./target/release/yellowstone-grpc --config yellowstone-grpc.toml
   ```

### Option 2: Commercial Provider

Use a commercial Dragon's Mouth provider like:
- [Triton One](https://triton.one/)
- [Helius](https://helius.xyz/)
- [GenesysGo](https://genesysgo.com/)

## Installing Monitoring Tools

### Prometheus (Optional)

For metrics collection:

1. **Using Docker:**
   ```bash
   docker run -d -p 9090:9090 prom/prometheus
   ```

2. **Using Docker Compose:**
   ```bash
   docker-compose up
   ```

### OpenTelemetry (Optional)

For distributed tracing:

```bash
cargo add opentelemetry
cargo add opentelemetry-otlp
```

## Development Tools

### For Parser Development

If you're developing custom parsers:

1. **Install Codama:**
   ```bash
   npm install -g @codama/cli
   ```

2. **Install Protocol Buffers:**
   ```bash
   # macOS
   brew install protobuf

   # Ubuntu/Debian
   sudo apt-get install protobuf-compiler

   # Or download from https://github.com/protocolbuffers/protobuf/releases
   ```

### IDE Setup

**VS Code:**
- Install the "rust-analyzer" extension
- Install the "CodeLLDB" extension for debugging

**Other Editors:**
- Follow the [Rust documentation](https://www.rust-lang.org/learn/get-started) for your editor

## Verifying Installation

Run this command to verify everything is working:

```bash
cargo run --bin yellowstone-vixen -- --help
```

You should see the help output for the Yellowstone Vixen CLI.

## Next Steps

- [Quick Start](../getting-started/quick-start.md) - Your first pipeline
- [Configuration](../getting-started/configuration.md) - Configure your setup
- [Examples](../examples/) - Learn from examples
