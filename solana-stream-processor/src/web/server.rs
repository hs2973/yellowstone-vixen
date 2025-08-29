//! Web server implementation for the Solana Stream Processor

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::time::Instant;
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, error};

use crate::error::ProcessorResult;
use crate::metrics::Metrics;
use crate::models::{HealthResponse, SseEvent};
use crate::web::sse::{SseManager, create_sse_response};

/// Application state shared across handlers
#[derive(Debug, Clone)]
pub struct AppState {
    pub metrics: Metrics,
    pub sse_manager: SseManager,
    pub start_time: Instant,
}

/// Web server for serving SSE streams, health checks, and metrics
#[derive(Debug)]
pub struct WebServer {
    port: u16,
    app_state: AppState,
}

impl WebServer {
    /// Create a new web server
    pub fn new(port: u16, metrics: Metrics) -> Self {
        let sse_manager = SseManager::new(1000); // Buffer size for SSE events
        let start_time = Instant::now();
        
        let app_state = AppState {
            metrics,
            sse_manager,
            start_time,
        };
        
        Self {
            port,
            app_state,
        }
    }
    
    /// Get the SSE sender for publishing events
    pub fn get_sse_sender(&self) -> broadcast::Sender<SseEvent> {
        self.app_state.sse_manager.get_sender()
    }
    
    /// Start the web server
    pub async fn start(self) -> ProcessorResult<()> {
        let app = self.create_app();
        
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port))
            .await
            .map_err(|e| crate::error::ProcessorError::WebServer(format!("Failed to bind to port {}: {}", self.port, e)))?;
        
        info!("Web server starting on port {}", self.port);
        info!("Health endpoint: http://localhost:{}/health", self.port);
        info!("Metrics endpoint: http://localhost:{}/metrics", self.port);
        info!("SSE stream endpoint: http://localhost:{}/events/stream", self.port);
        
        axum::serve(listener, app)
            .await
            .map_err(|e| crate::error::ProcessorError::WebServer(format!("Server error: {}", e)))?;
        
        Ok(())
    }
    
    /// Create the Axum application with all routes
    fn create_app(&self) -> Router {
        Router::new()
            .route("/health", get(health_handler))
            .route("/metrics", get(metrics_handler))
            .route("/events/stream", get(sse_stream_handler))
            .route("/", get(root_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(
                        CorsLayer::new()
                            .allow_origin(Any)
                            .allow_methods(Any)
                            .allow_headers(Any)
                    )
            )
            .with_state(self.app_state.clone())
    }
}

/// Root endpoint handler
async fn root_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "solana-stream-processor",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "health": "/health",
            "metrics": "/metrics", 
            "sse_stream": "/events/stream"
        }
    }))
}

/// Health check endpoint handler
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed();
    let response = HealthResponse::healthy(uptime.as_secs());
    
    (StatusCode::OK, Json(response))
}

/// Prometheus metrics endpoint handler
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Update uptime metric
    let uptime = state.start_time.elapsed();
    state.metrics.set_uptime(uptime);
    
    match state.metrics.gather() {
        Ok(metrics_text) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
                metrics_text,
            )
        },
        Err(e) => {
            error!("Failed to gather metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("Failed to gather metrics: {}", e),
            )
        }
    }
}

/// SSE stream endpoint handler
async fn sse_stream_handler(State(state): State<AppState>) -> impl IntoResponse {
    info!("New SSE connection established");
    
    // Update active connections metric
    // Note: This is a simple approximation. A more accurate count would require
    // tracking connection lifecycle events.
    state.metrics.active_sse_connections.inc();
    
    create_sse_response(state.sse_manager)
}