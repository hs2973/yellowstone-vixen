# Parsers API

This reference documents all available parsers in Yellowstone Vixen, their capabilities, and usage examples.

## Core Parser Traits

### InstructionParser

The core trait for parsing Solana transaction instructions:

```rust
pub trait InstructionParser {
    type Instruction;
    type Error;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error>;
}
```

**Parameters:**
- `instruction`: Raw Solana instruction data
- Returns: Parsed instruction or error

**Error Handling:**
- `ParseError::Filtered`: Skip processing this instruction
- `ParseError::Parsing(String)`: Parsing failed with details
- `ParseError::Validation(String)`: Validation failed

### AccountParser

The core trait for parsing Solana account state:

```rust
pub trait AccountParser {
    type Account;
    type Error;

    fn parse(&self, account_info: &AccountInfo) -> Result<Self::Account, Self::Error>;
}
```

**Parameters:**
- `account_info`: Raw account data and metadata
- Returns: Parsed account state or error

## Supported Program Parsers

### DeFi Protocol Parsers

#### Jupiter Swap Parser

**Program ID:** `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`

**Crate:** `yellowstone-vixen-jupiter-swap-parser`

**Supported Instructions:**
- `swap` - Token swap execution
- `route` - Multi-hop swap routing
- `sharedAccountsRoute` - Shared accounts routing
- `routeWithTokenLedger` - Route with token ledger
- `exactOutRoute` - Exact output routing

**Usage:**
```rust
use yellowstone_vixen_jupiter_swap_parser::JupiterSwapParser;
use yellowstone_vixen::Pipeline;

let pipeline = Pipeline::new(
    JupiterSwapParser,
    [DatabaseHandler::new()]
);
```

**Parsed Types:**
```rust
pub enum JupiterInstruction {
    Swap(SwapInstruction),
    Route(RouteInstruction),
    // ... other variants
}

pub struct SwapInstruction {
    pub token_program: Pubkey,
    pub user_transfer_authority: Pubkey,
    pub source_token_account: Pubkey,
    pub destination_token_account: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
    pub platform_fee_account: Option<Pubkey>,
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
}
```

#### Meteora Parsers

##### Meteora DLMM Parser

**Program ID:** `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`

**Crate:** `yellowstone-vixen-meteora-parser`

**Supported Instructions:**
- `initializeLbPair` - Initialize liquidity book pair
- `initializePosition` - Initialize liquidity position
- `addLiquidity` - Add liquidity to position
- `removeLiquidity` - Remove liquidity from position
- `swap` - Execute token swap
- `claimFee` - Claim accumulated fees
- `closePosition` - Close liquidity position

##### Meteora DAMM Parser

**Program ID:** `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`

**Crate:** `yellowstone-vixen-meteora-amm-parser`

##### Meteora Dynamic Bonding Curve Parser

**Program ID:** `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN`

**Crate:** `yellowstone-vixen-meteora-dbc-parser`

##### Meteora Pools Parser

**Program ID:** `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB`

**Crate:** `yellowstone-vixen-meteora-pools-parser`

##### Meteora Vault Parser

**Program ID:** `24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi`

**Crate:** `yellowstone-vixen-meteora-vault-parser`

#### Raydium Parsers

##### Raydium AMM v4 Parser

**Program ID:** `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`

**Crate:** `yellowstone-vixen-raydium-amm-v4-parser`

**Supported Instructions:**
- `swap` - Token swap
- `addLiquidity` - Add liquidity
- `removeLiquidity` - Remove liquidity
- `withdrawPnl` - Withdraw profits and losses

##### Raydium Concentrated Liquidity Parser

**Program ID:** `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`

**Crate:** `yellowstone-vixen-raydium-clmm-parser`

##### Raydium CPMM Parser

**Program ID:** `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C`

**Crate:** `yellowstone-vixen-raydium-cpmm-parser`

##### Raydium Launchpad Parser

**Program ID:** `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj`

**Crate:** `yellowstone-vixen-raydium-launchpad-parser`

#### Orca Whirlpool Parser

**Program ID:** `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`

**Crate:** `yellowstone-vixen-orca-whirlpool-parser`

**Supported Instructions:**
- `initializePool` - Initialize liquidity pool
- `initializeTickArray` - Initialize tick array
- `initializePosition` - Initialize liquidity position
- `openPosition` - Open liquidity position
- `closePosition` - Close liquidity position
- `swap` - Execute token swap
- `collectFees` - Collect accumulated fees

