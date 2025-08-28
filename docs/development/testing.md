# Testing

This guide covers comprehensive testing strategies for Yellowstone Vixen, including unit tests, integration tests, mock testing, and performance testing.

## Testing Overview

Yellowstone Vixen employs a multi-layered testing approach:

- **Unit Tests** - Test individual components in isolation
- **Integration Tests** - Test component interactions
- **Mock Tests** - Test with simulated Solana data
- **End-to-End Tests** - Test complete pipelines
- **Performance Tests** - Validate performance characteristics

## Unit Testing

### Parser Testing

```rust
#[cfg(test)]
mod parser_tests {
    use super::*;
    use yellowstone_vixen_core::Instruction;

    #[test]
    fn test_parse_valid_initialize_instruction() {
        let parser = MyProgramParser;

        // Create test instruction
        let instruction = Instruction {
            program_id: parser.program_id(),
            accounts: vec![],
            data: create_initialize_instruction_data(),
        };

        let result = parser.parse(&instruction);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        match parsed {
            MyInstruction::Initialize(data) => {
                assert_eq!(data.amount, 1000);
                assert!(!data.authority.is_empty());
            }
            _ => panic!("Expected Initialize instruction"),
        }
    }

    #[test]
    fn test_parse_invalid_discriminator() {
        let parser = MyProgramParser;

        let instruction = Instruction {
            program_id: parser.program_id(),
            accounts: vec![],
            data: vec![0xff; 16], // Invalid discriminator
        };

        let result = parser.parse(&instruction);
        assert!(matches!(result, Err(ParseError::Filtered)));
    }

    #[test]
    fn test_parse_wrong_program_id() {
        let parser = MyProgramParser;

        let instruction = Instruction {
            program_id: Pubkey::new_unique(), // Wrong program
            accounts: vec![],
            data: create_valid_instruction_data(),
        };

        let result = parser.parse(&instruction);
        assert!(matches!(result, Err(ParseError::Filtered)));
    }
}
```

### Handler Testing

```rust
#[cfg(test)]
mod handler_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    struct MockDatabase {
        data: Arc<Mutex<Vec<MyData>>>,
    }

    impl MockDatabase {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn get_data(&self) -> Vec<MyData> {
            self.data.lock().await.clone()
        }
    }

    #[derive(Debug, Clone)]
    struct MyData {
        id: String,
        value: u64,
    }

    struct TestHandler {
        db: MockDatabase,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                db: MockDatabase::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Handler<MyData> for TestHandler {
        async fn handle(&self, data: &MyData) -> HandlerResult<()> {
            self.db.data.lock().await.push(data.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handler_stores_data() {
        let handler = TestHandler::new();
        let test_data = MyData {
            id: "test-1".to_string(),
            value: 42,
        };

        let result = handler.handle(&test_data).await;
        assert!(result.is_ok());

        let stored = handler.db.get_data().await;
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id, "test-1");
        assert_eq!(stored[0].value, 42);
    }

    #[tokio::test]
    async fn test_handler_error_handling() {
        struct FailingHandler;

        #[async_trait::async_trait]
        impl Handler<MyData> for FailingHandler {
            async fn handle(&self, _data: &MyData) -> HandlerResult<()> {
                Err(anyhow::anyhow!("Simulated failure").into())
            }
        }

        let handler = FailingHandler;
        let test_data = MyData {
            id: "test".to_string(),
            value: 1,
        };

        let result = handler.handle(&test_data).await;
        assert!(result.is_err());
    }
}
```

## Integration Testing

### Pipeline Testing

