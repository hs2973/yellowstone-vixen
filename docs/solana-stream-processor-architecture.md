# Solana Stream Processor - Brownfield Architecture

## 1. Introduction

This document outlines the proposed architecture for the **Real-Time Solana Stream Processor**, a new Rust application built as an enhancement leveraging the existing `yellowstone-vixen` library. It is based on the requirements defined in `docs/prd.md` and the library analysis found in `docs/yellowstone-vixen-architecture.md`.

This architecture is designed to create a new, standalone application that uses the `yellowstone-vixen` framework as a dependency without modifying the library's code. The core of this design is a **unified handler** model that encapsulates all business logic for each program, ensuring a clean separation of concerns and efficient data processing.

### 1.1. Document Scope

This document details the architecture of the new `solana-stream-processor` application, focusing on its internal structure, integration with the `yellowstone-vixen` library, and interaction with external services like MongoDB and SSE clients.

### 1.2. Change Log

| Date | Version | Description | Author |
| --- | --- | --- | --- |
| August 29, 2025 | 1.0 | Initial architecture based on Brownfield PRD v2.0. | Winston |

---

## 2. Quick Reference - Key Files and Entry Points

The following files represent the core components of the proposed `solana-stream-processor` application:

-   **Main Entry Point**: `src/main.rs` - Responsible for configuration loading, `RuntimeBuilder` setup, web server initialization, and starting the processing pipeline.
-   **Configuration**: `src/config.rs` & `config.toml` - Manages all application-specific settings, including web server ports and MongoDB connection details.
-   **Unified Handler**: `src/handlers/unified_handler.rs` - Contains the primary business logic. A single handler struct will be responsible for filtering, SSE streaming, and database persistence.
-   **Data Models**: `src/models.rs` - Defines the simplified `EssentialData` struct that is passed between internal components and serialized for output.
-   **Metrics**: `src/metrics.rs` - Defines and registers the custom Prometheus metrics for monitoring application health and throughput.

---

## 3. High-Level Architecture

The application follows a pipeline processing model orchestrated by the `yellowstone-vixen` runtime.

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
[Unified Program Handler (in new app)]
             |
             +---> [Filter & Simplify Logic]
             |
             +---> [SSE Streaming Logic (non-blocking)]
             |
             +---> [MongoDB Persistence Logic (async)]
```

### 3.1. Technical Summary

The `solana-stream-processor` is a standalone Rust binary. It initializes the `yellowstone-vixen` runtime, registers the required program parsers, and attaches a single, custom **unified handler**. This handler receives parsed data for all specified programs. Inside the handler, a sequence of operations is performed: the data is filtered into an `EssentialData` model, pushed to an SSE manager, and then handed off to a MongoDB client for asynchronous writing.

### 3.2. Proposed Tech Stack

| Category | Technology | Notes |
| :--- | :--- | :--- |
| Language | Rust (1.70+) | |
| Runtime | Tokio | For all asynchronous operations. |
| Core Framework | `yellowstone-vixen` | Used for the core pipeline, sources, and parsers. |
| Web Server | `axum` | Chosen for its integration with Tokio and `tower` services. |
| Database | MongoDB | Accessed via the official `mongodb` Rust driver. |
| Monitoring | `prometheus`, `grafana` | Using the `prometheus` crate for metrics. |
| Serialization | `serde` | For configuration and JSON serialization. |

### 3.3. Repository Structure

-   **Type**: Polyrepo. The `solana-stream-processor` will reside in its own Git repository and will include the `yellowstone-vixen` repository as a dependency, likely via a Git submodule or a local path for development.

---

## 4. Source Tree and Module Organization

### 4.1. Proposed Project Structure

```text
solana-stream-processor/
├── Cargo.toml
├── config.toml
├── docker-compose.yml
├── go-client/
│   └── ...
└── src/
    ├── main.rs         // Entry point, runtime & server setup
    ├── config.rs       // Application configuration
    ├── error.rs        // Custom error types
    ├── handlers/
    │   └── unified_handler.rs // The core unified handler logic
    ├── models.rs       // Simplified data structures (e.g., EssentialData)
    ├── metrics.rs      // Prometheus metrics definitions
    └── web/
        ├── mod.rs      // Web server module
        └── sse.rs      // SSE broadcasting logic
