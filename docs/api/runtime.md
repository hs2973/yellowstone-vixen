# Runtime API

This reference documents the Yellowstone Vixen runtime API, including configuration, lifecycle management, and operational controls.

## Runtime Architecture

The Vixen runtime is the central orchestrator that manages parsers, handlers, and data flow. It provides:

- **Data Ingestion** - Connection to Yellowstone gRPC streams
- **Pipeline Management** - Coordination of parsing and handling
- **Resource Management** - Thread pools, connections, and memory
- **Monitoring** - Metrics collection and health monitoring
- **Lifecycle Control** - Startup, shutdown, and error recovery

## Runtime Builder

### Basic Construction

```rust
use yellowstone_vixen::Runtime;

let runtime = Runtime::builder()
    .instruction(Pipeline::new(parser, handlers))
    .account(Pipeline::new(account_parser, account_handlers))
    .build(config)
    .await?;
```

### Builder Methods

#### Pipeline Configuration

```rust
impl RuntimeBuilder {
    // Add instruction pipeline
    pub fn instruction<P, H>(self, pipeline: Pipeline<P, H>) -> Self
    where
        P: InstructionParser + Send + Sync + 'static,
        H: Handler<P::Instruction> + Send + Sync + 'static,

    // Add account pipeline
    pub fn account<P, H>(self, pipeline: Pipeline<P, H>) -> Self
    where
        P: AccountParser + Send + Sync + 'static,
        H: Handler<P::Account> + Send + Sync + 'static,

    // Add block metadata pipeline
    pub fn block_meta<H>(self, handler: H) -> Self
    where
        H: Handler<BlockMeta> + Send + Sync + 'static,
}
```

#### Source Configuration

```rust
impl RuntimeBuilder {
    // Configure Yellowstone gRPC source
    pub fn source<S>(self, source: S) -> Self
    where
        S: Source + Send + Sync + 'static,

    // Configure with Yellowstone config
    pub fn yellowstone_config(self, config: YellowstoneConfig) -> Self,

    // Set commitment level
    pub fn commitment_level(self, level: CommitmentLevel) -> Self,
}
```

#### Resource Management

```rust
impl RuntimeBuilder {
    // Configure thread pools
    pub fn worker_threads(self, count: usize) -> Self,

    // Configure buffer sizes
    pub fn buffer_config(self, config: BufferConfig) -> Self,

    // Configure metrics
    pub fn metrics<M>(self, metrics: M) -> Self
    where
        M: Metrics + Send + Sync + 'static,
}
```

#### Operational Configuration

```rust
impl RuntimeBuilder {
    // Set runtime configuration
    pub fn config(self, config: RuntimeConfig) -> Self,

    // Configure error handling
    pub fn error_handler<H>(self, handler: H) -> Self
    where
        H: ErrorHandler + Send + Sync + 'static,

    // Configure health checks
    pub fn health_check_interval(self, interval: Duration) -> Self,
}
```

## Runtime Configuration

### RuntimeConfig

```rust
pub struct RuntimeConfig {
    // Thread pool configuration
    pub worker_threads: usize,
    pub max_blocking_threads: usize,

    // Buffer configuration
    pub buffer_config: BufferConfig,

    // Network configuration
    pub network_config: NetworkConfig,

    // Metrics configuration
    pub metrics_config: MetricsConfig,

    // Error handling configuration
    pub error_config: ErrorConfig,
}
```

### BufferConfig

```rust
pub struct BufferConfig {
    // Maximum buffer capacity
    pub max_capacity: usize,

    // Batch processing size
    pub batch_size: usize,

    // Batch timeout
    pub batch_timeout: Duration,

    // Overflow policy
    pub overflow_policy: OverflowPolicy,
}

pub enum OverflowPolicy {
    // Drop oldest messages
    DropOldest,
    // Block until space available
    Block,
    // Return error
    Error,
}
```

### NetworkConfig

```rust
pub struct NetworkConfig {
    // Connection timeout
    pub connect_timeout: Duration,

    // Request timeout
    pub request_timeout: Duration,

    // Keep-alive interval
    pub keep_alive_interval: Duration,

    // Maximum connections per host
    pub max_connections_per_host: usize,

    // Connection pool size
    pub connection_pool_size: usize,
}
```

## Runtime Lifecycle

### Initialization

```rust
impl Runtime {
    // Create new runtime
    pub async fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        // Implementation
    }

    // Initialize from builder
    pub async fn from_builder(builder: RuntimeBuilder) -> Result<Self, RuntimeError> {
        // Implementation
    }
}
```

