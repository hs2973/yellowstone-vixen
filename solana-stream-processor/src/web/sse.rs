//! SSE (Server-Sent Events) handler for real-time data streaming

use axum::response::sse::{Event, Sse};
use futures_util::Stream;
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, info, warn};

use crate::models::SseEvent;

/// SSE manager for handling Server-Sent Events
#[derive(Debug, Clone)]
pub struct SseManager {
    sender: broadcast::Sender<SseEvent>,
}

impl SseManager {
    /// Create a new SSE manager
    pub fn new(buffer_size: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer_size);
        Self { sender }
    }
    
    /// Get a sender for publishing events
    pub fn get_sender(&self) -> broadcast::Sender<SseEvent> {
        self.sender.clone()
    }
    
    /// Create a new SSE stream for a client
    pub fn create_stream(&self) -> impl Stream<Item = Result<Event, Infallible>> {
        info!("New SSE client connected");
        
        let receiver = self.sender.subscribe();
        let stream = BroadcastStream::new(receiver);
        
        SseEventStream { stream }
    }
}

/// SSE event stream wrapper
struct SseEventStream {
    stream: BroadcastStream<SseEvent>,
}

impl Stream for SseEventStream {
    type Item = Result<Event, Infallible>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(sse_event))) => {
                debug!("Sending SSE event: {}", sse_event.event_type);
                
                let event = Event::default()
                    .event(sse_event.event_type)
                    .data(sse_event.data);
                    
                let event = if let Some(id) = sse_event.id {
                    event.id(id)
                } else {
                    event
                };
                
                Poll::Ready(Some(Ok(event)))
            },
            Poll::Ready(Some(Err(e))) => {
                warn!("SSE stream error: {}", e);
                // Continue streaming despite errors
                Poll::Pending
            },
            Poll::Ready(None) => {
                debug!("SSE stream ended");
                Poll::Ready(None)
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create SSE response
pub fn create_sse_response(sse_manager: SseManager) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(sse_manager.create_stream())
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(30))
                .text("heartbeat")
        )
}