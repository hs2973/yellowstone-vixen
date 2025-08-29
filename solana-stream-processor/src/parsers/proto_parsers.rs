use std::sync::Arc;
use std::collections::HashMap;

use async_trait::async_trait;
use yellowstone_vixen_core::{AccountUpdate, InstructionUpdate, ParseResult, Prefilter, Pubkey};
use bson::{doc, Document};
use chrono::Utc;
use tracing::{debug, warn, error};
use uuid::Uuid;

use crate::models::{ParsedData, ParsedAccountData, ParsedInstructionData, TradingInstructionType, ProgramMetadata};

/// Unified parser that handles all supported Solana programs
/// Routes messages to appropriate program-specific parsers and converts to common format
pub struct UnifiedParser {
    program_metadata: HashMap<String, ProgramMetadata>,
}

impl UnifiedParser {
    /// Create a new unified parser with metadata for all supported programs
    pub fn new() -> Self {
        let programs = ProgramMetadata::all_programs();
        let mut program_metadata = HashMap::new();
        
        for program in programs {
            program_metadata.insert(program.program_id.clone(), program);
        }

        Self {
            program_metadata,
        }
    }

    /// Parse account update from any supported program
    pub async fn parse_account_update(&self, update: &AccountUpdate) -> ParseResult<Option<ParsedData>> {
        let account = match &update.account {
            Some(account) => account,
            None => {
                debug!("Account update missing account data");
                return Ok(None);
            }
        };

        let program_id = account.owner.to_string();
        let program_metadata = match self.program_metadata.get(&program_id) {
            Some(metadata) => metadata,
            None => {
                debug!("Unsupported program ID for account: {}", program_id);
                return Ok(None);
            }
        };

        // Parse the account data based on program type
        let parsed_data = self.parse_program_account(&program_id, &account.data).await?;

        let account_data = ParsedAccountData {
            id: Uuid::new_v4().to_string(),
            account_pubkey: update.pubkey.to_string(),
            program_id: program_id.clone(),
            program_name: program_metadata.name.clone(),
            slot: update.slot,
            block_time: None, // Will be filled in by the processor if available
            ingested_at: Utc::now(),
            raw_data: account.data.clone(),
            parsed_data,
            lamports: account.lamports,
            owner: account.owner.to_string(),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            metadata: HashMap::new(),
        };

        Ok(Some(ParsedData::Account(account_data)))
    }

    /// Parse instruction update from any supported program
    pub async fn parse_instruction_update(&self, update: &InstructionUpdate) -> ParseResult<Option<ParsedData>> {
        let instruction = match &update.instruction {
            Some(instruction) => instruction,
            None => {
                debug!("Instruction update missing instruction data");
                return Ok(None);
            }
        };

        let program_id = instruction.program_id.to_string();
        let program_metadata = match self.program_metadata.get(&program_id) {
            Some(metadata) => metadata,
            None => {
                debug!("Unsupported program ID for instruction: {}", program_id);
                return Ok(None);
            }
        };

        // Parse the instruction data based on program type
        let (parsed_data, instruction_type) = self.parse_program_instruction(&program_id, &instruction.data).await?;

        // Determine if this is a trading instruction
        let is_trading = TradingInstructionType::from_instruction_name(&instruction_type).is_some();

        // Extract account keys and token mints
        let accounts: Vec<String> = instruction.accounts.iter().map(|acc| acc.to_string()).collect();
        let token_mints = self.extract_token_mints(&program_id, &parsed_data, &accounts);

        let instruction_data = ParsedInstructionData {
            id: Uuid::new_v4().to_string(),
            signature: update.signature.to_string(),
            program_id: program_id.clone(),
            program_name: program_metadata.name.clone(),
            instruction_index: update.index as u32,
            slot: update.slot,
            block_time: None, // Will be filled in by the processor if available
            ingested_at: Utc::now(),
            raw_data: instruction.data.clone(),
            parsed_data,
            instruction_type,
            accounts,
            is_trading,
            token_mints,
            metadata: HashMap::new(),
        };

        Ok(Some(ParsedData::Instruction(instruction_data)))
    }

