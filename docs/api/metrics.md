# Metrics API

This reference documents the metrics collection and monitoring capabilities in Yellowstone Vixen, including built-in metrics, custom metrics, and integration with monitoring systems.

## Metrics Overview

Yellowstone Vixen provides comprehensive metrics collection for monitoring pipeline performance, health, and operational status. Metrics are collected at multiple levels:

- **System Level** - Runtime performance and resource usage
- **Pipeline Level** - Parser and handler performance
- **Component Level** - Individual parser/handler metrics
- **Custom Metrics** - User-defined metrics

## Built-in Metrics

### Runtime Metrics

```rust
pub struct RuntimeMetrics {
    // Message processing
    pub messages_received: Counter,
    pub messages_processed: Counter,
    pub messages_dropped: Counter,

    // Performance
    pub processing_latency: Histogram,
    pub throughput_per_second: Gauge,

    // Errors
    pub errors_total: Counter,
    pub errors_by_type: HashMap<String, Counter>,

    // Resources
    pub memory_usage_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
    pub active_connections: Gauge,

    // Queues
    pub input_queue_depth: Gauge,
    pub processing_queue_depth: Gauge,
    pub output_queue_depth: Gauge,
}
```

### Pipeline Metrics

```rust
pub struct PipelineMetrics {
    // Pipeline identification
    pub pipeline_id: String,
    pub pipeline_type: PipelineType,

    // Processing metrics
    pub items_processed: Counter,
    pub items_failed: Counter,
    pub processing_time: Histogram,

    // Error tracking
    pub last_error: Option<String>,
    pub error_count: Counter,

    // Performance
    pub average_latency: Histogram,
    pub throughput: Gauge,
}
```

### Parser Metrics

```rust
pub struct ParserMetrics {
    // Parsing statistics
    pub instructions_parsed: Counter,
    pub accounts_parsed: Counter,
    pub parse_errors: Counter,

    // Performance
    pub parse_time: Histogram,
    pub parse_rate: Gauge,

    // Data quality
    pub filtered_messages: Counter,
    pub invalid_messages: Counter,
}
```

### Handler Metrics

```rust
pub struct HandlerMetrics {
    // Processing statistics
    pub items_handled: Counter,
    pub handle_errors: Counter,

    // Performance
    pub handle_time: Histogram,
    pub handle_rate: Gauge,

    // Resource usage
    pub memory_allocated: Gauge,
    pub connections_used: Gauge,
}
```

## Metrics Collection

### Metrics Registry

```rust
use yellowstone_vixen::metrics::{MetricsRegistry, PrometheusRegistry};

let registry = PrometheusRegistry::new();

// Register runtime metrics
registry.register_runtime_metrics();

// Register pipeline metrics
registry.register_pipeline_metrics(pipeline_id, &pipeline);

// Start metrics server
registry.start_server("0.0.0.0:9090").await?;
```

### Automatic Metrics Collection

```rust
use yellowstone_vixen::Runtime;

let runtime = Runtime::builder()
    .instruction(pipeline)
    .metrics(PrometheusRegistry::new())
    .enable_auto_metrics()
    .build(config)
    .await?;
```

### Manual Metrics Collection

```rust
impl MyHandler {
    async fn handle(&self, data: &MyData) -> HandlerResult<()> {
        let start = std::time::Instant::now();

        // Process data
        self.process_data(data).await?;

        let duration = start.elapsed();

        // Record metrics
        self.metrics.items_processed.inc();
        self.metrics.processing_time.observe(duration.as_secs_f64());

        Ok(())
    }
}
```

## Metrics Types

### Counters

Counters represent a single monotonically increasing value:

```rust
use yellowstone_vixen::metrics::Counter;

let counter = Counter::new("requests_total", "Total number of requests");
counter.inc(); // Increment by 1
counter.inc_by(5); // Increment by 5
```

### Gauges

Gauges represent a single value that can go up and down:

```rust
use yellowstone_vixen::metrics::Gauge;

let gauge = Gauge::new("queue_depth", "Current queue depth");
gauge.set(10.0); // Set to specific value
gauge.inc(); // Increment by 1
gauge.dec(); // Decrement by 1
gauge.add(5.0); // Add value
```

### Histograms

Histograms track the distribution of values:

```rust
use yellowstone_vixen::metrics::Histogram;

let histogram = Histogram::new(
    "request_duration_seconds",
    "Request duration in seconds",
    vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0] // Buckets
);

histogram.observe(1.2); // Record observation
```

### Summaries

Summaries calculate quantiles over a sliding time window:

