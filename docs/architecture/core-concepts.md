# Core Concepts

This guide explains the fundamental concepts that make up Yellowstone Vixen and how they work together to process Solana blockchain data.

## Overview

Yellowstone Vixen is built around three core components:

1. **Parsers** - Transform raw blockchain data into structured types
2. **Handlers** - Process parsed data and perform actions
3. **Pipelines** - Connect parsers to handlers and manage data flow

## Parsers

Parsers are responsible for converting raw Solana data into structured, type-safe Rust types.

### Types of Parsers

#### Account Parsers
Account parsers process account state changes:

```rust
use yellowstone_vixen_core::AccountParser;

pub struct TokenAccountParser;

impl AccountParser for TokenAccountParser {
    type Account = TokenAccount;
    type Error = ParseError;

    fn parse(&self, account_info: &AccountInfo) -> Result<Self::Account, Self::Error> {
        // Parse raw account data into TokenAccount struct
        todo!()
    }
}
```

#### Instruction Parsers
Instruction parsers process transaction instructions:

```rust
use yellowstone_vixen_core::InstructionParser;

pub struct TokenInstructionParser;

impl InstructionParser for TokenInstructionParser {
    type Instruction = TokenInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        // Parse raw instruction data into TokenInstruction enum
        todo!()
    }
}
```

### Parser Architecture

Parsers follow a consistent pattern:

1. **Input Processing** - Receive raw Solana data
2. **Validation** - Check data integrity and format
3. **Deserialization** - Convert bytes to structured data
4. **Transformation** - Apply business logic and enrich data
5. **Output** - Return typed data or error

## Handlers

Handlers process the parsed data and perform actions like logging, storage, or triggering workflows.

### Handler Interface

```rust
use yellowstone_vixen::Handler;

pub struct DatabaseHandler {
    pool: DatabasePool,
}

impl<V> Handler<V> for DatabaseHandler
where
    V: Serialize + Send + Sync,
{
    async fn handle(&self, value: &V) -> HandlerResult<()> {
        // Store parsed data in database
        self.pool.store(value).await?;
        Ok(())
    }
}
```

### Handler Types

#### Synchronous Handlers
Process data immediately:

```rust
impl<V> Handler<V> for Logger {
    async fn handle(&self, value: &V) -> HandlerResult<()> {
        tracing::info!(?value);
        Ok(())
    }
}
```

#### Asynchronous Handlers
Process data with async operations:

```rust
impl<V> Handler<V> for AsyncProcessor {
    async fn handle(&self, value: &V) -> HandlerResult<()> {
        tokio::spawn(async move {
            // Long-running async operation
            process_async(value).await
        });
        Ok(())
    }
}
```

#### Batch Handlers
Accumulate data before processing:

```rust
impl<V> Handler<V> for BatchHandler {
    async fn handle(&self, value: &V) -> HandlerResult<()> {
        self.buffer.push(value.clone());

        if self.buffer.len() >= self.batch_size {
            self.process_batch().await?;
        }

        Ok(())
    }
}
```

## Pipelines

Pipelines connect parsers to handlers and manage the flow of data through the system.

### Creating Pipelines

```rust
use yellowstone_vixen::Pipeline;

// Create a pipeline with one parser and multiple handlers
let pipeline = Pipeline::new(
    TokenAccountParser,
    [
        Logger,
        DatabaseHandler::new(pool),
        MetricsHandler::new(registry),
    ]
);
```

### Pipeline Execution Flow

1. **Data Ingestion** - Raw Solana data enters the system
2. **Parsing** - Parser converts raw data to structured types
3. **Filtering** - Optional filtering based on parser results
4. **Handler Execution** - Each handler processes the parsed data
5. **Error Handling** - Errors are logged and handled appropriately
6. **Metrics** - Performance metrics are collected

## Runtime

The runtime orchestrates the execution of pipelines and manages system resources.

### Runtime Builder Pattern

