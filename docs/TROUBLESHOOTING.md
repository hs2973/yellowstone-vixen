# Troubleshooting Guide

This guide provides solutions to common issues you might encounter when using Yellowstone Vixen.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Configuration Problems](#configuration-problems)
- [Connection Issues](#connection-issues)
- [Performance Problems](#performance-problems)
- [Parser Issues](#parser-issues)
- [gRPC Streaming Issues](#grpc-streaming-issues)
- [Metrics & Monitoring Issues](#metrics--monitoring-issues)
- [Memory & Resource Issues](#memory--resource-issues)
- [Debug Tools & Techniques](#debug-tools--techniques)

## Installation Issues

### Protocol Buffer Compiler Missing

**Error:**
```
Could not find `protoc`. If `protoc` is installed, try setting the `PROTOC` environment variable
```

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# macOS
brew install protobuf

# Windows (using chocolatey)
choco install protoc

# Manual installation
# Download from https://github.com/protocolbuffers/protobuf/releases
```

### Rust Toolchain Issues

**Error:**
```
error: toolchain 'stable-x86_64-unknown-linux-gnu' is not installed
```

**Solution:**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update existing installation
rustup update

# Set default toolchain
rustup default stable
```

### Compilation Errors with Dependencies

**Error:**
```
error: failed to compile `yellowstone-vixen`
```

**Solution:**
```bash
# Clean and rebuild
cargo clean && cargo build

# Update dependencies
cargo update

# Check for version conflicts
cargo tree --duplicates
```

## Configuration Problems

### Missing Configuration File

**Error:**
```
Error reading config file: No such file or directory
```

**Solution:**
```bash
# Create from example
cp Vixen.example.toml Vixen.toml

# Edit with your settings
nano Vixen.toml
```

### Invalid TOML Syntax

**Error:**
```
Error parsing config: invalid TOML
```

**Solution:**
```bash
# Validate TOML syntax
python3 -c "import toml; toml.load('Vixen.toml')"

# Or use online validator: https://www.toml-lint.com/
```

### Environment Variable Issues

**Error:**
```
Environment variable X_TOKEN not found
```

**Solution:**
```bash
# Check environment variables
env | grep -i token

# Set required variables
export X_TOKEN="your-token-here"
export ENDPOINT="https://api.example.com"

# Verify settings
echo $X_TOKEN
echo $ENDPOINT
```

### Configuration Validation Errors

**Error:**
```
Missing config section "source"
```

**Solution:**
```toml
# Ensure all required sections are present
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "your-token"

[metrics]  # Optional but recommended
endpoint = "0.0.0.0:9090"
```

## Connection Issues

### Dragon's Mouth Connection Failures

**Error:**
```
gRPC transport error: Connection refused
```

**Diagnosis:**
```bash
# Test network connectivity
curl -I https://yellowstone-api.triton.one

# Check DNS resolution
nslookup yellowstone-api.triton.one

# Test with grpcurl
grpcurl -plaintext -H "x-token: your-token" \
    yellowstone-api.triton.one:443 list
```

**Solutions:**

1. **Check credentials:**
```toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "verify-this-token-is-correct"
```

2. **Network connectivity:**
```bash
# Check firewall rules
sudo iptables -L

# Test with different ports
telnet yellowstone-api.triton.one 443
```

3. **TLS issues:**
```toml
[source]
endpoint = "https://yellowstone-api.triton.one"
tls = true  # Ensure TLS is enabled for HTTPS
```

### Authentication Errors

**Error:**
```
Unauthenticated: Invalid token
```

**Solutions:**

1. **Verify token format:**
```bash
# Check token length and format
echo $X_TOKEN | wc -c
echo $X_TOKEN | head -c 20
```

2. **Check token expiration:**
```bash
# Contact your provider to verify token status
```

3. **Test with curl:**
```bash
curl -H "x-token: $X_TOKEN" https://yellowstone-api.triton.one/health
```

### Timeout Issues

**Error:**
```
Request timeout after 60 seconds
```

**Solutions:**

1. **Increase timeout:**
```toml
[source]
timeout = 120  # Increase from default 60 seconds
```

2. **Network optimization:**
```toml
[source]
keepalive-time = 30
keepalive-timeout = 5
keepalive-permit-without-calls = true
```

## Performance Problems

### High Latency

**Symptoms:**
- Slow processing of events
- High `vixen_pipeline_lag_seconds` metric

**Diagnosis:**
```bash
# Check system resources
top
htop
iostat 1

# Monitor Vixen metrics
curl http://localhost:9090/metrics | grep latency
```

**Solutions:**

1. **Increase worker threads:**
```toml
[buffer]
num-workers = 8  # Increase based on CPU cores
batch-size = 500  # Larger batches
```

2. **Optimize parsing:**
```rust
// Use early returns in parsers
impl Parser for MyParser {
    async fn parse(&self, input: &Input) -> Result<Output, ParseError> {
        // Fast rejection
        if input.should_skip() {
            return Err(ParseError::Filtered);
        }
        // ... expensive parsing
    }
}
```

3. **Reduce handler complexity:**
```rust
// Avoid blocking operations in handlers
impl Handler<T> for MyHandler {
    async fn handle(&self, value: &T) -> HandlerResult<()> {
        // Offload heavy work to background tasks
        let tx = self.background_tx.clone();
        tokio::spawn(async move {
            heavy_processing(value).await;
        });
        Ok(())
    }
}
```

### High Memory Usage

**Symptoms:**
- Process memory growth
- Out of memory errors

**Diagnosis:**
```bash
# Monitor memory usage
ps aux | grep vixen
cat /proc/$(pidof vixen)/status | grep VmRSS

# Check for memory leaks
valgrind --tool=memcheck ./target/debug/vixen
```

**Solutions:**

1. **Reduce buffer sizes:**
```toml
[buffer]
buffer-size = 500   # Reduce from default 1000
max-memory-mb = 512 # Set memory limit
```

2. **Optimize data structures:**
```rust
// Use references instead of cloning
impl Handler<MyData> for MyHandler {
    async fn handle(&self, data: &MyData) -> HandlerResult<()> {
        // Process without cloning
        self.process_ref(data).await
    }
}
```

3. **Enable graceful degradation:**
```toml
[buffer]
graceful-shutdown = true
shutdown-timeout = 30
```

### High CPU Usage

**Symptoms:**
- 100% CPU utilization
- Slow system response

**Solutions:**

1. **Balance worker threads:**
```toml
[buffer]
num-workers = 4  # Don't exceed CPU core count
```

2. **Add filtering:**
```rust
// Filter early to reduce processing
let prefilter = Prefilter::builder()
    .transaction_accounts_include([important_account])
    .build();
```

3. **Use async properly:**
```rust
// Don't block the async runtime
impl Handler<T> for MyHandler {
    async fn handle(&self, value: &T) -> HandlerResult<()> {
        // Use tokio::task::spawn_blocking for CPU-intensive work
        let result = tokio::task::spawn_blocking(|| {
            cpu_intensive_work(value)
        }).await?;
        Ok(())
    }
}
```

## Parser Issues

### Parse Errors

**Error:**
```
Parse error: Invalid discriminator
```

**Diagnosis:**
```bash
# Enable debug logging
RUST_LOG=yellowstone_vixen::parser=debug cargo run

# Check account data format
echo "Account data: " $(base64 <<< "your_account_data")
```

**Solutions:**

1. **Validate input data:**
```rust
impl Parser for MyParser {
    async fn parse(&self, account: &AccountUpdate) -> Result<Output, ParseError> {
        // Check minimum size
        if account.account.data.len() < 8 {
            return Err(ParseError::Other("Data too small".into()));
        }
        
        // Validate program ownership
        if account.account.owner != MY_PROGRAM_ID {
            return Err(ParseError::Filtered);
        }
        
        // ... continue parsing
    }
}
```

2. **Handle version differences:**
```rust
fn parse_versioned_data(data: &[u8]) -> Result<MyData, ParseError> {
    let version = data[0];
    match version {
        1 => parse_v1_data(&data[1..]),
        2 => parse_v2_data(&data[1..]),
        _ => Err(ParseError::Other(format!("Unsupported version: {}", version).into()))
    }
}
```

### Missing Program Data

**Symptoms:**
- No parsed data despite transactions
- Empty instruction streams

**Solutions:**

1. **Check program ID:**
```rust
const MY_PROGRAM_ID: Pubkey = pubkey!("YourProgramId11111111111111111111111111111");

// Verify in parser
if ix.instruction.program_id != MY_PROGRAM_ID {
    return Err(ParseError::Filtered);
}
```

2. **Verify subscription filters:**
```toml
[source]
# Ensure your program is included
programs = ["YourProgramId11111111111111111111111111111"]
```

3. **Check transaction filters:**
```rust
// Make sure prefilters aren't too restrictive
let prefilter = Prefilter::builder()
    .transaction_accounts_include([/* not too specific */])
    .build();
```

## gRPC Streaming Issues

### Stream Connection Failures

**Error:**
```
grpc transport error: Connection refused (os error 61)
```

**Solutions:**

1. **Check server status:**
```bash
# Verify server is running
ps aux | grep vixen
netstat -tlnp | grep 3030
```

2. **Test local connectivity:**
```bash
# Test with grpcurl
grpcurl -plaintext localhost:3030 list
```

3. **Check firewall:**
```bash
# Allow gRPC port
sudo ufw allow 3030/tcp
```

### Stream Data Issues

**Error:**
```
Empty stream / No data received
```

**Solutions:**

1. **Verify subscription:**
```bash
# Test subscription
grpcurl -plaintext -d '{"program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"}' \
    localhost:3030 vixen.stream.ProgramStreams/Subscribe
```

2. **Check program activity:**
```bash
# Monitor logs for parsing activity
RUST_LOG=yellowstone_vixen_stream=debug cargo run
```

3. **Validate program ID:**
```bash
# Ensure program ID is correct and active
solana account TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
```

### Protocol Buffer Issues

**Error:**
```
Cannot decode protobuf message
```

**Solutions:**

1. **Regenerate protos:**
```bash
# Clean and rebuild
cargo clean
cargo build
```

2. **Check descriptor sets:**
```rust
// Ensure descriptor set is included
Server::builder()
    .descriptor_set(MY_DESCRIPTOR_SET)
    .build(config)
```

## Metrics & Monitoring Issues

### Prometheus Metrics Not Available

**Error:**
```
Connection refused on http://localhost:9090/metrics
```

**Solutions:**

1. **Enable metrics:**
```toml
[metrics]
endpoint = "0.0.0.0:9090"
```

2. **Check port binding:**
```bash
netstat -tlnp | grep 9090
```

3. **Test metrics endpoint:**
```bash
curl http://localhost:9090/metrics
```

### Missing Custom Metrics

**Symptoms:**
- Built-in metrics work but custom metrics don't appear

**Solutions:**

1. **Verify metric registration:**
```rust
impl<I: Instrumenter> MyHandler<I> {
    pub fn new(instrumenter: &I) -> Self {
        Self {
            custom_counter: instrumenter.counter("my_metric_total"),
        }
    }
}
```

2. **Check metric updates:**
```rust
impl Handler<T> for MyHandler {
    async fn handle(&self, value: &T) -> HandlerResult<()> {
        // Ensure metrics are actually updated
        self.custom_counter.inc();
        Ok(())
    }
}
```

### Grafana Dashboard Issues

**Symptoms:**
- Dashboard shows "No data"
- Connection errors

**Solutions:**

1. **Check Prometheus data source:**
```
URL: http://localhost:9091
Access: Server (default)
```

2. **Verify PromQL queries:**
```promql
# Test simple query
vixen_transactions_processed_total

# Check metric names
{__name__=~"vixen.*"}
```

## Memory & Resource Issues

### Out of Memory Errors

**Error:**
```
thread 'main' panicked at 'out of memory'
```

**Solutions:**

1. **Reduce memory footprint:**
```toml
[buffer]
buffer-size = 100
num-workers = 2
max-memory-mb = 256
```

2. **Optimize data handling:**
```rust
// Use streaming instead of collecting
async fn process_stream(&self, mut stream: Stream) {
    while let Some(item) = stream.next().await {
        self.process_item(item).await?;
        // Don't collect all items in memory
    }
}
```

3. **Enable memory monitoring:**
```rust
// Add memory usage metrics
let memory_gauge = instrumenter.gauge("memory_usage_bytes");
tokio::spawn(async move {
    loop {
        let usage = get_memory_usage();
        memory_gauge.set(usage as f64);
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
});
```

### File Descriptor Limits

**Error:**
```
Too many open files (os error 24)
```

**Solutions:**

1. **Increase limits:**
```bash
# Temporary increase
ulimit -n 4096

# Permanent increase (add to /etc/security/limits.conf)
* soft nofile 4096
* hard nofile 8192
```

2. **Monitor file descriptors:**
```bash
# Check current usage
lsof -p $(pidof vixen) | wc -l

# Monitor continuously
watch "lsof -p $(pidof vixen) | wc -l"
```

## Debug Tools & Techniques

### Enable Debug Logging

```bash
# Full debug logging
RUST_LOG=debug cargo run

# Specific component logging
RUST_LOG=yellowstone_vixen::parser=debug,yellowstone_vixen_stream=info cargo run

# JSON structured logging
RUST_LOG=info LOG_FORMAT=json cargo run
```

### Performance Profiling

```bash
# CPU profiling with perf
perf record -g cargo run --release
perf report

# Memory profiling with valgrind
valgrind --tool=massif cargo run
```

### Network Debugging

```bash
# Monitor network traffic
sudo tcpdump -i any port 443

# Test gRPC connectivity
grpcurl -plaintext -v localhost:3030 list

# Check TLS handshake
openssl s_client -connect yellowstone-api.triton.one:443
```

### Configuration Validation

```bash
# Dry run to validate config
cargo run -- --config Vixen.toml --dry-run

# Check configuration parsing
RUST_LOG=yellowstone_vixen::config=debug cargo run
```

### Health Checks

```bash
# Basic health check
curl http://localhost:9090/health

# Detailed status
curl http://localhost:9090/metrics | grep vixen_status

# gRPC health check
grpcurl -plaintext localhost:3030 grpc.health.v1.Health/Check
```

### Emergency Recovery

If Vixen becomes unresponsive:

1. **Graceful shutdown:**
```bash
# Send SIGTERM for graceful shutdown
kill -TERM $(pidof vixen)
```

2. **Force termination:**
```bash
# If graceful shutdown fails
kill -KILL $(pidof vixen)
```

3. **Restart with minimal config:**
```toml
# Use minimal configuration for recovery
[source]
endpoint = "https://api.devnet.solana.com"

[buffer]
buffer-size = 10
num-workers = 1
```

This troubleshooting guide should help you diagnose and resolve most common issues with Yellowstone Vixen. For additional support, check the project's GitHub issues or contact the maintainers.