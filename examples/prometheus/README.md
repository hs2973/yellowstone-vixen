# Prometheus Metrics Example

This example demonstrates comprehensive metrics integration using Prometheus with Yellowstone Vixen. It shows how to collect, export, and visualize metrics from Solana program parsing operations.

## Overview

This example showcases:

- **Built-in Prometheus Integration**: Out-of-the-box metrics collection
- **Custom Metrics**: Application-specific metric creation
- **FilterPipeline Usage**: Advanced transaction filtering with metrics
- **Shared Data Features**: Access to transaction signatures and slot numbers
- **Dashboard Setup**: Pre-configured Prometheus and Grafana dashboards
- **Performance Monitoring**: Real-time monitoring of parsing performance

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Dragon's      │    │   Vixen         │    │   Prometheus    │
│   Mouth         │───▶│   Runtime       │───▶│   Server        │
│   (Data Source) │    │   (Metrics)     │    │   (Collection)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │   Custom        │    │   Grafana       │
                       │   Handlers      │    │   Dashboard     │
                       │   (Business)    │    │   (Visualization)│
                       └─────────────────┘    └─────────────────┘
```

## Features Demonstrated

### 1. Raydium AMM v4 Parser with Filtering

The example focuses on Raydium AMM v4 transactions with advanced filtering:

```rust
// Filter transactions that include specific Raydium accounts
.instruction(FilterPipeline::new(
    RaydiumAmmV4IxParser,
    [RaydiumAmmV4IxLogger],
    Prefilter::builder()
        .transaction_accounts_include([
            Pubkey::from_str("CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW").unwrap(),
        ])
        .transaction_accounts_required([
            Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(),
        ])
        .build()
))
```

### 2. Shared Data Access

Demonstrates accessing transaction-wide context:

```rust
impl Handler<InstructionUpdateOutput<RaydiumAmmV4ProgramIx>> for RaydiumAmmV4IxLogger {
    async fn handle(&self, value: &InstructionUpdateOutput<RaydiumAmmV4ProgramIx>) -> HandlerResult<()> {
        match &value.parsed_ix {
            RaydiumAmmV4ProgramIx::SwapBaseIn(accounts, data) => {
                tracing::info!(
                    signature = %value.signature,
                    slot = value.slot,
                    amount_in = data.amount_in,
                    amount_out = data.amount_out,
                    "Raydium swap base in detected"
                );
            }
            RaydiumAmmV4ProgramIx::SwapBaseOut(accounts, data) => {
                tracing::info!(
                    signature = %value.signature,
                    slot = value.slot,
                    amount_in = data.amount_in,
                    amount_out = data.amount_out,
                    "Raydium swap base out detected"
                );
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 3. Built-in Prometheus Metrics

Vixen automatically collects and exports these metrics:

- `vixen_transactions_processed_total`: Total transactions processed
- `vixen_accounts_processed_total`: Total accounts processed
- `vixen_instructions_processed_total`: Total instructions processed
- `vixen_parse_errors_total`: Total parsing errors by parser
- `vixen_handler_errors_total`: Total handler execution errors
- `vixen_processing_duration_seconds`: Processing time distributions
- `vixen_pipeline_lag_seconds`: Lag between event time and processing

## Configuration

### Vixen Configuration (config.toml)

```toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "your-api-token-here"
timeout = 60
keepalive-time = 30

[buffer]
buffer-size = 1000
num-workers = 4
batch-size = 100

[metrics]
# Prometheus metrics endpoint
endpoint = "0.0.0.0:9090"
# Metrics collection interval (seconds)
interval = 15
# Custom labels for all metrics
labels = { environment = "production", service = "raydium-parser" }

[logging]
level = "info"
format = "json"
```

### Prometheus Configuration (prometheus.yml)

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "raydium_rules.yml"

scrape_configs:
  - job_name: 'vixen-raydium'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
    metrics_path: /metrics
    
  - job_name: 'vixen-metrics'
    static_configs:
      - targets: ['host.docker.internal:9090']
    scrape_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

## Running the Example

### 1. Start the Vixen Parser

```bash
cd examples/prometheus
cargo run -- --config config.toml
```

The application will start processing Raydium transactions and expose metrics at `http://localhost:9090/metrics`.

### 2. Start Prometheus and Grafana (Docker)

Start the monitoring stack using Docker Compose:

```bash
# Start Prometheus and Grafana
docker-compose up -d

# Check status
docker-compose ps
```

Services will be available at:
- **Prometheus**: http://localhost:9091 (Web UI)
- **Grafana**: http://localhost:3000 (Dashboard)
- **Vixen Metrics**: http://localhost:9090/metrics (Raw metrics)

### 3. Access Grafana Dashboard

1. **Login to Grafana**: http://localhost:3000
   - Username: `admin`
   - Password: `admin`

2. **Import Dashboard**: Use the pre-configured dashboard at `grafana/dashboards/vixen-raydium.json`

3. **Explore Metrics**: View real-time Raydium swap activity and performance metrics

## Available Metrics

### Core Vixen Metrics

```
# Transaction processing
vixen_transactions_processed_total{parser="raydium_amm_v4"} 1250

# Account processing  
vixen_accounts_processed_total{parser="raydium_amm_v4"} 3750

# Instruction processing
vixen_instructions_processed_total{parser="raydium_amm_v4",instruction_type="swap_base_in"} 450
vixen_instructions_processed_total{parser="raydium_amm_v4",instruction_type="swap_base_out"} 320

# Error tracking
vixen_parse_errors_total{parser="raydium_amm_v4",error_type="invalid_data"} 5
vixen_handler_errors_total{handler="raydium_logger",error_type="timeout"} 2

# Performance metrics
vixen_processing_duration_seconds_bucket{parser="raydium_amm_v4",le="0.001"} 1150
vixen_processing_duration_seconds_bucket{parser="raydium_amm_v4",le="0.01"} 1240
vixen_processing_duration_seconds_bucket{parser="raydium_amm_v4",le="+Inf"} 1250

# Pipeline lag
vixen_pipeline_lag_seconds{parser="raydium_amm_v4"} 0.025
```

### Custom Application Metrics

Add custom metrics for business logic:

```rust
use yellowstone_vixen::metrics::{Counter, Histogram, Instrumenter};

#[derive(Debug)]
pub struct RaydiumMetrics<I: Instrumenter> {
    swap_volume_usd: Histogram<I>,
    large_swaps_count: Counter<I>,
    pool_interactions: Counter<I>,
}

impl<I: Instrumenter> RaydiumMetrics<I> {
    pub fn new(instrumenter: &I) -> Self {
        Self {
            swap_volume_usd: instrumenter
                .histogram("raydium_swap_volume_usd")
                .with_description("Volume of swaps in USD"),
            large_swaps_count: instrumenter
                .counter("raydium_large_swaps_total")
                .with_description("Number of swaps over $10k"),
            pool_interactions: instrumenter
                .counter("raydium_pool_interactions_total")
                .with_description("Pool interactions by type"),
        }
    }
}

impl<I: Instrumenter> Handler<RaydiumAmmV4ProgramIx> for RaydiumMetrics<I> {
    async fn handle(&self, ix: &RaydiumAmmV4ProgramIx) -> HandlerResult<()> {
        match ix {
            RaydiumAmmV4ProgramIx::SwapBaseIn(accounts, data) => {
                let volume_usd = calculate_usd_value(data.amount_in, &accounts.token_mint);
                self.swap_volume_usd.record(volume_usd);
                
                if volume_usd > 10000.0 {
                    self.large_swaps_count.inc();
                }
                
                self.pool_interactions.inc_with_labels(&[("type", "swap_in")]);
            }
            RaydiumAmmV4ProgramIx::AddLiquidity(accounts, data) => {
                self.pool_interactions.inc_with_labels(&[("type", "add_liquidity")]);
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Monitoring Queries

### Prometheus Queries

Useful PromQL queries for monitoring:

```promql
# Swap rate per minute
rate(vixen_instructions_processed_total{instruction_type="swap_base_in"}[1m])

# Average processing latency
rate(vixen_processing_duration_seconds_sum[5m]) / rate(vixen_processing_duration_seconds_count[5m])

# Error rate percentage  
rate(vixen_parse_errors_total[5m]) / rate(vixen_transactions_processed_total[5m]) * 100

# Large swap alert
increase(raydium_large_swaps_total[1m]) > 0

# Pipeline lag spike detection
vixen_pipeline_lag_seconds > 1.0
```

### Grafana Dashboard Panels

#### 1. Transaction Processing Rate
```json
{
  "targets": [
    {
      "expr": "rate(vixen_transactions_processed_total[1m])",
      "legendFormat": "{{parser}} transactions/sec"
    }
  ],
  "title": "Transaction Processing Rate"
}
```

#### 2. Swap Volume Distribution
```json
{
  "targets": [
    {
      "expr": "histogram_quantile(0.95, rate(raydium_swap_volume_usd_bucket[5m]))",
      "legendFormat": "95th percentile"
    },
    {
      "expr": "histogram_quantile(0.50, rate(raydium_swap_volume_usd_bucket[5m]))",
      "legendFormat": "Median"
    }
  ],
  "title": "Swap Volume Distribution"
}
```

#### 3. Error Rate Monitoring
```json
{
  "targets": [
    {
      "expr": "rate(vixen_parse_errors_total[5m])",
      "legendFormat": "{{parser}} - {{error_type}}"
    }
  ],
  "title": "Parse Error Rate"
}
```

## Alerting Rules

### Prometheus Alerting Rules (raydium_rules.yml)

```yaml
groups:
  - name: raydium_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(vixen_parse_errors_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected in Raydium parser"
          description: "Error rate is {{ $value }} errors/sec for parser {{ $labels.parser }}"

      - alert: HighLatency
        expr: vixen_pipeline_lag_seconds > 5.0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "High processing latency detected"
          description: "Pipeline lag is {{ $value }} seconds"

      - alert: LargeSwapDetected
        expr: increase(raydium_large_swaps_total[1m]) > 0
        labels:
          severity: info
        annotations:
          summary: "Large Raydium swap detected"
          description: "{{ $value }} large swaps (>$10k) in the last minute"

      - alert: ProcessingStalled
        expr: rate(vixen_transactions_processed_total[5m]) == 0
        for: 30s
        labels:
          severity: critical
        annotations:
          summary: "Transaction processing has stalled"
          description: "No transactions processed in the last 5 minutes"
```

## Custom Handler Examples

### Volume Tracking Handler

```rust
#[derive(Debug)]
pub struct VolumeTracker {
    daily_volume: Arc<Mutex<f64>>,
    volume_histogram: Histogram<Registry>,
}

impl Handler<RaydiumAmmV4ProgramIx> for VolumeTracker {
    async fn handle(&self, ix: &RaydiumAmmV4ProgramIx) -> HandlerResult<()> {
        if let Some(volume) = extract_swap_volume(ix) {
            // Update daily volume
            {
                let mut daily = self.daily_volume.lock().await;
                *daily += volume;
            }
            
            // Record in histogram
            self.volume_histogram.record(volume);
            
            // Log large swaps
            if volume > 10000.0 {
                tracing::warn!(volume = volume, "Large swap detected");
            }
        }
        Ok(())
    }
}
```

### Pool Analytics Handler

```rust
#[derive(Debug)]
pub struct PoolAnalytics {
    pool_volumes: Arc<Mutex<HashMap<Pubkey, f64>>>,
    pool_counter: Counter<Registry>,
}

impl Handler<RaydiumAmmV4ProgramIx> for PoolAnalytics {
    async fn handle(&self, ix: &RaydiumAmmV4ProgramIx) -> HandlerResult<()> {
        match ix {
            RaydiumAmmV4ProgramIx::SwapBaseIn(accounts, data) => {
                self.track_pool_activity(&accounts.pool, data.amount_in as f64);
            }
            RaydiumAmmV4ProgramIx::AddLiquidity(accounts, data) => {
                self.track_liquidity_addition(&accounts.pool, data);
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Performance Optimization

### High-Throughput Configuration

For high-volume monitoring:

```toml
[buffer]
buffer-size = 5000
num-workers = 8
batch-size = 500

[metrics]
interval = 5  # More frequent updates
export-batch-size = 1000
```

### Memory-Efficient Metrics

```rust
// Use sampling for high-cardinality metrics
impl MetricsHandler {
    fn should_sample(&self, volume: f64) -> bool {
        // Sample 100% of large swaps, 1% of small swaps
        volume > 1000.0 || rand::random::<f64>() < 0.01
    }
}
```

## Troubleshooting

### Common Issues

1. **Missing Metrics**: Check that Prometheus is scraping the correct endpoint
2. **High Memory Usage**: Reduce metric cardinality or increase export frequency
3. **Connection Errors**: Verify Vixen is running and accessible on port 9090
4. **Dashboard Issues**: Ensure Grafana data source is configured correctly

### Debug Commands

```bash
# Check metrics endpoint
curl http://localhost:9090/metrics | grep raydium

# Test Prometheus connectivity
curl http://localhost:9091/api/v1/targets

# Check Docker services
docker-compose logs prometheus
docker-compose logs grafana
```

This example provides a complete monitoring solution for Solana program parsing with real-time metrics, alerting, and visualization capabilities.