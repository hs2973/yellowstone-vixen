# Supported Programs

Yellowstone Vixen supports parsing events from a wide range of Solana programs, covering DeFi protocols, NFT marketplaces, gaming platforms, and more. This page provides comprehensive information about all supported programs.

## Overview

| Category | Count | Description |
|----------|-------|-------------|
| DeFi Protocols | 12 | DEXes, lending, yield farming |
| NFT Marketplaces | 3 | NFT trading and marketplaces |
| Gaming | 2 | Gaming and metaverse protocols |
| Specialized | 4 | Staking, launchpads, and utilities |

## DeFi Protocols

### Jupiter Aggregator v6

**Program ID:** `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`

**Parser:** `yellowstone-vixen-jupiter-swap-parser`

**Description:** Jupiter is Solana's leading DEX aggregator, providing optimal swap routing across multiple liquidity sources.

**Supported Instructions:**
- `swap` - Execute token swaps with optimal routing
- `route` - Multi-hop swap routing
- `sharedAccountsRoute` - Shared accounts routing for efficiency
- `routeWithTokenLedger` - Route with token ledger support
- `exactOutRoute` - Exact output amount routing
- `swapWithTokenLedger` - Swap with token ledger

**Key Features:**
- Cross-DEX routing optimization
- MEV protection
- Gasless transactions support
- Real-time price discovery

**Usage Example:**
```rust
use yellowstone_vixen_jupiter_swap_parser::JupiterSwapParser;

let parser = JupiterSwapParser;
```

### Meteora Protocols

Meteora provides several innovative AMM designs for Solana.

#### Meteora DLMM (Dynamic Liquidity Market Maker)

**Program ID:** `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`

**Parser:** `yellowstone-vixen-meteora-parser`

**Description:** Dynamic AMM with concentrated liquidity similar to Uniswap V3.

**Supported Instructions:**
- `initializeLbPair` - Initialize liquidity book pair
- `initializePosition` - Initialize liquidity position
- `addLiquidity` - Add liquidity to position
- `removeLiquidity` - Remove liquidity from position
- `swap` - Execute token swap
- `claimFee` - Claim accumulated fees
- `closePosition` - Close liquidity position
- `initializePositionPda` - Initialize position PDA
- `initializePositionByOperator` - Initialize position by operator
- `updatePositionOperator` - Update position operator

#### Meteora DAMM (Dynamic AMM v2)

**Program ID:** `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`

**Parser:** `yellowstone-vixen-meteora-amm-parser`

**Description:** Advanced AMM with dynamic fee adjustment.

#### Meteora Dynamic Bonding Curve

**Program ID:** `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN`

**Parser:** `yellowstone-vixen-meteora-dbc-parser`

**Description:** Bonding curve AMM for fair token launches.

#### Meteora Pools

**Program ID:** `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB`

**Parser:** `yellowstone-vixen-meteora-pools-parser`

**Description:** Standard AMM pools with enhanced features.

#### Meteora Vault

**Program ID:** `24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi`

**Parser:** `yellowstone-vixen-meteora-vault-parser`

**Description:** Vault system for automated liquidity management.

### Raydium Protocols

Raydium is one of Solana's largest DEXes and AMMs.

#### Raydium AMM v4

**Program ID:** `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`

**Parser:** `yellowstone-vixen-raydium-amm-v4-parser`

**Description:** Raydium's main AMM program for standard liquidity pools.

**Supported Instructions:**
- `swap` - Token swap
- `addLiquidity` - Add liquidity
- `removeLiquidity` - Remove liquidity
- `withdrawPnl` - Withdraw profits and losses

#### Raydium Concentrated Liquidity

**Program ID:** `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`

**Parser:** `yellowstone-vixen-raydium-clmm-parser`

**Description:** Concentrated liquidity AMM similar to Uniswap V3.

#### Raydium CPMM (Constant Product Market Maker)

**Program ID:** `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C`

**Parser:** `yellowstone-vixen-raydium-cpmm-parser`

**Description:** Constant product market maker with enhanced features.

#### Raydium Launchpad

**Program ID:** `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj`

**Parser:** `yellowstone-vixen-raydixen-launchpad-parser`

**Description:** Token launch platform with fair distribution mechanisms.

### Orca Whirlpool

**Program ID:** `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`

**Parser:** `yellowstone-vixen-orca-whirlpool-parser`

**Description:** Orca's concentrated liquidity AMM with innovative features.

**Supported Instructions:**
- `initializePool` - Initialize liquidity pool
- `initializeTickArray` - Initialize tick array
- `initializePosition` - Initialize liquidity position
- `openPosition` - Open liquidity position
- `closePosition` - Close liquidity position
- `swap` - Execute token swap
- `collectFees` - Collect accumulated fees
- `increaseLiquidity` - Increase liquidity in position
- `decreaseLiquidity` - Decrease liquidity in position

**Key Features:**
- Concentrated liquidity
- Multiple fee tiers
- Protocol fees
- Position management

## NFT and Gaming Protocols

### Pump.fun Protocols

Pump.fun is a viral meme coin and NFT launch platform.

#### Pump.fun AMM

**Program ID:** `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`

**Parser:** `yellowstone-vixen-pump-swaps-parser`

**Description:** AMM for Pump.fun tokens with bonding curve mechanics.

#### Pump.fun Main

**Program ID:** `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`

**Parser:** `yellowstone-vixen-pumpfun-parser`

**Description:** Main Pump.fun program for token launches and trading.

### Moonshot

**Program ID:** `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG`

**Parser:** `yellowstone-vixen-moonshot-parser`

**Description:** Moonshot is a DEX aggregator focused on low-slippage trades.

