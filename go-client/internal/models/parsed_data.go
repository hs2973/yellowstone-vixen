// Package models contains data structures for Solana blockchain data
// These models mirror the Rust ParsedData structures from the stream processor
package models

import (
	"time"
)

// ParsedData represents parsed blockchain data from Solana programs
type ParsedData struct {
	Type string `json:"type"`
	// Account or Instruction will be populated based on Type
	Account     *ParsedAccountData     `json:"Account,omitempty"`
	Instruction *ParsedInstructionData `json:"Instruction,omitempty"`
}

// ParsedAccountData represents parsed account data from Solana programs
type ParsedAccountData struct {
	ID           string                 `json:"id"`
	AccountPubkey string                `json:"account_pubkey"`
	ProgramID    string                 `json:"program_id"`
	ProgramName  string                 `json:"program_name"`
	Slot         uint64                 `json:"slot"`
	BlockTime    *time.Time             `json:"block_time,omitempty"`
	IngestedAt   time.Time              `json:"ingested_at"`
	RawData      []byte                 `json:"raw_data"`
	ParsedData   map[string]interface{} `json:"parsed_data"`
	Lamports     uint64                 `json:"lamports"`
	Owner        string                 `json:"owner"`
	Executable   bool                   `json:"executable"`
	RentEpoch    uint64                 `json:"rent_epoch"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// ParsedInstructionData represents parsed instruction data from Solana programs
type ParsedInstructionData struct {
	ID              string                 `json:"id"`
	Signature       string                 `json:"signature"`
	ProgramID       string                 `json:"program_id"`
	ProgramName     string                 `json:"program_name"`
	InstructionIndex uint32                `json:"instruction_index"`
	Slot            uint64                 `json:"slot"`
	BlockTime       *time.Time             `json:"block_time,omitempty"`
	IngestedAt      time.Time              `json:"ingested_at"`
	RawData         []byte                 `json:"raw_data"`
	ParsedData      map[string]interface{} `json:"parsed_data"`
	InstructionType string                 `json:"instruction_type"`
	Accounts        []string               `json:"accounts"`
	IsTrading       bool                   `json:"is_trading"`
	TokenMints      []string               `json:"token_mints"`
	Metadata        map[string]interface{} `json:"metadata"`
}

// ProgramMetadata represents metadata about supported Solana programs
type ProgramMetadata struct {
	ProgramID        string   `json:"program_id"`
	Name             string   `json:"name"`
	Description      string   `json:"description"`
	InstructionTypes []string `json:"instruction_types"`
	SupportsTrading  bool     `json:"supports_trading"`
	AccountTypes     []string `json:"account_types"`
}

// APIResponse represents the standard API response format
type APIResponse struct {
	Success bool        `json:"success"`
	Data    interface{} `json:"data,omitempty"`
	Error   *string     `json:"error,omitempty"`
	Count   *int        `json:"count,omitempty"`
}

// HealthStatus represents the health check response
type HealthStatus struct {
	Status    string    `json:"status"`
	MongoDB   bool      `json:"mongodb"`
	Timestamp time.Time `json:"timestamp"`
	Version   string    `json:"version"`
}

// RecentDataQuery represents query parameters for recent data endpoint
type RecentDataQuery struct {
	Program        string `url:"program,omitempty"`
	TokenMint      string `url:"token_mint,omitempty"`
	Limit          int    `url:"limit,omitempty"`
	CollectionType string `url:"collection_type,omitempty"`
}

// SlotRangeQuery represents query parameters for slot range queries
type SlotRangeQuery struct {
	MinSlot uint64 `url:"min_slot"`
	MaxSlot uint64 `url:"max_slot"`
	Limit   int    `url:"limit,omitempty"`
}

// TradingInstructionType represents different types of trading instructions
type TradingInstructionType string

const (
	TradingBuy            TradingInstructionType = "buy"
	TradingSell           TradingInstructionType = "sell"
	TradingSwap           TradingInstructionType = "swap"
	TradingCreate         TradingInstructionType = "create"
	TradingInitialize     TradingInstructionType = "initialize"
	TradingAddLiquidity   TradingInstructionType = "add_liquidity"
	TradingRemoveLiquidity TradingInstructionType = "remove_liquidity"
	TradingTrade          TradingInstructionType = "trade"
	TradingLaunch         TradingInstructionType = "launch"
	TradingMigrate        TradingInstructionType = "migrate"
)

// SupportedPrograms returns metadata for all supported Solana programs
func SupportedPrograms() []ProgramMetadata {
	return []ProgramMetadata{
		{
			ProgramID:        "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
			Name:             "SPL Token Program",
			Description:      "Standard SPL token operations",
			InstructionTypes: []string{"transfer", "mint", "burn", "approve"},
			SupportsTrading:  false,
			AccountTypes:     []string{"token_account", "mint"},
		},
		{
			ProgramID:        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
			Name:             "Pump.fun",
			Description:      "Pump.fun token launching and trading",
			InstructionTypes: []string{"create", "buy", "sell", "withdraw"},
			SupportsTrading:  true,
			AccountTypes:     []string{"bonding_curve", "global_pool"},
		},
		{
			ProgramID:        "39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg",
			Name:             "Pump.fun AMM",
			Description:      "Pump.fun automated market maker for swaps",
			InstructionTypes: []string{"swap", "create_pool"},
			SupportsTrading:  true,
			AccountTypes:     []string{"pool", "swap_state"},
		},
		{
			ProgramID:        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
			Name:             "Raydium Liquidity Pool V4",
			Description:      "Raydium AMM V4 liquidity pools",
			InstructionTypes: []string{"swap", "add_liquidity", "remove_liquidity", "initialize"},
			SupportsTrading:  true,
			AccountTypes:     []string{"pool_state", "target_orders"},
		},
		{
			ProgramID:        "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo",
			Name:             "Meteora DLMM",
			Description:      "Meteora Dynamic Liquidity Market Maker",
			InstructionTypes: []string{"swap", "add_liquidity", "remove_liquidity"},
			SupportsTrading:  true,
			AccountTypes:     []string{"lb_pair", "bin"},
		},
	}
}

// IsTradingInstruction checks if an instruction type is trading-related
func IsTradingInstruction(instructionType string) bool {
	tradingTypes := map[string]bool{
		"buy":             true,
		"sell":            true,
		"swap":            true,
		"create":          true,
		"initialize":      true,
		"add_liquidity":   true,
		"remove_liquidity": true,
		"trade":           true,
		"launch":          true,
		"migrate":         true,
	}
	return tradingTypes[instructionType]
}