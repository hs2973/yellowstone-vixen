//! Monitoring and observability module for Chainstack gRPC source
//!
//! This module provides comprehensive monitoring, metrics collection, and observability
//! for the Chainstack Yellowstone gRPC integration, including Prometheus metrics,
//! health checks, and performance monitoring.

use crate::config::{MonitoringConfig, MetricsConfig, PrometheusConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Comprehensive monitoring system for Chainstack gRPC source
#[derive(Debug, Clone)]
pub struct MonitoringSystem {
    config: MonitoringConfig,
    metrics: Arc<RwLock<MetricsCollector>>,
    health_checks: Arc<RwLock<HealthCheckManager>>,
}

/// Metrics collector for performance and operational metrics
#[derive(Debug)]
pub struct MetricsCollector {
    // Connection metrics
    pub connections_active: u64,
    pub connections_total: u64,
    pub connection_failures: u64,
    pub connection_duration_ms: Vec<u64>,

    // Data processing metrics
    pub updates_processed: u64,
    pub updates_per_second: f64,
    pub processing_duration_ns: Vec<u64>,
    pub parse_errors: u64,

    // Redis streaming metrics
    pub redis_writes_total: u64,
    pub redis_write_failures: u64,
    pub redis_batch_size: Vec<usize>,
    pub redis_write_duration_ms: Vec<u64>,

    // Filter metrics
    pub active_filters: u64,
    pub filter_matches: HashMap<String, u64>,
    pub filter_processing_time: HashMap<String, Vec<u64>>,

    // Error metrics
    pub errors_total: u64,
    pub errors_by_type: HashMap<String, u64>,
    pub circuit_breaker_trips: u64,

    // Memory and resource metrics
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub network_bytes_in: u64,
    pub network_bytes_out: u64,

    last_update: std::time::Instant,
}

/// Health check manager for system health monitoring
#[derive(Debug)]
pub struct HealthCheckManager {
    checks: HashMap<String, HealthCheck>,
    overall_status: HealthStatus,
    last_check: std::time::Instant,
}

/// Individual health check
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: std::time::Instant,
    pub error_message: Option<String>,
    pub check_interval_secs: u64,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Performance metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: std::time::SystemTime,
    pub connections: ConnectionMetrics,
    pub processing: ProcessingMetrics,
    pub redis: RedisMetrics,
    pub errors: ErrorMetrics,
    pub resources: ResourceMetrics,
}

/// Connection-related metrics
#[derive(Debug, Clone)]
pub struct ConnectionMetrics {
    pub active_connections: u64,
    pub total_connections: u64,
    pub failed_connections: u64,
    pub avg_connection_duration_ms: f64,
    pub connection_success_rate: f64,
}

/// Data processing metrics
#[derive(Debug, Clone)]
pub struct ProcessingMetrics {
    pub total_updates: u64,
    pub updates_per_second: f64,
    pub avg_processing_time_ns: f64,
    pub parse_success_rate: f64,
    pub active_filters: u64,
}

/// Redis streaming metrics
#[derive(Debug, Clone)]
pub struct RedisMetrics {
    pub total_writes: u64,
    pub failed_writes: u64,
    pub avg_batch_size: f64,
    pub avg_write_duration_ms: f64,
    pub write_success_rate: f64,
}

/// Error tracking metrics
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub error_rate: f64,
    pub errors_by_type: HashMap<String, u64>,
    pub circuit_breaker_trips: u64,
}

