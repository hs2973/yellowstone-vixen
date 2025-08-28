# Stream Parser Example

This example demonstrates the full capabilities of Yellowstone Vixen's streaming architecture. It sets up a comprehensive gRPC server that serves real-time streams of parsed accounts and transaction updates from multiple Solana programs.

## Overview

The stream parser example showcases:

- **Multi-Program Support**: Parsing 15+ different Solana programs simultaneously
- **Real-time gRPC Streaming**: Serving parsed data to external clients
- **Advanced Filtering**: Using FilterPipeline for transaction-level filtering
- **Shared Data Features**: Accessing transaction signatures, slot numbers, and metadata
- **Protocol Buffer Integration**: Type-safe streaming with auto-generated clients

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Dragon's      │    │   Stream        │    │   gRPC          │
│   Mouth         │───▶│   Server        │───▶│   Clients       │
│   (Source)      │    │   (Vixen)       │    │   (Multiple)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   Protocol      │
                       │   Buffers       │
                       │   (Type Safety) │
                       └─────────────────┘
```

## Supported Programs

This example includes parsers for the following Solana programs:

| Program | Address | Supported Operations |
|---------|---------|---------------------|
| **SPL Token** | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | Mint, Account, Transfer, Burn |
| **Token Extensions** | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | All extensions, Advanced features |
| **Raydium AMM v4** | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` | Swaps, Liquidity, Pool creation |
| **Jupiter Swap** | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Route swaps, Price discovery |
| **Meteora DLMM** | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Dynamic liquidity, Bin operations |
| **Pump.fun** | `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P` | Token launches, Bonding curves |
| **Orca Whirlpools** | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` | Concentrated liquidity, Position management |
| **Moonshot** | `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG` | Token launches, Community features |

## Advanced Features Demonstrated

### 1. FilterPipeline Usage

The example shows how to use `FilterPipeline` for transaction-level filtering:

```rust
// Filter Raydium transactions by specific accounts
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

### 2. Shared Data Features

Access transaction-wide context including signatures and slot numbers:

```rust
impl Handler<InstructionUpdateOutput<RaydiumAmmV4ProgramIx>> for RaydiumAmmV4IxLogger {
    async fn handle(&self, value: &InstructionUpdateOutput<RaydiumAmmV4ProgramIx>) -> HandlerResult<()> {
        tracing::info!(
            signature = %value.signature,
            slot = value.slot,
            instruction = ?value.parsed_ix,
            "Raydium swap detected"
        );
        Ok(())
    }
}
```

### 3. Protocol Buffer Streaming

Type-safe streaming with auto-generated protocol buffer definitions:

```rust
Server::<_, YellowstoneGrpcSource>::builder()
    .descriptor_set(parser::token::DESCRIPTOR_SET)
    .descriptor_set(METEORA_DESCRIPTOR_SET)
    .descriptor_set(RAYDIUM_AMM_V4_DESCRIPTOR_SET)
    // ... more descriptor sets
```

## Running the Example

### Prerequisites

1. **Configuration File**: Create a `Vixen.toml` configuration file:

```toml
[source]
endpoint = "https://yellowstone-api.triton.one"
x-token = "your-api-token-here"
timeout = 60

[grpc]
address = "[::]:3030"

[metrics]
endpoint = "0.0.0.0:9090"
```

2. **Dependencies**: Ensure you have the required tools:

```bash
# Install grpcurl for client testing
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# Or using package manager
brew install grpcurl  # macOS
apt install grpcurl   # Ubuntu
```

### Starting the Server

Navigate to the example directory and start the server:

```bash
cd examples/stream-parser
RUST_LOG=info cargo run -- --config "$(pwd)/../../Vixen.toml"
```

The server will start and begin processing data from the configured source, with logs indicating:

```
INFO  yellowstone_vixen_stream: Starting gRPC server on [::]:3030
INFO  yellowstone_vixen: Connected to data source
INFO  example_vixen_stream_parser: Raydium swap detected signature=... slot=...
```

## Client Usage

### 1. Service Discovery

List all available gRPC services:

```bash
grpcurl -plaintext 127.0.0.1:3030 list
```

Output:
```
grpc.reflection.v1alpha.ServerReflection
vixen.stream.ProgramStreams
```

### 2. Service Introspection

Examine the streaming service interface:

```bash
grpcurl -plaintext 127.0.0.1:3030 describe vixen.stream.ProgramStreams
```

Output:
```
vixen.stream.ProgramStreams is a service:
service ProgramStreams {
  rpc Subscribe ( .vixen.stream.SubscribeRequest ) returns ( stream .google.protobuf.Any );
}
```

### 3. Streaming Program Data

Subscribe to specific program streams:

#### Token Program Stream
```bash
grpcurl -plaintext -d '{"program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"}' \
    127.0.0.1:3030 vixen.stream.ProgramStreams/Subscribe
```

#### Raydium AMM v4 Stream
```bash
grpcurl -plaintext -d '{"program": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"}' \
    127.0.0.1:3030 vixen.stream.ProgramStreams/Subscribe
```

