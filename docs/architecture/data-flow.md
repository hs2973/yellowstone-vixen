# Data Flow

This guide explains how data flows through Yellowstone Vixen, from raw Solana events to processed outputs, with detailed diagrams and examples.

## Overview

Understanding data flow is crucial for:
- Debugging pipeline issues
- Optimizing performance
- Designing custom parsers and handlers
- Troubleshooting production problems

## High-Level Data Flow

```
Raw Solana Data → Ingestion → Parsing → Processing → Output
      ↓               ↓         ↓          ↓         ↓
 Yellowstone     gRPC      Parser    Handler   Storage/
   Events       Stream   Processing  Execution  Analytics
```

## Detailed Data Flow

### 1. Data Ingestion

#### gRPC Stream Reception

Raw data enters through Yellowstone gRPC streams:

```rust
// Raw gRPC message structure
pub struct SubscribeUpdate {
    pub filters: Vec<SubscribeUpdateFilter>,
    pub account: Option<SubscribeUpdateAccount>,
    pub transaction: Option<SubscribeUpdateTransaction>,
    pub block_meta: Option<SubscribeUpdateBlockMeta>,
}
```

**Ingestion Process:**
1. **Connection Establishment** - Connect to Yellowstone endpoint
2. **Stream Subscription** - Subscribe to desired programs/accounts
3. **Message Reception** - Receive protobuf-encoded messages
4. **Message Decoding** - Decode protobuf to Rust structs
5. **Initial Validation** - Basic message integrity checks

#### Message Routing

Messages are routed based on type and content:

```rust
match update {
    SubscribeUpdate { account: Some(account), .. } => {
        route_to_account_pipeline(account)
    }
    SubscribeUpdate { transaction: Some(tx), .. } => {
        route_to_instruction_pipeline(tx)
    }
    SubscribeUpdate { block_meta: Some(block), .. } => {
        route_to_block_pipeline(block)
    }
    _ => skip_message()
}
```

### 2. Parsing Stage

#### Account Parsing Flow

```
Raw Account Data → Validation → Deserialization → Enrichment → Structured Output
      ↓                ↓             ↓              ↓            ↓
 AccountInfo      Check Owner    Decode Bytes   Add Metadata  TokenAccount
   Update         & Data Size    to Struct       (Slot, Time)   Struct
```

**Detailed Steps:**

1. **Input Validation**
   ```rust
   // Validate account belongs to expected program
   if account.owner != expected_program_id {
       return ParseError::Filtered;
   }

   // Check data size matches expected
   if account.data.len() != expected_size {
       return ParseError::Validation("Invalid data size".to_string());
   }
   ```

2. **Data Deserialization**
   ```rust
   // Deserialize raw bytes to structured data
   let token_account: TokenAccount = borsh::from_slice(&account.data)?;
   ```

3. **Data Enrichment**
   ```rust
   // Add contextual information
   let enriched = EnrichedTokenAccount {
       account: token_account,
       slot: update.slot,
       timestamp: get_timestamp(update.slot),
       program_id: account.owner,
   };
   ```

#### Instruction Parsing Flow

```
Raw Transaction → Instruction Extraction → Validation → Parsing → Structured Output
      ↓                     ↓                ↓          ↓            ↓
TransactionInfo     Find Relevant       Check Format  Decode Data  SwapInstruction
   Update           Instructions         & Accounts               Enum Variant
```

**Detailed Steps:**

1. **Instruction Extraction**
   ```rust
   for instruction in &transaction.transaction.message.instructions {
       if instruction.program_id == target_program {
           parse_instruction(instruction);
       }
   }
   ```

2. **Instruction Validation**
   ```rust
   // Validate instruction format
   if instruction.data.len() < 8 {
       return ParseError::Validation("Instruction too short".to_string());
   }

   // Check discriminator
   let discriminator = u64::from_le_bytes(instruction.data[..8].try_into()?);
   ```

3. **Data Parsing**
   ```rust
   match discriminator {
       SWAP_DISCRIMINATOR => {
           let swap_ix: SwapInstruction = borsh::from_slice(&instruction.data[8..])?;
           Ok(ParsedInstruction::Swap(swap_ix))
       }
       // ... other instruction types
   }
   ```

### 3. Processing Stage

#### Handler Execution Flow