```rust
use yellowstone_vixen::Runtime;

let runtime = Runtime::builder()
    .account(token_account_pipeline)
    .instruction(token_instruction_pipeline)
    .metrics(Prometheus::new())
    .commitment_level(CommitmentLevel::Confirmed)
    .build(config)
    .await?;
```

### Runtime Features

- **Multi-threading** - Parallel processing of different data types
- **Resource Management** - Connection pooling and resource limits
- **Graceful Shutdown** - Clean shutdown with proper cleanup
- **Health Monitoring** - Built-in health checks and metrics
- **Error Recovery** - Automatic retry and error recovery

## Data Flow

Understanding the complete data flow is crucial for building effective pipelines.

### Account Data Flow

```
Raw Account Update → Account Parser → Structured Account → Handlers → Actions
      ↓                     ↓              ↓                ↓           ↓
 Yellowstone        Validation &      TokenAccount     Database    Storage
   gRPC              Deserialization   struct         Metrics     Analytics
```

### Instruction Data Flow

```
Raw Transaction → Instruction Parser → Structured Instruction → Handlers → Actions
      ↓                    ↓                    ↓                  ↓         ↓
 Yellowstone         Validation &         TokenInstruction      Queue    Processing
   gRPC               Deserialization          enum            Alerts   Notifications
```

## Error Handling

Vixen provides comprehensive error handling at every level:

### Parser Errors

```rust
pub enum ParseError {
    /// Parser received undesired update
    Filtered,
    /// Parser encountered an error
    Parsing(String),
    /// Validation failed
    Validation(String),
}
```

### Handler Errors

```rust
pub type HandlerResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
```

### Runtime Error Handling

- **Parser Errors** - Logged and metrics updated, processing continues
- **Handler Errors** - Logged, handler may be retried or skipped
- **System Errors** - May trigger graceful shutdown or recovery

## Metrics and Observability

Vixen includes built-in metrics for monitoring pipeline performance:

### Standard Metrics

- **Throughput** - Events processed per second
- **Latency** - Time from ingestion to processing
- **Error Rates** - Parser and handler error percentages
- **Queue Depth** - Number of pending events
- **Resource Usage** - Memory and CPU utilization

### Custom Metrics

```rust
impl<V> Handler<V> for MetricsHandler {
    async fn handle(&self, value: &V) -> HandlerResult<()> {
        self.counter.inc();
        self.histogram.record(self.start_time.elapsed());
        Ok(())
    }
}
```

## Best Practices

### Parser Design

1. **Keep parsers focused** - One parser per data type
2. **Handle errors gracefully** - Use `ParseError::Filtered` for expected skips
3. **Validate thoroughly** - Check data integrity before processing
4. **Document assumptions** - Comment on data format expectations

### Handler Design

1. **Make handlers idempotent** - Safe to reprocess the same data
2. **Handle failures gracefully** - Don't crash on individual failures
3. **Use async appropriately** - Don't block on I/O operations
4. **Monitor performance** - Add metrics for latency and throughput

### Pipeline Design

1. **Start simple** - Begin with basic logging handlers
2. **Add monitoring early** - Include metrics from the start
3. **Test thoroughly** - Use mock data for testing
4. **Plan for scale** - Design for expected data volumes

## Advanced Concepts

### Custom Parsers

For programs not yet supported, you can create custom parsers:

```rust
#[derive(Parser)]
#[program(id = "MyProgram111111111111111111111111111")]
pub struct MyProgramParser;

impl InstructionParser for MyProgramParser {
    // Implementation
}
```

### Composite Handlers

Combine multiple handlers for complex processing:

```rust
pub struct CompositeHandler<H1, H2> {
    handler1: H1,
    handler2: H2,
}

impl<H1, H2, V> Handler<V> for CompositeHandler<H1, H2>
where
    H1: Handler<V>,
    H2: Handler<V>,
{
    async fn handle(&self, value: &V) -> HandlerResult<()> {
        self.handler1.handle(value).await?;
        self.handler2.handle(value).await?;
        Ok(())
    }
}
```

This architecture provides the foundation for building robust, scalable Solana data pipelines with Yellowstone Vixen.
