# Adding Programs

This guide provides a comprehensive walkthrough for adding support for new Solana programs to Yellowstone Vixen, from initial assessment to production deployment.

## Program Assessment

### 1. Evaluate Program Suitability

Before adding a new program, consider:

**Program Characteristics:**
- **Activity Level** - Is the program actively used?
- **Stability** - Has the program been stable for several months?
- **Community Size** - Is there an active developer community?
- **Documentation** - Are there good docs and examples?

**Technical Considerations:**
- **IDL Availability** - Does the program have a public IDL?
- **Instruction Complexity** - How complex are the program's instructions?
- **Event Structure** - What events does the program emit?
- **Update Frequency** - How often does the program get updated?

### 2. Gather Program Information

```bash
# Get program details
solana program show <PROGRAM_ID>

# Check program size and deployment
solana program dump <PROGRAM_ID> program.dump

# Get IDL if available
anchor idl fetch <PROGRAM_ID> --outfile program.idl

# Or find IDL in program repository
```

### 3. Analyze Program Usage

```bash
# Check recent activity
solana transaction-history <PROGRAM_ID> --limit 10

# Monitor account changes
solana account <ACCOUNT_PUBKEY> --output json
```

## Planning the Parser

### 1. Define Scope

**What to Parse:**
- [ ] Instruction parsing (most important)
- [ ] Account state parsing
- [ ] Event parsing
- [ ] Error handling

**Priority Features:**
- [ ] Basic instruction parsing
- [ ] Account state tracking
- [ ] Error recovery
- [ ] Performance optimization

### 2. Design Parser Architecture

```rust
// Example parser structure
pub struct MyProgramParser;

impl InstructionParser for MyProgramParser {
    type Instruction = MyProgramInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        // Implementation
        todo!()
    }
}

impl AccountParser for MyProgramParser {
    type Account = MyProgramAccount;
    type Error = ParseError;

    fn parse(&self, account_info: &AccountInfo) -> Result<Self::Account, Self::Error> {
        // Implementation
        todo!()
    }
}
```

### 3. Define Data Structures

```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub enum MyProgramInstruction {
    Initialize(InitializeData),
    Execute(ExecuteData),
    Close,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct MyProgramAccount {
    pub authority: Pubkey,
    pub data: Vec<u8>,
    pub bump: u8,
    pub created_at: i64,
}
```

## Implementation Approaches

### Option 1: Manual Implementation

Best for simple programs or when you need full control.

#### 1. Create Parser Crate

```bash
cargo new --lib my-program-parser
```

#### 2. Implement Parser Logic

```rust
// src/lib.rs
use yellowstone_vixen_core::*;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub enum MyProgramInstruction {
    Initialize { amount: u64, authority: Pubkey },
    Execute { data: Vec<u8> },
    Close,
}

pub struct MyProgramParser;

impl MyProgramParser {
    pub fn program_id() -> Pubkey {
        "MyProgram111111111111111111111111111".parse().unwrap()
    }

    const INITIALIZE_DISCRIMINATOR: u64 = 0x12345678abcdef12;
    const EXECUTE_DISCRIMINATOR: u64 = 0x87654321fedcba21;
    const CLOSE_DISCRIMINATOR: u64 = 0x1111111122222222;
}

impl InstructionParser for MyProgramParser {
    type Instruction = MyProgramInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        if instruction.program_id != Self::program_id() {
            return Err(ParseError::Filtered);
        }

        if instruction.data.len() < 8 {
            return Err(ParseError::Validation("Instruction too short".to_string()));
        }

        let discriminator = u64::from_le_bytes(instruction.data[..8].try_into().unwrap());

        match discriminator {
            Self::INITIALIZE_DISCRIMINATOR => {
                let data: InitializeData = borsh::from_slice(&instruction.data[8..])
                    .map_err(|e| ParseError::Parsing(e.to_string()))?;
                Ok(MyProgramInstruction::Initialize(data))
            }
            Self::EXECUTE_DISCRIMINATOR => {
                let data: ExecuteData = borsh::from_slice(&instruction.data[8..])
                    .map_err(|e| ParseError::Parsing(e.to_string()))?;
                Ok(MyProgramInstruction::Execute(data))
            }
            Self::CLOSE_DISCRIMINATOR => {
                Ok(MyProgramInstruction::Close)
            }
            _ => Err(ParseError::Filtered),
        }
    }
}
```