### Token Program Parsers

#### SPL Token Parser

**Program ID:** `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`

**Crate:** `yellowstone-vixen-parser`

**Supported Instructions:**
- `initializeMint` - Initialize token mint
- `initializeAccount` - Initialize token account
- `transfer` - Transfer tokens
- `approve` - Approve token delegation
- `revoke` - Revoke token delegation
- `setAuthority` - Set account authority
- `mintTo` - Mint tokens
- `burn` - Burn tokens
- `closeAccount` - Close token account
- `freezeAccount` - Freeze token account
- `thawAccount` - Thaw token account

**Parsed Types:**
```rust
pub enum TokenInstruction {
    InitializeMint(InitializeMintData),
    InitializeAccount(InitializeAccountData),
    Transfer(TransferData),
    // ... other variants
}

pub struct TransferData {
    pub amount: u64,
}
```

#### Associated Token Program Parser

**Program ID:** `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`

**Crate:** `yellowstone-vixen-parser`

**Supported Instructions:**
- `create` - Create associated token account
- `createIdempotent` - Create associated token account if it doesn't exist
- `recoverNested` - Recover lamports from nested associated token accounts

### NFT and Gaming Parsers

#### Pump.fun Parsers

##### Pump.fun AMM Parser

**Program ID:** `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`

**Crate:** `yellowstone-vixen-pump-swaps-parser`

##### Pump.fun Main Parser

**Program ID:** `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`

**Crate:** `yellowstone-vixen-pumpfun-parser`

#### Moonshot Parser

**Program ID:** `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG`

**Crate:** `yellowstone-vixen-moonshot-parser`

#### Boop.fun Parser

**Program ID:** `boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4`

**Crate:** `yellowstone-vixen-boop-parser`

### Specialized Parsers

#### Kamino Limit Orders Parser

**Program ID:** `LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF`

**Crate:** `yellowstone-vixen-kamino-limit-orders-parser`

#### Virtuals Parser

**Program ID:** `5U3EU2ubXtK84QcRjWVmYt9RaDyA8gKxdUrPFXmZyaki`

**Crate:** `yellowstone-vixen-virtuals-parser`

## Parser Usage Patterns

### Basic Usage

```rust
use yellowstone_vixen_jupiter_swap_parser::JupiterSwapParser;
use yellowstone_vixen::Pipeline;

// Create parser instance
let parser = JupiterSwapParser;

// Use in pipeline
let pipeline = Pipeline::new(parser, [handler]);
```

### Multiple Parsers

```rust
use yellowstone_vixen::{Pipeline, Runtime};

// Create multiple pipelines
let jupiter_pipeline = Pipeline::new(JupiterSwapParser, [jupiter_handler]);
let meteora_pipeline = Pipeline::new(MeteoraParser, [meteora_handler]);
let raydium_pipeline = Pipeline::new(RaydiumParser, [raydium_handler]);

// Add to runtime
let runtime = Runtime::builder()
    .instruction(jupiter_pipeline)
    .instruction(meteora_pipeline)
    .instruction(raydium_pipeline)
    .build(config)
    .await?;
```

### Custom Parser Integration

```rust
use yellowstone_vixen_core::*;

// Implement custom parser
pub struct MyCustomParser;

impl InstructionParser for MyCustomParser {
    type Instruction = MyInstruction;
    type Error = ParseError;

    fn parse(&self, instruction: &Instruction) -> Result<Self::Instruction, Self::Error> {
        // Custom parsing logic
        todo!()
    }
}

// Use in pipeline
let pipeline = Pipeline::new(MyCustomParser, [handler]);
```

## Parser Configuration

### Program-Specific Configuration

Some parsers support configuration options:

```toml
[[programs]]
name = "Jupiter"
address = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"

[programs.config]
# Filter specific instruction types
instructions = ["swap", "route"]
# Minimum swap amount
min_swap_amount = 1000000
```

### Parser Options

```rust
// Configure parser behavior
let parser = JupiterSwapParser::new()
    .with_min_amount(1000000)
    .with_allowed_instructions(vec!["swap", "route"])
    .with_validation_level(ValidationLevel::Strict);
```

## Error Handling

### Parser Errors

```rust
match parser.parse(&instruction) {
    Ok(parsed) => {
        // Handle successful parsing
        process_instruction(parsed).await?;
    }
    Err(ParseError::Filtered) => {
        // Instruction was filtered out (normal)
        tracing::debug!("Instruction filtered");
    }
    Err(ParseError::Parsing(msg)) => {
        // Parsing error
        tracing::error!("Parsing failed: {}", msg);
    }
    Err(ParseError::Validation(msg)) => {
        // Validation error
        tracing::warn!("Validation failed: {}", msg);
    }
}
```