### Startup

```rust
impl Runtime {
    // Start processing
    pub async fn run(self) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Start with timeout
    pub async fn run_for(self, duration: Duration) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Start and return handle for control
    pub fn run_handle(self) -> (RuntimeHandle, RuntimeTask) {
        // Implementation
    }
}
```

### RuntimeHandle

```rust
pub struct RuntimeHandle {
    // Control the runtime
    command_sender: mpsc::Sender<RuntimeCommand>,
}

impl RuntimeHandle {
    // Graceful shutdown
    pub async fn shutdown(self) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Pause processing
    pub async fn pause(&self) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Resume processing
    pub async fn resume(&self) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Get runtime status
    pub async fn status(&self) -> Result<RuntimeStatus, RuntimeError> {
        // Implementation
    }

    // Update configuration
    pub async fn update_config(&self, config: RuntimeConfig) -> Result<(), RuntimeError> {
        // Implementation
    }
}
```

### RuntimeStatus

```rust
pub struct RuntimeStatus {
    // Runtime state
    pub state: RuntimeState,

    // Performance metrics
    pub metrics: RuntimeMetrics,

    // Pipeline statuses
    pub pipelines: Vec<PipelineStatus>,

    // Uptime
    pub uptime: Duration,

    // Last error (if any)
    pub last_error: Option<RuntimeError>,
}

pub enum RuntimeState {
    Initializing,
    Running,
    Paused,
    Error,
    Shutdown,
}
```

### Shutdown

```rust
impl Runtime {
    // Graceful shutdown
    pub async fn shutdown(self) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Force shutdown
    pub async fn shutdown_now(self) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Shutdown with timeout
    pub async fn shutdown_with_timeout(self, timeout: Duration) -> Result<(), RuntimeError> {
        // Implementation
    }
}
```

## Runtime Monitoring

### Metrics Collection

```rust
pub struct RuntimeMetrics {
    // Throughput metrics
    pub messages_processed: Counter,
    pub messages_per_second: Gauge,

    // Latency metrics
    pub processing_latency: Histogram,

    // Error metrics
    pub errors_total: Counter,
    pub errors_by_type: HashMap<String, Counter>,

    // Resource metrics
    pub memory_usage: Gauge,
    pub cpu_usage: Gauge,
    pub active_connections: Gauge,

    // Queue metrics
    pub queue_depth: Gauge,
    pub queue_capacity: Gauge,
}
```

### Health Checks

```rust
pub trait HealthCheck {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthCheckResult;
}

pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: Option<HashMap<String, String>>,
}

pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

### Logging

```rust
impl Runtime {
    // Enable debug logging
    pub fn enable_debug_logging(&mut self) -> &mut Self {
        // Implementation
    }

    // Set log level
    pub fn set_log_level(&mut self, level: tracing::Level) -> &mut Self {
        // Implementation
    }

    // Configure structured logging
    pub fn configure_logging(&mut self, config: LoggingConfig) -> &mut Self {
        // Implementation
    }
}
```

## Error Handling

### RuntimeError Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Parser error: {0}")]
    Parser(String),

    #[error("Handler error: {0}")]
    Handler(String),

    #[error("Resource error: {0}")]
    Resource(String),

    #[error("System error: {0}")]
    System(#[from] std::io::Error),
}
```

### Error Recovery

```rust
impl Runtime {
    // Configure error recovery
    pub fn with_error_recovery(&mut self, config: ErrorRecoveryConfig) -> &mut Self {
        // Implementation
    }

    // Handle errors
    async fn handle_error(&self, error: RuntimeError) -> Result<(), RuntimeError> {
        // Implementation
    }
}
```

### ErrorRecoveryConfig

```rust
pub struct ErrorRecoveryConfig {
    // Maximum retry attempts
    pub max_retries: usize,

    // Retry backoff strategy
    pub backoff_strategy: BackoffStrategy,

    // Error classification
    pub error_classification: HashMap<String, ErrorAction>,
}

pub enum ErrorAction {
    Retry,
    Skip,
    Shutdown,
    Custom(Box<dyn ErrorHandler>),
}

pub enum BackoffStrategy {
    Fixed(Duration),
    Exponential { base: Duration, max: Duration },
    Linear { increment: Duration, max: Duration },
}
```

## Performance Tuning

### Thread Pool Configuration

```rust
pub struct ThreadPoolConfig {
    // Number of worker threads
    pub worker_threads: usize,

    // Thread stack size
    pub stack_size: usize,

    // Thread name prefix
    pub name_prefix: String,

    // CPU affinity
    pub cpu_affinity: Option<Vec<usize>>,
}
```