```
Parsed Data → Handler Selection → Pre-Processing → Main Processing → Post-Processing
     ↓               ↓                  ↓              ↓              ↓
Structured     Match Handler        Validation     Business      Metrics/
  Output       by Type             & Filtering     Logic        Cleanup
```

**Handler Pipeline:**

1. **Handler Selection**
   ```rust
   // Route to appropriate handlers based on data type
   match parsed_data {
       ParsedData::TokenAccount(account) => {
           token_handlers.process(account).await
       }
       ParsedData::SwapInstruction(swap) => {
           swap_handlers.process(swap).await
       }
   }
   ```

2. **Pre-Processing**
   ```rust
   // Validate and filter data
   if !self.should_process(&data) {
       return Ok(()); // Skip processing
   }

   // Apply transformations
   let processed_data = self.preprocess(data)?;
   ```

3. **Main Processing**
   ```rust
   // Execute business logic
   self.database.store(&processed_data).await?;
   self.metrics.record_processing(&processed_data);
   ```

4. **Post-Processing**
   ```rust
   // Cleanup and finalization
   self.cleanup_resources().await;
   self.update_metrics();
   ```

### 4. Output Stage

#### Multiple Output Destinations

Data can be sent to multiple destinations simultaneously:

```
Processed Data → Database → Message Queue → Metrics → Logs
      ↓             ↓            ↓           ↓        ↓
  Storage      Async        Real-time    Monitoring  Debugging
  Layer       Processing    Events       & Alerting  & Auditing
```

**Output Types:**

1. **Database Storage**
   ```rust
   impl Handler<TokenAccount> for DatabaseHandler {
       async fn handle(&self, account: &TokenAccount) -> HandlerResult<()> {
           sqlx::query!(
               "INSERT INTO token_accounts (address, mint, amount, slot)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (address) DO UPDATE SET
                    amount = EXCLUDED.amount,
                    slot = EXCLUDED.slot",
               account.address,
               account.mint,
               account.amount,
               account.slot
           )
           .execute(&self.pool)
           .await?;
           Ok(())
       }
   }
   ```

2. **Message Queue Publishing**
   ```rust
   impl Handler<SwapEvent> for QueueHandler {
       async fn handle(&self, event: &SwapEvent) -> HandlerResult<()> {
           let message = serde_json::to_string(event)?;
           self.queue.publish("swap-events", message).await?;
           Ok(())
       }
   }
   ```

3. **Metrics Collection**
   ```rust
   impl Handler<AnyEvent> for MetricsHandler {
       async fn handle(&self, event: &AnyEvent) -> HandlerResult<()> {
           self.events_processed.inc();
           self.processing_latency.observe(start_time.elapsed());
           Ok(())
       }
   }
   ```

## Error Handling Flow

### Error Propagation

```
Error Occurs → Error Classification → Error Handling → Recovery Action → Logging
     ↓                ↓                   ↓              ↓            ↓
  Parse/Handler    Transient/         Retry/         Reconnect/     Metrics
   Failure         Permanent          Skip/         Failover       Update
```

**Error Types and Handling:**

1. **Transient Errors**
   ```rust
   match error {
       Error::ConnectionLost => {
           retry_with_backoff().await
       }
       Error::Timeout => {
           retry_with_timeout().await
       }
   }
   ```

2. **Permanent Errors**
   ```rust
   match error {
       Error::InvalidData => {
           log_error_and_skip()
       }
       Error::ConfigurationError => {
           shutdown_pipeline()
       }
   }
   ```

3. **Handler Errors**
   ```rust
   match handler_error {
       HandlerError::DatabaseError => {
           retry_with_exponential_backoff().await
       }
       HandlerError::ValidationError => {
           log_and_continue()
       }
   }
   ```

## Performance Considerations

### Buffering and Batching

```
Input Stream → Buffer → Batch Processor → Handler → Output
     ↓           ↓            ↓            ↓        ↓
  Raw Data    Accumulate   Group Messages  Process  Results
             Messages      for Efficiency  Batch    Batch
```

**Buffering Strategy:**

```rust
pub struct BufferManager {
    buffer: VecDeque<ParsedData>,
    max_size: usize,
    batch_size: usize,
    timeout: Duration,
}

impl BufferManager {
    pub async fn process(&mut self) -> Result<(), Error> {
        loop {
            // Wait for batch to fill or timeout
            let batch = self.collect_batch().await?;

            // Process batch
            self.handler.process_batch(batch).await?;

            // Update metrics
            self.metrics.record_batch_processed(batch.len());
        }
    }
}
```

