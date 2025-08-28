# Creating Parsers

This comprehensive guide walks you through creating custom parsers for Yellowstone Vixen, from understanding the architecture to implementing production-ready parsers.

## Parser Architecture Overview

### Core Concepts

Parsers in Yellowstone Vixen are responsible for transforming raw Solana blockchain data into structured, type-safe Rust types. The parser ecosystem consists of:

- **Instruction Parsers** - Parse transaction instructions
- **Account Parsers** - Parse account state changes
- **Event Parsers** - Parse program-generated events

### Parser Interface

All parsers implement common traits:

```rust
use yellowstone_vixen_core::*;

pub trait InstructionParser {
    type Instruction;
    type Error;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error>;
}

pub trait AccountParser {
    type Account;
    type Error;

    fn parse(&self, account_info: &AccountInfo) -> Result<Self::Account, Self::Error>;
}
```

## Planning Your Parser

### 1. Understand the Target Program

Before creating a parser, you need:

- **Program ID** - The Solana program you want to parse
- **IDL (Interface Definition Language)** - Program's API definition
- **Instruction Format** - How instructions are structured
- **Account Layout** - How accounts store data
- **Events** - What events the program emits

### 2. Gather Resources

```bash
# Find program ID
solana program show <program-name>

# Get IDL if available
anchor idl fetch <program-id>

# Or examine on-chain program
solana program dump <program-id> program.dump
```

### 3. Analyze Data Structures

Understand the program's data formats:

```rust
// Example: Token Program Account
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate: Option<Pubkey>,
    pub state: AccountState,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<Pubkey>,
}
```

## Creating an Instruction Parser

### Basic Structure

```rust
use yellowstone_vixen_core::*;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub enum MyInstruction {
    Initialize(InitializeData),
    Execute(ExecuteData),
    Close(CloseData),
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct InitializeData {
    pub param1: u64,
    pub param2: Pubkey,
}

pub struct MyProgramParser;

impl InstructionParser for MyProgramParser {
    type Instruction = MyInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        // Implementation
        todo!()
    }
}
```

### Implementation Steps

#### 1. Define Instruction Types

```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub enum MyInstruction {
    Initialize {
        param1: u64,
        param2: Pubkey,
    },
    Execute {
        amount: u64,
        recipient: Pubkey,
    },
    Close,
}
```

#### 2. Implement Parsing Logic

```rust
impl InstructionParser for MyProgramParser {
    type Instruction = MyInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        // Validate program ID
        if instruction.program_id != self.program_id() {
            return Err(ParseError::Filtered);
        }

        // Check minimum instruction size
        if instruction.data.len() < 8 {
            return Err(ParseError::Validation(
                "Instruction data too short".to_string()
            ));
        }

        // Extract discriminator (first 8 bytes)
        let discriminator = u64::from_le_bytes(
            instruction.data[..8].try_into()
                .map_err(|_| ParseError::Validation("Invalid discriminator".to_string()))?
        );

        // Parse based on discriminator
        match discriminator {
            INITIALIZE_DISCRIMINATOR => {
                let data: InitializeData = borsh::from_slice(&instruction.data[8..])
                    .map_err(|e| ParseError::Parsing(e.to_string()))?;
                Ok(MyInstruction::Initialize(data))
            }
            EXECUTE_DISCRIMINATOR => {
                let data: ExecuteData = borsh::from_slice(&instruction.data[8..])
                    .map_err(|e| ParseError::Parsing(e.to_string()))?;
                Ok(MyInstruction::Execute(data))
            }
            CLOSE_DISCRIMINATOR => {
                Ok(MyInstruction::Close)
            }
            _ => Err(ParseError::Filtered)
        }
    }
}
```

#### 3. Add Helper Methods

```rust
impl MyProgramParser {
    pub fn program_id(&self) -> Pubkey {
        "MyProgram111111111111111111111111111".parse().unwrap()
    }

    pub const INITIALIZE_DISCRIMINATOR: u64 = 0x12345678abcdef12;
    pub const EXECUTE_DISCRIMINATOR: u64 = 0x87654321fedcba21;
    pub const CLOSE_DISCRIMINATOR: u64 = 0x1111111122222222;
}
```

## Creating an Account Parser

### Basic Structure

