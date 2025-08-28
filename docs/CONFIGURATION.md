# Configuration Reference

This document provides a comprehensive reference for configuring Yellowstone Vixen.

## Table of Contents

- [Configuration Overview](#configuration-overview)
- [Configuration Sources](#configuration-sources)
- [Core Configuration](#core-configuration)
- [Source Configuration](#source-configuration)
- [Metrics Configuration](#metrics-configuration)
- [Stream Server Configuration](#stream-server-configuration)
- [Environment Variables](#environment-variables)
- [Configuration Examples](#configuration-examples)
- [Validation & Error Handling](#validation--error-handling)

## Configuration Overview

Yellowstone Vixen uses a hierarchical configuration system that supports:

- **TOML Files**: Primary configuration format
- **Environment Variables**: Override any configuration value
- **Command Line Arguments**: Runtime parameter overrides
- **Programmatic Configuration**: Rust API for dynamic configuration

Configuration follows a type-safe approach with automatic validation and sensible defaults.

## Configuration Sources

### Priority Order

Configuration values are resolved in the following priority order (highest to lowest):

1. **Command Line Arguments**: `--config`, `--endpoint`, etc.
2. **Environment Variables**: `ENDPOINT`, `X_TOKEN`, etc.
3. **Configuration Files**: `Vixen.toml`, `config.toml`, etc.
4. **Default Values**: Built-in defaults for optional values

### File Formats

#### TOML (Recommended)
```toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "your-token-here"

[metrics]
endpoint = "0.0.0.0:9090"
```

#### Environment Variables
```bash
export ENDPOINT="https://yellowstone-api.triton.one"
export X_TOKEN="your-token-here"
export METRICS_ENDPOINT="0.0.0.0:9090"
```

## Core Configuration

### Runtime Configuration

```toml
[buffer]
# Buffer size for processing pipeline (default: 1000)
buffer-size = 1000

# Number of worker threads (default: CPU count)
num-workers = 4

# Batch processing size (default: 100)
batch-size = 100

# Maximum memory usage in MB (default: unlimited)
max-memory-mb = 1024

# Enable graceful shutdown (default: true)
graceful-shutdown = true

# Shutdown timeout in seconds (default: 30)
shutdown-timeout = 30
```

### Commitment Level

```rust
// Programmatic configuration
yellowstone_vixen::Runtime::builder()
    .commitment_level(yellowstone_vixen::CommitmentLevel::Confirmed)
    .build(config)
    .run();
```

Available commitment levels:
- `Processed`: Fastest, least reliable
- `Confirmed`: Balanced (default)
- `Finalized`: Slowest, most reliable

### Logging Configuration

```toml
[logging]
# Log level (default: "info")
level = "debug"

# Log format: "json" or "pretty" (default: "pretty")
format = "json"

# Include timestamps (default: true)
timestamps = true

# Include line numbers (default: false in release)
line-numbers = true

# Log file path (optional)
file = "/var/log/vixen.log"

# Log rotation settings
max-file-size = "100MB"
max-files = 5
```

## Source Configuration

### Yellowstone gRPC Source

```toml
[source]
# Yellowstone gRPC endpoint (required)
endpoint = "https://yellowstone-api.triton.one"

# Authentication token (required)
x-token = "your-api-token-here"

# Connection timeout in seconds (default: 60)
timeout = 60

# Enable TLS (default: true for https://)
tls = true

# Custom gRPC keepalive settings
keepalive-time = 30        # seconds
keepalive-timeout = 5      # seconds
keepalive-permit-without-calls = true

# Retry configuration
max-retries = 3
retry-delay = 5           # seconds
backoff-multiplier = 2.0

# Subscription filters
accounts = ["account1", "account2"]  # Optional: filter by accounts
programs = ["program1", "program2"]  # Optional: filter by programs
transactions = true                   # Subscribe to transactions (default: true)
slots = true                         # Subscribe to slots (default: true)
blocks = true                        # Subscribe to blocks (default: true)

# Advanced gRPC settings
max-receive-message-size = 4194304   # 4MB default
max-send-message-size = 4194304      # 4MB default
compression = "gzip"                 # Optional: "gzip" or "none"
```

### Yellowstone Fumarole Source

```toml
[source]
# Fumarole endpoint
endpoint = "https://fumarole-api.triton.one"

# Authentication token
x-token = "your-api-token-here"

# Subscriber name for ordered processing
subscriber-name = "my_subscriber_group"

# Consumer group settings
consumer-group = "vixen-processors"
auto-commit = true
commit-interval = 5000    # milliseconds

# Offset management
auto-offset-reset = "latest"  # "earliest" or "latest"
```

### Solana RPC Source

```toml
[source]
# Solana RPC endpoint
endpoint = "https://api.mainnet-beta.solana.com"

# WebSocket endpoint for real-time updates
ws-endpoint = "wss://api.mainnet-beta.solana.com"

# Rate limiting
requests-per-second = 100
burst-capacity = 200

# Polling intervals
account-poll-interval = 1000    # milliseconds
slot-poll-interval = 400        # milliseconds

# Batch processing
batch-size = 100
max-batch-wait = 100           # milliseconds
```

### Snapshot Source

```toml
[source]
# Snapshot file path or directory
path = "/path/to/snapshot"

# Processing mode: "sequential" or "parallel"
mode = "parallel"

# Number of parallel workers (default: CPU count)
workers = 8

# Chunk size for parallel processing
chunk-size = 10000

# Skip invalid accounts
skip-invalid = true

# Progress reporting interval
progress-interval = 10000      # accounts
```

## Metrics Configuration

### Prometheus Configuration

```toml
[metrics]
# Prometheus metrics endpoint (default: disabled)
endpoint = "0.0.0.0:9090"

# Metrics path (default: "/metrics")
path = "/metrics"

# Update interval in seconds (default: 15)
interval = 15

# Basic authentication (optional)
username = "admin"
password = "secret"

# TLS configuration (optional)
tls-cert = "/path/to/cert.pem"
tls-key = "/path/to/key.pem"

# Custom labels
labels = { environment = "production", region = "us-east-1" }

# Metric buckets for histograms
latency-buckets = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
size-buckets = [1, 10, 100, 1000, 10000, 100000]
```

### OpenTelemetry Configuration

```toml
[metrics]
# OpenTelemetry collector endpoint
endpoint = "http://localhost:4317"

# Service information
service-name = "yellowstone-vixen"
service-version = "0.4.0"
service-namespace = "solana"

# Resource attributes
resource-attributes = { 
    deployment.environment = "production",
    service.instance.id = "vixen-001"
}

# Export configuration
export-timeout = 30000         # milliseconds
export-batch-size = 512
export-schedule-delay = 5000   # milliseconds

# Sampling configuration
trace-sampling-ratio = 1.0     # 0.0 to 1.0
```

### Custom Metrics

```rust
// Programmatic metrics configuration
use yellowstone_vixen::metrics::{Prometheus, OpenTelemetry};

// Prometheus
yellowstone_vixen::Runtime::builder()
    .metrics(Prometheus)
    .build(config)
    .run();

// OpenTelemetry  
let meter_provider = /* your setup */;
yellowstone_vixen::Runtime::builder()
    .metrics(OpenTelemetry::new(meter_provider))
    .build(config)
    .run();

// No metrics
yellowstone_vixen::Runtime::builder()
    .build(config)
    .run();
```

## Stream Server Configuration

### gRPC Server

```toml
[grpc]
# Server bind address (default: "[::]:3030")
address = "0.0.0.0:3030"

# TLS configuration (optional)
tls-cert = "/path/to/server.crt"
tls-key = "/path/to/server.key"
tls-ca = "/path/to/ca.crt"

# Server limits
max-connections = 1000
max-concurrent-streams = 100
max-frame-size = 16777216      # 16MB

# Keepalive settings
keepalive-time = 60            # seconds
keepalive-timeout = 20         # seconds
keepalive-permit-without-calls = false

# Compression
enable-compression = true
compression-level = 6          # 1-9 for gzip

# Request/response limits
max-request-size = 4194304     # 4MB
max-response-size = 16777216   # 16MB
```

### Stream Configuration

```rust
use yellowstone_vixen_stream::Server;

Server::<_, YellowstoneGrpcSource>::builder()
    // Protocol buffer descriptor sets
    .descriptor_set(TOKEN_DESCRIPTOR_SET)
    .descriptor_set(RAYDIUM_DESCRIPTOR_SET)
    
    // Parser configuration
    .account(Proto::new(TokenAccountParser))
    .instruction(Proto::new(RaydiumIxParser))
    
    // Build with config
    .build(config)
    .run();
```

## Environment Variables

### Variable Naming Convention

Environment variables use SCREAMING_SNAKE_CASE and follow the configuration structure:

- `source.endpoint` → `ENDPOINT` or `SOURCE_ENDPOINT`
- `source.x-token` → `X_TOKEN` or `SOURCE_X_TOKEN`
- `metrics.endpoint` → `METRICS_ENDPOINT`
- `buffer.buffer-size` → `BUFFER_SIZE` or `BUFFER_BUFFER_SIZE`

### Complete Environment Variable Reference

#### Source Configuration
```bash
# Yellowstone gRPC
export ENDPOINT="https://yellowstone-api.triton.one"
export X_TOKEN="your-token"
export TIMEOUT="60"
export TLS="true"
export KEEPALIVE_TIME="30"
export KEEPALIVE_TIMEOUT="5"
export MAX_RETRIES="3"
export RETRY_DELAY="5"

# Fumarole
export SUBSCRIBER_NAME="my_subscriber"
export CONSUMER_GROUP="vixen-group"
export AUTO_OFFSET_RESET="latest"

# Solana RPC
export WS_ENDPOINT="wss://api.mainnet-beta.solana.com"
export REQUESTS_PER_SECOND="100"
export BATCH_SIZE="100"
```

#### Runtime Configuration
```bash
export BUFFER_SIZE="1000"
export NUM_WORKERS="4"
export BATCH_SIZE="100"
export MAX_MEMORY_MB="1024"
export GRACEFUL_SHUTDOWN="true"
export SHUTDOWN_TIMEOUT="30"
```

#### Metrics Configuration
```bash
export METRICS_ENDPOINT="0.0.0.0:9090"
export METRICS_PATH="/metrics"
export METRICS_INTERVAL="15"
export METRICS_USERNAME="admin"
export METRICS_PASSWORD="secret"
```

#### gRPC Configuration
```bash
export GRPC_ADDRESS="0.0.0.0:3030"
export GRPC_TLS_CERT="/path/to/cert.pem"
export GRPC_TLS_KEY="/path/to/key.pem"
export GRPC_MAX_CONNECTIONS="1000"
```

#### Logging Configuration
```bash
export LOG_LEVEL="info"
export LOG_FORMAT="json"
export LOG_FILE="/var/log/vixen.log"
export RUST_LOG="yellowstone_vixen=debug"
```

## Configuration Examples

### Production Configuration

```toml
# Production Vixen.toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "${X_TOKEN}"  # From environment
timeout = 120
keepalive-time = 30
max-retries = 5
retry-delay = 10

[buffer]
buffer-size = 5000
num-workers = 8
batch-size = 500
max-memory-mb = 2048
graceful-shutdown = true

[metrics]
endpoint = "0.0.0.0:9090"
interval = 10
labels = { environment = "production", service = "vixen" }

[grpc]
address = "0.0.0.0:3030"
max-connections = 1000
enable-compression = true

[logging]
level = "info"
format = "json"
file = "/var/log/vixen.log"
```

### Development Configuration

```toml
# Development Vixen.toml
[source]
endpoint = "https://api.devnet.solana.com"
timeout = 30

[buffer]
buffer-size = 100
num-workers = 2
batch-size = 10

[metrics]
endpoint = "127.0.0.1:9090"
interval = 5

[logging]
level = "debug"
format = "pretty"
timestamps = true
line-numbers = true
```

### Testing Configuration

```toml
# Test Vixen.toml
[source]
# Mock source configuration
type = "mock"
fixtures-path = "./fixtures"

[buffer]
buffer-size = 10
num-workers = 1
batch-size = 1

[logging]
level = "trace"
format = "pretty"
```

### High-Performance Configuration

```toml
# High-performance Vixen.toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "${X_TOKEN}"
timeout = 60
max-receive-message-size = 16777216  # 16MB
compression = "gzip"

[buffer]
buffer-size = 10000
num-workers = 16
batch-size = 1000
max-memory-mb = 8192

[metrics]
endpoint = "0.0.0.0:9090"
interval = 5
latency-buckets = [0.0001, 0.001, 0.01, 0.1, 1.0]

[grpc]
address = "0.0.0.0:3030"
max-connections = 5000
max-concurrent-streams = 1000
enable-compression = true
compression-level = 1  # Fast compression
```

## Validation & Error Handling

### Configuration Validation

Vixen performs comprehensive validation at startup:

```rust
// Example validation errors
ConfigError::MissingField("source.endpoint")
ConfigError::InvalidValue("timeout must be positive")
ConfigError::InvalidFormat("invalid endpoint URL")
ConfigError::IncompatibleOptions("cannot use both TLS cert and key without CA")
```

### Common Configuration Issues

#### Missing Required Fields
```toml
# ❌ Missing required fields
[source]
# endpoint is required
# x-token is required for authenticated sources

# ✅ Correct configuration
[source]
endpoint = "https://api.example.com"
x-token = "your-token"
```

#### Invalid Values
```toml
# ❌ Invalid values
[buffer]
buffer-size = -1          # Must be positive
num-workers = 0           # Must be positive
timeout = "invalid"       # Must be number

# ✅ Correct values
[buffer]
buffer-size = 1000
num-workers = 4
```

#### Type Mismatches
```toml
# ❌ Type mismatches
[metrics]
interval = "15"           # Should be number, not string
enable = "true"           # Should be boolean

# ✅ Correct types
[metrics]
interval = 15
enable = true
```

### Environment Variable Issues

#### Case Sensitivity
```bash
# ❌ Wrong case
export endpoint="https://api.example.com"  # lowercase
export X_token="token"                     # mixed case

# ✅ Correct case
export ENDPOINT="https://api.example.com"
export X_TOKEN="token"
```

#### Invalid JSON in Environment
```bash
# ❌ Invalid JSON for complex types
export LABELS="{environment: production}"  # Missing quotes

# ✅ Valid JSON
export LABELS='{"environment": "production"}'
```

### Configuration Testing

Test your configuration with the `--dry-run` flag:

```bash
# Validate configuration without starting
cargo run -- --config Vixen.toml --dry-run

# Check with specific environment
X_TOKEN=your-token cargo run -- --config Vixen.toml --dry-run
```

### Configuration Debugging

Enable debug logging for configuration issues:

```bash
# Show configuration resolution
RUST_LOG=yellowstone_vixen::config=debug cargo run -- --config Vixen.toml

# Show all environment variables
RUST_LOG=yellowstone_vixen=debug cargo run -- --config Vixen.toml
```