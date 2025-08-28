# Handlers API

This reference documents the handler interface, built-in handlers, and patterns for creating custom handlers in Yellowstone Vixen.

## Handler Interface

### Core Handler Trait

All handlers implement the `Handler` trait:

```rust
#[async_trait::async_trait]
pub trait Handler<T> {
    async fn handle(&self, data: &T) -> HandlerResult<()>;
}
```

**Parameters:**
- `data`: Parsed data from a parser (instruction or account)
- Returns: `Result<(), HandlerError>` indicating success or failure

**Error Handling:**
```rust
pub type HandlerResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
```

### Handler Lifecycle

Handlers follow a consistent lifecycle:

1. **Initialization** - Set up resources (database connections, etc.)
2. **Processing** - Handle incoming data
3. **Cleanup** - Release resources on shutdown
4. **Error Recovery** - Handle and recover from errors

## Built-in Handlers

### Logging Handlers

#### Basic Logger

```rust
use yellowstone_vixen::handlers::Logger;

pub struct Logger;

#[async_trait::async_trait]
impl<T> Handler<T> for Logger
where
    T: Debug + Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        tracing::info!(?data);
        Ok(())
    }
}
```

**Usage:**
```rust
let pipeline = Pipeline::new(parser, [Logger]);
```

**Configuration:**
- Log level controlled by `RUST_LOG` environment variable
- Structured logging with `tracing` crate
- Automatic JSON formatting for complex types

#### Structured Logger

```rust
use yellowstone_vixen::handlers::StructuredLogger;

pub struct StructuredLogger {
    format: LogFormat,
    level: tracing::Level,
}

impl StructuredLogger {
    pub fn new() -> Self {
        Self {
            format: LogFormat::Json,
            level: tracing::Level::INFO,
        }
    }

    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }
}
```

### Database Handlers

#### PostgreSQL Handler

```rust
use yellowstone_vixen::handlers::PostgresHandler;

pub struct PostgresHandler {
    pool: sqlx::PgPool,
    table_name: String,
}

impl PostgresHandler {
    pub async fn new(database_url: &str, table_name: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::PgPool::connect(database_url).await?;
        Ok(Self {
            pool,
            table_name: table_name.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl<T> Handler<T> for PostgresHandler
where
    T: Serialize + Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        let json_data = serde_json::to_value(data)?;
        sqlx::query(
            "INSERT INTO $1 (data, created_at) VALUES ($2, NOW())"
        )
        .bind(&self.table_name)
        .bind(json_data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

**Features:**
- Automatic table creation
- JSON storage for flexible schemas
- Connection pooling
- Transaction support

#### ClickHouse Handler

```rust
use yellowstone_vixen::handlers::ClickHouseHandler;

pub struct ClickHouseHandler {
    client: clickhouse::Client,
    table: String,
}

#[async_trait::async_trait]
impl<T> Handler<T> for ClickHouseHandler
where
    T: Serialize + Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        let mut insert = self.client.insert(&self.table)?;
        insert.write(data).await?;
        insert.end().await?;
        Ok(())
    }
}
```

**Features:**
- High-performance columnar storage
- Real-time analytics support
- Compression and indexing
- Distributed processing

### Message Queue Handlers

#### Kafka Handler

```rust
use yellowstone_vixen::handlers::KafkaHandler;

pub struct KafkaHandler {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    key_extractor: Box<dyn Fn(&T) -> String + Send + Sync>,
}

#[async_trait::async_trait]
impl<T> Handler<T> for KafkaHandler
where
    T: Serialize + Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        let payload = serde_json::to_vec(data)?;
        let key = (self.key_extractor)(data);

        let record = rdkafka::producer::FutureRecord::to(&self.topic)
            .payload(&payload)
            .key(&key);

        self.producer.send(record, Duration::from_secs(0)).await?;
        Ok(())
    }
}
```

**Features:**
- Asynchronous message publishing
- Key-based partitioning
- Delivery guarantees
- Monitoring and metrics

#### Redis Handler

```rust
use yellowstone_vixen::handlers::RedisHandler;

pub struct RedisHandler {
    client: redis::Client,
    key_pattern: String,
}

#[async_trait::async_trait]
impl<T> Handler<T> for RedisHandler
where
    T: Serialize + Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        let json_data = serde_json::to_string(data)?;

        let key = format!("{}:{}", self.key_pattern, generate_id());
        redis::cmd("SET")
            .arg(&key)
            .arg(json_data)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }
}
```

### Metrics Handlers

#### Prometheus Handler

```rust
use yellowstone_vixen::handlers::PrometheusHandler;

pub struct PrometheusHandler {
    registry: prometheus::Registry,
    counters: HashMap<String, prometheus::Counter>,
    histograms: HashMap<String, prometheus::Histogram>,
}

