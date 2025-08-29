use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Application metrics for monitoring and observability
/// Simple in-memory metrics that can be exported for Prometheus
pub struct Metrics {
    // Stream processing metrics
    stream_messages_processed: AtomicU64,
    stream_messages_errors: AtomicU64,
    stream_connections: AtomicU64,
    stream_events_sent: AtomicU64,
    stream_errors: AtomicU64,
    
    // Database operation metrics (stored as HashMap for labels)
    database_operations: Arc<std::sync::Mutex<HashMap<String, u64>>>,
    
    // API request metrics
    api_requests: Arc<std::sync::Mutex<HashMap<String, u64>>>,
    
    // gRPC connection status
    grpc_connection_status: AtomicU64,
    grpc_reconnections: AtomicU64,
    
    // Timing metrics (stored as histograms)
    processing_latencies: Arc<std::sync::Mutex<Vec<f64>>>,
    database_durations: Arc<std::sync::Mutex<Vec<f64>>>,
    api_durations: Arc<std::sync::Mutex<Vec<f64>>>,
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            stream_messages_processed: AtomicU64::new(0),
            stream_messages_errors: AtomicU64::new(0),
            stream_connections: AtomicU64::new(0),
            stream_events_sent: AtomicU64::new(0),
            stream_errors: AtomicU64::new(0),
            database_operations: Arc::new(std::sync::Mutex::new(HashMap::new())),
            api_requests: Arc::new(std::sync::Mutex::new(HashMap::new())),
            grpc_connection_status: AtomicU64::new(0),
            grpc_reconnections: AtomicU64::new(0),
            processing_latencies: Arc::new(std::sync::Mutex::new(Vec::new())),
            database_durations: Arc::new(std::sync::Mutex::new(Vec::new())),
            api_durations: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_metrics(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut output = String::new();
        
        // Stream metrics
        output.push_str(&format!(
            "# HELP stream_messages_processed_total Total number of stream messages processed\n\
             # TYPE stream_messages_processed_total counter\n\
             stream_messages_processed_total {}\n\n",
            self.stream_messages_processed.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP stream_messages_errors_total Total number of stream message processing errors\n\
             # TYPE stream_messages_errors_total counter\n\
             stream_messages_errors_total {}\n\n",
            self.stream_messages_errors.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP stream_connections_active Number of active stream connections\n\
             # TYPE stream_connections_active gauge\n\
             stream_connections_active {}\n\n",
            self.stream_connections.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP stream_events_sent_total Total number of events sent to stream clients\n\
             # TYPE stream_events_sent_total counter\n\
             stream_events_sent_total {}\n\n",
            self.stream_events_sent.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP grpc_connection_status Status of gRPC connection (1 = connected, 0 = disconnected)\n\
             # TYPE grpc_connection_status gauge\n\
             grpc_connection_status {}\n\n",
            self.grpc_connection_status.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP grpc_reconnections_total Total number of gRPC reconnection attempts\n\
             # TYPE grpc_reconnections_total counter\n\
             grpc_reconnections_total {}\n\n",
            self.grpc_reconnections.load(Ordering::Relaxed)
        ));

        // Database operations
        if let Ok(db_ops) = self.database_operations.lock() {
            output.push_str("# HELP database_operations_total Total number of database operations\n");
            output.push_str("# TYPE database_operations_total counter\n");
            for (key, value) in db_ops.iter() {
                output.push_str(&format!("database_operations_total{{operation=\"{}\"}} {}\n", key, value));
            }
            output.push('\n');
        }

        // API requests
        if let Ok(api_reqs) = self.api_requests.lock() {
            output.push_str("# HELP api_requests_total Total number of API requests\n");
            output.push_str("# TYPE api_requests_total counter\n");
            for (key, value) in api_reqs.iter() {
                output.push_str(&format!("api_requests_total{{endpoint=\"{}\"}} {}\n", key, value));
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Increment stream messages processed counter
    pub fn increment_stream_messages_processed(&self) {
        self.stream_messages_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment stream messages errors counter
    pub fn increment_stream_messages_errors(&self) {
        self.stream_messages_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record processing latency
    pub fn record_processing_latency(&self, duration: Duration) {
        if let Ok(mut latencies) = self.processing_latencies.lock() {
            latencies.push(duration.as_secs_f64());
            // Keep only last 1000 measurements to avoid memory growth
            if latencies.len() > 1000 {
                let len = latencies.len();
                latencies.drain(0..(len - 1000));
            }
        }
    }

    /// Increment/decrement stream connections gauge
    pub fn increment_stream_connections(&self) {
        self.stream_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_stream_connections(&self) {
        self.stream_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Increment stream events sent counter
    pub fn increment_stream_events_sent(&self) {
        self.stream_events_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment stream errors counter
    pub fn increment_stream_errors(&self) {
        self.stream_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record database operation metrics
    pub fn increment_database_operations(&self, operation: &str, status: &str) {
        let key = format!("{}_{}", operation, status);
        if let Ok(mut ops) = self.database_operations.lock() {
            *ops.entry(key).or_insert(0) += 1;
        }
    }

    pub fn record_database_operation_duration(&self, _operation: &str, _status: &str, duration: Duration) {
        if let Ok(mut durations) = self.database_durations.lock() {
            durations.push(duration.as_secs_f64());
            // Keep only last 1000 measurements to avoid memory growth
            if durations.len() > 1000 {
                let len = durations.len();
                durations.drain(0..(len - 1000));
            }
        }
    }

    /// Record API request metrics
    pub fn increment_api_requests(&self, endpoint: &str, status: &str) {
        let key = format!("{}_{}", endpoint, status);
        if let Ok(mut reqs) = self.api_requests.lock() {
            *reqs.entry(key).or_insert(0) += 1;
        }
    }

    pub fn record_api_request_duration(&self, _endpoint: &str, duration: Duration) {
        if let Ok(mut durations) = self.api_durations.lock() {
            durations.push(duration.as_secs_f64());
            // Keep only last 1000 measurements to avoid memory growth
            if durations.len() > 1000 {
                let len = durations.len();
                durations.drain(0..(len - 1000));
            }
        }
    }

    /// Set gRPC connection status
    pub fn set_grpc_connected(&self, connected: bool) {
        self.grpc_connection_status.store(if connected { 1 } else { 0 }, Ordering::Relaxed);
    }

    /// Increment gRPC reconnection counter
    pub fn increment_grpc_reconnections(&self) {
        self.grpc_reconnections.fetch_add(1, Ordering::Relaxed);
    }
}

/// Global metrics instance
static GLOBAL_METRICS: once_cell::sync::Lazy<Arc<Metrics>> = once_cell::sync::Lazy::new(|| Arc::new(Metrics::new()));

/// Get the global metrics instance
pub fn global_metrics() -> Arc<Metrics> {
    GLOBAL_METRICS.clone()
}

/// Helper macro for timing operations
#[macro_export]
macro_rules! time_operation {
    ($metrics:expr, $operation:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed();
        $metrics.record_processing_latency(duration);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        
        // Test that counters can be incremented
        metrics.increment_stream_messages_processed();
        metrics.increment_stream_messages_errors();
        
        // Test that gauges can be set
        metrics.set_grpc_connected(true);
        metrics.set_grpc_connected(false);
        
        // Test that histograms can record values
        metrics.record_processing_latency(Duration::from_millis(100));
        metrics.record_database_operation_duration("insert", "success", Duration::from_millis(50));
        metrics.record_api_request_duration("/health", Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let metrics = Metrics::new();
        
        // Add some sample data
        metrics.increment_stream_messages_processed();
        metrics.record_processing_latency(Duration::from_millis(100));
        
        // Test export
        let result = metrics.export_metrics().await;
        assert!(result.is_ok());
        
        let metrics_text = result.unwrap();
        assert!(metrics_text.contains("stream_messages_processed_total"));
    }

    #[test]
    fn test_global_metrics() {
        let metrics1 = global_metrics();
        let metrics2 = global_metrics();
        
        // Should be the same instance
        assert!(Arc::ptr_eq(&metrics1, &metrics2));
    }
}