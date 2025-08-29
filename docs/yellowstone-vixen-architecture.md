# Yellowstone Vixen Library Brownfield Architecture

## Introduction

This document captures the CURRENT STATE of the `yellowstone-vixen` library codebase. It is created with the specific goal of enabling a developer to build the **Real-Time Solana Stream Processor** as defined in `docs/prd.md`.

This document focuses on the library's public interfaces, data flow, and extension points, particularly the `Pipeline` and `Handler` traits, which are central to the planned project. It treats the library as a "black box" to some extent, detailing *how to use it* rather than documenting every internal implementation detail.

### Document Scope

Focused on areas relevant to implementing a custom, multi-program stream processor with custom handlers for filtering, SSE, and MongoDB, as per the PRD.

### Change Log

| Date | Version | Description | Author |
| --- | --- | --- | --- |
| August 29, 2025 | 1.0 | Initial brownfield analysis for PRD implementation | Winston |

## Quick Reference - Key Files for Library Usage

To understand how to use this library, focus on the following files:

- **Core Handler Interface**: `crates/runtime/src/handler.rs` - Defines the `Handler` trait, the primary extension point for custom logic.
- **Pipeline Construction**: `crates/runtime/src/builder.rs` - Defines the `RuntimeBuilder` used to configure and create a processing pipeline.
- **Core Data Structures**: `crates/core/src/instruction.rs` - Defines the `InstructionUpdate` struct that handlers will receive.
- **Primary Usage Example**: `examples/stream-parser/src/main.rs` - Demonstrates how to initialize the runtime, configure sources, add parsers, and attach a handler.
- **Configuration**: `Vixen.example.toml` - Shows the configuration structure for data sources and other runtime options.

## High-Level Architecture

`yellowstone-vixen` is a Rust framework for building high-performance data processing pipelines for Solana blockchain data. Its architecture is designed to be modular and extensible.

The core data flow is as follows:

`[Data Source(s)] -> [Vixen Runtime] -> [Parsers] -> [Custom Handler(s)]`

1.  **Data Sources**: Connect to sources like Yellowstone gRPC or a standard Solana RPC to receive raw transaction/block data. These are configured in the `.toml` file.
2.  **Vixen Runtime**: The core engine that orchestrates the flow of data from sources to parsers and handlers. It manages connections, buffering, and message dispatching.
3.  **Parsers**: A collection of specialized crates (`*-parser`) that decode raw on-chain data for specific programs (e.g., `pumpfun-parser`, `moonshot-parser`) into structured Rust objects.
4.  **Handlers**: User-defined logic that processes the parsed data. This is where all custom business logic, like filtering, storing, or streaming, is implemented by creating structs that implement the `Handler` trait.

### Actual Tech Stack

| Category | Technology | Version/Path | Notes |
| --- | --- | --- | --- |
| Language | Rust | 1.70+ | As per `rust-toolchain.toml`. |
| Runtime | Tokio | | For asynchronous operations. |
| gRPC | `yellowstone-grpc-client` | 9 | For connecting to Yellowstone gRPC streams. |
| RPC | `solana-rpc-client` | | For connecting to standard Solana RPC endpoints. |
| Serialization | `serde`, `prost` | | For configuration and Protobuf message handling. |
| Core Framework | `yellowstone-vixen-runtime` | `crates/runtime` | The core pipeline and handler engine. |

### Repository Structure Reality Check

- **Type**: Monorepo containing multiple Rust crates.
- **Package Manager**: `cargo`.
- **Notable**: The project is organized into `crates` for different functionalities (core, runtime, parsers, sources). The `examples` directory is critical for understanding library usage.

## Source Tree and Module Organization

### Project Structure (Actual)

```text
yellowstone-vixen/
├── crates/
│   ├── core/            # Core data types (InstructionUpdate, Pubkey, etc.)
│   ├── runtime/         # Main pipeline engine, Handler trait, RuntimeBuilder
│   ├── parser/          # Generic SPL token parser and parsing helpers
│   ├── proto/           # Protobuf definitions for internal streaming format
│   ├── *-parser/        # Individual crates for parsing specific program data
│   ├── *-source/        # Crates for different data sources (gRPC, RPC)
│   └── mock/            # Mocking utilities for testing
├── examples/            # CRITICAL: Usage examples showing how to build applications
├── Vixen.example.toml   # Example configuration file
└── Cargo.toml           # Workspace definition
```

### Key Modules and Their Purpose

- **`crates/runtime`**: The heart of the framework. It provides the `Runtime` and `RuntimeBuilder` to construct a pipeline. Its most important component is the `Handler` trait in `handler.rs`.
- **`crates/core`**: Provides the fundamental data structures that flow through the system, like `TransactionUpdate` and `InstructionUpdate`. Handlers will primarily interact with these types.
- **`crates/parser`**: Contains the parsers for common programs like SPL Token.
- **`crates/*-parser`**: Each of these crates is responsible for parsing the instructions and accounts of a specific on-chain program (e.g., `pumpfun-parser` for Pump.fun). They are added to the runtime builder to enable processing for that program.
- **`crates/*-source`**: These crates implement the logic for connecting to different data sources. The selection of a source is typically handled via the configuration file.

## Data Models and APIs

The primary "API" of this library is the set of traits and structs a developer uses to build a pipeline.

### The `Handler` Trait

This is the main extension point. Any custom logic must be implemented in a struct that conforms to this trait, defined in `crates/runtime/src/handler.rs`.

A simplified view of the trait:

```rust
#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&mut self, msg: &Message) -> Result<()>;
}
```