```rust
use yellowstone_vixen_core::*;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct MyAccount {
    pub field1: u64,
    pub field2: Pubkey,
    pub field3: Vec<u8>,
}

pub struct MyAccountParser;

impl AccountParser for MyAccountParser {
    type Account = MyAccount;
    type Error = ParseError;

    fn parse(&self, account_info: &AccountInfo) -> Result<Self::Account, Self::Error> {
        // Implementation
        todo!()
    }
}
```

### Implementation Steps

#### 1. Define Account Structure

```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct MyAccount {
    pub authority: Pubkey,
    pub data: Vec<u8>,
    pub bump: u8,
    pub created_at: i64,
}
```

#### 2. Implement Parsing Logic

```rust
impl AccountParser for MyAccountParser {
    type Account = MyAccount;
    type Error = ParseError;

    fn parse(&self, account_info: &AccountInfo) -> Result<Self::Account, Self::Error> {
        // Validate owner
        if account_info.owner != self.program_id() {
            return Err(ParseError::Filtered);
        }

        // Validate data size
        if account_info.data.len() < std::mem::size_of::<MyAccount>() {
            return Err(ParseError::Validation(
                "Account data too small".to_string()
            ));
        }

        // Deserialize account data
        let account: MyAccount = borsh::from_slice(&account_info.data)
            .map_err(|e| ParseError::Parsing(e.to_string()))?;

        // Additional validation
        if !self.is_valid_account(&account) {
            return Err(ParseError::Validation(
                "Invalid account state".to_string()
            ));
        }

        Ok(account)
    }
}
```

#### 3. Add Validation Methods

```rust
impl MyAccountParser {
    pub fn program_id(&self) -> Pubkey {
        "MyProgram111111111111111111111111111".parse().unwrap()
    }

    pub fn is_valid_account(&self, account: &MyAccount) -> bool {
        // Implement account validation logic
        !account.data.is_empty() && account.bump <= 255
    }
}
```

## Using Codama for Parser Generation

### Automated Parser Generation

For complex programs, use Codama to generate parsers automatically:

#### 1. Set Up Codama Project

```bash
# Create package.json
npm init -y

# Install Codama
npm install @codama/renderers-vixen-parser
```

#### 2. Create Generation Script

```javascript
// codama.cjs
const { rootNodeFromAnchor } = require("@codama/nodes-from-anchor");
const { renderVixenParsers } = require("@codama/renderers-vixen-parser");
const { readJson } = require("@codama/renderers-core");

async function generateParser() {
    // Load IDL
    const idl = readJson('./program.idl');

    // Create root node
    const rootNode = rootNodeFromAnchor(idl);

    // Generate parser
    const output = renderVixenParsers(rootNode, {
        name: 'MyProgramParser',
        crateName: 'my-program-parser',
    });

    // Write to file
    require('fs').writeFileSync('./src/generated.rs', output);
}

generateParser();
```

#### 3. Run Generation

```bash
node codama.cjs
```

### Customizing Generated Parsers

Generated parsers can be customized:

```rust
// Use generated parser as base
use generated::MyProgramParser as GeneratedParser;

// Create wrapper with custom logic
pub struct MyCustomParser {
    generated: GeneratedParser,
    custom_config: CustomConfig,
}

impl InstructionParser for MyCustomParser {
    type Instruction = MyInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        // Pre-processing
        self.validate_instruction(instruction)?;

        // Use generated parser
        let result = self.generated.parse(instruction)?;

        // Post-processing
        self.enrich_instruction(result)
    }
}
```

## Testing Your Parser

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use yellowstone_vixen_core::test_utils::*;

    #[test]
    fn test_parse_initialize_instruction() {
        let parser = MyProgramParser;
        let instruction = create_test_instruction(
            INITIALIZE_DISCRIMINATOR,
            InitializeData {
                param1: 100,
                param2: Pubkey::new_unique(),
            }
        );

        let result = parser.parse(&instruction);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        match parsed {
            MyInstruction::Initialize(data) => {
                assert_eq!(data.param1, 100);
            }
            _ => panic!("Expected Initialize instruction"),
        }
    }

    #[test]
    fn test_parse_invalid_instruction() {
        let parser = MyProgramParser;
        let instruction = create_invalid_instruction();

        let result = parser.parse(&instruction);
        assert!(matches!(result, Err(ParseError::Filtered)));
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use yellowstone_vixen_mock::*;
    use super::*;

    #[tokio::test]
    async fn test_parser_with_mock_data() {
        // Load test fixtures
        let fixtures = FixtureLoader::new("./fixtures")
            .load_transactions("test_txs.json")
            .build();

        // Create mock environment
        let mock_env = MockEnvironment::new(fixtures);

        // Test parser
        let parser = MyProgramParser;
        let results = mock_env.run_parser(parser).await;

        // Verify results
        assert!(!results.is_empty());
        for result in results {
            assert!(result.is_ok());
        }
    }
}
```

### Property-Based Testing

```rust
#[cfg(test)]
mod property_tests {
    use proptest::*;
    use super::*;