impl PrometheusHandler {
    pub fn new() -> Self {
        let registry = prometheus::Registry::new();
        // Register metrics...
        Self {
            registry,
            counters: HashMap::new(),
            histograms: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl<T> Handler<T> for PrometheusHandler {
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        // Update metrics based on data
        if let Some(counter) = self.counters.get("events_processed") {
            counter.inc();
        }
        Ok(())
    }
}
```

**Features:**
- Automatic metric collection
- Custom metric definitions
- Integration with Prometheus ecosystem
- Alerting support

### File Handlers

#### JSON Lines Handler

```rust
use yellowstone_vixen::handlers::JsonLinesHandler;

pub struct JsonLinesHandler {
    writer: tokio::io::BufWriter<tokio::fs::File>,
}

impl JsonLinesHandler {
    pub async fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = tokio::fs::File::create(path).await?;
        let writer = tokio::io::BufWriter::new(file);
        Ok(Self { writer })
    }
}

#[async_trait::async_trait]
impl<T> Handler<T> for JsonLinesHandler
where
    T: Serialize + Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        let json_line = serde_json::to_string(data)?;
        self.writer.write_all(json_line.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        Ok(())
    }
}
```

**Features:**
- Line-delimited JSON format
- Buffered writing for performance
- Automatic file rotation
- Compression support

## Creating Custom Handlers

### Basic Custom Handler

```rust
use async_trait::async_trait;
use yellowstone_vixen_core::{Handler, HandlerResult};

pub struct CustomHandler {
    config: CustomConfig,
    state: Arc<Mutex<HandlerState>>,
}

impl CustomHandler {
    pub fn new(config: CustomConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(HandlerState::default())),
        }
    }
}

#[async_trait]
impl<T> Handler<T> for CustomHandler
where
    T: Send + Sync + MyTrait, // Add trait bounds as needed
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        // Custom processing logic
        self.process_data(data).await?;
        self.update_metrics(data).await?;
        Ok(())
    }
}

impl CustomHandler {
    async fn process_data(&self, data: &T) -> HandlerResult<()> {
        // Implementation
        Ok(())
    }

    async fn update_metrics(&self, data: &T) -> HandlerResult<()> {
        // Implementation
        Ok(())
    }
}
```

### Handler with State

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct StatefulHandler<T> {
    state: Arc<Mutex<HandlerState>>,
    _phantom: std::marker::PhantomData<T>,
}

struct HandlerState {
    processed_count: u64,
    last_processed: Option<std::time::Instant>,
    errors: Vec<String>,
}

#[async_trait]
impl<T> Handler<T> for StatefulHandler<T>
where
    T: Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        let mut state = self.state.lock().await;

        // Update state
        state.processed_count += 1;
        state.last_processed = Some(std::time::Instant::now());

        // Process data
        match self.process_with_state(data, &mut state).await {
            Ok(()) => Ok(()),
            Err(e) => {
                state.errors.push(e.to_string());
                Err(e)
            }
        }
    }
}
```

### Batch Handler

```rust
use tokio::sync::mpsc;

pub struct BatchHandler<T> {
    batch_size: usize,
    timeout: Duration,
    sender: mpsc::Sender<Vec<T>>,
}

impl<T> BatchHandler<T>
where
    T: Send + Clone + 'static,
{
    pub fn new(batch_size: usize, processor: impl BatchProcessor<T> + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel(100);

        tokio::spawn(async move {
            Self::batch_processor(receiver, batch_size, processor).await;
        });

        Self {
            batch_size,
            timeout: Duration::from_secs(1),
            sender,
        }
    }

    async fn batch_processor(
        mut receiver: mpsc::Receiver<Vec<T>>,
        batch_size: usize,
        mut processor: impl BatchProcessor<T>,
    ) {
        let mut batch = Vec::with_capacity(batch_size);

        loop {
            match tokio::time::timeout(Duration::from_secs(1), receiver.recv()).await {
                Ok(Some(items)) => {
                    batch.extend(items);
                    if batch.len() >= batch_size {
                        processor.process_batch(batch).await;
                        batch = Vec::with_capacity(batch_size);
                    }
                }
                Ok(None) => break, // Channel closed
                Err(_) => {
                    // Timeout - process current batch
                    if !batch.is_empty() {
                        processor.process_batch(batch).await;
                        batch = Vec::with_capacity(batch_size);
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<T> Handler<T> for BatchHandler<T>
where
    T: Send + Clone + 'static,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        self.sender.send(vec![data.clone()]).await?;
        Ok(())
    }
}
```

## Handler Patterns

### Chain of Responsibility

```rust
pub struct HandlerChain<H1, H2> {
    handler1: H1,
    handler2: H2,
}

#[async_trait]
impl<H1, H2, T> Handler<T> for HandlerChain<H1, H2>
where
    H1: Handler<T>,
    H2: Handler<T>,
    T: Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        self.handler1.handle(data).await?;
        self.handler2.handle(data).await?;
        Ok(())
    }
}
```

