use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bson::{Document, doc};
use std::collections::HashMap;

/// Common structure for all parsed account data
/// Stores account updates from various Solana programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAccountData {
    /// Unique identifier for this record
    pub id: String,
    /// Solana account public key
    pub account_pubkey: String,
    /// Program ID that owns this account
    pub program_id: String,
    /// Program name for easy identification
    pub program_name: String,
    /// Blockchain slot number
    pub slot: u64,
    /// Block timestamp
    pub block_time: Option<DateTime<Utc>>,
    /// Ingestion timestamp when we processed this
    pub ingested_at: DateTime<Utc>,
    /// Raw account data
    pub raw_data: Vec<u8>,
    /// Parsed data specific to the program
    pub parsed_data: Document,
    /// Account lamports (SOL balance)
    pub lamports: u64,
    /// Account owner program ID
    pub owner: String,
    /// Account executable flag
    pub executable: bool,
    /// Account rent epoch
    pub rent_epoch: u64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Common structure for all parsed instruction data
/// Stores transaction instructions from various Solana programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedInstructionData {
    /// Unique identifier for this record
    pub id: String,
    /// Transaction signature
    pub signature: String,
    /// Program ID that executed this instruction
    pub program_id: String,
    /// Program name for easy identification
    pub program_name: String,
    /// Instruction index within the transaction
    pub instruction_index: u32,
    /// Blockchain slot number
    pub slot: u64,
    /// Block timestamp
    pub block_time: Option<DateTime<Utc>>,
    /// Ingestion timestamp when we processed this
    pub ingested_at: DateTime<Utc>,
    /// Raw instruction data
    pub raw_data: Vec<u8>,
    /// Parsed instruction data specific to the program
    pub parsed_data: Document,
    /// Instruction type/name
    pub instruction_type: String,
    /// Account keys involved in this instruction
    pub accounts: Vec<String>,
    /// Whether this instruction is trading-related
    pub is_trading: bool,
    /// Token mint addresses involved (if applicable)
    pub token_mints: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Enum representing the different types of parsed data we handle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParsedData {
    #[serde(rename = "account")]
    Account(ParsedAccountData),
    #[serde(rename = "instruction")]
    Instruction(ParsedInstructionData),
}

impl ParsedData {
    /// Get the program name from the parsed data
    pub fn program_name(&self) -> &str {
        match self {
            ParsedData::Account(data) => &data.program_name,
            ParsedData::Instruction(data) => &data.program_name,
        }
    }

    /// Get the slot number from the parsed data
    pub fn slot(&self) -> u64 {
        match self {
            ParsedData::Account(data) => data.slot,
            ParsedData::Instruction(data) => data.slot,
        }
    }

    /// Get the ingestion timestamp
    pub fn ingested_at(&self) -> DateTime<Utc> {
        match self {
            ParsedData::Account(data) => data.ingested_at,
            ParsedData::Instruction(data) => data.ingested_at,
        }
    }

    /// Convert to BSON document for MongoDB storage
    pub fn to_bson_document(&self) -> Result<Document, bson::ser::Error> {
        bson::to_document(self)
    }

    /// Get collection name for MongoDB storage
    pub fn collection_name(&self) -> String {
        let program = self.program_name().to_lowercase().replace(['.', ' ', '-'], "_");
        match self {
            ParsedData::Account(_) => format!("{}_accounts", program),
            ParsedData::Instruction(_) => format!("{}_instructions", program),
        }
    }
}

/// Trading-related instruction types that we filter for
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TradingInstructionType {
    Buy,
    Sell,
    Swap,
    Create,
    Initialize,
    AddLiquidity,
    RemoveLiquidity,
    Trade,
    Launch,
    Migrate,
}

impl TradingInstructionType {
    /// Check if an instruction name matches any trading types
    pub fn from_instruction_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            s if s.contains("buy") => Some(Self::Buy),
            s if s.contains("sell") => Some(Self::Sell),
            s if s.contains("swap") => Some(Self::Swap),
            s if s.contains("create") => Some(Self::Create),
            s if s.contains("initialize") || s.contains("init") => Some(Self::Initialize),
            s if s.contains("add") && s.contains("liquidity") => Some(Self::AddLiquidity),
            s if s.contains("remove") && s.contains("liquidity") => Some(Self::RemoveLiquidity),
            s if s.contains("trade") => Some(Self::Trade),
            s if s.contains("launch") => Some(Self::Launch),
            s if s.contains("migrate") => Some(Self::Migrate),
            _ => None,
        }
    }

    /// Get all trading instruction types as strings
    pub fn all_types() -> Vec<&'static str> {
        vec![
            "buy", "sell", "swap", "create", "initialize", "init", 
            "add_liquidity", "remove_liquidity", "trade", "launch", "migrate"
        ]
    }
}

