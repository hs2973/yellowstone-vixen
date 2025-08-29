package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/sirupsen/logrus"
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
	url    string
	logger *logrus.Logger
}

// NewSSEClient creates a new SSE client
func NewSSEClient(url string) *SSEClient {
	logger := logrus.New()
	logger.SetLevel(logrus.InfoLevel)
	
	return &SSEClient{
		url:    url,
		logger: logger,
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
				c.logger.WithError(err).Error("SSE connection failed, retrying in 5 seconds...")
				time.Sleep(5 * time.Second)
				continue
			}
		}
	}
}

// connectOnce handles a single connection attempt
func (c *SSEClient) connectOnce(ctx context.Context) error {
	c.logger.WithField("url", c.url).Info("Connecting to SSE stream...")
	
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
	
	c.logger.Info("Connected to SSE stream successfully")
	
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
			c.logger.Debug("Received heartbeat")
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
		c.logger.WithFields(logrus.Fields{
			"event_type": eventType,
			"data":       data,
		}).Debug("Received unknown event type")
	}
}

// handleInstructionEvent processes instruction events
func (c *SSEClient) handleInstructionEvent(data string) {
	var essentialData EssentialData
	if err := json.Unmarshal([]byte(data), &essentialData); err != nil {
		c.logger.WithError(err).Error("Failed to parse instruction data")
		return
	}
	
	// Process the instruction data
	c.processInstruction(&essentialData)
}

// processInstruction handles the business logic for processing instructions
func (c *SSEClient) processInstruction(data *EssentialData) {
	c.logger.WithFields(logrus.Fields{
		"program_id":         data.ProgramID,
		"instruction_type":   data.InstructionType,
		"transaction_sig":    data.TransactionSignature,
		"slot":              data.Slot,
		"blockchain_time":   time.Unix(data.BlockchainTimestamp, 0),
		"ingestion_time":    time.Unix(data.IngestionTimestamp, 0),
	}).Info("Processed instruction")
	
	// Add your custom business logic here
	// For example:
	// - Store in local database
	// - Forward to another service
	// - Trigger alerts based on certain conditions
	// - Calculate statistics
	
	// Example: Log token transfers
	if data.InstructionType == "transfer" && data.TokenMint != nil {
		c.logger.WithFields(logrus.Fields{
			"token_mint": *data.TokenMint,
			"type":       "token_transfer",
		}).Info("Token transfer detected")
	}
}

func main() {
	// Configuration
	sseURL := "http://localhost:8080/events/stream"
	if len(log.Args()) > 1 {
		sseURL = log.Args()[1]
	}
	
	// Create SSE client
	client := NewSSEClient(sseURL)
	
	// Create context for graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	
	// Start the client
	log.Printf("Starting Solana Stream Processor Go Client")
	log.Printf("Connecting to: %s", sseURL)
	
	if err := client.Connect(ctx); err != nil {
		log.Fatalf("Client failed: %v", err)
	}
}