```rust
#[cfg(test)]
mod integration_tests {
    use yellowstone_vixen::*;
    use yellowstone_vixen_mock::*;
    use super::*;

    #[tokio::test]
    async fn test_complete_pipeline() {
        // Set up mock data
        let mock_data = MockDataBuilder::new()
            .add_instruction(create_test_instruction())
            .add_account(create_test_account())
            .build();

        // Create pipeline
        let pipeline = Pipeline::new(
            MyProgramParser,
            vec![
                DatabaseHandler::new().boxed(),
                MetricsHandler::new().boxed(),
            ]
        );

        // Create mock environment
        let mock_env = MockEnvironment::new(mock_data);

        // Run pipeline
        let result = mock_env.run_pipeline(pipeline).await;

        // Verify results
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.processed_instructions, 1);
        assert_eq!(stats.processed_accounts, 1);
        assert_eq!(stats.errors, 0);
    }

    #[tokio::test]
    async fn test_pipeline_error_recovery() {
        // Set up data with some invalid entries
        let mock_data = MockDataBuilder::new()
            .add_instruction(create_valid_instruction())
            .add_instruction(create_invalid_instruction())
            .add_instruction(create_valid_instruction())
            .build();

        let pipeline = Pipeline::new(MyProgramParser, vec![TestHandler::new().boxed()]);
        let mock_env = MockEnvironment::new(mock_data);

        let result = mock_env.run_pipeline(pipeline).await;

        // Should process valid instructions and skip invalid ones
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.processed_instructions, 2); // 2 valid out of 3
        assert_eq!(stats.errors, 1);
    }
}
```

### Cross-Component Testing

```rust
#[cfg(test)]
mod cross_component_tests {
    use super::*;

    #[tokio::test]
    async fn test_parser_handler_integration() {
        let parser = MyProgramParser;
        let handler = TestHandler::new();

        // Create raw instruction
        let raw_instruction = create_raw_instruction();

        // Parse instruction
        let parsed = parser.parse(&raw_instruction).unwrap();

        // Handle parsed data
        let result = handler.handle(&parsed).await;

        // Verify end-to-end flow
        assert!(result.is_ok());
        assert_eq!(handler.get_processed_count().await, 1);
    }

    #[tokio::test]
    async fn test_multiple_handlers() {
        let parser = MyProgramParser;
        let handler1 = CountingHandler::new();
        let handler2 = LoggingHandler::new();

        let pipeline = Pipeline::new(
            parser,
            vec![handler1.boxed(), handler2.boxed()]
        );

        let mock_env = MockEnvironment::new(create_test_data());
        let result = mock_env.run_pipeline(pipeline).await;

        assert!(result.is_ok());

        // Verify both handlers were called
        assert_eq!(handler1.get_count().await, 1);
        assert!(handler2.has_logs().await);
    }
}
```

## Mock Testing

### Using the Mock Crate

```rust
use yellowstone_vixen_mock::*;

#[tokio::test]
async fn test_with_fixtures() {
    // Load fixtures from files
    let fixtures = FixtureLoader::new("./fixtures")
        .load_accounts("test_accounts.json")
        .load_transactions("test_transactions.json")
        .load_block_meta("test_blocks.json")
        .build();

    // Create mock source
    let mock_source = MockSource::new(fixtures)
        .with_speed_multiplier(2.0) // 2x speed for faster tests
        .with_error_injection(0.1); // 10% error rate

    // Create runtime with mock source
    let runtime = Runtime::builder()
        .source(mock_source)
        .instruction(Pipeline::new(MyParser, vec![TestHandler::new()]))
        .build()
        .await
        .unwrap();

    // Run for specific duration
    let result = runtime.run_for(Duration::from_secs(10)).await;

    // Verify results
    assert!(result.is_ok());
    let stats = runtime.get_stats().await;
    assert!(stats.processed_instructions > 0);
}
```

### Creating Custom Fixtures

```rust
#[cfg(test)]
mod fixture_tests {
    use yellowstone_vixen_mock::*;
    use serde_json;

    fn create_token_fixture() -> FixtureData {
        FixtureData {
            accounts: vec![
                AccountFixture {
                    pubkey: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                    owner: "11111111111111111111111111111112".to_string(),
                    lamports: 1000000,
                    data: base64::encode(create_token_account_data()),
                    executable: false,
                    rent_epoch: 0,
                }
            ],
            transactions: vec![
                TransactionFixture {
                    signature: "test_sig_1".to_string(),
                    slot: 12345,
                    block_time: Some(1640995200),
                    instructions: vec![
                        InstructionFixture {
                            program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                            accounts: vec![
                                AccountMetaFixture {
                                    pubkey: "source_account".to_string(),
                                    is_signer: true,
                                    is_writable: true,
                                }
                            ],
                            data: base64::encode(create_transfer_instruction_data()),
                        }
                    ],
                }
            ],
            block_meta: vec![],
        }
    }

    #[tokio::test]
    async fn test_with_custom_fixture() {
        let fixture = create_token_fixture();
        let mock_source = MockSource::new(fixture.into());

        let runtime = Runtime::builder()
            .source(mock_source)
            .instruction(Pipeline::new(TokenParser, vec![TestHandler::new()]))
            .build()
            .await
            .unwrap();

        let result = runtime.run_for(Duration::from_secs(5)).await;
        assert!(result.is_ok());
    }
}
```