### Conditional Handler

```rust
pub struct ConditionalHandler<H, F> {
    handler: H,
    condition: F,
}

impl<H, F, T> ConditionalHandler<H, F>
where
    F: Fn(&T) -> bool,
{
    pub fn new(handler: H, condition: F) -> Self {
        Self { handler, condition }
    }
}

#[async_trait]
impl<H, F, T> Handler<T> for ConditionalHandler<H, F>
where
    H: Handler<T>,
    F: Fn(&T) -> bool + Send + Sync,
    T: Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        if (self.condition)(data) {
            self.handler.handle(data).await?;
        }
        Ok(())
    }
}
```

### Retry Handler

```rust
pub struct RetryHandler<H> {
    handler: H,
    max_attempts: usize,
    backoff: ExponentialBackoff,
}

#[async_trait]
impl<H, T> Handler<T> for RetryHandler<H>
where
    H: Handler<T>,
    T: Send + Sync,
{
    async fn handle(&self, data: &T) -> HandlerResult<()> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match self.handler.handle(data).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if attempt >= self.max_attempts {
                        return Err(e);
                    }

                    let delay = self.backoff.delay(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}
```

## Error Handling

### Handler Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
```

### Error Recovery Strategies

```rust
impl ResilientHandler {
    pub async fn handle_with_recovery(&self, data: &T) -> HandlerResult<()> {
        match self.handler.handle(data).await {
            Ok(()) => Ok(()),
            Err(e) => {
                // Log error
                tracing::error!("Handler failed: {}", e);

                // Attempt recovery
                if let Err(recovery_err) = self.attempt_recovery(data).await {
                    tracing::error!("Recovery also failed: {}", recovery_err);
                    return Err(e); // Return original error
                }

                Ok(())
            }
        }
    }

    async fn attempt_recovery(&self, data: &T) -> HandlerResult<()> {
        // Implementation of recovery logic
        // e.g., retry with different parameters, use fallback handler, etc.
        Ok(())
    }
}
```

## Performance Optimization

### Async Handler Patterns

```rust
impl OptimizedHandler {
    pub async fn handle_concurrent(&self, data: &[T]) -> HandlerResult<()> {
        let tasks: Vec<_> = data.iter()
            .map(|item| {
                let handler = &self.handler;
                async move { handler.handle(item).await }
            })
            .collect();

        let results = futures::future::join_all(tasks).await;

        for result in results {
            result?;
        }

        Ok(())
    }
}
```

### Resource Pooling

```rust
use deadpool::managed::Pool;

pub struct PooledHandler<H> {
    pool: Pool<H>,
}

impl<H, T> PooledHandler<H>
where
    H: Handler<T> + Send + Sync + 'static,
{
    pub async fn handle_pooled(&self, data: &T) -> HandlerResult<()> {
        let handler = self.pool.get().await?;
        handler.handle(data).await?;
        Ok(())
    }
}
```

## Testing Handlers

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use yellowstone_vixen_core::test_utils::*;

    #[tokio::test]
    async fn test_handler_processes_data() {
        let handler = TestHandler::new();
        let test_data = create_test_data();

        let result = handler.handle(&test_data).await;
        assert!(result.is_ok());

        // Verify handler state
        assert_eq!(handler.get_processed_count().await, 1);
    }

    #[tokio::test]
    async fn test_handler_error_handling() {
        let handler = FailingHandler::new();

        let result = handler.handle(&create_test_data()).await;
        assert!(result.is_err());
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use yellowstone_vixen_mock::*;
    use super::*;

    #[tokio::test]
    async fn test_handler_with_parser() {
        let fixtures = FixtureLoader::new("./fixtures")
            .load_transactions("test_txs.json")
            .build();

        let mock_env = MockEnvironment::new(fixtures);
        let parser = TestParser;
        let handler = TestHandler::new();

        let pipeline = Pipeline::new(parser, vec![handler.boxed()]);
        let result = mock_env.run_pipeline(pipeline).await;

        assert!(result.is_ok());
    }
}
```

## Handler Best Practices

### Design Principles

1. **Idempotency** - Safe to process the same data multiple times
2. **Error Isolation** - Handler failures shouldn't crash the system
3. **Resource Management** - Properly manage connections and resources
4. **Observability** - Include logging and metrics
5. **Performance** - Optimize for throughput and latency

### Operational Considerations

1. **Monitoring** - Track handler performance and errors
2. **Scaling** - Design for horizontal scaling
3. **Backpressure** - Handle overload gracefully
4. **Maintenance** - Plan for updates and migrations
5. **Security** - Validate inputs and protect sensitive data

This comprehensive handler API reference provides the foundation for building robust, scalable data processing pipelines with Yellowstone Vixen.