    proptest! {
        #[test]
        fn test_parser_doesnt_crash_on_random_data(data in prop::collection::vec(prop::num::u8::ANY, 0..1000)) {
            let parser = MyProgramParser;
            let instruction = Instruction {
                program_id: Pubkey::new_unique(),
                accounts: vec![],
                data,
            };

            // Parser should not panic
            let _ = parser.parse(&instruction);
        }
    }
}
```

## Error Handling

### Comprehensive Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Parser filtered this update")]
    Filtered,

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Parsing failed: {0}")]
    Parsing(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] borsh::io::Error),
}
```

### Error Recovery Strategies

```rust
impl InstructionParser for ResilientParser {
    type Instruction = MyInstruction;
    type Error = ParserError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        match self.try_parse(instruction) {
            Ok(result) => Ok(result),
            Err(ParserError::Parsing(_)) => {
                // Log parsing errors but don't fail completely
                tracing::warn!("Failed to parse instruction: {:?}", instruction);
                Err(ParserError::Filtered)
            }
            Err(e) => Err(e),
        }
    }
}
```

## Performance Optimization

### Efficient Parsing

```rust
impl FastParser {
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

        Self::DISCRIMINATORS.iter()
            .position(|&d| d == discriminator)
    }
}
```

### Memory Management

```rust
impl MemoryEfficientParser {
    // Reuse buffers to avoid allocations
    thread_local! {
        static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    }

    fn parse_with_buffer(&self, data: &[u8]) -> Result<MyInstruction, ParserError> {
        Self::BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            buf.clear();
            buf.extend_from_slice(data);

            // Parse using buffer
            borsh::from_slice(buf)
                .map_err(|e| ParserError::Parsing(e.to_string()))
        })
    }
}
```

## Packaging and Distribution

### Creating a Parser Crate

```toml
# Cargo.toml
[package]
name = "yellowstone-vixen-my-parser"
version = "0.1.0"
edition = "2021"

[dependencies]
yellowstone-vixen-core = { path = "../../crates/core", version = "0.4" }
borsh = "0.10"
solana-program = "1.16"

[dev-dependencies]
yellowstone-vixen-mock = { path = "../../crates/mock", version = "0.4" }
```

### Publishing to Crates.io

```bash
# Test before publishing
cargo test
cargo publish --dry-run

# Publish
cargo publish
```

## Best Practices

### Code Quality

1. **Comprehensive Testing** - Test all code paths and edge cases
2. **Error Handling** - Handle errors gracefully without crashing
3. **Documentation** - Document all public APIs and complex logic
4. **Performance** - Optimize for speed and memory usage
5. **Maintainability** - Write clear, readable code

### Security Considerations

1. **Input Validation** - Validate all inputs thoroughly
2. **Resource Limits** - Prevent resource exhaustion attacks
3. **Access Control** - Respect program ownership and permissions
4. **Data Integrity** - Verify data authenticity and integrity

### Maintenance

1. **Version Compatibility** - Keep up with Solana and Yellowstone updates
2. **Deprecation Notices** - Warn about deprecated features
3. **Migration Guides** - Help users migrate between versions
4. **Community Support** - Engage with users and contributors

## Examples

### Complete Parser Examples

See the `crates/` directory for complete parser implementations:

- **Token Program Parser** - `crates/parser/src/token_program.rs`
- **Jupiter Parser** - `crates/jupiter-swap-parser/`
- **Meteora Parser** - `crates/meteora-parser/`

### Real-World Usage

```rust
use yellowstone_vixen::*;
use yellowstone_vixen_my_parser::MyProgramParser;

// Create pipeline
let pipeline = Pipeline::new(
    MyProgramParser,
    [
        DatabaseHandler::new(database),
        MetricsHandler::new(metrics),
        AlertHandler::new(alerts),
    ]
);

// Use in runtime
let runtime = Runtime::builder()
    .instruction(pipeline)
    .build(config)
    .await?;
```

This guide provides the foundation for creating robust, efficient parsers for Yellowstone Vixen. Remember to test thoroughly and follow best practices for production-ready code.