## End-to-End Testing

### Full Pipeline Testing

```rust
#[cfg(test)]
mod e2e_tests {
    use super::*;
    use yellowstone_vixen::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_full_runtime_lifecycle() {
        // Create comprehensive test setup
        let config = create_test_config();
        let runtime = Runtime::builder()
            .with_config(config)
            .instruction(Pipeline::new(
                MyProgramParser,
                vec![
                    DatabaseHandler::new().boxed(),
                    MetricsHandler::new().boxed(),
                    AlertHandler::new().boxed(),
                ]
            ))
            .account(Pipeline::new(
                MyAccountParser,
                vec![AccountHandler::new().boxed()]
            ))
            .build()
            .await
            .unwrap();

        // Start runtime
        let runtime_handle = tokio::spawn(async move {
            runtime.run().await
        });

        // Wait a bit for processing
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Send test data through mock source
        // (Assuming runtime has a way to inject test data)

        // Stop runtime gracefully
        runtime_handle.abort();

        // Verify final state
        // (Check database, metrics, logs, etc.)
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // Test invalid configurations
        let invalid_config = create_invalid_config();

        let result = Runtime::builder()
            .with_config(invalid_config)
            .build()
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::Config(_)));
    }
}
```

## Performance Testing

### Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parser(c: &mut Criterion) {
    let parser = MyProgramParser;
    let instruction = create_test_instruction();

    c.bench_function("parse_instruction", |b| {
        b.iter(|| {
            let result = parser.parse(black_box(&instruction));
            black_box(result);
        })
    });
}

fn benchmark_handler(c: &mut Criterion) {
    let handler = TestHandler::new();
    let data = create_test_data();

    c.bench_function("handle_data", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handler.handle(black_box(&data)).await;
                black_box(result);
            });
        })
    });
}