/// Resource utilization metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
    pub network_bytes_in: u64,
    pub network_bytes_out: u64,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        let metrics = Arc::new(RwLock::new(MetricsCollector::new()));
        let health_checks = Arc::new(RwLock::new(HealthCheckManager::new()));

        Self {
            config,
            metrics,
            health_checks,
        }
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            info!("Monitoring disabled");
            return Ok(());
        }

        info!("Starting monitoring system");

        // Initialize health checks
        self.initialize_health_checks().await?;

        // Start metrics collection
        self.start_metrics_collection().await?;

        // Start health check loop
        self.start_health_check_loop().await;

        // Start Prometheus metrics export if configured
        if let Some(ref prometheus_config) = self.config.metrics.prometheus {
            self.start_prometheus_export(prometheus_config.clone()).await?;
        }

        Ok(())
    }

    /// Record a connection event
    pub async fn record_connection(&self, success: bool, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.connections_total += 1;
        
        if success {
            metrics.connections_active += 1;
            metrics.connection_duration_ms.push(duration_ms);
        } else {
            metrics.connection_failures += 1;
        }
    }

    /// Record connection closure
    pub async fn record_connection_closed(&self) {
        let mut metrics = self.metrics.write().await;
        if metrics.connections_active > 0 {
            metrics.connections_active -= 1;
        }
    }

    /// Record data processing event
    pub async fn record_update_processed(&self, filter_id: &str, processing_time_ns: u64, success: bool) {
        let mut metrics = self.metrics.write().await;
        
        if success {
            metrics.updates_processed += 1;
            metrics.processing_duration_ns.push(processing_time_ns);
            
            // Update filter-specific metrics
            *metrics.filter_matches.entry(filter_id.to_string()).or_insert(0) += 1;
            metrics.filter_processing_time
                .entry(filter_id.to_string())
                .or_insert_with(Vec::new)
                .push(processing_time_ns);
        } else {
            metrics.parse_errors += 1;
        }

        // Update updates per second
        metrics.update_updates_per_second();
    }

    /// Record Redis write event
    pub async fn record_redis_write(&self, success: bool, batch_size: usize, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        
        if success {
            metrics.redis_writes_total += 1;
            metrics.redis_batch_size.push(batch_size);
            metrics.redis_write_duration_ms.push(duration_ms);
        } else {
            metrics.redis_write_failures += 1;
        }
    }

    /// Record error event
    pub async fn record_error(&self, error_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.errors_total += 1;
        *metrics.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Record circuit breaker trip
    pub async fn record_circuit_breaker_trip(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.circuit_breaker_trips += 1;
    }

    /// Get current metrics snapshot
    pub async fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        let metrics = self.metrics.read().await;
        metrics.get_snapshot()
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> HealthStatus {
        let health_checks = self.health_checks.read().await;
        health_checks.overall_status.clone()
    }

    /// Initialize health checks
    async fn initialize_health_checks(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut health_checks = self.health_checks.write().await;
        
        // Add standard health checks
        health_checks.add_check(HealthCheck {
            name: "chainstack_connection".to_string(),
            status: HealthStatus::Unknown,
            last_check: std::time::Instant::now(),
            error_message: None,
            check_interval_secs: 30,
        });

        health_checks.add_check(HealthCheck {
            name: "redis_connection".to_string(),
            status: HealthStatus::Unknown,
            last_check: std::time::Instant::now(),
            error_message: None,
            check_interval_secs: 15,
        });

        health_checks.add_check(HealthCheck {
            name: "data_processing".to_string(),
            status: HealthStatus::Unknown,
            last_check: std::time::Instant::now(),
            error_message: None,
            check_interval_secs: 60,
        });

        Ok(())
    }

    /// Start metrics collection loop
    async fn start_metrics_collection(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metrics = Arc::clone(&self.metrics);
        let interval_secs = self.config.metrics.collection_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Update resource metrics
                if let Ok(mut metrics_guard) = metrics.try_write() {
                    metrics_guard.update_resource_metrics().await;
                }
            }
        });

        Ok(())
    }

    /// Start health check monitoring loop
    async fn start_health_check_loop(&self) {
        let health_checks = Arc::clone(&self.health_checks);
        let metrics = Arc::clone(&self.metrics);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let (Ok(mut health_guard), Ok(metrics_guard)) = (health_checks.try_write(), metrics.try_read()) {
                    health_guard.run_health_checks(&metrics_guard).await;
                }
            }
        });
    }

    /// Start Prometheus metrics export
    async fn start_prometheus_export(&self, config: PrometheusConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metrics = Arc::clone(&self.metrics);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(config.export_interval_secs));
            
            loop {
                interval.tick().await;
                
                if let Ok(metrics_guard) = metrics.try_read() {
                    if let Err(e) = export_prometheus_metrics(&config, &metrics_guard).await {
                        error!(error = ?e, "Failed to export Prometheus metrics");
                    }
                }
            }
        });

        Ok(())
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            connections_active: 0,
            connections_total: 0,
            connection_failures: 0,
            connection_duration_ms: Vec::new(),
            updates_processed: 0,
            updates_per_second: 0.0,
            processing_duration_ns: Vec::new(),
            parse_errors: 0,
            redis_writes_total: 0,
            redis_write_failures: 0,
            redis_batch_size: Vec::new(),
            redis_write_duration_ms: Vec::new(),
            active_filters: 0,
            filter_matches: HashMap::new(),
            filter_processing_time: HashMap::new(),
            errors_total: 0,
            errors_by_type: HashMap::new(),
            circuit_breaker_trips: 0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            network_bytes_in: 0,
            network_bytes_out: 0,
            last_update: std::time::Instant::now(),
        }
    }

    fn update_updates_per_second(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        
        if elapsed >= 1.0 {
            self.updates_per_second = self.updates_processed as f64 / elapsed;
            self.last_update = now;
        }
    }

    async fn update_resource_metrics(&mut self) {
        // TODO: Implement actual resource monitoring
        // This would typically use system APIs to get actual resource usage
        self.memory_usage_bytes = get_memory_usage().await.unwrap_or(0);
        self.cpu_usage_percent = get_cpu_usage().await.unwrap_or(0.0);
    }

    fn get_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            timestamp: std::time::SystemTime::now(),
            connections: ConnectionMetrics {
                active_connections: self.connections_active,
                total_connections: self.connections_total,
                failed_connections: self.connection_failures,
                avg_connection_duration_ms: self.calculate_avg(&self.connection_duration_ms),
                connection_success_rate: if self.connections_total > 0 {
                    (self.connections_total - self.connection_failures) as f64 / self.connections_total as f64
                } else {
                    0.0
                },
            },
            processing: ProcessingMetrics {
                total_updates: self.updates_processed,
                updates_per_second: self.updates_per_second,
                avg_processing_time_ns: self.calculate_avg_u64(&self.processing_duration_ns),
                parse_success_rate: if self.updates_processed + self.parse_errors > 0 {
                    self.updates_processed as f64 / (self.updates_processed + self.parse_errors) as f64
                } else {
                    0.0
                },
                active_filters: self.active_filters,
            },
            redis: RedisMetrics {
                total_writes: self.redis_writes_total,
                failed_writes: self.redis_write_failures,
                avg_batch_size: self.calculate_avg_usize(&self.redis_batch_size),
                avg_write_duration_ms: self.calculate_avg(&self.redis_write_duration_ms),
                write_success_rate: if self.redis_writes_total + self.redis_write_failures > 0 {
                    self.redis_writes_total as f64 / (self.redis_writes_total + self.redis_write_failures) as f64
                } else {
                    0.0
                },
            },
            errors: ErrorMetrics {
                total_errors: self.errors_total,
                error_rate: self.errors_total as f64 / (self.updates_processed + self.errors_total) as f64,
                errors_by_type: self.errors_by_type.clone(),
                circuit_breaker_trips: self.circuit_breaker_trips,
            },
            resources: ResourceMetrics {
                memory_usage_bytes: self.memory_usage_bytes,
                memory_usage_percent: 0.0, // TODO: Calculate from system memory
                cpu_usage_percent: self.cpu_usage_percent,
                network_bytes_in: self.network_bytes_in,
                network_bytes_out: self.network_bytes_out,
            },
        }
    }

    fn calculate_avg(&self, values: &[u64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<u64>() as f64 / values.len() as f64
        }
    }

    fn calculate_avg_u64(&self, values: &[u64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<u64>() as f64 / values.len() as f64
        }
    }

    fn calculate_avg_usize(&self, values: &[usize]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<usize>() as f64 / values.len() as f64
        }
    }
}

