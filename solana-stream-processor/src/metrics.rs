//! Metrics module for monitoring the Solana Stream Processor

use prometheus::{
    Counter, Histogram, IntGauge, Registry, Encoder, TextEncoder,
    HistogramOpts, Opts, register_counter_with_registry,
    register_histogram_with_registry, register_int_gauge_with_registry,
};
use std::time::Instant;

/// Metrics registry and collectors
#[derive(Debug, Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub messages_processed_total: Counter,
    pub messages_filtered_total: Counter,
    pub sse_messages_sent_total: Counter,
    pub mongodb_writes_total: Counter,
    pub mongodb_write_errors_total: Counter,
    pub processing_duration_seconds: Histogram,
    pub active_sse_connections: IntGauge,
    pub last_processed_slot: IntGauge,
    pub uptime_seconds: IntGauge,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        let messages_processed_total = register_counter_with_registry!(
            Opts::new(
                "stream_messages_processed_total",
                "Total number of messages processed"
            ),
            registry
        )?;
        
        let messages_filtered_total = register_counter_with_registry!(
            Opts::new(
                "stream_messages_filtered_total", 
                "Total number of messages that passed filtering"
            ),
            registry
        )?;
        
        let sse_messages_sent_total = register_counter_with_registry!(
            Opts::new(
                "sse_messages_sent_total",
                "Total number of messages sent via SSE"
            ),
            registry
        )?;
        
        let mongodb_writes_total = register_counter_with_registry!(
            Opts::new(
                "mongodb_writes_total",
                "Total number of MongoDB write operations"
            ),
            registry
        )?;
        
        let mongodb_write_errors_total = register_counter_with_registry!(
            Opts::new(
                "mongodb_write_errors_total",
                "Total number of MongoDB write errors"
            ),
            registry
        )?;
        
        let processing_duration_seconds = register_histogram_with_registry!(
            HistogramOpts::new(
                "processing_duration_seconds",
                "Duration of message processing in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            registry
        )?;
        
        let active_sse_connections = register_int_gauge_with_registry!(
            Opts::new(
                "active_sse_connections",
                "Number of active SSE connections"
            ),
            registry
        )?;
        
        let last_processed_slot = register_int_gauge_with_registry!(
            Opts::new(
                "last_processed_slot",
                "Last processed blockchain slot"
            ),
            registry
        )?;
        
        let uptime_seconds = register_int_gauge_with_registry!(
            Opts::new(
                "uptime_seconds",
                "Application uptime in seconds"
            ),
            registry
        )?;
        
        Ok(Self {
            registry,
            messages_processed_total,
            messages_filtered_total,
            sse_messages_sent_total,
            mongodb_writes_total,
            mongodb_write_errors_total,
            processing_duration_seconds,
            active_sse_connections,
            last_processed_slot,
            uptime_seconds,
        })
    }
    
    /// Record a processed message
    pub fn record_message_processed(&self) {
        self.messages_processed_total.inc();
    }
    
    /// Record a filtered message
    pub fn record_message_filtered(&self) {
        self.messages_filtered_total.inc();
    }
    
    /// Record an SSE message sent
    pub fn record_sse_message_sent(&self) {
        self.sse_messages_sent_total.inc();
    }
    
    /// Record a MongoDB write
    pub fn record_mongodb_write(&self) {
        self.mongodb_writes_total.inc();
    }
    
    /// Record a MongoDB write error
    pub fn record_mongodb_write_error(&self) {
        self.mongodb_write_errors_total.inc();
    }
    
    /// Record processing duration
    pub fn record_processing_duration(&self, duration: std::time::Duration) {
        self.processing_duration_seconds.observe(duration.as_secs_f64());
    }
    
    /// Update active SSE connections count
    pub fn set_active_sse_connections(&self, count: i64) {
        self.active_sse_connections.set(count);
    }
    
    /// Update last processed slot
    pub fn set_last_processed_slot(&self, slot: u64) {
        self.last_processed_slot.set(slot as i64);
    }
    
    /// Update uptime
    pub fn set_uptime(&self, uptime: std::time::Duration) {
        self.uptime_seconds.set(uptime.as_secs() as i64);
    }
    
    /// Gather all metrics in Prometheus format
    pub fn gather(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}

/// Timer helper for measuring durations
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize metrics registry
pub fn initialize_metrics() -> Metrics {
    Metrics::new().expect("Failed to initialize metrics")
}