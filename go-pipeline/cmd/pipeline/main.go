// Package main implements the second stage Go pipeline for high-throughput
// data processing from Redis streams to PostgreSQL database.
//
// This pipeline is designed to handle 700,000+ packets per second efficiently
// using the architecture:
// Redis Stream Consumer → Worker Pool → Batch Processor → PostgreSQL
package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/hs2973/yellowstone-vixen/go-pipeline/internal/config"
	"github.com/hs2973/yellowstone-vixen/go-pipeline/internal/pipeline"
	"github.com/hs2973/yellowstone-vixen/go-pipeline/pkg/logger"
	"github.com/hs2973/yellowstone-vixen/go-pipeline/pkg/metrics"
	"go.uber.org/zap"
)

var (
	configPath = flag.String("config", "config.yaml", "Path to configuration file")
	version    = flag.Bool("version", false, "Show version information")
)

// Version information
var (
	Version   = "dev"
	GitCommit = "unknown"
	BuildTime = "unknown"
)

func main() {
	flag.Parse()

	if *version {
		fmt.Printf("Yellowstone Vixen Go Pipeline\n")
		fmt.Printf("Version: %s\n", Version)
		fmt.Printf("Git Commit: %s\n", GitCommit)
		fmt.Printf("Build Time: %s\n", BuildTime)
		os.Exit(0)
	}

	// Load configuration
	cfg, err := config.Load(*configPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to load config: %v\n", err)
		os.Exit(1)
	}

	// Initialize logger
	log, err := logger.New(cfg.Logging)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer log.Sync()

	log.Info("Starting Yellowstone Vixen Go Pipeline",
		zap.String("version", Version),
		zap.String("git_commit", GitCommit),
		zap.String("build_time", BuildTime),
	)

	// Initialize metrics
	metricsServer := metrics.NewServer(cfg.Metrics)
	if err := metricsServer.Start(); err != nil {
		log.Fatal("Failed to start metrics server", zap.Error(err))
	}
	defer metricsServer.Stop()

	// Create pipeline
	pipeline, err := pipeline.New(cfg, log)
	if err != nil {
		log.Fatal("Failed to create pipeline", zap.Error(err))
	}

	// Setup graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	// Start pipeline
	errChan := make(chan error, 1)
	go func() {
		errChan <- pipeline.Run(ctx)
	}()

	// Wait for shutdown signal or error
	select {
	case <-sigChan:
		log.Info("Received shutdown signal")
		cancel()
		
		// Wait for graceful shutdown with timeout
		shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer shutdownCancel()
		
		if err := pipeline.Shutdown(shutdownCtx); err != nil {
			log.Error("Pipeline shutdown error", zap.Error(err))
		} else {
			log.Info("Pipeline shutdown complete")
		}

	case err := <-errChan:
		if err != nil {
			log.Error("Pipeline error", zap.Error(err))
			os.Exit(1)
		}
	}
}