### Memory Management

```rust
pub struct MemoryConfig {
    // Maximum memory usage
    pub max_memory: usize,

    // Buffer allocation strategy
    pub allocation_strategy: AllocationStrategy,

    // Garbage collection settings
    pub gc_config: GcConfig,
}

pub enum AllocationStrategy {
    // Pre-allocate fixed buffers
    Fixed,

    // Dynamic allocation with limits
    Dynamic,

    // Memory pool allocation
    Pooled,
}
```

### Performance Monitoring

```rust
impl Runtime {
    // Get performance profile
    pub async fn performance_profile(&self) -> Result<PerformanceProfile, RuntimeError> {
        // Implementation
    }

    // Enable profiling
    pub fn enable_profiling(&mut self) -> &mut Self {
        // Implementation
    }

    // Collect performance metrics
    pub async fn collect_metrics(&self) -> Result<PerformanceMetrics, RuntimeError> {
        // Implementation
    }
}
```

## Advanced Features

### Hot Reloading

```rust
impl Runtime {
    // Reload configuration
    pub async fn reload_config(&self, config: RuntimeConfig) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Reload pipelines
    pub async fn reload_pipelines(&self, pipelines: Vec<Pipeline>) -> Result<(), RuntimeError> {
        // Implementation
    }
}
```

### Dynamic Scaling

```rust
impl Runtime {
    // Scale worker threads
    pub async fn scale_workers(&self, new_count: usize) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Auto-scaling configuration
    pub fn with_auto_scaling(&mut self, config: AutoScalingConfig) -> &mut Self {
        // Implementation
    }
}
```

### Distributed Runtime

```rust
pub struct DistributedRuntime {
    // Node configuration
    node_id: String,

    // Cluster configuration
    cluster_config: ClusterConfig,

    // Coordination
    coordinator: Box<dyn Coordinator>,
}

impl DistributedRuntime {
    // Join cluster
    pub async fn join_cluster(&self, seeds: Vec<String>) -> Result<(), RuntimeError> {
        // Implementation
    }

    // Leave cluster
    pub async fn leave_cluster(&self) -> Result<(), RuntimeError> {
        // Implementation
    }
}
```

## Testing Runtime

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use yellowstone_vixen_mock::*;

    #[tokio::test]
    async fn test_runtime_initialization() {
        let config = create_test_config();
        let runtime = Runtime::builder()
            .config(config)
            .build()
            .await;

        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_runtime_lifecycle() {
        let runtime = create_test_runtime().await;
        let handle = runtime.run_handle();

        // Test status
        let status = handle.status().await.unwrap();
        assert_eq!(status.state, RuntimeState::Running);

        // Test shutdown
        handle.shutdown().await.unwrap();
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
    async fn test_runtime_with_pipelines() {
        let fixtures = FixtureLoader::new("./fixtures")
            .load_transactions("test_txs.json")
            .build();

        let mock_env = MockEnvironment::new(fixtures);

        let runtime = Runtime::builder()
            .instruction(Pipeline::new(TestParser, vec![TestHandler::new()]))
            .source(mock_env.source())
            .build()
            .await
            .unwrap();

        let result = runtime.run_for(Duration::from_secs(10)).await;
        assert!(result.is_ok());
    }
}
```

## Best Practices

### Configuration

1. **Environment-Specific Configs** - Use different configs for dev/staging/prod
2. **Configuration Validation** - Validate configs at startup
3. **Secrets Management** - Don't hardcode secrets in config files
4. **Config Versioning** - Version control configuration changes

### Operations

1. **Monitoring** - Set up comprehensive monitoring and alerting
2. **Logging** - Configure appropriate log levels and structured logging
3. **Health Checks** - Implement and monitor health check endpoints
4. **Resource Limits** - Set appropriate resource limits and quotas

### Performance

1. **Load Testing** - Regularly test with expected load patterns
2. **Profiling** - Use profiling tools to identify bottlenecks
3. **Optimization** - Tune thread pools, buffer sizes, and memory usage
4. **Scaling** - Plan for horizontal and vertical scaling

### Reliability

1. **Error Handling** - Implement comprehensive error handling and recovery
2. **Graceful Shutdown** - Ensure clean shutdown with proper cleanup
3. **Backups** - Regular backups of critical data and configurations
4. **Disaster Recovery** - Plan for and test disaster recovery procedures

This runtime API reference provides comprehensive guidance for configuring, operating, and optimizing Yellowstone Vixen runtimes in production environments.