#### Jupiter Swap Stream
```bash
grpcurl -plaintext -d '{"program": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"}' \
    127.0.0.1:3030 vixen.stream.ProgramStreams/Subscribe
```

### 4. Advanced Filtering

Subscribe with additional filters (if supported by the client):

```bash
# Filter by specific account types or instruction types
grpcurl -plaintext -d '{
    "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "filters": {
        "account_types": ["Mint", "Account"],
        "instruction_types": ["Transfer", "Burn"]
    }
}' 127.0.0.1:3030 vixen.stream.ProgramStreams/Subscribe
```

## Generated Client Libraries

The stream server automatically generates protocol buffer definitions that can be used to create type-safe clients in multiple languages.

### Python Client Example

```python
import grpc
from vixen.stream import program_streams_pb2, program_streams_pb2_grpc

# Connect to the stream
channel = grpc.insecure_channel('localhost:3030')
stub = program_streams_pb2_grpc.ProgramStreamsStub(channel)

# Subscribe to Raydium streams
request = program_streams_pb2.SubscribeRequest(
    program="675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
)

# Process the stream
for response in stub.Subscribe(request):
    print(f"Received: {response}")
```

### JavaScript/TypeScript Client Example

```typescript
import * as grpc from '@grpc/grpc-js';
import { ProgramStreamsClient } from './generated/vixen/stream/program_streams';

const client = new ProgramStreamsClient(
    'localhost:3030',
    grpc.credentials.createInsecure()
);

const request = {
    program: '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8'
};

const stream = client.subscribe(request);

stream.on('data', (response) => {
    console.log('Received:', response);
});

stream.on('error', (error) => {
    console.error('Stream error:', error);
});
```

### Rust Client Example

```rust
use tonic::Request;
use vixen_stream::program_streams_client::ProgramStreamsClient;
use vixen_stream::SubscribeRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ProgramStreamsClient::connect("http://127.0.0.1:3030").await?;
    
    let request = Request::new(SubscribeRequest {
        program: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
    });
    
    let mut stream = client.subscribe(request).await?.into_inner();
    
    while let Some(response) = stream.message().await? {
        println!("Received: {:?}", response);
    }
    
    Ok(())
}
```

## Monitoring and Observability

### Prometheus Metrics

The example includes Prometheus metrics at `http://localhost:9090/metrics`:

- `vixen_instructions_processed_total`: Instructions processed by program
- `vixen_accounts_processed_total`: Accounts processed by program  
- `vixen_grpc_connections_total`: Active gRPC connections
- `vixen_stream_latency_seconds`: Stream processing latency

### Health Checks

Check server health:

```bash
grpcurl -plaintext 127.0.0.1:3030 grpc.health.v1.Health/Check
```

### Debug Logging

Enable detailed logging for troubleshooting:

```bash
RUST_LOG=yellowstone_vixen=debug,yellowstone_vixen_stream=debug \
cargo run -- --config Vixen.toml
```

## Customization Examples

### Adding a New Program Parser

1. **Add the parser dependency**:
```toml
[dependencies]
my-custom-parser = { path = "../path/to/parser" }
```

2. **Include in the server**:
```rust
use my_custom_parser::{
    AccountParser as MyAccParser,
    InstructionParser as MyIxParser,
    DESCRIPTOR_SET as MY_DESCRIPTOR_SET,
};

Server::<_, YellowstoneGrpcSource>::builder()
    .descriptor_set(MY_DESCRIPTOR_SET)
    .account(Proto::new(MyAccParser))
    .instruction(Proto::new(MyIxParser))
    // ... existing parsers
    .build(config)
    .run();
```

### Custom Handler Implementation

```rust
#[derive(Debug)]
pub struct DatabaseHandler {
    pool: sqlx::PgPool,
}

impl Handler<RaydiumAmmV4ProgramIx> for DatabaseHandler {
    async fn handle(&self, ix: &RaydiumAmmV4ProgramIx) -> HandlerResult<()> {
        match ix {
            RaydiumAmmV4ProgramIx::SwapBaseIn(accounts, data) => {
                // Store swap data in database
                sqlx::query!(
                    "INSERT INTO swaps (signature, amount_in, amount_out) VALUES ($1, $2, $3)",
                    data.signature.to_string(),
                    data.amount_in as i64,
                    data.amount_out as i64
                )
                .execute(&self.pool)
                .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Troubleshooting

### Common Issues

1. **Connection Errors**: Verify your Dragon's Mouth credentials and endpoint
2. **Missing Data**: Check that your filters are not too restrictive
3. **High Memory Usage**: Reduce buffer sizes or add more aggressive filtering
4. **gRPC Errors**: Ensure the server is running and accessible

### Performance Tuning

1. **Increase Buffer Sizes**: For high-throughput scenarios
2. **Add More Workers**: Scale processing threads based on CPU cores
3. **Optimize Filters**: Use prefilters to reduce unnecessary processing
4. **Batch Operations**: Implement batching in custom handlers

This example provides a solid foundation for building production-ready Solana data streaming applications with Yellowstone Vixen.
