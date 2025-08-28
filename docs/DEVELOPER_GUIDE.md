# Developer Guide

This guide provides comprehensive information for developers who want to build custom parsers, handlers, and extend Yellowstone Vixen.

## Table of Contents

- [Getting Started](#getting-started)
- [Creating Custom Parsers](#creating-custom-parsers)
- [Creating Custom Handlers](#creating-custom-handlers)
- [Advanced Filtering](#advanced-filtering)
- [Protocol Buffer Integration](#protocol-buffer-integration)
- [Testing Custom Components](#testing-custom-components)
- [Performance Optimization](#performance-optimization)
- [Error Handling Best Practices](#error-handling-best-practices)
- [Contributing Back](#contributing-back)

## Getting Started

### Development Environment Setup

1. **Install Rust**: Use rustup to install the latest stable Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Install protobuf compiler**: Required for protocol buffer generation
```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# macOS
brew install protobuf

# Windows
# Download from https://github.com/protocolbuffers/protobuf/releases
```

3. **Clone and build Vixen**:
```bash
git clone https://github.com/rpcpool/yellowstone-vixen.git
cd yellowstone-vixen
cargo build
```

4. **Set up development dependencies**:
```bash
cargo install cargo-watch  # For auto-recompilation
cargo install grpcurl      # For testing gRPC endpoints
```

### Project Structure for Custom Components

Create a new crate for your custom parser:

```
my-custom-parser/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── account_parser.rs
│   ├── instruction_parser.rs
│   └── proto_def.rs
├── proto/
│   └── my_program.proto
├── tests/
│   └── integration_tests.rs
└── fixtures/
    ├── accounts/
    └── transactions/
```

## Creating Custom Parsers

### Account Parser Implementation

Account parsers transform raw account data into typed Rust structures.

#### 1. Define Your Account Structure

```rust
// src/lib.rs
use yellowstone_vixen_core::Pubkey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyProgramAccount {
    pub authority: Pubkey,
    pub balance: u64,
    pub created_at: i64,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum MyProgramAccountType {
    UserAccount(MyProgramAccount),
    PoolAccount(PoolAccount),
    ConfigAccount(ConfigAccount),
}
```

#### 2. Implement the Account Parser

```rust
// src/account_parser.rs
use yellowstone_vixen_core::{AccountUpdate, ParseError, Parser};
use std::borrow::Cow;

pub struct AccountParser;

impl Parser for AccountParser {
    type Input = AccountUpdate;
    type Output = MyProgramAccountType;
    
    async fn parse(&self, account: &AccountUpdate) -> Result<Self::Output, ParseError> {
        // Check if this account belongs to our program
        if account.account.owner != MY_PROGRAM_ID {
            return Err(ParseError::Filtered);
        }
        
        // Check minimum data size
        if account.account.data.len() < 8 {
            return Err(ParseError::Other("Account data too small".into()));
        }
        
        // Parse discriminator to determine account type
        let discriminator = u64::from_le_bytes(
            account.account.data[0..8].try_into()
                .map_err(|e| ParseError::Other(format!("Invalid discriminator: {}", e).into()))?
        );
        
        match discriminator {
            USER_ACCOUNT_DISCRIMINATOR => {
                let account = parse_user_account(&account.account.data[8..])?;
                Ok(MyProgramAccountType::UserAccount(account))
            }
            POOL_ACCOUNT_DISCRIMINATOR => {
                let account = parse_pool_account(&account.account.data[8..])?;
                Ok(MyProgramAccountType::PoolAccount(account))
            }
            CONFIG_ACCOUNT_DISCRIMINATOR => {
                let account = parse_config_account(&account.account.data[8..])?;
                Ok(MyProgramAccountType::ConfigAccount(account))
            }
            _ => Err(ParseError::Other(format!("Unknown account type: {}", discriminator).into()))
        }
    }
    
    fn id(&self) -> Cow<str> {
        "my_program::AccountParser".into()
    }
}

// Helper parsing functions
fn parse_user_account(data: &[u8]) -> Result<MyProgramAccount, ParseError> {
    if data.len() < 48 { // authority(32) + balance(8) + created_at(8)
        return Err(ParseError::Other("User account data too small".into()));
    }
    
    let authority = Pubkey::try_from(&data[0..32])
        .map_err(|e| ParseError::Other(format!("Invalid authority: {}", e).into()))?;
    
    let balance = u64::from_le_bytes(data[32..40].try_into().unwrap());
    let created_at = i64::from_le_bytes(data[40..48].try_into().unwrap());
    
    // Parse remaining metadata
    let metadata = data[48..].to_vec();
    
    Ok(MyProgramAccount {
        authority,
        balance,
        created_at,
        metadata,
    })
}
```

### Instruction Parser Implementation

Instruction parsers decode transaction instructions into typed data structures.

#### 1. Define Instruction Types

```rust
// src/lib.rs
#[derive(Debug, Clone)]
pub enum MyProgramInstruction {
    Initialize(InitializeAccounts, InitializeData),
    Deposit(DepositAccounts, DepositData),
    Withdraw(WithdrawAccounts, WithdrawData),
    Transfer(TransferAccounts, TransferData),
}

#[derive(Debug, Clone)]
pub struct InitializeAccounts {
    pub user: Pubkey,
    pub account: Pubkey,
    pub system_program: Pubkey,
}

#[derive(Debug, Clone)]
pub struct InitializeData {
    pub initial_balance: u64,
    pub metadata: Vec<u8>,
}

// Similar structs for other instruction types...
```

#### 2. Implement the Instruction Parser

```rust
// src/instruction_parser.rs
use yellowstone_vixen_core::{InstructionUpdate, ParseError, Parser, instruction::InstructionUpdateExt};

pub struct InstructionParser;

impl Parser for InstructionParser {
    type Input = InstructionUpdate;
    type Output = MyProgramInstruction;
    
    async fn parse(&self, ix: &InstructionUpdate) -> Result<Self::Output, ParseError> {
        // Check if this is our program
        if ix.instruction.program_id != MY_PROGRAM_ID {
            return Err(ParseError::Filtered);
        }
        
        // Check minimum instruction data size
        if ix.instruction.data.is_empty() {
            return Err(ParseError::Other("Empty instruction data".into()));
        }
        
        // Parse instruction discriminator
        let instruction_type = ix.instruction.data[0];
        let data = &ix.instruction.data[1..];
        
        match instruction_type {
            0 => self.parse_initialize(ix, data),
            1 => self.parse_deposit(ix, data),
            2 => self.parse_withdraw(ix, data),
            3 => self.parse_transfer(ix, data),
            _ => Err(ParseError::Other(format!("Unknown instruction type: {}", instruction_type).into()))
        }
    }
    
    fn id(&self) -> Cow<str> {
        "my_program::InstructionParser".into()
    }
}

impl InstructionParser {
    fn parse_initialize(&self, ix: &InstructionUpdate, data: &[u8]) -> Result<MyProgramInstruction, ParseError> {
        // Parse accounts
        if ix.instruction.accounts.len() < 3 {
            return Err(ParseError::Other("Initialize instruction requires 3 accounts".into()));
        }
        
        let accounts = InitializeAccounts {
            user: ix.account_at(0)?,
            account: ix.account_at(1)?,
            system_program: ix.account_at(2)?,
        };
        
        // Parse instruction data
        if data.len() < 8 {
            return Err(ParseError::Other("Initialize data too small".into()));
        }
        
        let initial_balance = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let metadata = data[8..].to_vec();
        
        let instruction_data = InitializeData {
            initial_balance,
            metadata,
        };
        
        Ok(MyProgramInstruction::Initialize(accounts, instruction_data))
    }
    
    fn parse_deposit(&self, ix: &InstructionUpdate, data: &[u8]) -> Result<MyProgramInstruction, ParseError> {
        // Similar implementation for deposit...
        todo!("Implement deposit parsing")
    }
    
    // ... other parsing methods
}
```

#### 3. Add Helper Extensions

Create utility extensions for common parsing tasks:

```rust
// src/instruction_parser.rs
use yellowstone_vixen_core::instruction::ParseError;

trait InstructionUpdateExt {
    fn account_at(&self, index: usize) -> Result<Pubkey, ParseError>;
    fn require_account_count(&self, count: usize) -> Result<(), ParseError>;
}

impl InstructionUpdateExt for InstructionUpdate {
    fn account_at(&self, index: usize) -> Result<Pubkey, ParseError> {
        self.instruction.accounts.get(index)
            .copied()
            .ok_or_else(|| ParseError::Other(format!("Missing account at index {}", index).into()))
    }
    
    fn require_account_count(&self, count: usize) -> Result<(), ParseError> {
        if self.instruction.accounts.len() < count {
            return Err(ParseError::Other(
                format!("Expected {} accounts, got {}", count, self.instruction.accounts.len()).into()
            ));
        }
        Ok(())
    }
}
```

## Creating Custom Handlers

Handlers implement business logic that processes parsed data.

### Simple Handler Implementation

```rust
// src/handlers.rs
use yellowstone_vixen::{Handler, HandlerResult};
use tracing::{info, warn, error};

#[derive(Debug)]
pub struct LoggingHandler;

impl Handler<MyProgramInstruction> for LoggingHandler {
    async fn handle(&self, instruction: &MyProgramInstruction) -> HandlerResult<()> {
        match instruction {
            MyProgramInstruction::Initialize(accounts, data) => {
                info!(
                    user = %accounts.user,
                    initial_balance = data.initial_balance,
                    "User account initialized"
                );
            }
            MyProgramInstruction::Deposit(accounts, data) => {
                info!(
                    user = %accounts.user,
                    amount = data.amount,
                    "Deposit processed"
                );
            }
            MyProgramInstruction::Withdraw(accounts, data) => {
                warn!(
                    user = %accounts.user,
                    amount = data.amount,
                    "Withdrawal processed"
                );
            }
            MyProgramInstruction::Transfer(accounts, data) => {
                info!(
                    from = %accounts.from,
                    to = %accounts.to,
                    amount = data.amount,
                    "Transfer processed"
                );
            }
        }
        Ok(())
    }
}
```

### Database Handler Implementation

```rust
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct DatabaseHandler {
    pool: PgPool,
}

impl DatabaseHandler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Handler<MyProgramInstruction> for DatabaseHandler {
    async fn handle(&self, instruction: &MyProgramInstruction) -> HandlerResult<()> {
        match instruction {
            MyProgramInstruction::Deposit(accounts, data) => {
                // Insert deposit record
                sqlx::query!(
                    r#"
                    INSERT INTO deposits (user_pubkey, amount, signature, slot, timestamp)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                    accounts.user.to_string(),
                    data.amount as i64,
                    data.signature.to_string(),
                    data.slot as i64,
                    data.timestamp
                )
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?;
                
                // Update user balance
                sqlx::query!(
                    r#"
                    UPDATE user_accounts 
                    SET balance = balance + $1, updated_at = $2
                    WHERE pubkey = $3
                    "#,
                    data.amount as i64,
                    Utc::now(),
                    accounts.user.to_string()
                )
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?;
            }
            
            MyProgramInstruction::Withdraw(accounts, data) => {
                // Similar implementation for withdrawals
                // Check balance, insert withdrawal record, update balance
                self.process_withdrawal(accounts, data).await?;
            }
            
            // Handle other instruction types...
            _ => {}
        }
        
        Ok(())
    }
}

impl DatabaseHandler {
    async fn process_withdrawal(&self, accounts: &WithdrawAccounts, data: &WithdrawData) -> HandlerResult<()> {
        // Start a transaction
        let mut tx = self.pool.begin().await
            .map_err(|e| format!("Transaction error: {}", e))?;
        
        // Check current balance
        let current_balance: Option<i64> = sqlx::query_scalar!(
            "SELECT balance FROM user_accounts WHERE pubkey = $1",
            accounts.user.to_string()
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
        
        let current_balance = current_balance.unwrap_or(0);
        if current_balance < data.amount as i64 {
            return Err("Insufficient balance".into());
        }
        
        // Insert withdrawal record
        sqlx::query!(
            r#"
            INSERT INTO withdrawals (user_pubkey, amount, signature, slot, timestamp)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            accounts.user.to_string(),
            data.amount as i64,
            data.signature.to_string(),
            data.slot as i64,
            data.timestamp
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
        
        // Update balance
        sqlx::query!(
            r#"
            UPDATE user_accounts 
            SET balance = balance - $1, updated_at = $2
            WHERE pubkey = $3
            "#,
            data.amount as i64,
            Utc::now(),
            accounts.user.to_string()
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
        
        // Commit transaction
        tx.commit().await
            .map_err(|e| format!("Transaction commit error: {}", e))?;
        
        Ok(())
    }
}
```

### Metrics Handler Implementation

```rust
use yellowstone_vixen::metrics::{Counter, Histogram, Instrumenter};

#[derive(Debug)]
pub struct MetricsHandler<I: Instrumenter> {
    instruction_counter: Counter<I>,
    deposit_amount: Histogram<I>,
    withdrawal_amount: Histogram<I>,
    processing_duration: Histogram<I>,
}

impl<I: Instrumenter> MetricsHandler<I> {
    pub fn new(instrumenter: &I) -> Self {
        Self {
            instruction_counter: instrumenter.counter("my_program_instructions_total")
                .with_description("Total number of instructions processed"),
            deposit_amount: instrumenter.histogram("my_program_deposit_amount")
                .with_description("Deposit amounts"),
            withdrawal_amount: instrumenter.histogram("my_program_withdrawal_amount")
                .with_description("Withdrawal amounts"),
            processing_duration: instrumenter.histogram("my_program_processing_duration_seconds")
                .with_description("Time spent processing instructions"),
        }
    }
}

impl<I: Instrumenter> Handler<MyProgramInstruction> for MetricsHandler<I> {
    async fn handle(&self, instruction: &MyProgramInstruction) -> HandlerResult<()> {
        let start_time = std::time::Instant::now();
        
        // Count the instruction
        self.instruction_counter.inc_by(1.0);
        
        match instruction {
            MyProgramInstruction::Deposit(_, data) => {
                self.deposit_amount.record(data.amount as f64);
            }
            MyProgramInstruction::Withdraw(_, data) => {
                self.withdrawal_amount.record(data.amount as f64);
            }
            _ => {}
        }
        
        // Record processing time
        let duration = start_time.elapsed();
        self.processing_duration.record(duration.as_secs_f64());
        
        Ok(())
    }
}
```

## Advanced Filtering

### Using FilterPipeline

For complex filtering logic that depends on transaction context:

```rust
use yellowstone_vixen::filter_pipeline::FilterPipeline;
use yellowstone_vixen_core::{Prefilter, Pubkey};
use std::str::FromStr;

// Create a filtered pipeline that only processes transactions involving specific accounts
let important_accounts = [
    Pubkey::from_str("11111111111111111111111111111112").unwrap(), // System program
    Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(), // Token program
];

let prefilter = Prefilter::builder()
    .transaction_accounts_include(important_accounts)
    .build();

let filtered_pipeline = FilterPipeline::new(
    MyInstructionParser,
    [LoggingHandler, MetricsHandler],
    prefilter
);

// Use in runtime
yellowstone_vixen::Runtime::builder()
    .instruction(filtered_pipeline)
    .build(config)
    .run();
```

### Custom Prefilter Logic

Implement custom filtering logic within your parser:

```rust
impl Parser for MyInstructionParser {
    async fn parse(&self, ix: &InstructionUpdate) -> Result<Self::Output, ParseError> {
        // Program ID check
        if ix.instruction.program_id != MY_PROGRAM_ID {
            return Err(ParseError::Filtered);
        }
        
        // Custom filtering logic
        if !self.should_process_instruction(ix) {
            return Err(ParseError::Filtered);
        }
        
        // Parse the instruction
        self.do_parse(ix)
    }
}

impl MyInstructionParser {
    fn should_process_instruction(&self, ix: &InstructionUpdate) -> bool {
        // Only process instructions with sufficient SOL transfers
        if let Some(tx) = &ix.transaction {
            let total_sol_transferred = tx.meta.as_ref()
                .map(|meta| meta.post_balances.iter().sum::<u64>() - meta.pre_balances.iter().sum::<u64>())
                .unwrap_or(0);
            
            return total_sol_transferred > 1_000_000; // More than 0.001 SOL
        }
        
        true
    }
}
```

## Protocol Buffer Integration

### Defining Protocol Buffers

Create protocol buffer definitions for your parsed data:

```protobuf
// proto/my_program.proto
syntax = "proto3";

package my_program;

// Account messages
message MyProgramAccount {
  string authority = 1;
  uint64 balance = 2;
  int64 created_at = 3;
  bytes metadata = 4;
}

message PoolAccount {
  string token_a_mint = 1;
  string token_b_mint = 2;
  uint64 token_a_amount = 3;
  uint64 token_b_amount = 4;
}

// Instruction messages
message InitializeInstruction {
  InitializeAccounts accounts = 1;
  InitializeData data = 2;
}

message InitializeAccounts {
  string user = 1;
  string account = 2;
  string system_program = 3;
}

message InitializeData {
  uint64 initial_balance = 1;
  bytes metadata = 2;
}

// Union type for all instructions
message MyProgramInstruction {
  oneof instruction {
    InitializeInstruction initialize = 1;
    DepositInstruction deposit = 2;
    WithdrawInstruction withdraw = 3;
    TransferInstruction transfer = 4;
  }
}
```

### Protocol Buffer Build Configuration

```rust
// build.rs
use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Build protocol buffers
    tonic_build::configure()
        .build_server(false)  // We only need the types, not the server
        .file_descriptor_set_path(out_dir.join("my_program_descriptor.bin"))
        .compile(&["proto/my_program.proto"], &["proto"])
        .unwrap();
}
```

### Implementing Proto Wrapper

```rust
// src/proto_def.rs
use yellowstone_vixen_core::proto::Proto;

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("my_program");
}

// Export the descriptor set for the stream server
pub const DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("my_program_descriptor");

// Implement conversion from parsed data to protobuf
impl From<MyProgramAccount> for proto::MyProgramAccount {
    fn from(account: MyProgramAccount) -> Self {
        Self {
            authority: account.authority.to_string(),
            balance: account.balance,
            created_at: account.created_at,
            metadata: account.metadata,
        }
    }
}

impl From<MyProgramInstruction> for proto::MyProgramInstruction {
    fn from(instruction: MyProgramInstruction) -> Self {
        use proto::my_program_instruction::Instruction;
        
        let instruction = match instruction {
            MyProgramInstruction::Initialize(accounts, data) => {
                Instruction::Initialize(proto::InitializeInstruction {
                    accounts: Some(proto::InitializeAccounts {
                        user: accounts.user.to_string(),
                        account: accounts.account.to_string(),
                        system_program: accounts.system_program.to_string(),
                    }),
                    data: Some(proto::InitializeData {
                        initial_balance: data.initial_balance,
                        metadata: data.metadata,
                    }),
                })
            }
            // ... other instruction types
        };
        
        Self {
            instruction: Some(instruction),
        }
    }
}
```

## Testing Custom Components

### Unit Tests with Mock Data

```rust
// tests/parser_tests.rs
use yellowstone_vixen_mock::{account_fixture, tx_fixture};
use my_custom_parser::{AccountParser, InstructionParser};

#[tokio::test]
async fn test_account_parsing() {
    let parser = AccountParser;
    
    // Test with real devnet data
    let account = account_fixture!("account_address", &parser);
    
    match account {
        MyProgramAccountType::UserAccount(user_account) => {
            assert_eq!(user_account.balance, expected_balance);
            assert_eq!(user_account.authority, expected_authority);
        }
        _ => panic!("Expected user account"),
    }
}

#[tokio::test]
async fn test_instruction_parsing() {
    let parser = InstructionParser;
    
    // Test with real transaction data
    let instructions = tx_fixture!("transaction_signature", &parser);
    
    assert!(!instructions.is_empty());
    
    match &instructions[0] {
        MyProgramInstruction::Deposit(accounts, data) => {
            assert_eq!(data.amount, expected_amount);
            assert_eq!(accounts.user, expected_user);
        }
        _ => panic!("Expected deposit instruction"),
    }
}

#[tokio::test]
async fn test_invalid_data_handling() {
    let parser = AccountParser;
    
    // Create account update with invalid data
    let invalid_account = AccountUpdate {
        pubkey: test_pubkey(),
        account: Account {
            owner: MY_PROGRAM_ID,
            data: vec![1, 2, 3], // Too small
            lamports: 0,
            executable: false,
            rent_epoch: 0,
        },
        slot: 100,
        write_version: 1,
    };
    
    let result = parser.parse(&invalid_account).await;
    assert!(result.is_err());
}
```

### Integration Tests

```rust
// tests/integration_tests.rs
use yellowstone_vixen::{Runtime, Pipeline};
use yellowstone_vixen_mock::MockSource;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_end_to_end_pipeline() {
    let (tx, mut rx) = mpsc::channel(100);
    
    // Create a test handler that sends results to our channel
    let test_handler = TestHandler::new(tx);
    
    // Set up runtime with mock source
    let mock_source = MockSource::from_fixtures("test_fixtures/");
    
    let runtime = Runtime::builder()
        .source(mock_source)
        .account(Pipeline::new(AccountParser, [test_handler.clone()]))
        .instruction(Pipeline::new(InstructionParser, [test_handler]))
        .build(test_config());
    
    // Run for a short time
    let runtime_handle = tokio::spawn(runtime.run());
    
    // Collect results
    let mut results = Vec::new();
    while let Some(result) = rx.recv().await {
        results.push(result);
        if results.len() >= expected_count {
            break;
        }
    }
    
    runtime_handle.abort();
    
    // Verify results
    assert_eq!(results.len(), expected_count);
    // ... additional assertions
}

#[derive(Debug, Clone)]
struct TestHandler {
    tx: mpsc::Sender<TestResult>,
}

impl TestHandler {
    fn new(tx: mpsc::Sender<TestResult>) -> Self {
        Self { tx }
    }
}

impl Handler<MyProgramInstruction> for TestHandler {
    async fn handle(&self, instruction: &MyProgramInstruction) -> HandlerResult<()> {
        let result = TestResult::from(instruction.clone());
        self.tx.send(result).await.ok();
        Ok(())
    }
}
```

### Performance Testing

```rust
// benches/parser_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_account_parsing(c: &mut Criterion) {
    let parser = AccountParser;
    let account_data = create_test_account_data();
    
    c.bench_function("account_parsing", |b| {
        b.iter(|| {
            black_box(parser.parse(black_box(&account_data)))
        })
    });
}

fn benchmark_instruction_parsing(c: &mut Criterion) {
    let parser = InstructionParser;
    let instruction_data = create_test_instruction_data();
    
    c.bench_function("instruction_parsing", |b| {
        b.iter(|| {
            black_box(parser.parse(black_box(&instruction_data)))
        })
    });
}

criterion_group!(benches, benchmark_account_parsing, benchmark_instruction_parsing);
criterion_main!(benches);
```

## Performance Optimization

### Memory Optimization

1. **Avoid Unnecessary Allocations**:
```rust
// ❌ Allocates a new Vec every time
fn parse_bad(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

// ✅ Returns a slice when possible
fn parse_good(data: &[u8]) -> &[u8] {
    &data[8..] // Just return a slice
}

// ✅ Use Cow for conditional allocation
fn parse_cow(data: &[u8]) -> Cow<[u8]> {
    if needs_processing(data) {
        Cow::Owned(process_data(data))
    } else {
        Cow::Borrowed(data)
    }
}
```

2. **Reuse Buffers**:
```rust
use std::cell::RefCell;

thread_local! {
    static DECODE_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024));
}

fn parse_with_buffer(data: &[u8]) -> Result<ParsedData, ParseError> {
    DECODE_BUFFER.with(|buffer| {
        let mut buf = buffer.borrow_mut();
        buf.clear();
        buf.extend_from_slice(data);
        
        // Use buf for parsing
        parse_from_buffer(&buf)
    })
}
```

### CPU Optimization

1. **Early Returns**:
```rust
impl Parser for OptimizedParser {
    async fn parse(&self, ix: &InstructionUpdate) -> Result<Self::Output, ParseError> {
        // Fast path: check program ID first
        if ix.instruction.program_id != MY_PROGRAM_ID {
            return Err(ParseError::Filtered);
        }
        
        // Fast path: check minimum size
        if ix.instruction.data.len() < MIN_INSTRUCTION_SIZE {
            return Err(ParseError::Filtered);
        }
        
        // More expensive checks only if we need them
        self.full_parse(ix).await
    }
}
```

2. **Batch Processing in Handlers**:
```rust
#[derive(Debug)]
pub struct BatchingHandler {
    batch: Arc<Mutex<Vec<MyProgramInstruction>>>,
    batch_size: usize,
}

impl Handler<MyProgramInstruction> for BatchingHandler {
    async fn handle(&self, instruction: &MyProgramInstruction) -> HandlerResult<()> {
        let mut batch = self.batch.lock().await;
        batch.push(instruction.clone());
        
        if batch.len() >= self.batch_size {
            let items = std::mem::take(&mut *batch);
            drop(batch); // Release lock
            
            self.process_batch(items).await?;
        }
        
        Ok(())
    }
}
```

## Error Handling Best Practices

### Parser Error Handling

```rust
impl Parser for RobustParser {
    async fn parse(&self, input: &Input) -> Result<Self::Output, ParseError> {
        // Use ParseError::Filtered for intentional skips
        if input.should_be_skipped() {
            return Err(ParseError::Filtered);
        }
        
        // Provide detailed error messages for debugging
        let data = input.data.get(8..)
            .ok_or_else(|| ParseError::Other("Data too small for header".into()))?;
        
        // Handle specific parsing errors with context
        let discriminator = parse_discriminator(data)
            .map_err(|e| ParseError::Other(format!("Invalid discriminator: {}", e).into()))?;
        
        match discriminator {
            VALID_DISCRIMINATOR => self.parse_valid_data(data),
            _ => Err(ParseError::Other(format!("Unknown discriminator: {}", discriminator).into()))
        }
    }
}
```

### Handler Error Recovery

```rust
impl Handler<MyData> for ResilientHandler {
    async fn handle(&self, data: &MyData) -> HandlerResult<()> {
        // Implement retry logic for transient errors
        let mut retries = 0;
        let max_retries = 3;
        
        loop {
            match self.try_process(data).await {
                Ok(()) => return Ok(()),
                Err(e) if retries < max_retries && is_retryable(&e) => {
                    retries += 1;
                    tokio::time::sleep(Duration::from_millis(100 * retries)).await;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to process data after {} retries: {}", retries, e);
                    return Err(e);
                }
            }
        }
    }
}

fn is_retryable(error: &Box<dyn std::error::Error + Send + Sync>) -> bool {
    // Check if the error is worth retrying
    error.to_string().contains("connection") || 
    error.to_string().contains("timeout")
}
```

## Contributing Back

### Preparing for Contribution

1. **Follow Coding Standards**:
   - Use `cargo fmt` for formatting
   - Run `cargo clippy` for linting
   - Add comprehensive tests
   - Include documentation

2. **Add Parser Metadata**:
```rust
// Add to your parser crate
pub const PROGRAM_ID: &str = "YourProgramId11111111111111111111111111111";
pub const PROGRAM_NAME: &str = "Your Program Name";
pub const PARSER_VERSION: &str = "1.0.0";

// Export descriptor set for stream integration
pub const DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));
```

3. **Create Example Usage**:
```rust
// examples/my_program.rs
use yellowstone_vixen::{Runtime, Pipeline};
use my_program_parser::{AccountParser, InstructionParser, LoggingHandler};

fn main() {
    Runtime::builder()
        .account(Pipeline::new(AccountParser, [LoggingHandler]))
        .instruction(Pipeline::new(InstructionParser, [LoggingHandler]))
        .build(config)
        .run();
}
```

4. **Add Documentation**:
   - README.md with usage examples
   - API documentation with rustdoc
   - Integration examples

### Submission Process

1. Fork the repository
2. Create a feature branch
3. Add your parser crate to the workspace
4. Update the main README with your parser
5. Add integration tests
6. Submit a pull request

The Yellowstone Vixen team welcomes contributions and will work with you to integrate high-quality parsers into the main repository.