criterion_group!(benches, benchmark_parser, benchmark_handler);
criterion_main!(benches);
```

### Load Testing

```rust
#[cfg(test)]
mod load_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    #[tokio::test]
    async fn test_concurrent_processing() {
        let parser = Arc::new(MyProgramParser);
        let semaphore = Arc::new(Semaphore::new(100)); // Limit concurrency

        let tasks: Vec<_> = (0..1000).map(|i| {
            let parser = parser.clone();
            let semaphore = semaphore.clone();
            let instruction = create_test_instruction_with_id(i);

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                parser.parse(&instruction)
            })
        }).collect();

        let results = futures::future::join_all(tasks).await;

        let success_count = results.iter()
            .filter(|r| r.as_ref().unwrap().is_ok())
            .count();

        assert_eq!(success_count, 1000);
    }

    #[tokio::test]
    async fn test_memory_usage() {
        let parser = MyProgramParser;

        // Monitor memory before
        let start_mem = get_memory_usage();

        // Process many instructions
        for i in 0..10000 {
            let instruction = create_test_instruction_with_id(i);
            let _ = parser.parse(&instruction);
        }

        // Monitor memory after
        let end_mem = get_memory_usage();

        // Assert memory growth is reasonable
        let growth = end_mem - start_mem;
        assert!(growth < 10 * 1024 * 1024); // Less than 10MB growth
    }

    fn get_memory_usage() -> usize {
        // Platform-specific memory measurement
        // This is a simplified example
        0
    }
}
```

## Property-Based Testing

### Using Proptest

```rust
use proptest::prelude::*;

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_parser_never_panics(data in prop::collection::vec(prop::num::u8::ANY, 8..1000)) {
            let parser = MyProgramParser;
            let instruction = Instruction {
                program_id: parser.program_id(),
                accounts: vec![],
                data,
            };

            // Parser should handle any input gracefully
            let result = parser.parse(&instruction);
            // Result should be either Ok or Filtered, never panic
            assert!(result.is_ok() || matches!(result, Err(ParseError::Filtered)));
        }

        #[test]
        fn test_parser_roundtrip(
            amount in prop::num::u64::ANY,
            authority in prop::array::uniform32(prop::num::u8::ANY)
        ) {
            let original = MyInstruction::Initialize(InitializeData {
                amount,
                authority: Pubkey::from(authority),
            });

            // Serialize
            let data = borsh::to_vec(&original).unwrap();

            // Create instruction
            let instruction = Instruction {
                program_id: MyProgramParser.program_id(),
                accounts: vec![],
                data: [INITIALIZE_DISCRIMINATOR.to_le_bytes().as_slice(), &data].concat(),
            };

            // Parse
            let parser = MyProgramParser;
            let parsed = parser.parse(&instruction).unwrap();

            // Verify roundtrip
            assert_eq!(parsed, original);
        }
    }
}
```

## Test Utilities

### Test Helpers

```rust
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use yellowstone_vixen_core::*;

    pub fn create_test_instruction() -> Instruction {
        Instruction {
            program_id: MyProgramParser.program_id(),
            accounts: vec![
                AccountMeta {
                    pubkey: Pubkey::new_unique(),
                    is_signer: true,
                    is_writable: true,
                }
            ],
            data: create_initialize_instruction_data(),
        }
    }

    pub fn create_initialize_instruction_data() -> Vec<u8> {
        let data = InitializeData {
            amount: 1000,
            authority: Pubkey::new_unique(),
        };
        [INITIALIZE_DISCRIMINATOR.to_le_bytes().as_slice(), &borsh::to_vec(&data).unwrap()].concat()
    }

    pub fn create_invalid_instruction() -> Instruction {
        Instruction {
            program_id: MyProgramParser.program_id(),
            accounts: vec![],
            data: vec![0xff; 4], // Too short
        }
    }

    pub async fn with_timeout<F, T>(future: F, timeout: Duration) -> Result<T, ()>
    where
        F: Future<Output = T>,
    {
        tokio::time::timeout(timeout, future)
            .await
            .map_err(|_| ())
    }
}
```

### Test Configuration

```rust
#[cfg(test)]
pub mod test_config {
    use super::*;

    pub fn create_test_config() -> Config {
        Config {
            grpc: GrpcConfig {
                endpoint: "http://localhost:50051".to_string(),
                token: None,
            },
            programs: vec![
                ProgramConfig {
                    name: "Test Program".to_string(),
                    address: "MyProgram111111111111111111111111111".to_string(),
                }
            ],
            metrics: MetricsConfig {
                enabled: false,
                prometheus: None,
            },
        }
    }

    pub fn create_invalid_config() -> Config {
        Config {
            grpc: GrpcConfig {
                endpoint: "invalid-endpoint".to_string(),
                token: None,
            },
            programs: vec![],
            metrics: MetricsConfig {
                enabled: true,
                prometheus: Some(PrometheusConfig {
                    port: 0, // Invalid port
                }),
            },
        }
    }
}
```

## Continuous Integration

### GitHub Actions Example

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2

    - name: Run tests
      run: cargo test --all-features

    - name: Run clippy
      run: cargo clippy --all-features -- -D warnings

    - name: Check formatting
      run: cargo fmt --all -- --check

    - name: Run benchmarks
      run: cargo bench
```

### Test Coverage

```yaml
# Add to CI workflow
- name: Generate coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Html

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    file: ./tarpaulin-report.html
```

## Best Practices

### Test Organization

1. **Unit tests** in the same file as the code they test
2. **Integration tests** in a separate `tests/` directory
3. **Mock tests** using the mock crate
4. **Performance tests** in a separate benchmark file

### Test Quality

1. **Test all code paths** including error conditions
2. **Use descriptive test names** that explain what they're testing
3. **Test edge cases** and boundary conditions
4. **Avoid flaky tests** that depend on timing or external state
5. **Keep tests fast** to encourage frequent running

### Test Maintenance

1. **Update tests** when changing code
2. **Remove obsolete tests** when removing features
3. **Add tests for bug fixes** to prevent regressions
4. **Review test coverage** regularly
5. **Document test setup** for complex scenarios

This comprehensive testing approach ensures Yellowstone Vixen components are reliable, performant, and maintainable.