#### 3. Add Comprehensive Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use yellowstone_vixen_core::test_utils::*;

    #[test]
    fn test_parse_initialize() {
        let parser = MyProgramParser;
        let instruction = create_test_instruction(
            MyProgramParser::INITIALIZE_DISCRIMINATOR,
            InitializeData { amount: 1000, authority: Pubkey::new_unique() }
        );

        let result = parser.parse(&instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            MyProgramInstruction::Initialize(data) => {
                assert_eq!(data.amount, 1000);
            }
            _ => panic!("Expected Initialize instruction"),
        }
    }

    #[test]
    fn test_parse_invalid_discriminator() {
        let parser = MyProgramParser;
        let instruction = Instruction {
            program_id: MyProgramParser::program_id(),
            accounts: vec![],
            data: vec![0xff; 16],
        };

        let result = parser.parse(&instruction);
        assert!(matches!(result, Err(ParseError::Filtered)));
    }
}
```

### Option 2: Codama-Generated Parser

Best for complex programs with available IDLs.

#### 1. Set Up Codama

```bash
# Install Codama
npm install -g @codama/cli

# Create codama.cjs
const { rootNodeFromAnchor } = require('@codama/nodes-from-anchor');
const { renderVixenParsers } = require('@codama/renderers-vixen-parser');
const { readJson } = require('@codama/renderers-core');

async function generateParser() {
    const idl = readJson('./program.idl');
    const rootNode = rootNodeFromAnchor(idl);

    const output = renderVixenParsers(rootNode, {
        name: 'MyProgramParser',
        crateName: 'my-program-parser',
        options: {
            generateTests: true,
            includeComments: true,
        }
    });

    require('fs').writeFileSync('./src/generated.rs', output);
}

generateParser();
```

#### 2. Customize Generated Parser

```rust
// src/lib.rs
pub mod generated;

use generated::MyProgramParser as GeneratedParser;

pub struct MyProgramParser {
    base: GeneratedParser,
    custom_config: CustomConfig,
}

impl MyProgramParser {
    pub fn new(config: CustomConfig) -> Self {
        Self {
            base: GeneratedParser,
            custom_config: config,
        }
    }
}

impl InstructionParser for MyProgramParser {
    type Instruction = MyProgramInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        // Custom validation
        self.validate_instruction(instruction)?;

        // Use generated parser
        let result = self.base.parse(instruction)?;

        // Custom enrichment
        self.enrich_instruction(result)
    }
}
```

## Testing Strategy

### 1. Unit Tests

```rust
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_all_instruction_types() {
        let parser = MyProgramParser;
        let instructions = create_all_instruction_types();

        for instruction in instructions {
            let result = parser.parse(&instruction);
            assert!(result.is_ok(), "Failed to parse {:?}", instruction);
        }
    }

    #[test]
    fn test_error_conditions() {
        let parser = MyProgramParser;

        // Test invalid program ID
        let invalid_program = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![],
            data: vec![0; 8],
        };
        assert!(matches!(parser.parse(&invalid_program), Err(ParseError::Filtered)));

        // Test malformed data
        let malformed = Instruction {
            program_id: MyProgramParser::program_id(),
            accounts: vec![],
            data: vec![0; 4], // Too short
        };
        assert!(matches!(parser.parse(&malformed), Err(ParseError::Validation(_))));
    }
}
```

### 2. Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use yellowstone_vixen_mock::*;
    use super::*;

    #[tokio::test]
    async fn test_with_real_data() {
        // Load real program data
        let fixtures = FixtureLoader::new("./fixtures")
            .load_from_chain(MyProgramParser::program_id())
            .build();

        let mock_env = MockEnvironment::new(fixtures);
        let parser = MyProgramParser;

        let results = mock_env.run_parser(parser).await;

        assert!(!results.is_empty());
        for result in results {
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_pipeline_integration() {
        let parser = MyProgramParser;
        let handler = TestHandler::new();

        let pipeline = Pipeline::new(parser, vec![handler.boxed()]);

        let mock_env = MockEnvironment::new(create_test_data());
        let result = mock_env.run_pipeline(pipeline).await;

        assert!(result.is_ok());
    }
}
```