```rust
use yellowstone_vixen::metrics::Summary;

let summary = Summary::new(
    "request_duration_quantiles",
    "Request duration quantiles",
    vec![0.5, 0.9, 0.95, 0.99] // Quantiles
);

summary.observe(1.2); // Record observation
```

## Custom Metrics

### Defining Custom Metrics

```rust
use yellowstone_vixen::metrics::{MetricsRegistry, Counter, Gauge, Histogram};

pub struct CustomMetrics {
    pub trades_processed: Counter,
    pub active_connections: Gauge,
    pub trade_value: Histogram,
    pub error_rate: Gauge,
}

impl CustomMetrics {
    pub fn new(registry: &MetricsRegistry) -> Self {
        Self {
            trades_processed: registry.register_counter(
                "trades_processed_total",
                "Total number of trades processed"
            ),
            active_connections: registry.register_gauge(
                "active_connections",
                "Number of active connections"
            ),
            trade_value: registry.register_histogram(
                "trade_value_usd",
                "Trade value in USD",
                vec![1.0, 10.0, 100.0, 1000.0, 10000.0]
            ),
            error_rate: registry.register_gauge(
                "error_rate_percent",
                "Error rate as percentage"
            ),
        }
    }
}
```

### Using Custom Metrics in Handlers

```rust
pub struct TradeHandler {
    metrics: CustomMetrics,
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl Handler<TradeData> for TradeHandler {
    async fn handle(&self, trade: &TradeData) -> HandlerResult<()> {
        // Record trade processing
        self.metrics.trades_processed.inc();

        // Record trade value
        self.metrics.trade_value.observe(trade.value_usd);

        // Process trade
        match self.db.store_trade(trade).await {
            Ok(()) => {
                // Success - could update success rate
            }
            Err(e) => {
                // Error - update error rate
                self.metrics.error_rate.set(calculate_error_rate());
                return Err(e.into());
            }
        }

        Ok(())
    }
}
```

### Business Logic Metrics

```rust
pub struct BusinessMetrics {
    pub large_trades: Counter,
    pub small_trades: Counter,
    pub arbitrage_opportunities: Counter,
    pub liquidity_events: Counter,
}

impl BusinessMetrics {
    pub fn record_trade(&self, trade: &TradeData) {
        if trade.value_usd > 10000.0 {
            self.large_trades.inc();
        } else {
            self.small_trades.inc();
        }

        // Check for arbitrage opportunities
        if self.is_arbitrage_opportunity(trade) {
            self.arbitrage_opportunities.inc();
        }
    }
}
```

## Metrics Integration

### Prometheus Integration

```rust
use yellowstone_vixen::metrics::PrometheusRegistry;

let registry = PrometheusRegistry::new();

// Start HTTP server for Prometheus scraping
registry.start_server("0.0.0.0:9090").await?;

// Metrics will be available at http://localhost:9090/metrics
```

### OpenTelemetry Integration

```rust
use yellowstone_vixen::metrics::OpenTelemetryRegistry;

let registry = OpenTelemetryRegistry::new(
    "my-service",
    "v1.0.0",
    "http://localhost:4317" // OTLP endpoint
);

// Export metrics to OpenTelemetry collector
registry.start_export().await?;
```

### Custom Exporters

```rust
use yellowstone_vixen::metrics::{MetricsRegistry, MetricsExporter};

pub struct CustomExporter {
    endpoint: String,
    client: reqwest::Client,
}

impl MetricsExporter for CustomExporter {
    async fn export(&self, metrics: &MetricsSnapshot) -> Result<(), ExportError> {
        let payload = serde_json::to_string(metrics)?;
        self.client
            .post(&self.endpoint)
            .body(payload)
            .send()
            .await?;
        Ok(())
    }
}
```

## Metrics Configuration

### Configuration Options

```toml
[metrics]
# Enable metrics collection
enabled = true

# Metrics prefix
prefix = "yellowstone_vixen"

# Collection interval (seconds)
collection_interval = 10

# Export configuration
[metrics.export]
# Export format
format = "prometheus"  # prometheus, opentelemetry, custom

# Export interval (seconds)
interval = 30

# Prometheus configuration
[metrics.prometheus]
# Server address
address = "0.0.0.0:9090"

# Metrics path
path = "/metrics"

# OpenTelemetry configuration
[metrics.opentelemetry]
# Collector endpoint
endpoint = "http://localhost:4317"

# Service information
service_name = "yellowstone-vixen"
service_version = "0.4.0"
```

### Runtime Configuration

