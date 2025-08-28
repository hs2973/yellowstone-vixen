# Yellowstone Vixen Architecture Guide

This guide provides a detailed overview of Yellowstone Vixen's architecture, design principles, and core components.

## Table of Contents

- [Design Principles](#design-principles)
- [Component Overview](#component-overview)
- [Data Flow](#data-flow)
- [Parser Architecture](#parser-architecture)
- [Handler System](#handler-system)
- [Filtering & Prefiltering](#filtering--prefiltering)
- [Metrics & Observability](#metrics--observability)
- [Configuration System](#configuration-system)
- [Error Handling](#error-handling)
- [Performance Considerations](#performance-considerations)

## Design Principles

Yellowstone Vixen is built on several key design principles:

### 1. Modularity
- **Pluggable Components**: All major components (sources, parsers, handlers, metrics) are pluggable
- **Crate Separation**: Each parser is a separate crate, allowing independent versioning and usage
- **Trait-Based Design**: Behavior is defined through traits, enabling easy extension and testing

### 2. Performance
- **Zero-Copy Parsing**: Minimize data copying through efficient parsing strategies
- **Async-First**: Built on Tokio for high-concurrency workloads
- **Buffered Processing**: Intelligent buffering to handle traffic spikes
- **Filtering at Source**: Apply filters early to reduce processing overhead

### 3. Reliability
- **Error Isolation**: Errors in one parser don't affect others
- **Graceful Degradation**: Continue processing even when individual components fail
- **Comprehensive Monitoring**: Built-in metrics and observability
- **Testing Framework**: Extensive testing capabilities with fixture support

### 4. Developer Experience
- **Type Safety**: Leverage Rust's type system for compile-time correctness
- **Rich Tooling**: Comprehensive testing and development tools
- **Clear APIs**: Simple, intuitive interfaces for common use cases
- **Extensive Documentation**: Detailed guides and examples

## Component Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Yellowstone Vixen                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   Sources   │  │   Runtime   │  │   Parsers   │  │   Handlers  │      │
│  │             │  │             │  │             │  │             │      │
│  │ Yellowstone │──┤ Buffering   │──┤ Program     │──┤ Custom      │      │
│  │ Solana RPC  │  │ Routing     │  │ Logic       │  │ Logic       │      │
│  │ Snapshots   │  │ Error       │  │ Filtering   │  │ Storage     │      │
│  │ Mock Data   │  │ Handling    │  │ Validation  │  │ Metrics     │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
│                           │                                   │            │
│  ┌─────────────┐         │          ┌─────────────┐         │            │
│  │   Metrics   │─────────┴──────────┤ gRPC Stream │─────────┘            │
│  │             │                    │             │                      │
│  │ Prometheus  │                    │ Real-time   │                      │
│  │ OpenTelemetry│                   │ API         │                      │
│  │ Custom      │                    │ Multi-lang  │                      │
│  └─────────────┘                    └─────────────┘                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Core Crates

#### 1. Runtime (`yellowstone-vixen`)
The main orchestration engine that:
- Manages data source connections
- Implements buffering and backpressure handling
- Routes events to appropriate parsers
- Handles error recovery and logging
- Coordinates metrics collection

**Key Types:**
- `Runtime<M, S>`: Main runtime struct parameterized by metrics and source
- `RuntimeBuilder`: Builder pattern for runtime configuration
- `Pipeline<P, H>`: Combines a parser with handlers

#### 2. Core (`yellowstone-vixen-core`)
Foundational types and traits:
- `Parser` trait: Core parsing interface
- `ParserId`: Unique identifier for parsers
- `Prefilter`: Account-based filtering system
- Update types: `AccountUpdate`, `InstructionUpdate`, etc.

**Key Traits:**
```rust
pub trait Parser {
    type Input: Sync;
    type Output: Send + Sync;
    
    async fn parse(&self, input: &Self::Input) -> Result<Self::Output, ParseError>;
    fn id(&self) -> Cow<str>;
}
```

#### 3. Stream (`yellowstone-vixen-stream`)
gRPC streaming server implementation:
- Protocol buffer service definitions
- Real-time streaming of parsed events
- Multi-client support with backpressure
- Dynamic service registration

#### 4. Sources
Data source implementations:
- **Yellowstone gRPC**: Production-ready Dragon's Mouth integration
- **Solana RPC**: Development and fallback source
- **Snapshot**: Historical data processing
- **Mock**: Testing and development

#### 5. Parsers
Program-specific parsing logic:
- 30+ supported Solana programs
- Account and instruction parsers
- Protocol buffer definitions
- Shared parsing utilities

## Data Flow

The data flow through Yellowstone Vixen follows a clear pipeline:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Dragon's  │    │   Source    │    │   Runtime   │    │   Buffer    │
│   Mouth     │───▶│  Adapter    │───▶│  Receiver   │───▶│   Queue     │
│   Stream    │    │             │    │             │    │             │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                                  │
                                                                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Handler   │    │   Parser    │    │ Prefilter   │    │   Worker    │
│  Execution  │◀───│ Execution   │◀───│  Check      │◀───│    Pool     │
│             │    │             │    │             │    │             │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### 1. Data Ingestion
- **Source Adapter**: Normalizes data from different sources into common types
- **Connection Management**: Handles reconnections, retries, and error recovery
- **Rate Limiting**: Implements backpressure to prevent overwhelming downstream components

### 2. Buffering & Routing
- **Buffered Queue**: Smooths traffic spikes and provides processing guarantees
- **Event Routing**: Determines which parsers should process each event
- **Prefiltering**: Early filtering to reduce processing overhead

### 3. Parsing & Processing
- **Parser Execution**: Runs program-specific parsing logic
- **Error Handling**: Isolates parser errors and continues processing
- **Context Sharing**: Provides shared transaction context to all parsers

### 4. Handler Execution
- **Parallel Execution**: Handlers run concurrently for maximum throughput
- **Error Isolation**: Handler errors don't affect other handlers or parsers
- **Metrics Collection**: Automatic metrics collection for all operations

## Parser Architecture

Parsers are the core business logic components that transform raw Solana data into structured, typed data.

### Parser Types

#### Account Parsers
Parse account data into program-specific structures:

```rust
impl Parser for TokenProgramAccountParser {
    type Input = AccountUpdate;
    type Output = TokenProgramAccount;
    
    async fn parse(&self, account: &AccountUpdate) -> Result<Self::Output, ParseError> {
        match account.account.owner.as_ref() {
            TOKEN_PROGRAM_ID => {
                // Parse token account data
                parse_token_account(&account.account.data)
            }
            _ => Err(ParseError::Filtered), // Not our program
        }
    }
}
```

#### Instruction Parsers
Parse transaction instructions with program-specific logic:

```rust
impl Parser for RaydiumAmmV4InstructionParser {
    type Input = InstructionUpdate;
    type Output = RaydiumAmmV4Instruction;
    
    async fn parse(&self, ix: &InstructionUpdate) -> Result<Self::Output, ParseError> {
        if ix.instruction.program_id != RAYDIUM_AMM_V4_PROGRAM_ID {
            return Err(ParseError::Filtered);
        }
        
        // Parse instruction data
        match ix.instruction.data[0] {
            0 => parse_initialize_instruction(&ix.instruction.data[1..]),
            1 => parse_swap_instruction(&ix.instruction.data[1..]),
            _ => Err(ParseError::Other("Unknown instruction".into())),
        }
    }
}
```

### Parser Best Practices

1. **Use `ParseError::Filtered`** for irrelevant updates to avoid error logging
2. **Validate Program IDs** early to prevent unnecessary processing
3. **Handle Incomplete Data** gracefully with appropriate error messages
4. **Minimize Allocations** for high-performance parsing
5. **Add Comprehensive Tests** using the mock framework

## Handler System

Handlers implement custom business logic that processes parsed data.

### Handler Trait

```rust
pub trait Handler<T> {
    async fn handle(&self, value: &T) -> HandlerResult<()>;
}

pub type HandlerResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
```

### Handler Patterns

#### Database Storage Handler
```rust
#[derive(Debug)]
pub struct DatabaseHandler {
    pool: sqlx::PgPool,
}

impl Handler<SwapInstruction> for DatabaseHandler {
    async fn handle(&self, swap: &SwapInstruction) -> HandlerResult<()> {
        sqlx::query!(
            "INSERT INTO swaps (signature, amount_in, amount_out, timestamp) VALUES ($1, $2, $3, $4)",
            swap.signature.to_string(),
            swap.amount_in as i64,
            swap.amount_out as i64,
            swap.timestamp
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

#### Notification Handler
```rust
#[derive(Debug)]
pub struct SlackNotifier {
    webhook_url: String,
    min_volume: u64,
}

impl Handler<SwapInstruction> for SlackNotifier {
    async fn handle(&self, swap: &SwapInstruction) -> HandlerResult<()> {
        if swap.amount_in > self.min_volume {
            let message = format!(
                "Large swap detected: {} tokens for signature {}",
                swap.amount_in, swap.signature
            );
            
            self.send_slack_message(&message).await?;
        }
        
        Ok(())
    }
}
```

### Handler Composition

Multiple handlers can be composed in pipelines:

```rust
yellowstone_vixen::Runtime::builder()
    .instruction(Pipeline::new(
        SwapParser,
        [
            DatabaseHandler::new(pool),
            MetricsHandler::new(),
            SlackNotifier::new(webhook_url, min_volume),
        ]
    ))
    .build(config)
    .run();
```

## Filtering & Prefiltering

Vixen provides multiple levels of filtering to optimize performance:

### 1. Source-Level Filtering
Configure data sources to only subscribe to relevant accounts:

```toml
[source]
# Only subscribe to specific accounts
accounts = ["account1", "account2"]
# Only subscribe to specific programs  
programs = ["program1", "program2"]
```

### 2. Prefiltering
Account-based filtering before parsing:

```rust
use yellowstone_vixen_core::Prefilter;

let prefilter = Prefilter::builder()
    .transaction_accounts_include([account1, account2])
    .transaction_accounts_required([required_account])
    .build();
```

### 3. FilterPipeline
Advanced transaction filtering with custom logic:

```rust
use yellowstone_vixen::filter_pipeline::FilterPipeline;

let filtered_pipeline = FilterPipeline::new(
    MyParser,
    [MyHandler],
    prefilter
);
```

### 4. Parser-Level Filtering
Custom filtering logic within parsers:

```rust
impl Parser for MyParser {
    async fn parse(&self, input: &Input) -> Result<Output, ParseError> {
        // Custom filtering logic
        if !self.should_process(input) {
            return Err(ParseError::Filtered);
        }
        
        // Parse the input
        self.do_parse(input)
    }
}
```

## Metrics & Observability

Vixen provides comprehensive observability through multiple metrics backends:

### Prometheus Integration

```rust
use yellowstone_vixen::metrics::Prometheus;

yellowstone_vixen::Runtime::builder()
    .metrics(Prometheus)
    .build(config)
    .run();
```

### Built-in Metrics

- **Throughput Metrics**: Events processed per second by type
- **Error Metrics**: Parse errors, handler errors, connection errors
- **Latency Metrics**: Processing time distributions
- **Resource Metrics**: Memory usage, connection counts

### Custom Metrics

Implement custom metrics collection:

```rust
use yellowstone_vixen::metrics::{Counter, Histogram, Instrumenter};

#[derive(Debug)]
pub struct CustomMetrics<I: Instrumenter> {
    swap_volume: Histogram<I>,
    large_swaps: Counter<I>,
}

impl<I: Instrumenter> CustomMetrics<I> {
    pub fn new(instrumenter: &I) -> Self {
        Self {
            swap_volume: instrumenter.histogram("swap_volume_usd"),
            large_swaps: instrumenter.counter("large_swaps_total"),
        }
    }
}

impl<I: Instrumenter> Handler<SwapInstruction> for CustomMetrics<I> {
    async fn handle(&self, swap: &SwapInstruction) -> HandlerResult<()> {
        self.swap_volume.record(swap.volume_usd);
        
        if swap.volume_usd > 10000.0 {
            self.large_swaps.inc();
        }
        
        Ok(())
    }
}
```

## Configuration System

Vixen uses a hierarchical configuration system supporting multiple formats:

### Configuration Sources
1. **TOML Files**: Primary configuration format
2. **Environment Variables**: Override any configuration value
3. **Command Line Args**: Runtime parameter overrides
4. **Defaults**: Sensible defaults for all optional values

### Configuration Structure

```rust
#[derive(Debug, Args)]
pub struct VixenConfig<M, S> 
where
    M: Args,  // Metrics configuration
    S: Args,  // Source configuration
{
    pub source: S,
    pub buffer: BufferConfig,
    pub metrics: OptConfig<M>,
}
```

### Environment Variable Mapping

Configuration values can be overridden with environment variables:

```bash
# source.endpoint -> ENDPOINT
export ENDPOINT="https://api.example.com"

# source.timeout -> TIMEOUT  
export TIMEOUT="120"

# metrics.export-interval -> METRICS_EXPORT_INTERVAL
export METRICS_EXPORT_INTERVAL="30"
```

## Error Handling

Vixen implements comprehensive error handling with isolation and recovery:

### Error Types

#### ParseError
Errors specific to parsing operations:
```rust
pub enum ParseError {
    Filtered,           // Skip processing (not an error)
    Other(Box<dyn Error>), // Actual parsing error
}
```

#### Handler Errors
Errors from handler execution are isolated and logged without stopping the pipeline.

#### Connection Errors
Source connection errors trigger automatic reconnection with exponential backoff.

### Error Recovery Strategies

1. **Parser Errors**: Log and continue processing other parsers
2. **Handler Errors**: Log and continue processing other handlers  
3. **Connection Errors**: Automatic reconnection with backoff
4. **Resource Errors**: Graceful degradation and alerting

### Error Monitoring

All errors are automatically:
- Logged with structured logging
- Counted in metrics systems
- Traced for debugging purposes
- Isolated to prevent cascade failures

## Performance Considerations

### Memory Management
- **Zero-Copy Parsing**: Minimize data copying through efficient parsing
- **Bounded Buffers**: Prevent unbounded memory growth
- **Object Pooling**: Reuse expensive objects where possible

### CPU Optimization
- **Async Processing**: Maximize CPU utilization with async/await
- **Worker Pools**: Distribute work across multiple threads
- **Efficient Parsing**: Optimized parsing algorithms for each program

### Network Optimization
- **Connection Pooling**: Reuse connections to external services
- **Batching**: Batch operations where possible
- **Compression**: Use compression for large data transfers

### Monitoring Performance

Key performance metrics to monitor:
- **Processing Latency**: Time from event receipt to handler completion
- **Throughput**: Events processed per second by type
- **Memory Usage**: Heap usage and growth patterns
- **CPU Utilization**: Processing efficiency across workers
- **Network Bandwidth**: Data transfer rates and efficiency

### Scaling Considerations

1. **Horizontal Scaling**: Run multiple Vixen instances with load balancing
2. **Vertical Scaling**: Increase worker threads and buffer sizes
3. **Source Scaling**: Use multiple data source connections
4. **Handler Scaling**: Distribute handlers across instances
5. **Database Scaling**: Use connection pooling and read replicas