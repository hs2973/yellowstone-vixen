use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

use axum::{
    extract::{Query, State, Path},
    http::{StatusCode, HeaderMap, HeaderValue},
    response::{Response, IntoResponse, sse::{Event, KeepAlive}},
    routing::{get, post},
    Router, Json,
};
use axum::response::sse::Sse;
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use bson::Document;

use crate::config::ServerConfig;
use crate::models::{ParsedData, ProgramMetadata};
use crate::handlers::MongoHandler;
use crate::metrics::Metrics;

/// Web server handler for streaming APIs and data access
/// Provides SSE streaming endpoints and REST APIs for querying data
pub struct StreamHandler {
    config: ServerConfig,
    mongo_handler: Arc<MongoHandler>,
    metrics: Arc<Metrics>,
    /// Broadcast channel for real-time data streaming
    broadcast_tx: broadcast::Sender<ParsedData>,
}

/// Query parameters for recent data endpoint
#[derive(Debug, Deserialize)]
pub struct RecentDataQuery {
    /// Program name to query
    pub program: Option<String>,
    /// Token mint address to filter by
    pub token_mint: Option<String>,
    /// Maximum number of records to return
    pub limit: Option<u32>,
    /// Collection type: "accounts" or "instructions"
    pub collection_type: Option<String>,
}

/// Query parameters for slot range queries
#[derive(Debug, Deserialize)]
pub struct SlotRangeQuery {
    /// Minimum slot number
    pub min_slot: u64,
    /// Maximum slot number
    pub max_slot: u64,
    /// Maximum number of records to return
    pub limit: Option<u32>,
}

/// Response structure for API endpoints
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub count: Option<usize>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            count: None,
        }
    }

    pub fn success_with_count(data: T, count: usize) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            count: Some(count),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            count: None,
        }
    }
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub mongo_handler: Arc<MongoHandler>,
    pub metrics: Arc<Metrics>,
    pub broadcast_tx: broadcast::Sender<ParsedData>,
}

impl StreamHandler {
    /// Create a new stream handler with the given configuration
    pub fn new(
        config: ServerConfig,
        mongo_handler: Arc<MongoHandler>,
        metrics: Arc<Metrics>,
    ) -> Self {
        // Create broadcast channel for real-time streaming
        // Buffer size should be large enough to handle burst traffic
        let (broadcast_tx, _) = broadcast::channel(10000);

        Self {
            config,
            mongo_handler,
            metrics,
            broadcast_tx,
        }
    }

    /// Get the broadcast sender for publishing data
    pub fn get_broadcast_sender(&self) -> broadcast::Sender<ParsedData> {
        self.broadcast_tx.clone()
    }

