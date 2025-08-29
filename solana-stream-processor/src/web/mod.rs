//! Web module for HTTP server, SSE streaming, and API endpoints

pub mod server;
pub mod sse;

pub use server::WebServer;
pub use sse::SseManager;