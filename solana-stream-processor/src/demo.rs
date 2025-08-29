//! Simple test version of the Solana Stream Processor for demonstration
//! This version tests the web server and SSE functionality without external dependencies

use std::time::Duration;
use tokio::time::interval;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Json, sse::{Event, Sse}},
    routing::get,
    Router,
};
use serde_json::json;
use tokio::sync::broadcast;
use futures_util::Stream;
use tokio_stream::wrappers::BroadcastStream;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
struct SseEvent {
    event_type: String,
    data: String,
}

#[derive(Debug, Clone)]
struct AppState {
    sse_sender: broadcast::Sender<SseEvent>,
}

async fn health_handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp(),
        "service": "solana-stream-processor-demo"
    })))
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(json!({
        "service": "solana-stream-processor-demo",
        "version": "0.1.0",
        "endpoints": {
            "health": "/health",
            "sse_stream": "/events/stream"
        }
    }))
}

async fn sse_handler(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let receiver = state.sse_sender.subscribe();
    let stream = BroadcastStream::new(receiver);
    
    Sse::new(
        tokio_stream::StreamExt::map(stream, |result| {
            match result {
                Ok(sse_event) => {
                    Ok(Event::default()
                        .event(sse_event.event_type)
                        .data(sse_event.data))
                }
                Err(_) => {
                    Ok(Event::default().data("heartbeat"))
                }
            }
        })
    ).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("heartbeat")
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Solana Stream Processor Demo");

    // Create SSE broadcaster
    let (sse_sender, _) = broadcast::channel(1000);
    
    let app_state = AppState {
        sse_sender: sse_sender.clone(),
    };
    
    // Create the Axum app
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/events/stream", get(sse_handler))
        .with_state(app_state);
    
    // Start the demo data generator
    let demo_sender = sse_sender.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        let mut counter = 0;
        
        loop {
            interval.tick().await;
            counter += 1;
            
            let demo_data = json!({
                "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "token_mint": "So11111111111111111111111111111111111111112",
                "transaction_signature": format!("demo_signature_{}", counter),
                "instruction_type": "transfer",
                "instruction_data": {
                    "amount": 1000000 + counter * 100,
                    "source": "demo_source",
                    "destination": "demo_destination"
                },
                "blockchain_timestamp": chrono::Utc::now().timestamp(),
                "ingestion_timestamp": chrono::Utc::now().timestamp(),
                "slot": 123456789 + counter
            });
            
            let event = SseEvent {
                event_type: "instruction".to_string(),
                data: demo_data.to_string(),
            };
            
            match demo_sender.send(event) {
                Ok(receiver_count) => {
                    tracing::info!("Sent demo instruction #{} to {} receivers", counter, receiver_count);
                }
                Err(_) => {
                    tracing::debug!("No SSE receivers connected");
                }
            }
        }
    });
    
    // Start the server
    let port = 8080;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await?;
    
    tracing::info!("Demo server starting on port {}", port);
    tracing::info!("Visit:");
    tracing::info!("  - http://localhost:{}/", port);
    tracing::info!("  - http://localhost:{}/health", port);
    tracing::info!("  - http://localhost:{}/events/stream", port);
    tracing::info!("");
    tracing::info!("Test the SSE stream with:");
    tracing::info!("  curl -N -H \"Accept: text/event-stream\" http://localhost:{}/events/stream", port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}