# Yellowstone Vixen Documentation

Welcome to the comprehensive documentation for Yellowstone Vixen, a powerful framework for building program-aware, real-time Solana data pipelines.

## Quick Start

- **[Main README](../README.md)**: Overview, features, and quick start guide
- **[Configuration Examples](../Vixen.example.toml)**: Example configuration file
- **[Stream Parser Example](../examples/stream-parser/README.md)**: Complete streaming example
- **[Prometheus Example](../examples/prometheus/README.md)**: Metrics integration example

## Core Documentation

### 📚 Architecture & Design
- **[Architecture Guide](./ARCHITECTURE.md)**: Detailed system architecture, design principles, and component overview
- **[Configuration Reference](./CONFIGURATION.md)**: Complete configuration options and environment variables

### 🛠 Development
- **[Developer Guide](./DEVELOPER_GUIDE.md)**: Creating custom parsers, handlers, and extending Vixen
- **[Troubleshooting Guide](./TROUBLESHOOTING.md)**: Common issues and solutions

## Examples & Tutorials

### Real-World Examples
- **[Stream Parser](../examples/stream-parser/)**: Multi-program gRPC streaming server
- **[Prometheus Metrics](../examples/prometheus/)**: Metrics collection and monitoring
- **[OpenTelemetry](../examples/opentelemetry/)**: Distributed tracing integration
- **[Tracing](../examples/streams-tracing/)**: Advanced debugging and profiling

### Component Documentation
- **[Parser Library](../crates/parser/README.md)**: Built-in SPL Token and Token Extension parsers
- **[Mock Testing](../crates/mock/README.md)**: Offline testing with fixtures
- **[Source Connectors](../crates/yellowstone-grpc-source/README.md)**: Data source implementations

## Key Features

### 🏗 Core Architecture
- **Parser + Handler Pattern**: Modular, composable data processing pipelines
- **Multiple Data Sources**: Yellowstone gRPC, Solana RPC, snapshots, and mock data
- **Advanced Filtering**: Transaction-level filtering with account inclusion/exclusion
- **Shared Context**: Access to transaction signatures, slot numbers, and metadata

### 📊 Monitoring & Observability
- **Prometheus Integration**: Built-in metrics with custom dashboards
- **OpenTelemetry Support**: Distributed tracing and advanced telemetry
- **Performance Monitoring**: Latency, throughput, and error tracking
- **Health Checks**: Comprehensive status monitoring

### 🔄 Real-time Streaming
- **gRPC API**: Type-safe streaming with protocol buffers
- **Multi-client Support**: Serve multiple consumers simultaneously  
- **Backpressure Handling**: Automatic flow control and buffering
- **Client Libraries**: Generated clients for multiple languages

### 🧪 Development & Testing
- **Mock Framework**: Offline testing with real devnet data
- **Fixture Management**: Reusable test data and scenarios
- **Performance Testing**: Benchmarking and optimization tools
- **Debug Tooling**: Comprehensive logging and tracing

## Supported Programs

Yellowstone Vixen includes parsers for 30+ Solana programs:

| Category | Programs | Parser Crates |
|----------|----------|---------------|
| **Core SPL** | Token, Token Extensions, Associated Token | `yellowstone-vixen-parser` |
| **DEX Aggregators** | Jupiter v6 | `yellowstone-vixen-jupiter-swap-parser` |
| **AMM Protocols** | Raydium v4, Raydium CLMM, Raydium CPMM, Orca Whirlpools | `yellowstone-vixen-raydium-*-parser`, `yellowstone-vixen-orca-*-parser` |
| **Meteora Ecosystem** | DLMM, DAMM v2, Pools, Vault, DBC | `yellowstone-vixen-meteora-*-parser` |
| **Launchpads** | Pump.fun, Moonshot, Raydium Launchpad | `yellowstone-vixen-*-parser` |
| **DeFi Protocols** | Kamino Limit Orders | `yellowstone-vixen-kamino-*-parser` |

## Getting Started

### 1. Installation