### Boop.fun

**Program ID:** `boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4`

**Parser:** `yellowstone-vixen-boop-parser`

**Description:** Boop.fun is a gaming and NFT platform on Solana.

## Specialized Protocols

### Kamino Limit Orders

**Program ID:** `LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF`

**Parser:** `yellowstone-vixen-kamino-limit-orders-parser`

**Description:** Kamino provides advanced limit order functionality for Solana.

### Virtuals

**Program ID:** `5U3EU2ubXtK84QcRjWVmYt9RaDyA8gKxdUrPFXmZyaki`

**Parser:** `yellowstone-vixen-virtuals-parser`

**Description:** Virtuals protocol for virtual assets and gaming.

## Core Solana Programs

### SPL Token Program

**Program ID:** `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`

**Parser:** `yellowstone-vixen-parser`

**Description:** Solana's standard token program for SPL tokens.

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
- `transferChecked` - Transfer with amount validation
- `approveChecked` - Approve with amount validation
- `mintToChecked` - Mint with amount validation
- `burnChecked` - Burn with amount validation

### Associated Token Program

**Program ID:** `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`

**Parser:** `yellowstone-vixen-parser`

**Description:** Program for creating associated token accounts.

**Supported Instructions:**
- `create` - Create associated token account
- `createIdempotent` - Create associated token account if it doesn't exist
- `recoverNested` - Recover lamports from nested associated token accounts

## Program Information Table

| Program | Address | Parser | Category | Status |
|---------|---------|--------|----------|--------|
| Jupiter Aggregator v6 | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | jupiter-swap-parser | DeFi | ✅ Active |
| Meteora DLMM | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | meteora-parser | DeFi | ✅ Active |
| Meteora DAMM v2 | `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG` | meteora-amm-parser | DeFi | ✅ Active |
| Meteora DBC | `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN` | meteora-dbc-parser | DeFi | ✅ Active |
| Meteora Pools | `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB` | meteora-pools-parser | DeFi | ✅ Active |
| Meteora Vault | `24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi` | meteora-vault-parser | DeFi | ✅ Active |
| Raydium AMM v4 | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` | raydium-amm-v4-parser | DeFi | ✅ Active |
| Raydium CLMM | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` | raydium-clmm-parser | DeFi | ✅ Active |
| Raydium CPMM | `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C` | raydium-cpmm-parser | DeFi | ✅ Active |
| Raydium Launchpad | `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj` | raydium-launchpad-parser | DeFi | ✅ Active |
| Orca Whirlpools | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` | orca-whirlpool-parser | DeFi | ✅ Active |
| Pump.fun AMM | `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` | pump-swaps-parser | NFT/Gaming | ✅ Active |
| Pump.fun | `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P` | pumpfun-parser | NFT/Gaming | ✅ Active |
| Moonshot | `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG` | moonshot-parser | DeFi | ✅ Active |
| Boop.fun | `boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4` | boop-parser | NFT/Gaming | ✅ Active |
| Kamino Limit Orders | `LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF` | kamino-limit-orders-parser | DeFi | ✅ Active |
| Virtuals | `5U3EU2ubXtK84QcRjWVmYt9RaDyA8gKxdUrPFXmZyaki` | virtuals-parser | Gaming | ✅ Active |
| SPL Token | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | parser | Core | ✅ Active |
| Associated Token | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | parser | Core | ✅ Active |

## Adding New Programs

To add support for a new program:

1. **Identify the Program** - Get the program ID and understand its functionality
2. **Obtain IDL** - Get the program's interface definition
3. **Create Parser** - Use Codama or manual implementation
4. **Add Tests** - Comprehensive test coverage
5. **Submit PR** - Contribute back to the project

See [Creating Parsers](../development/creating-parsers.md) for detailed instructions.

## Program Health Status

### Active Programs ✅
These programs are actively maintained and supported:
- Jupiter Aggregator v6
- All Meteora protocols
- Raydium protocols
- Orca Whirlpool
- SPL Token programs

### Beta Programs 🧪
These programs have parsers but may need additional testing:
- Pump.fun protocols
- Moonshot
- Boop.fun

### Deprecated Programs ⚠️
These programs are no longer actively developed:
- None currently

## Performance Characteristics

| Program Category | Typical Throughput | Memory Usage | Notes |
|------------------|-------------------|--------------|-------|
| DeFi Protocols | 10,000-50,000 tx/s | Medium | Complex parsing logic |
| NFT Marketplaces | 5,000-20,000 tx/s | Low | Simple event structures |
| Core Programs | 50,000+ tx/s | Low | Optimized parsing |
| Gaming Protocols | 2,000-10,000 tx/s | High | Complex state management |

## Getting Help

### Program-Specific Issues

If you encounter issues with a specific program:

1. **Check Program Status** - Verify the program is still active
2. **Review Recent Changes** - Check for program updates
3. **Test with Mock Data** - Use mock testing to isolate issues
4. **Open Issue** - Report problems on GitHub

### Adding New Programs

To request support for a new program:

1. **Gather Information** - Program ID, IDL, documentation
2. **Create Issue** - Use the "New Program Request" template
3. **Provide Examples** - Sample transactions and expected output
4. **Community Support** - Get community feedback and contributions

## Contributing

We welcome contributions for new program support! See our [Contributing Guide](../development/contributing.md) for details on:

- Adding new parsers
- Improving existing parsers
- Testing and validation
- Documentation updates

---

*Last updated: August 2025. Program support is continuously evolving. Check the [GitHub repository](https://github.com/rpcpool/yellowstone-vixen) for the latest updates.*