    /// Parse account data for a specific program
    async fn parse_program_account(&self, program_id: &str, data: &[u8]) -> ParseResult<Document> {
        match program_id {
            // SPL Token Program
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" => {
                self.parse_spl_token_account(data).await
            }
            // Pump.fun
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" => {
                self.parse_pumpfun_account(data).await
            }
            // Raydium AMM V4
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => {
                self.parse_raydium_amm_v4_account(data).await
            }
            // Add other programs as needed
            _ => {
                warn!("No specific parser for program {}, using generic parser", program_id);
                self.parse_generic_account(data).await
            }
        }
    }

    /// Parse instruction data for a specific program
    async fn parse_program_instruction(&self, program_id: &str, data: &[u8]) -> ParseResult<(Document, String)> {
        match program_id {
            // SPL Token Program
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" => {
                self.parse_spl_token_instruction(data).await
            }
            // Pump.fun
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" => {
                self.parse_pumpfun_instruction(data).await
            }
            // Raydium AMM V4
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => {
                self.parse_raydium_amm_v4_instruction(data).await
            }
            // Add other programs as needed
            _ => {
                warn!("No specific parser for program {}, using generic parser", program_id);
                self.parse_generic_instruction(data).await
            }
        }
    }

    /// Parse SPL Token account data
    async fn parse_spl_token_account(&self, data: &[u8]) -> ParseResult<Document> {
        // This is a simplified parser - in production you would use the actual SPL token parser
        if data.len() < 165 { // SPL token account is 165 bytes
            return Ok(doc! { "type": "invalid", "reason": "insufficient_data" });
        }

        // For demonstration, we'll create a basic structure
        // In production, you would use yellowstone-vixen-parser crate
        Ok(doc! {
            "type": "token_account",
            "mint": hex::encode(&data[0..32]),
            "owner": hex::encode(&data[32..64]),
            "amount": u64::from_le_bytes(data[64..72].try_into().unwrap_or([0; 8])),
            "raw_size": data.len()
        })
    }

    /// Parse Pump.fun account data
    async fn parse_pumpfun_account(&self, data: &[u8]) -> ParseResult<Document> {
        // Placeholder for Pump.fun account parsing
        // In production, you would use yellowstone-vixen-pumpfun-parser
        Ok(doc! {
            "type": "pumpfun_account",
            "data_size": data.len(),
            "raw_data": hex::encode(data)
        })
    }

    /// Parse Raydium AMM V4 account data
    async fn parse_raydium_amm_v4_account(&self, data: &[u8]) -> ParseResult<Document> {
        // Placeholder for Raydium AMM V4 account parsing
        // In production, you would use yellowstone-vixen-raydium-amm-v4-parser
        Ok(doc! {
            "type": "raydium_amm_v4_account",
            "data_size": data.len(),
            "raw_data": hex::encode(data)
        })
    }

    /// Generic account parser for unsupported programs
    async fn parse_generic_account(&self, data: &[u8]) -> ParseResult<Document> {
        Ok(doc! {
            "type": "generic_account",
            "data_size": data.len(),
            "data_hash": sha256::digest(data)
        })
    }

    /// Parse SPL Token instruction data
    async fn parse_spl_token_instruction(&self, data: &[u8]) -> ParseResult<(Document, String)> {
        if data.is_empty() {
            return Ok((doc! { "type": "invalid" }, "unknown".to_string()));
        }

        let instruction_type = match data[0] {
            0 => "initialize_mint",
            1 => "initialize_account", 
            2 => "initialize_multisig",
            3 => "transfer",
            4 => "approve",
            5 => "revoke",
            6 => "set_authority",
            7 => "mint_to",
            8 => "burn",
            9 => "close_account",
            _ => "unknown",
        };

        let parsed = doc! {
            "instruction_id": data[0],
            "type": instruction_type,
            "data_size": data.len()
        };

        Ok((parsed, instruction_type.to_string()))
    }

    /// Parse Pump.fun instruction data
    async fn parse_pumpfun_instruction(&self, data: &[u8]) -> ParseResult<(Document, String)> {
        // Placeholder for Pump.fun instruction parsing
        // In production, you would use yellowstone-vixen-pumpfun-parser
        let instruction_type = if !data.is_empty() {
            match data[0] {
                0 => "create",
                1 => "buy",
                2 => "sell",
                _ => "unknown",
            }
        } else {
            "unknown"
        };

        let parsed = doc! {
            "type": "pumpfun_instruction",
            "instruction_type": instruction_type,
            "data_size": data.len()
        };

        Ok((parsed, instruction_type.to_string()))
    }

    /// Parse Raydium AMM V4 instruction data
    async fn parse_raydium_amm_v4_instruction(&self, data: &[u8]) -> ParseResult<(Document, String)> {
        // Placeholder for Raydium AMM V4 instruction parsing
        // In production, you would use yellowstone-vixen-raydium-amm-v4-parser
        let instruction_type = if !data.is_empty() {
            match data[0] {
                0 => "initialize",
                1 => "swap",
                2 => "add_liquidity",
                3 => "remove_liquidity",
                _ => "unknown",
            }
        } else {
            "unknown"
        };

        let parsed = doc! {
            "type": "raydium_amm_v4_instruction",
            "instruction_type": instruction_type,
            "data_size": data.len()
        };

        Ok((parsed, instruction_type.to_string()))
    }

    /// Generic instruction parser
    async fn parse_generic_instruction(&self, data: &[u8]) -> ParseResult<(Document, String)> {
        let parsed = doc! {
            "type": "generic_instruction",
            "data_size": data.len(),
            "data_hash": sha256::digest(data)
        };

        Ok((parsed, "generic".to_string()))
    }

    /// Extract token mint addresses from parsed data and accounts
    fn extract_token_mints(&self, program_id: &str, parsed_data: &Document, accounts: &[String]) -> Vec<String> {
        let mut token_mints = Vec::new();

        // Extract mints from parsed data
        if let Ok(mint) = parsed_data.get_str("mint") {
            token_mints.push(mint.to_string());
        }

        // For SPL Token program, accounts might contain mint addresses
        if program_id == "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" && accounts.len() > 1 {
            // First account is usually the mint for many SPL token instructions
            token_mints.push(accounts[0].clone());
        }

        // Remove duplicates
        token_mints.sort();
        token_mints.dedup();

        token_mints
    }

    /// Get prefilter for all supported programs
    pub fn get_prefilter(&self) -> Prefilter {
        let program_ids: Vec<Pubkey> = self.program_metadata
            .keys()
            .filter_map(|id| id.parse().ok())
            .collect();

        Prefilter::builder()
            .account_owners(program_ids.clone())
            .build()
            .expect("Failed to build prefilter")
    }
}

