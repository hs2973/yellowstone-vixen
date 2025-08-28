# Yellowstone Vixen Documentation

Welcome to Yellowstone Vixen, a high-performance Rust framework for building Solana data pipelines using Yellowstone gRPC streams. This documentation provides comprehensive guidance for understanding, using, and contributing to the framework.

## 🚀 Quick Start

New to Yellowstone Vixen? Start here to get up and running quickly.

- **[Quick Start Guide](./getting-started/quick-start.md)** - Get your first pipeline running in minutes
- **[Installation](./getting-started/installation.md)** - Install dependencies and set up your environment
- **[Configuration](./getting-started/configuration.md)** - Configure your pipelines and runtime

## 🏗️ Architecture

Understand the core concepts and architecture that make Yellowstone Vixen powerful.

- **[Core Concepts](./architecture/core-concepts.md)** - Fundamental building blocks and patterns
- **[Runtime Architecture](./architecture/runtime.md)** - How the runtime orchestrates your pipelines
- **[Data Flow](./architecture/data-flow.md)** - How data moves through your pipelines
- **[Error Handling](./architecture/error-handling.md)** - Robust error handling and recovery

## �️ Development

Everything you need to develop with and contribute to Yellowstone Vixen.

- **[Contributing](./development/contributing.md)** - How to contribute to the project
- **[Creating Parsers](./development/creating-parsers.md)** - Build custom parsers for Solana programs
- **[Testing](./development/testing.md)** - Comprehensive testing strategies and best practices
- **[Codama Integration](./development/codama-integration.md)** - Automated parser generation with Codama

## � API Reference

Complete API documentation for all components.

- **[Parsers](./api/parsers.md)** - Instruction and account parser interfaces
- **[Handlers](./api/handlers.md)** - Handler trait and built-in implementations
- **[Runtime](./api/runtime.md)** - Runtime configuration and pipeline management
- **[Metrics](./api/metrics.md)** - Metrics collection and monitoring APIs

## 📋 Supported Programs

Documentation for all supported Solana programs and protocols.

- **[Supported Programs](./programs/supported-programs.md)** - Complete list of supported programs
- **[Adding Programs](./programs/adding-programs.md)** - How to add support for new programs

## � Examples

Practical examples to help you build real-world applications.

- **[Basic Pipeline](./examples/basic-pipeline.md)** - Simple pipeline with logging
- **[Database Integration](./examples/database-integration.md)** - Store parsed data in databases
- **[Metrics and Monitoring](./examples/metrics-monitoring.md)** - Set up observability
- **[Custom Handlers](./examples/custom-handlers.md)** - Build custom data processors

## 🔧 Advanced Topics

Deep dives into advanced features and optimization techniques.

- **[Performance Optimization](./advanced/performance.md)** - Optimize for high-throughput scenarios
- **[Custom Transports](./advanced/custom-transports.md)** - Implement custom data transport layers
- **[Plugin Architecture](./advanced/plugins.md)** - Extend Yellowstone Vixen with plugins
- **[Troubleshooting](./advanced/troubleshooting.md)** - Debug and resolve common issues

## � Reference

Additional reference materials and specifications.

- **[Configuration Schema](./reference/configuration-schema.md)** - Complete configuration reference
- **[Error Codes](./reference/error-codes.md)** - All error types and their meanings
- **[Migration Guide](./reference/migration.md)** - Migrate between versions
- **[Changelog](./reference/changelog.md)** - Version history and changes

## 🤝 Community & Support

- **GitHub Repository**: [rpcpool/yellowstone-vixen](https://github.com/rpcpool/yellowstone-vixen)
- **Issues**: [Report bugs and request features](https://github.com/rpcpool/yellowstone-vixen/issues)
- **Discussions**: [Community discussions](https://github.com/rpcpool/yellowstone-vixen/discussions)
- **Dragon's Mouth Documentation**: [Yellowstone gRPC streams](https://docs.triton.one/project-yellowstone/dragons-mouth-grpc-subscriptions)

## 📄 License

Yellowstone Vixen is licensed under the MIT License. See [LICENSE](../LICENSE) for details.

---

## 🎯 Key Features

Yellowstone Vixen provides:

- **🚀 High Performance** - Built in Rust with async processing for maximum throughput
- **🔧 Modular Architecture** - Pluggable parsers and handlers for flexible pipelines
- **📊 Built-in Observability** - Prometheus metrics and OpenTelemetry tracing
- **🛡️ Robust Error Handling** - Comprehensive error recovery and monitoring
- **🔄 Real-time Processing** - Process Solana data as it happens
- **📈 Scalable** - Handle high-volume data streams with ease
- **� Extensible** - Easy to add support for new programs and protocols

## 🏃‍♂️ Getting Started in 3 Steps

1. **Install Dependencies**
   ```bash
   cargo add yellowstone-vixen
   ```

2. **Create Your Pipeline**
   ```rust
   use yellowstone_vixen::{Pipeline, Logger};

   let pipeline = Pipeline::new(
       MyParser,
       vec![Logger.boxed()]
   );
   ```

3. **Run It**
   ```rust
   pipeline.run().await?;
   ```

That's it! You're now processing Solana data in real-time.

## 📈 Use Cases

Yellowstone Vixen is perfect for:

- **DeFi Analytics** - Track DEX trades, liquidity changes, and protocol metrics
- **NFT Marketplaces** - Monitor NFT sales, listings, and collection activity
- **Gaming Analytics** - Track in-game transactions and player activity
- **Wallet Services** - Monitor wallet activity and transaction patterns
- **Risk Monitoring** - Detect suspicious activity and anomalies
- **Research** - Analyze on-chain data for academic or business research

## �🤝 Contributing

We welcome contributions! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated. See our [Contributing Guide](./development/contributing.md) to get started.

## 📞 Support

Need help? Here's how to get support:

1. **Check the Docs** - Most questions are answered in this documentation
2. **Search Issues** - Check existing GitHub issues for similar problems
3. **Ask the Community** - Join our Discord or GitHub Discussions
4. **Report Bugs** - Use GitHub Issues for bugs and feature requests

## 🙏 Acknowledgments

Yellowstone Vixen builds upon the excellent work of:

- **Yellowstone** - High-performance Solana gRPC streams
- **Anchor** - Solana smart contract framework
- **Tokio** - Async runtime for Rust
- **Prometheus** - Metrics collection and monitoring
- **OpenTelemetry** - Observability and tracing

This project is developed by [ABK Labs](https://abklabs.com/) and [Triton One](https://triton.one/).

---

*This documentation is continuously updated. If you find errors or have suggestions for improvement, please [open an issue](https://github.com/rpcpool/yellowstone-vixen/issues) or [submit a pull request](https://github.com/rpcpool/yellowstone-vixen/pulls).*

*For the main README, see [README.md](../README.md) in the project root.*