### 3. Performance Tests

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
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

    criterion_group!(benches, benchmark_parser);
    criterion_main!(benches);
}
```

## Documentation

### 1. Parser Documentation

```rust
//! # My Program Parser
//!
//! This crate provides parsing functionality for My Program on Solana.
//!
//! ## Features
//!
//! - Parse all instruction types
//! - Account state parsing
//! - Comprehensive error handling
//! - High performance
//!
//! ## Usage
//!
//! ```rust
//! use my_program_parser::MyProgramParser;
//! use yellowstone_vixen::Pipeline;
//!
//! let parser = MyProgramParser;
//! let pipeline = Pipeline::new(parser, handlers);
//! ```
//!
//! ## Supported Instructions
//!
//! - `Initialize` - Initialize program state
//! - `Execute` - Execute program logic
//! - `Close` - Close program accounts
```

### 2. API Documentation

```rust
/// Parses My Program instructions from Solana transactions.
///
/// This parser handles all instruction types defined in My Program,
/// converting raw transaction data into structured Rust types.
///
/// # Examples
///
/// ```
/// use my_program_parser::MyProgramParser;
///
/// let parser = MyProgramParser;
/// let instruction = create_test_instruction();
/// let parsed = parser.parse(&instruction).unwrap();
/// ```
///
/// # Errors
///
/// Returns `ParseError::Filtered` if the instruction is not for this program.
/// Returns `ParseError::Validation` for malformed instructions.
/// Returns `ParseError::Parsing` for deserialization errors.
pub struct MyProgramParser;
```

## Packaging and Distribution

### 1. Crate Configuration

```toml
# Cargo.toml
[package]
name = "yellowstone-vixen-my-program-parser"
version = "0.1.0"
edition = "2021"
description = "Parser for My Program on Solana"
license = "MIT OR Apache-2.0"
repository = "https://github.com/rpcpool/yellowstone-vixen"

[dependencies]
yellowstone-vixen-core = { path = "../../crates/core", version = "0.4" }
borsh = "0.10"
solana-program = "1.16"

[dev-dependencies]
yellowstone-vixen-mock = { path = "../../crates/mock", version = "0.4" }
```

### 2. Feature Flags

```toml
[features]
default = ["std"]
std = ["borsh/std", "solana-program/std"]
serde = ["dep:serde", "borsh/serde"]
```

### 3. Publishing to Crates.io

```bash
# Test before publishing
cargo test
cargo publish --dry-run

# Publish
cargo publish
```

## Integration with Main Repository

### 1. Add to Workspace

```toml
# Cargo.toml (root)
[workspace]
members = [
    # ... existing members
    "crates/my-program-parser"
]

[workspace.dependencies]
yellowstone-vixen-my-program-parser = { path = "crates/my-program-parser", version = "0.1.0" }
```

### 2. Update Main Parser Crate

```rust
// crates/parser/src/lib.rs
pub mod my_program {
    pub use yellowstone_vixen_my_program_parser::*;
}
```

### 3. Add to Examples

```rust
// examples/my-program-example.rs
use yellowstone_vixen::Pipeline;
use yellowstone_vixen_my_program_parser::MyProgramParser;

#[tokio::main]
async fn main() {
    let pipeline = Pipeline::new(
        MyProgramParser,
        vec![Logger.boxed()]
    );

    // Run pipeline
}
```

## Performance Optimization

### 1. Memory Optimization

```rust
impl MyProgramParser {
    // Reuse buffers to avoid allocations
    thread_local! {
        static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    }

    fn parse_with_buffer(&self, data: &[u8]) -> Result<MyProgramInstruction, ParseError> {
        Self::BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            buf.clear();
            buf.extend_from_slice(data);

            borsh::from_slice(buf)
                .map_err(|e| ParseError::Parsing(e.to_string()))
        })
    }
}
```

### 2. CPU Optimization

```rust
impl MyProgramParser {
    // Pre-compile discriminators for faster matching
    const DISCRIMINATORS: &[u64] = &[
        INITIALIZE_DISCRIMINATOR,
        EXECUTE_DISCRIMINATOR,
        CLOSE_DISCRIMINATOR,
    ];