### Parallel Processing

```
Input → Splitter → Worker 1 → Merger → Output
    ↓       ↓         ↓        ↓       ↓
  Data   Distribute  Process  Combine Results
         to Workers  Parallel
```

**Parallel Execution:**

```rust
pub struct ParallelProcessor<H> {
    handlers: Vec<H>,
    executor: ThreadPoolExecutor,
}

impl<H, T> ParallelProcessor<H>
where
    H: Handler<T> + Send + Sync,
    T: Send + Sync,
{
    pub async fn process_parallel(&self, data: Vec<T>) -> Result<(), Error> {
        let tasks: Vec<_> = data.into_iter()
            .zip(self.handlers.iter())
            .map(|(item, handler)| {
                let handler = handler.clone();
                tokio::spawn(async move {
                    handler.handle(&item).await
                })
            })
            .collect();

        // Wait for all tasks to complete
        for task in tasks {
            task.await??;
        }

        Ok(())
    }
}
```

## Monitoring Data Flow

### Metrics Collection Points

```
Ingestion → Parsing → Processing → Output
     ↓         ↓          ↓         ↓
   Counters  Timers    Gauges   Histograms
  (Volume)  (Latency) (Queue)  (Distribution)
```

**Key Metrics:**

```rust
pub struct DataFlowMetrics {
    // Volume metrics
    pub messages_ingested: Counter,
    pub messages_parsed: Counter,
    pub messages_processed: Counter,

    // Performance metrics
    pub ingestion_latency: Histogram,
    pub parsing_latency: Histogram,
    pub processing_latency: Histogram,

    // Queue metrics
    pub input_queue_depth: Gauge,
    pub processing_queue_depth: Gauge,

    // Error metrics
    pub parse_errors: Counter,
    pub processing_errors: Counter,
}
```

### Tracing Data Flow

Enable distributed tracing for end-to-end visibility:

```rust
#[tracing::instrument(skip(data))]
async fn process_data_flow(data: &RawData) -> Result<(), Error> {
    let span = tracing::info_span!("data_flow",
        message_id = %data.id,
        data_type = %data.data_type
    );

    let _enter = span.enter();

    // Ingestion
    tracing::info!("Starting data ingestion");
    let ingested = ingest_data(data).await?;

    // Parsing
    tracing::info!("Starting data parsing");
    let parsed = parse_data(&ingested).instrument(tracing::info_span!("parsing")).await?;

    // Processing
    tracing::info!("Starting data processing");
    let result = process_data(&parsed).instrument(tracing::info_span!("processing")).await?;

    tracing::info!("Data flow completed successfully");
    Ok(result)
}
```

## Troubleshooting Data Flow Issues

### Common Issues and Solutions

1. **Data Loss**
   - **Symptom**: Missing events in output
   - **Causes**: Buffer overflow, parsing errors, handler failures
   - **Solutions**: Increase buffer sizes, improve error handling, add retries

2. **High Latency**
   - **Symptom**: Slow end-to-end processing
   - **Causes**: Large batches, slow handlers, network issues
   - **Solutions**: Tune batch sizes, optimize handlers, improve network

3. **Processing Errors**
   - **Symptom**: Frequent handler failures
   - **Causes**: Invalid data, resource constraints, bugs
   - **Solutions**: Add validation, improve error handling, fix bugs

4. **Resource Exhaustion**
   - **Symptom**: Memory/CPU spikes, crashes
   - **Causes**: Large data volumes, memory leaks, unbounded queues
   - **Solutions**: Implement limits, add monitoring, fix leaks

### Debugging Tools

**Data Flow Tracing:**
```bash
# Enable detailed logging
export RUST_LOG=trace

# Trace specific data
export RUST_LOG=yellowstone_vixen=debug,yellowstone_vixen::parser=trace
```

**Metrics Monitoring:**
```bash
# Check Prometheus metrics
curl http://localhost:9090/metrics | grep vixen
```

**Performance Profiling:**
```rust
// Add profiling to handlers
#[tracing::instrument(skip(data))]
async fn profiled_handler(&self, data: &T) -> HandlerResult<()> {
    let start = Instant::now();
    let result = self.inner.handle(data).await;
    let duration = start.elapsed();

    tracing::info!("Handler completed in {:?}", duration);
    result
}
```

This comprehensive data flow understanding will help you build, debug, and optimize your Yellowstone Vixen pipelines effectively.