impl Default for UnifiedParser {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function for SHA256 hashing
mod sha256 {
    use std::fmt::Write;

    pub fn digest(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("{:x}", hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_parser_creation() {
        let parser = UnifiedParser::new();
        assert!(!parser.program_metadata.is_empty());
        
        // Check that major programs are included
        assert!(parser.program_metadata.contains_key("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"));
        assert!(parser.program_metadata.contains_key("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"));
    }

    #[test]
    fn test_token_mint_extraction() {
        let parser = UnifiedParser::new();
        let parsed_data = doc! { "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" };
        let accounts = vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()];
        
        let mints = parser.extract_token_mints("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb", &parsed_data, &accounts);
        assert!(!mints.is_empty());
        assert!(mints.contains(&"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()));
    }

    #[tokio::test]
    async fn test_spl_token_instruction_parsing() {
        let parser = UnifiedParser::new();
        
        // Test transfer instruction (opcode 3)
        let data = vec![3, 0, 0, 0, 0, 0, 0, 0, 100]; // Transfer 100 tokens
        let (parsed, instruction_type) = parser.parse_spl_token_instruction(&data).await.unwrap();
        
        assert_eq!(instruction_type, "transfer");
        assert_eq!(parsed.get_i32("instruction_id").unwrap(), 3);
    }
}