    fn fast_discriminator_match(&self, data: &[u8]) -> Option<usize> {
        if data.len() < 8 {
            return None;
        }

        let discriminator = u64::from_le_bytes(data[..8].try_into().ok()?);
        Self::DISCRIMINATORS.iter().position(|&d| d == discriminator)
    }
}
```

### 3. Batch Processing

```rust
impl MyProgramParser {
    fn parse_batch(&self, instructions: &[Instruction]) -> Vec<Result<MyProgramInstruction, ParseError>> {
        instructions.iter()
            .map(|instruction| self.parse(instruction))
            .collect()
    }
}
```

## Error Handling and Recovery

### 1. Comprehensive Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum MyProgramParseError {
    #[error("IDL parsing error: {0}")]
    IdlParsing(String),

    #[error("Instruction parsing error: {0}")]
    InstructionParsing(String),

    #[error("Account parsing error: {0}")]
    AccountParsing(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unsupported instruction version: {0}")]
    UnsupportedVersion(u32),
}
```

### 2. Error Recovery Strategies

```rust
impl MyProgramParser {
    fn parse_with_recovery(&self, instruction: &Instruction) -> Result<MyProgramInstruction, ParseError> {
        match self.parse(instruction) {
            Ok(result) => Ok(result),
            Err(ParseError::Parsing(_)) => {
                // Try alternative parsing method
                self.fallback_parse(instruction)
            }
            Err(e) => Err(e),
        }
    }

    fn fallback_parse(&self, instruction: &Instruction) -> Result<MyProgramInstruction, ParseError> {
        // Simplified parsing for error recovery
        // Implementation depends on program specifics
        todo!()
    }
}
```

## Maintenance and Updates

### 1. Version Compatibility

```rust
impl MyProgramParser {
    const SUPPORTED_VERSIONS: &[u32] = &[1, 2];

    fn parse_with_version(&self, instruction: &Instruction, version: u32) -> Result<MyProgramInstruction, ParseError> {
        if !Self::SUPPORTED_VERSIONS.contains(&version) {
            return Err(ParseError::Validation(format!("Unsupported version: {}", version)));
        }

        match version {
            1 => self.parse_v1(instruction),
            2 => self.parse_v2(instruction),
            _ => unreachable!(),
        }
    }
}
```

### 2. Update Process

```bash
# When program updates
node codama.cjs  # Regenerate if using Codama

# Update tests
cargo test

# Update documentation
cargo doc

# Run integration tests
cargo test --test integration
```

## Contributing Back

### 1. Create Pull Request

```bash
# Fork and clone
git clone https://github.com/your-username/yellowstone-vixen.git
cd yellowstone-vixen

# Create branch
git checkout -b add-my-program-parser

# Add parser crate
cp -r crates/my-program-parser crates/

# Commit changes
git add .
git commit -m "feat: add My Program parser"

# Push and create PR
git push origin add-my-program-parser
```

### 2. PR Checklist

- [ ] Parser implementation complete
- [ ] Comprehensive test coverage
- [ ] Documentation added
- [ ] Performance benchmarks included
- [ ] Integration with main repo
- [ ] Example usage provided
- [ ] Changelog updated

### 3. Review Process

1. **Automated Checks** - CI runs tests and linting
2. **Code Review** - Maintainers review implementation
3. **Testing** - Additional testing may be requested
4. **Documentation Review** - Docs are checked for completeness
5. **Merge** - PR is merged and released

## Best Practices

### Code Quality

1. **Comprehensive Testing** - Test all code paths and edge cases
2. **Error Handling** - Handle errors gracefully without crashing
3. **Documentation** - Document all public APIs and complex logic
4. **Performance** - Optimize for speed and memory usage
5. **Maintainability** - Write clear, readable code

### Parser Design

1. **Idempotent Parsing** - Safe to parse the same data multiple times
2. **Forward Compatibility** - Handle new instruction types gracefully
3. **Clear Error Messages** - Provide helpful error messages for debugging
4. **Resource Efficiency** - Minimize memory allocations and CPU usage
5. **Extensibility** - Design for easy addition of new features

### Testing

1. **Unit Tests** - Test individual functions and methods
2. **Integration Tests** - Test with mock Solana data
3. **Performance Tests** - Benchmark parsing performance
4. **Edge Case Testing** - Test with malformed and edge case data
5. **Regression Testing** - Ensure updates don't break existing functionality

This comprehensive guide provides everything needed to successfully add new program support to Yellowstone Vixen. Remember to follow the established patterns, write thorough tests, and contribute back to the community!