```bash
# Add to your Cargo.toml
[dependencies]
yellowstone-vixen = "0.4.0"
yellowstone-vixen-yellowstone-grpc-source = "0.2.0"
yellowstone-vixen-parser = "0.4.0"  # For SPL Token parsers
```

### 2. Basic Setup

```rust
use yellowstone_vixen::{Runtime, Pipeline};
use yellowstone_vixen_parser::token_program::{AccountParser, InstructionParser};
use yellowstone_vixen_yellowstone_grpc_source::YellowstoneGrpcSource;

fn main() {
    Runtime::builder()
        .source(YellowstoneGrpcSource::new())
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .metrics(yellowstone_vixen::metrics::Prometheus)
        .build(config)
        .run();
}
```

### 3. Configuration

```toml
# Vixen.toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "your-api-token"

[metrics]
endpoint = "0.0.0.0:9090"
```

### 4. Running

```bash
RUST_LOG=info cargo run -- --config Vixen.toml
```

## Development Workflow

### 1. Parser Development
1. **Study the Program**: Understand the Solana program's account and instruction layouts
2. **Define Structures**: Create Rust structs for account and instruction data
3. **Implement Parsers**: Implement the `Parser` trait for account and instruction parsing
4. **Add Protocol Buffers**: Define protobuf messages for gRPC streaming
5. **Write Tests**: Use the mock framework for comprehensive testing

### 2. Handler Development
1. **Define Business Logic**: Implement the `Handler` trait for custom processing
2. **Add Error Handling**: Handle errors gracefully with proper recovery
3. **Implement Metrics**: Add custom metrics for monitoring and alerting
4. **Performance Testing**: Benchmark and optimize for production workloads

### 3. Integration Testing
1. **Mock Data**: Use fixtures for repeatable integration tests
2. **End-to-end Testing**: Test complete pipelines with realistic data
3. **Performance Testing**: Load testing with production-like scenarios
4. **Monitoring Setup**: Configure metrics and alerting for production

## Best Practices

### Performance
- **Early Filtering**: Use prefilters and early returns to reduce processing overhead
- **Efficient Parsing**: Minimize allocations and use zero-copy techniques where possible
- **Async Design**: Leverage async/await for high-concurrency workloads
- **Resource Management**: Monitor memory usage and configure appropriate limits

### Reliability
- **Error Isolation**: Ensure errors in one component don't affect others
- **Graceful Degradation**: Continue processing even when individual components fail
- **Monitoring**: Implement comprehensive metrics and alerting
- **Testing**: Use extensive testing with real-world data scenarios

### Security
- **Credential Management**: Use environment variables for sensitive configuration
- **Input Validation**: Validate all parsed data and handle malicious inputs
- **Resource Limits**: Set appropriate limits to prevent resource exhaustion
- **Access Control**: Secure gRPC endpoints and metrics endpoints appropriately

## Contributing

We welcome contributions to Yellowstone Vixen! Here's how to get started:

1. **Fork the Repository**: Create your own fork on GitHub
2. **Create a Parser**: Follow the [Developer Guide](./DEVELOPER_GUIDE.md) to create a new parser
3. **Add Tests**: Include comprehensive tests using the mock framework
4. **Update Documentation**: Add examples and documentation for your parser
5. **Submit a Pull Request**: Follow the contribution guidelines

### Parser Contributions
- **High-Quality Parsers**: We prioritize well-tested, production-ready parsers
- **Popular Programs**: Focus on widely-used Solana programs
- **Complete Implementation**: Include both account and instruction parsers
- **Protocol Buffer Support**: Add protobuf definitions for gRPC streaming

## Support & Community

- **GitHub Issues**: Report bugs and request features
- **Discussions**: Join community discussions and ask questions
- **Documentation**: Contribute to documentation and examples
- **Enterprise Support**: Contact [ABK Labs](https://abklabs.com/) or [Triton One](https://triton.one/) for commercial support

## License

Yellowstone Vixen is open source software. See the [LICENSE](../LICENSE) file for details.

---

**Ready to build powerful Solana data pipelines?** Start with the [Quick Start Guide](../README.md#quick-start) or explore the [examples](../examples/) to see Vixen in action!