### Error Recovery

```rust
impl ResilientParser {
    pub fn parse_with_fallback(&self, instruction: &Instruction) -> Result<MyInstruction, ParseError> {
        match self.parse(instruction) {
            Ok(result) => Ok(result),
            Err(ParseError::Parsing(_)) => {
                // Try alternative parsing method
                self.fallback_parse(instruction)
            }
            Err(e) => Err(e),
        }
    }
}
```

## Performance Characteristics

### Parser Performance

| Parser | Instructions/sec | Memory Usage | Notes |
|--------|------------------|--------------|-------|
| Jupiter Swap | ~50,000 | Low | Simple instruction structure |
| Meteora DLMM | ~30,000 | Medium | Complex account structures |
| Raydium CLMM | ~40,000 | Medium | Mathematical computations |
| SPL Token | ~100,000 | Low | Well-optimized |

### Optimization Tips

1. **Batch Processing** - Process multiple instructions together
2. **Memory Pooling** - Reuse allocated objects
3. **Lazy Parsing** - Parse only required fields initially
4. **Caching** - Cache frequently accessed data

## Testing Parsers

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use yellowstone_vixen_core::test_utils::*;

    #[test]
    fn test_parse_swap_instruction() {
        let parser = JupiterSwapParser;
        let instruction = create_test_swap_instruction();

        let result = parser.parse(&instruction);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        match parsed {
            JupiterInstruction::Swap(swap) => {
                assert_eq!(swap.amount, 1000000);
            }
            _ => panic!("Expected swap instruction"),
        }
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use yellowstone_vixen_mock::*;
    use super::*;

    #[tokio::test]
    async fn test_parser_with_fixtures() {
        let fixtures = FixtureLoader::new("./fixtures")
            .load_transactions("jupiter_txs.json")
            .build();

        let mock_env = MockEnvironment::new(fixtures);
        let parser = JupiterSwapParser;

        let results = mock_env.run_parser(parser).await;

        assert!(!results.is_empty());
        for result in results {
            assert!(result.is_ok());
        }
    }
}
```

## Extending Parsers

### Adding Custom Validation

```rust
impl ExtendedJupiterParser {
    pub fn parse_with_validation(&self, instruction: &Instruction) -> Result<JupiterInstruction, ParseError> {
        let parsed = self.base.parse(instruction)?;

        // Add custom validation
        match &parsed {
            JupiterInstruction::Swap(swap) => {
                if swap.amount == 0 {
                    return Err(ParseError::Validation("Swap amount cannot be zero".to_string()));
                }
            }
            _ => {}
        }

        Ok(parsed)
    }
}
```

### Adding Enrichment

```rust
impl EnrichedJupiterParser {
    pub async fn parse_with_enrichment(&self, instruction: &Instruction) -> Result<EnrichedJupiterInstruction, ParseError> {
        let parsed = self.base.parse(instruction)?;

        // Add enrichment data
        let enriched = match parsed {
            JupiterInstruction::Swap(swap) => {
                let price = self.get_token_price(swap.source_mint, swap.destination_mint).await?;
                EnrichedJupiterInstruction::Swap(EnrichedSwap {
                    base: swap,
                    estimated_price: price,
                    timestamp: get_current_timestamp(),
                })
            }
            // ... other cases
        };

        Ok(enriched)
    }
}
```

## Migration Guide

### Upgrading Parser Versions

When upgrading parser versions:

1. **Check Breaking Changes** - Review changelog for breaking changes
2. **Update Dependencies** - Update Cargo.toml with new versions
3. **Test Thoroughly** - Run full test suite with new version
4. **Update Configuration** - Update config files if needed
5. **Monitor Performance** - Check for performance regressions

### Migrating from Custom Parsers

```rust
// Before (custom parser)
impl InstructionParser for MyCustomParser {
    // Custom implementation
}

// After (using library parser)
use yellowstone_vixen_jupiter_swap_parser::JupiterSwapParser;

// Use library parser directly
let parser = JupiterSwapParser;

// Or wrap with custom logic
pub struct MyJupiterParser {
    base: JupiterSwapParser,
    custom_config: CustomConfig,
}
```

This comprehensive API reference provides everything needed to effectively use Yellowstone Vixen's parser ecosystem.
