# Configuration

This guide covers configuring Yellowstone Vixen for different use cases and environments.

## Configuration File Format

Yellowstone Vixen uses TOML for configuration. The main configuration file is typically named `Vixen.toml`.

## Basic Configuration

```toml
# gRPC connection settings
[grpc]
endpoint = "http://localhost:10000"
token = "your-auth-token"  # Optional, depending on your provider

# Programs to monitor
[[programs]]
name = "Token Program"
address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

[[programs]]
name = "Associated Token Program"
address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
```

## Advanced Configuration

### Multiple Endpoints

```toml
[grpc]
# Primary endpoint
endpoint = "http://primary-endpoint:10000"
token = "primary-token"

# Fallback endpoints
[[grpc.fallbacks]]
endpoint = "http://fallback-1:10000"
token = "fallback-token-1"

[[grpc.fallbacks]]
endpoint = "http://fallback-2:10000"
token = "fallback-token-2"
```

### Buffer Configuration

```toml
[buffer]
# Maximum number of messages to buffer
max_capacity = 10000
# Batch size for processing
batch_size = 100
# Timeout for batch processing (milliseconds)
batch_timeout_ms = 1000
```

### Metrics Configuration

```toml
[metrics]
# Enable Prometheus metrics
prometheus = true
# Metrics endpoint port
port = 9090
# Metrics path
path = "/metrics"

# OpenTelemetry configuration
[metrics.opentelemetry]
enabled = true
endpoint = "http://localhost:4317"
service_name = "yellowstone-vixen"
```

### Logging Configuration

```toml
[logging]
# Log level (trace, debug, info, warn, error)
level = "info"
# Log format (json, pretty)
format = "pretty"
# Enable file logging
file = "vixen.log"
```

## Program-Specific Configuration

### Jupiter Swap Parser

```toml
[[programs]]
name = "Jupiter Aggregator"
address = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"

[programs.config]
# Filter specific instruction types
instructions = ["swap", "route"]
# Minimum swap amount (in lamports)
min_swap_amount = 1000000
```

### Meteora Parser

```toml
[[programs]]
name = "Meteora DLMM"
address = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"

[programs.config]
# Track specific pool addresses
pools = [
    "pool_address_1",
    "pool_address_2"
]
# Position size thresholds
min_position_size = 10000000
```

## Environment Variables

You can use environment variables in your configuration:

```toml
[grpc]
endpoint = "${GRPC_ENDPOINT}"
token = "${GRPC_TOKEN}"
```

Set them in your environment:
```bash
export GRPC_ENDPOINT="http://localhost:10000"
export GRPC_TOKEN="your-token"
```

## Runtime Configuration

### Commitment Levels

```toml
[runtime]
# Solana commitment level
commitment_level = "confirmed"  # processed, confirmed, finalized
```

### Performance Tuning

```toml
[runtime]
# Number of worker threads
worker_threads = 4
# Maximum concurrent connections
max_connections = 10
# Request timeout (seconds)
request_timeout = 30
```

## Development Configuration

For development and testing:

```toml
[development]
# Enable debug mode
debug = true
# Use mock data instead of live Solana
mock_data = true
# Log all gRPC messages
log_grpc = true

# Mock data configuration
[mock]
# Path to fixture files
fixtures_path = "./fixtures"
# Replay speed multiplier
speed_multiplier = 1.0
```

## Production Configuration

For production deployments:

```toml
[production]
# Enable all optimizations
optimize = true
# Health check endpoint
health_check_port = 8080

[grpc]
endpoint = "https://production-endpoint:443"
token = "${PRODUCTION_TOKEN}"

# High availability
[ha]
# Enable leader election
leader_election = true
# Health check interval (seconds)
health_check_interval = 30
```

## Configuration Validation

Validate your configuration:

```bash
cargo run -- --config Vixen.toml --validate
```

## Example Configurations

### Simple Token Monitoring

```toml
[grpc]
endpoint = "http://localhost:10000"

[[programs]]
name = "Token Program"
address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

[metrics]
prometheus = true
```

### DeFi Protocol Monitoring

```toml
[grpc]
endpoint = "https://api.mainnet.solana.com"

[[programs]]
name = "Jupiter"
address = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"

[[programs]]
name = "Raydium"
address = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"

[[programs]]
name = "Meteora"
address = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"

[metrics]
prometheus = true
port = 9090

[runtime]
commitment_level = "confirmed"
```

## Configuration Best Practices

1. **Use environment variables** for secrets and environment-specific values
2. **Validate configurations** before deployment
3. **Use descriptive names** for programs and configurations
4. **Document custom configurations** with comments
5. **Version control** your configuration files
6. **Test configurations** in staging before production

## Troubleshooting

**Configuration not loading:**
- Check file permissions
- Verify TOML syntax
- Ensure all required fields are present

**Connection issues:**
- Verify endpoint URLs
- Check authentication tokens
- Confirm network connectivity

**Performance issues:**
- Adjust buffer sizes
- Tune worker thread counts
- Monitor resource usage