```rust
use yellowstone_vixen::Runtime;

let runtime = Runtime::builder()
    .instruction(pipeline)
    .metrics_config(MetricsConfig {
        enabled: true,
        collection_interval: Duration::from_secs(10),
        exporters: vec![
            Box::new(PrometheusExporter::new("0.0.0.0:9090")),
            Box::new(OpenTelemetryExporter::new("http://localhost:4317")),
        ],
    })
    .build(config)
    .await?;
```

## Monitoring and Alerting

### Health Checks

```rust
use yellowstone_vixen::metrics::HealthChecker;

pub struct PipelineHealthChecker {
    metrics: Arc<RuntimeMetrics>,
    thresholds: HealthThresholds,
}

#[async_trait::async_trait]
impl HealthChecker for PipelineHealthChecker {
    async fn check(&self) -> HealthCheckResult {
        let error_rate = self.metrics.errors_total.value() as f64
            / self.metrics.messages_processed.value() as f64;

        if error_rate > self.thresholds.max_error_rate {
            HealthCheckResult::unhealthy("High error rate")
        } else if error_rate > self.thresholds.warning_error_rate {
            HealthCheckResult::degraded("Elevated error rate")
        } else {
            HealthCheckResult::healthy()
        }
    }
}
```

### Alerting Rules

Example Prometheus alerting rules:

```yaml
groups:
  - name: yellowstone_vixen
    rules:
      - alert: HighErrorRate
        expr: rate(yellowstone_vixen_errors_total[5m]) / rate(yellowstone_vixen_messages_processed_total[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate in Yellowstone Vixen"

      - alert: LowThroughput
        expr: rate(yellowstone_vixen_messages_processed_total[5m]) < 100
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Low throughput in Yellowstone Vixen"
```

### Dashboard Examples

#### Grafana Dashboard JSON

```json
{
  "dashboard": {
    "title": "Yellowstone Vixen Metrics",
    "panels": [
      {
        "title": "Message Throughput",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(yellowstone_vixen_messages_processed_total[5m])",
            "legendFormat": "Messages/sec"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(yellowstone_vixen_errors_total[5m]) / rate(yellowstone_vixen_messages_processed_total[5m]) * 100",
            "legendFormat": "Error %"
          }
        ]
      }
    ]
  }
}
```

## Performance Considerations

### Metrics Overhead

- **Memory Usage** - Metrics storage scales with number of metrics
- **CPU Usage** - Metric collection and export has CPU overhead
- **Network Usage** - Exporting metrics consumes network bandwidth

### Optimization Strategies

```rust
// Use sampling for high-frequency metrics
let sampled_histogram = SampledHistogram::new(histogram, 0.1); // 10% sampling

// Batch metric updates
let batch = MetricBatch::new();
batch.queue(counter.inc());
batch.queue(gauge.set(value));
// Flush batch
batch.flush().await?;
```

### Resource Monitoring

```rust
pub struct ResourceMonitor {
    metrics: SystemMetrics,
    thresholds: ResourceThresholds,
}

impl ResourceMonitor {
    pub async fn check_resources(&self) -> Result<(), ResourceError> {
        let memory_usage = self.get_memory_usage();
        let cpu_usage = self.get_cpu_usage();

        if memory_usage > self.thresholds.max_memory {
            self.metrics.memory_pressure.inc();
            return Err(ResourceError::MemoryPressure);
        }

        if cpu_usage > self.thresholds.max_cpu {
            self.metrics.cpu_pressure.inc();
            return Err(ResourceError::CpuPressure);
        }

        Ok(())
    }
}
```

## Best Practices

### Metrics Naming

1. **Use Descriptive Names** - `requests_total` not `req`
2. **Include Units** - `duration_seconds` not `duration`
3. **Use Consistent Prefixes** - `yellowstone_vixen_*`
4. **Avoid Special Characters** - Use underscores instead of hyphens

### Metrics Collection

1. **Collect What Matters** - Focus on key performance indicators
2. **Use Appropriate Types** - Counters for events, gauges for states, histograms for distributions
3. **Set Reasonable Cardinality** - Avoid high-cardinality labels
4. **Document Metrics** - Include descriptions and units

### Alerting

1. **Set Meaningful Thresholds** - Based on historical data and business requirements
2. **Use Multiple Severity Levels** - Critical, warning, info
3. **Include Context** - Provide enough information for debugging
4. **Test Alerts** - Regularly test alerting rules

### Monitoring

1. **Monitor Trends** - Look at metric trends over time
2. **Set Baselines** - Establish normal operating ranges
3. **Correlate Metrics** - Look for relationships between metrics
4. **Automate Responses** - Use alerts to trigger automated responses

This comprehensive metrics API reference provides everything needed to implement robust monitoring and observability in Yellowstone Vixen applications.