/// Program metadata for organizing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    /// Program ID
    pub program_id: String,
    /// Human-readable program name
    pub name: String,
    /// Program description
    pub description: String,
    /// Supported instruction types
    pub instruction_types: Vec<String>,
    /// Whether this program supports trading operations
    pub supports_trading: bool,
    /// Account types this program manages
    pub account_types: Vec<String>,
}

impl ProgramMetadata {
    /// Get metadata for all supported programs
    pub fn all_programs() -> Vec<Self> {
        vec![
            Self {
                program_id: "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".to_string(),
                name: "SPL Token Program".to_string(),
                description: "Standard SPL token operations".to_string(),
                instruction_types: vec!["transfer", "mint", "burn", "approve"].into_iter().map(String::from).collect(),
                supports_trading: false,
                account_types: vec!["token_account", "mint"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(),
                name: "Pump.fun".to_string(),
                description: "Pump.fun token launching and trading".to_string(),
                instruction_types: vec!["create", "buy", "sell", "withdraw"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["bonding_curve", "global_pool"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg".to_string(),
                name: "Pump.fun AMM".to_string(),
                description: "Pump.fun automated market maker for swaps".to_string(),
                instruction_types: vec!["swap", "create_pool"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["pool", "swap_state"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "6bFQpPjNLsG8xRSPJ1KJSYwE7vfzC8hPxWVNozEqYLR".to_string(),
                name: "Raydium Launchpad".to_string(),
                description: "Raydium token launchpad".to_string(),
                instruction_types: vec!["launch", "contribute"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["launchpad", "contribution"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
                name: "Raydium Liquidity Pool V4".to_string(),
                description: "Raydium AMM V4 liquidity pools".to_string(),
                instruction_types: vec!["swap", "add_liquidity", "remove_liquidity", "initialize"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["pool_state", "target_orders"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "BaUEhvUqKhw8eZ1Zb93rjE5Lxj5n1jv2brT8cEa4Y9M".to_string(),
                name: "Boop.fun".to_string(),
                description: "Boop.fun platform".to_string(),
                instruction_types: vec!["create", "trade"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["boop_account"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG".to_string(),
                name: "Moonshot".to_string(),
                description: "Moonshot trading platform".to_string(),
                instruction_types: vec!["buy", "sell", "create"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["moonshot_pool"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_string(),
                name: "Meteora DLMM".to_string(),
                description: "Meteora Dynamic Liquidity Market Maker".to_string(),
                instruction_types: vec!["swap", "add_liquidity", "remove_liquidity"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["lb_pair", "bin"].into_iter().map(String::from).collect(),
            },
            Self {
                program_id: "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB".to_string(),
                name: "Meteora DAMM v2".to_string(),
                description: "Meteora Dynamic AMM v2".to_string(),
                instruction_types: vec!["swap", "add_liquidity", "remove_liquidity"].into_iter().map(String::from).collect(),
                supports_trading: true,
                account_types: vec!["pool", "vault"].into_iter().map(String::from).collect(),
            },
        ]
    }

    /// Find program metadata by program ID
    pub fn find_by_program_id(program_id: &str) -> Option<Self> {
        Self::all_programs().into_iter().find(|p| p.program_id == program_id)
    }

    /// Get all trading-enabled programs
    pub fn trading_programs() -> Vec<Self> {
        Self::all_programs().into_iter().filter(|p| p.supports_trading).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_instruction_type_detection() {
        assert_eq!(TradingInstructionType::from_instruction_name("buy_tokens"), Some(TradingInstructionType::Buy));
        assert_eq!(TradingInstructionType::from_instruction_name("sell_token"), Some(TradingInstructionType::Sell));
        assert_eq!(TradingInstructionType::from_instruction_name("swap_exact_tokens"), Some(TradingInstructionType::Swap));
        assert_eq!(TradingInstructionType::from_instruction_name("unknown_instruction"), None);
    }

    #[test]
    fn test_program_metadata() {
        let programs = ProgramMetadata::all_programs();
        assert!(!programs.is_empty());
        
        let trading_programs = ProgramMetadata::trading_programs();
        assert!(trading_programs.len() > 0);
        assert!(trading_programs.iter().all(|p| p.supports_trading));
    }

    #[test]
    fn test_parsed_data_collection_names() {
        let account_data = ParsedData::Account(ParsedAccountData {
            id: "test".to_string(),
            account_pubkey: "test".to_string(),
            program_id: "test".to_string(),
            program_name: "Pump.fun".to_string(),
            slot: 123,
            block_time: None,
            ingested_at: Utc::now(),
            raw_data: vec![],
            parsed_data: doc! {},
            lamports: 0,
            owner: "test".to_string(),
            executable: false,
            rent_epoch: 0,
            metadata: HashMap::new(),
        });

        assert_eq!(account_data.collection_name(), "pump_fun_accounts");
        assert_eq!(account_data.program_name(), "Pump.fun");
    }
}