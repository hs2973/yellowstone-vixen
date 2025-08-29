package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"
)

// EssentialData matches the Rust struct for parsed Solana data
type EssentialData struct {
	ProgramID            string                 `json:"program_id"`
	TokenMint            *string                `json:"token_mint"`
	TransactionSignature string                 `json:"transaction_signature"`
	InstructionType      string                 `json:"instruction_type"`
	InstructionData      map[string]interface{} `json:"instruction_data"`
	BlockchainTimestamp  int64                  `json:"blockchain_timestamp"`
	IngestionTimestamp   int64                  `json:"ingestion_timestamp"`
	Slot                 uint64                 `json:"slot"`
	Metadata             *map[string]interface{} `json:"metadata"`
}

// SSEClient handles Server-Sent Events connection to the Rust application
type SSEClient struct {
	url string
}

// NewSSEClient creates a new SSE client
func NewSSEClient(url string) *SSEClient {
	return &SSEClient{
		url: url,
	}
}

// Connect establishes connection to the SSE stream with automatic reconnection
func (c *SSEClient) Connect(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			if err := c.connectOnce(ctx); err != nil {
				log.Printf("SSE connection failed, retrying in 5 seconds: %v", err)
				time.Sleep(5 * time.Second)
				continue
			}
		}
	}
}

// connectOnce handles a single connection attempt
func (c *SSEClient) connectOnce(ctx context.Context) error {
	log.Printf("Connecting to SSE stream: %s", c.url)
	
	req, err := http.NewRequestWithContext(ctx, "GET", c.url, nil)
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}
	
	req.Header.Set("Accept", "text/event-stream")
	req.Header.Set("Cache-Control", "no-cache")
	
	client := &http.Client{
		Timeout: 0, // No timeout for SSE connections
	}
	
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to connect: %w", err)
	}
	defer resp.Body.Close()
	
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("unexpected status code: %d", resp.StatusCode)
	}
	
	log.Println("Connected to SSE stream successfully")
	
	scanner := bufio.NewScanner(resp.Body)
	var eventType string
	var data strings.Builder
	
	for scanner.Scan() {
		line := scanner.Text()
		
		// Handle different SSE line types
		switch {
		case strings.HasPrefix(line, "event:"):
			eventType = strings.TrimSpace(strings.TrimPrefix(line, "event:"))
		case strings.HasPrefix(line, "data:"):
			dataLine := strings.TrimSpace(strings.TrimPrefix(line, "data:"))
			data.WriteString(dataLine)
		case line == "":
			// Empty line indicates end of event
			if eventType != "" && data.Len() > 0 {
				c.handleEvent(eventType, data.String())
				eventType = ""
				data.Reset()
			}
		case strings.HasPrefix(line, "id:"):
			// Event ID (not used in this example)
		case strings.HasPrefix(line, ":"):
			// Comment line (heartbeat)
			log.Println("Received heartbeat")
		}
	}
	
	if err := scanner.Err(); err != nil {
		return fmt.Errorf("scanner error: %w", err)
	}
	
	return fmt.Errorf("connection closed")
}

// handleEvent processes incoming SSE events
func (c *SSEClient) handleEvent(eventType, data string) {
	switch eventType {
	case "instruction":
		c.handleInstructionEvent(data)
	default:
		log.Printf("Received unknown event type: %s, data: %s", eventType, data)
	}
}

// handleInstructionEvent processes instruction events
func (c *SSEClient) handleInstructionEvent(data string) {
	var essentialData EssentialData
	if err := json.Unmarshal([]byte(data), &essentialData); err != nil {
		log.Printf("Failed to parse instruction data: %v", err)
		return
	}
	
	// Process the instruction data
	c.processInstruction(&essentialData)
}

// processInstruction handles the business logic for processing instructions
func (c *SSEClient) processInstruction(data *EssentialData) {
	log.Printf("📦 Instruction Processed:")
	log.Printf("  Program ID: %s", data.ProgramID)
	log.Printf("  Type: %s", data.InstructionType)
	log.Printf("  Transaction: %s", data.TransactionSignature)
	log.Printf("  Slot: %d", data.Slot)
	log.Printf("  Blockchain Time: %s", time.Unix(data.BlockchainTimestamp, 0).Format(time.RFC3339))
	log.Printf("  Ingestion Time: %s", time.Unix(data.IngestionTimestamp, 0).Format(time.RFC3339))
	
	if data.TokenMint != nil {
		log.Printf("  Token Mint: %s", *data.TokenMint)
	}
	
	if amount, ok := data.InstructionData["amount"].(float64); ok {
		log.Printf("  Amount: %.0f", amount)
	}
	
	log.Println("  ---")
	
	// Add your custom business logic here
	// For example:
	// - Store in local database
	// - Forward to another service
	// - Trigger alerts based on certain conditions
	// - Calculate statistics
	
	// Example: Log token transfers
	if data.InstructionType == "transfer" && data.TokenMint != nil {
		log.Printf("💰 Token transfer detected: %s", *data.TokenMint)
	}
}

func main() {
	// Configuration
	sseURL := "http://localhost:8080/events/stream"
	if len(os.Args) > 1 {
		sseURL = os.Args[1]
	}
	
	// Create SSE client
	client := NewSSEClient(sseURL)
	
	// Create context for graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	
	// Start the client
	log.Printf("🚀 Starting Solana Stream Processor Go Client")
	log.Printf("📡 Connecting to: %s", sseURL)
	log.Println("Press Ctrl+C to stop")
	
	if err := client.Connect(ctx); err != nil {
		log.Fatalf("❌ Client failed: %v", err)
	}
}