```

### 4.2. Key Modules and Their Purpose

-   **`main`**: Initializes everything. Reads `config.toml`, sets up the `tracing` logger, builds the Vixen runtime by adding parsers and the `UnifiedHandler`, and starts the `axum` web server.
-   **`handlers::unified_handler`**: Contains the `UnifiedHandler` struct which implements the `vixen::Handler` trait. This is the heart of the application. It will contain instances of the SSE broadcaster and a MongoDB client. Its `handle` method will orchestrate the filtering and dispatching to the SSE and MongoDB components.
-   **`models`**: Defines the canonical, simplified `EssentialData` struct. This ensures that all components (filtering, SSE, MongoDB) work with a consistent, minimal data model.
-   **`web::sse`**: Implements the SSE broadcasting logic. It will likely use a `tokio::sync::broadcast` channel to distribute `EssentialData` messages to all connected HTTP clients. The `UnifiedHandler` will hold a `Sender` to this channel.
-   **`web`**: Contains the `axum` router setup, defining the `/health`, `/metrics`, and `/events/stream` endpoints.

---

## 5. Data Models and APIs

### 5.1. Data Models

-   **`EssentialData`**: A struct defined in `src/models.rs` that represents the simplified, filtered data.
    ```rust
    #[derive(Clone, serde::Serialize)]
    pub struct EssentialData {
        pub program_id: String,
        pub token_mint: String,
        pub transaction_signature: String,
        pub instruction_type: String, // e.g., "buy", "sell", "create"
        pub instruction_data: serde_json::Value,
        pub blockchain_timestamp: i64,
        pub ingestion_timestamp: i64,
    }
    ```

### 5.2. API Specifications

-   **`GET /events/stream`**: An SSE endpoint that streams a continuous flow of `EssentialData` objects as JSON events.
-   **`GET /health`**: A simple health check endpoint returning a `200 OK`.
-   **`GET /metrics`**: Exposes application and `yellowstone-vixen` metrics in Prometheus format.

---

## 6. Technical Implementation Details

### 6.1. Unified Handler Design

The key change from the initial PRD draft is the move to a unified handler. This avoids the complexity of handler chaining.

```rust
// In src/handlers/unified_handler.rs

pub struct UnifiedHandler {
    sse_tx: tokio::sync::broadcast::Sender<EssentialData>,
    mongo_client: mongodb::Client,
    // ... other shared resources
}

#[async_trait]
impl vixen::Handler for UnifiedHandler {
    async fn handle(&mut self, msg: &vixen::Message) -> Result<()> {
        if let vixen::Message::Instruction(inst) = msg {
            // 1. Filter and simplify the instruction into `EssentialData`
            if let Some(data) = self.filter_and_simplify(inst) {
                // 2. Send to SSE broadcaster. Ignore error if no receivers.
                _ = self.sse_tx.send(data.clone());

                // 3. Spawn a non-blocking task for the DB write.
                let client = self.mongo_client.clone();
                tokio::spawn(async move {
                    // ... logic to write `data` to MongoDB ...
                });
            }
        }
        Ok(())
    }
}
```

This design ensures that the `handle` method returns immediately, keeping the Vixen pipeline unblocked.

### 6.2. Concurrency and State Management

-   **SSE**: The `SseBroadcaster` will be created once and cloned into the `UnifiedHandler`. It is thread-safe.
-   **MongoDB**: The `mongodb::Client` is designed to be cloned and is safe to share across threads. Connection pooling is handled automatically by the driver.
-   **State**: The `UnifiedHandler` itself should be stateless. All data is contained within the `Message` it receives.

---

## 7. Impact Analysis (New Project)

As this is a new application, the impact is on creation rather than modification.

### 7.1. New Files & Modules Needed

All files listed in the **Proposed Project Structure** (section 4.1) will need to be created from scratch. This includes:
-   Setting up the Cargo project and dependencies.
-   Implementing the `config` module.
-   Implementing the `UnifiedHandler` and its filtering logic.
-   Implementing the `models`.
-   Implementing the `axum` web server with SSE and metrics endpoints.
-   Creating the `docker-compose.yml` for local services.
-   Creating the `go-client` boilerplate.

This architecture provides a clear roadmap for implementing the `solana-stream-processor` as a robust, maintainable, and performant application that effectively leverages the power of the `yellowstone-vixen` library.