- Your custom handlers (Filter, SSE, MongoDB) will each be a struct that implements `handle`.
- The `Message` enum contains different types of data, but for program processing, you will primarily match on `Message::Instruction(instruction)`.
- The `instruction` is of type `yellowstone_vixen_core::InstructionUpdate`, which contains the parsed data from one of the `*-parser` crates.

### The `RuntimeBuilder`

This is the entry point for building an application, found in `crates/runtime/src/builder.rs`. The typical workflow, as seen in `examples/stream-parser/src/main.rs`, is:

1.  Create a `RuntimeBuilder::new()`.
2.  Add parsers for each program you're interested in using `add_parser<T>()`.
3.  Add your custom handler using `add_handler()`. You can chain multiple handlers.
4.  Build the runtime with `build()`.
5.  Run the pipeline with `run()`.

## Development and Usage

### Local Development Setup

To use this library, you do not modify its code. Instead, you create a new Rust project and import the necessary crates as dependencies in your `Cargo.toml`.

Example `Cargo.toml` dependencies for the PRD project:

```toml
[dependencies]
yellowstone-vixen = { path = "path/to/yellowstone-vixen/crates/runtime" }
yellowstone-vixen-core = { path = "path/to/yellowstone-vixen/crates/core" }
yellowstone-vixen-parser = { path = "path/to/yellowstone-vixen/crates/parser" }
yellowstone-vixen-pumpfun-parser = { path = "path/to/yellowstone-vixen/crates/pumpfun-parser" }
yellowstone-vixen-moonshot-parser = { path = "path/to/yellowstone-vixen/crates/moonshot-parser" }
# ... other parsers and necessary crates like tokio, serde, etc.
```

### Configuration

The application is configured via a `.toml` file (e.g., `Vixen.toml`). This file defines:
- Which data source to use (`grpc` or `rpc`).
- Connection details for the selected source (endpoint, auth tokens).
- Filtering rules for the source (e.g., which program accounts to watch).

## PRD Impact Analysis: Building the Stream Processor

Based on the PRD, here is how to use the `yellowstone-vixen` library to build the required application.

### Crates to Import into the New Project

Your `solana-stream-processor` project will need to import the following crates from the `yellowstone-vixen` workspace:

- `yellowstone-vixen-runtime`: For `RuntimeBuilder` and the `Handler` trait.
- `yellowstone-vixen-core`: For `InstructionUpdate` and other core types.
- `yellowstone-vixen-parser`: For the SPL Token program parser.
- `yellowstone-vixen-pumpfun-parser`: For the Pump.fun parser.
- `yellowstone-vixen-boop-parser`: For the Boop.fun parser.
- `yellowstone-vixen-moonshot-parser`: For the Moonshot parser.

### New Modules/Files to Create (in the new project)

You will create a new application with the following custom components, as outlined in the PRD's proposed project structure:

- **`src/handlers/filter_handler.rs`**:
    - Will contain a `FilterHandler` struct that implements the `vixen::Handler` trait.
    - Its `handle` method will receive the full `InstructionUpdate` from the parsers.
    - It will filter these instructions down to the "essential fields" and pass the simplified data to the next handler in the chain.
- **`src/handlers/sse_handler.rs`**:
    - Will contain an `SseHandler` that also implements `vixen::Handler`.
    - It will receive the simplified data from the `FilterHandler`.
    - It will be responsible for sending this data to connected clients via Server-Sent Events. This should be a non-blocking operation.
- **`src/handlers/mongo_handler.rs`**:
    - Will contain a `MongoHandler` that implements `vixen::Handler`.
    - It will receive the simplified data and write it to MongoDB asynchronously.
    - This handler must not block the pipeline; database operations should be spawned onto a separate task.
- **`src/main.rs`**:
    - This will be the entry point of your application.
    - It will use `RuntimeBuilder` to construct the processing pipeline.
    - It will instantiate and chain your custom handlers: `FilterHandler -> SseHandler -> MongoHandler`.
    - It will add all the required program parsers from the library.
    - It will start the web server for SSE and the Vixen runtime.
- **`src/config.rs`**: To manage application-specific configuration, including MongoDB connection strings and web server settings, in addition to the Vixen-specific config.
- **`src/models.rs`**: To define the simplified Rust structs for trading and account data that will be used for SSE streaming and MongoDB storage.

### Integration Strategy

1.  **Chaining Handlers**: The `yellowstone-vixen` runtime does not have a built-in "pass-to-next" mechanism for handlers. The simplest way to create a chain is to have each handler hold an instance of the next handler in the chain and call it manually.

    *Example `FilterHandler`*:
    ```rust
    struct FilterHandler {
        next_handler: SseHandler,
    }

    #[async_trait]
    impl Handler for FilterHandler {
        async fn handle(&mut self, msg: &Message) -> Result<()> {
            if let Message::Instruction(inst) = msg {
                // 1. Filter and simplify the instruction
                let simplified_data = self.simplify(inst);
                
                // 2. Create a new message for the next handler
                let next_message = Message::Custom(Box::new(simplified_data));

                // 3. Pass to the next handler
                self.next_handler.handle(&next_message).await?;
            }
            Ok(())
        }
    }
    ```
    *(Note: You would need a way to pass custom data between handlers, perhaps by defining a custom message type and using the `Message::Custom` variant if the framework supports it, or by passing your simplified struct directly.)*

2.  **Concurrency**: For the SSE and MongoDB handlers, ensure all I/O operations are non-blocking. The `handle` function should return quickly. For MongoDB, this means spawning the database write operation using `tokio::spawn` so it runs in the background without blocking the processing of the next message in the pipeline.