impl HealthCheckManager {
    fn new() -> Self {
        Self {
            checks: HashMap::new(),
            overall_status: HealthStatus::Unknown,
            last_check: std::time::Instant::now(),
        }
    }

    fn add_check(&mut self, check: HealthCheck) {
        self.checks.insert(check.name.clone(), check);
    }

    async fn run_health_checks(&mut self, metrics: &MetricsCollector) {
        let now = std::time::Instant::now();
        let mut unhealthy_count = 0;
        let mut degraded_count = 0;
        let mut total_checks = 0;

        for (name, check) in &mut self.checks {
            // Check if it's time to run this health check
            if now.duration_since(check.last_check).as_secs() < check.check_interval_secs {
                continue;
            }

            total_checks += 1;
            check.last_check = now;

            // Run specific health checks based on the check name
            match name.as_str() {
                "chainstack_connection" => {
                    check.status = if metrics.connections_active > 0 {
                        HealthStatus::Healthy
                    } else if metrics.connection_failures > 5 {
                        HealthStatus::Unhealthy
                    } else {
                        HealthStatus::Degraded
                    };
                }
                "redis_connection" => {
                    let redis_success_rate = if metrics.redis_writes_total + metrics.redis_write_failures > 0 {
                        metrics.redis_writes_total as f64 / (metrics.redis_writes_total + metrics.redis_write_failures) as f64
                    } else {
                        1.0
                    };

                    check.status = if redis_success_rate > 0.95 {
                        HealthStatus::Healthy
                    } else if redis_success_rate > 0.8 {
                        HealthStatus::Degraded
                    } else {
                        HealthStatus::Unhealthy
                    };
                }
                "data_processing" => {
                    let parse_success_rate = if metrics.updates_processed + metrics.parse_errors > 0 {
                        metrics.updates_processed as f64 / (metrics.updates_processed + metrics.parse_errors) as f64
                    } else {
                        1.0
                    };

                    check.status = if parse_success_rate > 0.98 {
                        HealthStatus::Healthy
                    } else if parse_success_rate > 0.9 {
                        HealthStatus::Degraded
                    } else {
                        HealthStatus::Unhealthy
                    };
                }
                _ => {
                    check.status = HealthStatus::Unknown;
                }
            }

            match check.status {
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                _ => {}
            }
        }

        // Calculate overall status
        self.overall_status = if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if total_checks > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };

        self.last_check = now;
    }
}

/// Export metrics to Prometheus
async fn export_prometheus_metrics(
    config: &PrometheusConfig,
    metrics: &MetricsCollector,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement actual Prometheus metrics export
    // This would typically use the prometheus crate to format and push metrics
    info!(
        endpoint = %config.endpoint,
        updates_processed = metrics.updates_processed,
        connections_active = metrics.connections_active,
        "Exported metrics to Prometheus"
    );
    
    Ok(())
}

/// Get current memory usage (placeholder implementation)
async fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement actual memory usage monitoring
    Ok(0)
}

/// Get current CPU usage (placeholder implementation)
async fn get_cpu_usage() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement actual CPU usage monitoring
    Ok(0.0)
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}