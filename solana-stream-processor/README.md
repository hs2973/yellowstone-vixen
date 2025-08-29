# Solana Stream Processor

A real-time Solana data processing application built on the `yellowstone-vixen` framework. This application processes blockchain data through custom filters and outputs to both Server-Sent Events (SSE) streams and MongoDB for immediate access and historical analysis.

## Overview

The Solana Stream Processor implements the requirements defined in `docs/prd.md` and follows the architecture outlined in `docs/solana-stream-processor-architecture.md`. It uses the `yellowstone-vixen` library as a dependency to create a real-time, filtered, and multi-output stream of Solana program data.

## Features

- **Real-time Processing**: Processes Solana blockchain data in real-time using yellowstone-vixen
- **Multi-output**: Simultaneously streams data via SSE and stores in MongoDB
- **Filtering**: Intelligent filtering to extract essential trading and account data
- **Monitoring**: Built-in Prometheus metrics for production monitoring
- **Scalable**: Asynchronous, non-blocking architecture for high throughput
- **Go Client**: Includes a Go client example for consuming SSE streams

## Architecture

```
[Yellowstone gRPC / Solana RPC]
             |
             v
[Vixen Runtime (from library)]
             |
             v
[Built-in Parsers (from library)] -> [SPL Token, Pump.fun, etc.]
             |
             v
[Unified Program Handler]
             |
             +---> [Filter & Simplify Logic]
             |
             +---> [SSE Streaming Logic (non-blocking)]
             |
             +---> [MongoDB Persistence Logic (async)]
```

## Quick Start

### Prerequisites

- Rust 1.70+
- MongoDB 6.0+
- Go 1.21+ (for client)
- Docker & Docker Compose (for monitoring stack)

### Installation

1. **Clone and build the application:**
   ```bash
   cd solana-stream-processor
   cargo build --release
   ```

2. **Start the monitoring stack:**
   ```bash
   docker-compose up -d
   ```

3. **Configure the application:**
   Edit `config.toml` to set your MongoDB URI and data sources.

4. **Run the processor:**
   ```bash
   cargo run --release -- --config config.toml
   ```

### Testing the SSE Stream

You can test the SSE stream using curl:
```bash
curl -N -H "Accept: text/event-stream" http://localhost:8080/events/stream
```

Or use the included Go client:
```bash
cd go-client
go run main.go
```

## Configuration

The application is configured via `config.toml`. Key settings include:

- `mongodb_uri`: MongoDB connection string
- `web_server_port`: Port for the web server (default: 8080)
- `vixen.rpc_endpoint`: Solana RPC endpoint
- `vixen.programs`: List of program IDs to monitor

## API Endpoints

- `GET /` - Service information
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /events/stream` - SSE stream of processed data

## Data Model

The application outputs simplified data structures containing essential fields:

```json
{
  "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
  "token_mint": "So11111111111111111111111111111111111111112",
  "transaction_signature": "3Kw2...",
  "instruction_type": "transfer",
  "instruction_data": {...},
  "blockchain_timestamp": 1640995200,
  "ingestion_timestamp": 1640995201,
  "slot": 123456789,
  "metadata": null
}
```

## Monitoring

The application includes comprehensive monitoring:

- **Prometheus Metrics**: Available at `/metrics`
- **Grafana Dashboard**: Access at http://localhost:3000 (admin/admin)
- **Health Checks**: Available at `/health`

Key metrics include:
- `stream_messages_processed_total`
- `stream_messages_filtered_total`
- `sse_messages_sent_total`
- `mongodb_writes_total`
- `processing_duration_seconds`

## Development

### Project Structure

```
solana-stream-processor/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error handling
│   ├── models.rs            # Data structures
│   ├── metrics.rs           # Prometheus metrics
│   ├── handlers/            # Data processing handlers
│   │   └── unified_handler.rs
│   └── web/                 # Web server and SSE
│       ├── server.rs
│       └── sse.rs
├── go-client/               # Go client example
├── config.toml              # Configuration file
├── docker-compose.yml       # Local development stack
└── prometheus.yml           # Prometheus configuration
```

### Building and Testing

```bash
# Build the application
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Build optimized release
cargo build --release
```

## Performance

The application is designed to handle:
- **Throughput**: 5,000+ messages per second
- **Latency**: <10ms for SSE streaming, <200ms for MongoDB persistence
- **Memory**: <2GB RAM under normal load

## Contributing

This application is built as an enhancement to the yellowstone-vixen library without modifying the core library code. When contributing:

1. Follow the existing code structure and patterns
2. Maintain compatibility with the yellowstone-vixen API
3. Add appropriate tests for new functionality
4. Update documentation as needed

## License

MIT License - see the main repository LICENSE file for details.

## Related Documentation

- [Product Requirements Document](../docs/prd.md)
- [Architecture Document](../docs/solana-stream-processor-architecture.md)
- [Yellowstone Vixen Library Analysis](../docs/yellowstone-vixen-architecture.md)