    /// Build the Axum router with all endpoints
    pub fn build_router(&self) -> Router {
        let app_state = AppState {
            mongo_handler: self.mongo_handler.clone(),
            metrics: self.metrics.clone(),
            broadcast_tx: self.broadcast_tx.clone(),
        };

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            // Real-time streaming endpoint using Server-Sent Events
            .route("/events/stream", get(stream_events))
            // REST endpoint for querying recent data
            .route("/events/recent", get(get_recent_data))
            // REST endpoint for querying data by slot range
            .route("/events/range/:program/:collection_type", get(get_data_by_slot_range))
            // Health check endpoint
            .route("/health", get(health_check))
            // Metrics endpoint for Prometheus
            .route("/metrics", get(get_metrics))
            // Program metadata endpoint
            .route("/programs", get(get_programs))
            // Program-specific endpoints
            .route("/programs/:program/stats", get(get_program_stats))
            .with_state(app_state)
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(TimeoutLayer::new(Duration::from_secs(30)))
                    .layer(cors)
            )
    }

    /// Publish parsed data to all connected SSE streams
    pub async fn publish_data(&self, data: ParsedData) {
        let subscriber_count = self.broadcast_tx.receiver_count();
        
        if subscriber_count > 0 {
            match self.broadcast_tx.send(data) {
                Ok(_) => {
                    self.metrics.increment_stream_events_sent();
                    debug!("Published data to {} subscribers", subscriber_count);
                }
                Err(e) => {
                    warn!("Failed to publish data to stream: {}", e);
                    self.metrics.increment_stream_errors();
                }
            }
        }
    }

    /// Start the web server
    pub async fn start_server(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = self.build_router();
        let addr = self.config.bind_address;

        info!("Starting web server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Handler for the SSE streaming endpoint
/// Provides real-time stream of parsed data to connected clients
async fn stream_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    info!("New SSE connection established");
    state.metrics.increment_stream_connections();

    let rx = state.broadcast_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| async move {
            match result {
                Ok(data) => {
                    match serde_json::to_string(&data) {
                        Ok(json) => Some(Ok(Event::default().data(json))),
                        Err(e) => {
                            error!("Failed to serialize data for SSE: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("Broadcast receiver error: {}", e);
                    None
                }
            }
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Handler for querying recent data
async fn get_recent_data(
    Query(params): Query<RecentDataQuery>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Document>>>, StatusCode> {
    debug!("Recent data query: {:?}", params);

    let program = params.program.as_deref().unwrap_or("pump_fun");
    let limit = params.limit.unwrap_or(100).min(1000); // Cap at 1000 records

    let start_time = std::time::Instant::now();

    let result = if let Some(token_mint) = params.token_mint.as_deref() {
        // Query by token mint
        state.mongo_handler
            .query_recent_by_token_mint(program, token_mint, limit)
            .await
    } else {
        // Query recent data by slot range (last 1000 slots as example)
        let collection_type = params.collection_type.as_deref().unwrap_or("instructions");
        
        // For demo purposes, query recent data from a reasonable slot range
        // In production, you might want to track the latest slot more precisely
        let max_slot = u64::MAX;
        let min_slot = max_slot.saturating_sub(1000);
        
        state.mongo_handler
            .query_by_slot_range(program, collection_type, min_slot, max_slot, Some(limit))
            .await
    };

    let duration = start_time.elapsed();
    state.metrics.record_api_request_duration("/events/recent", duration);

    match result {
        Ok(data) => {
            let count = data.len();
            state.metrics.increment_api_requests("/events/recent", "success");
            Ok(Json(ApiResponse::success_with_count(data, count)))
        }
        Err(e) => {
            error!("Failed to query recent data: {}", e);
            state.metrics.increment_api_requests("/events/recent", "error");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handler for querying data by slot range
async fn get_data_by_slot_range(
    Path((program, collection_type)): Path<(String, String)>,
    Query(params): Query<SlotRangeQuery>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Document>>>, StatusCode> {
    debug!("Slot range query: program={}, collection_type={}, range={}-{}", 
           program, collection_type, params.min_slot, params.max_slot);

    // Validate collection type
    if collection_type != "accounts" && collection_type != "instructions" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate slot range
    if params.min_slot > params.max_slot {
        return Err(StatusCode::BAD_REQUEST);
    }

    let limit = params.limit.unwrap_or(1000).min(10000); // Cap at 10k records
    let start_time = std::time::Instant::now();

    match state.mongo_handler
        .query_by_slot_range(&program, &collection_type, params.min_slot, params.max_slot, Some(limit))
        .await
    {
        Ok(data) => {
            let duration = start_time.elapsed();
            let count = data.len();
            state.metrics.record_api_request_duration("/events/range", duration);
            state.metrics.increment_api_requests("/events/range", "success");
            Ok(Json(ApiResponse::success_with_count(data, count)))
        }
        Err(e) => {
            let duration = start_time.elapsed();
            error!("Failed to query data by slot range: {}", e);
            state.metrics.record_api_request_duration("/events/range", duration);
            state.metrics.increment_api_requests("/events/range", "error");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let mongo_healthy = state.mongo_handler.health_check().await;
    
    let status = if mongo_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = ApiResponse::success(serde_json::json!({
        "status": if mongo_healthy { "healthy" } else { "unhealthy" },
        "mongodb": mongo_healthy,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }));

    (status, Json(response))
}

/// Prometheus metrics endpoint
async fn get_metrics(State(state): State<AppState>) -> impl IntoResponse {
    match state.metrics.export_metrics().await {
        Ok(metrics_text) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "content-type",
                HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
            );
            (StatusCode::OK, headers, metrics_text)
        }
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                "Failed to export metrics".to_string(),
            )
        }
    }
}

/// Get supported programs metadata
async fn get_programs() -> Json<ApiResponse<Vec<ProgramMetadata>>> {
    let programs = ProgramMetadata::all_programs();
    Json(ApiResponse::success(programs))
}

/// Get statistics for a specific program
async fn get_program_stats(
    Path(program): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // This is a placeholder for program-specific statistics
    // In a full implementation, you would query MongoDB for counts,
    // recent activity, etc.
    
    let stats = serde_json::json!({
        "program": program,
        "status": "active",
        "last_updated": chrono::Utc::now().to_rfc3339(),
        // Add more statistics as needed
    });

    Ok(Json(ApiResponse::success(stats)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_api_response_creation() {
        let success_response = ApiResponse::success("test data");
        assert!(success_response.success);
        assert!(success_response.data.is_some());
        assert!(success_response.error.is_none());

        let error_response: ApiResponse<()> = ApiResponse::error("test error".to_string());
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());
    }

    #[test]
    fn test_query_params_deserialization() {
        // These tests would typically use a testing framework
        // to validate query parameter parsing
        let recent_query = RecentDataQuery {
            program: Some("pump_fun".to_string()),
            token_mint: Some("token123".to_string()),
            limit: Some(50),
            collection_type: Some("instructions".to_string()),
        };

        assert_eq!(recent_query.program.as_deref(), Some("pump_fun"));
        assert_eq!(recent_query.limit, Some(50));
    }
}