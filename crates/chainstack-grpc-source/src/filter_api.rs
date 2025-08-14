//! Filter API for real-time filter management
//!
//! This module provides a REST API for managing filters on the Chainstack gRPC source
//! in real-time, allowing dynamic adjustment of filter rules without restart.

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{error, info};
use yellowstone_vixen_core::{Prefilter, PrefilterBuilder};

/// Filter API server state
#[derive(Debug, Clone)]
pub struct FilterApiState {
    /// Current filters
    pub filters: Arc<RwLock<HashMap<String, Prefilter>>>,
    /// Channel to send filter updates to the runtime
    pub filter_update_tx: Arc<mpsc::UnboundedSender<FilterUpdate>>,
}

/// Filter update message sent to the runtime
#[derive(Debug, Clone)]
pub struct FilterUpdate {
    /// Parser ID for the filter
    pub parser_id: String,
    /// New prefilter to apply
    pub prefilter: Prefilter,
}

/// Request body for updating filters
#[derive(Debug, Deserialize)]
pub struct UpdateFilterRequest {
    /// Parser ID to update filter for
    pub parser_id: String,
    /// Account filters
    pub accounts: Option<Vec<String>>,
    /// Account owner filters
    pub account_owners: Option<Vec<String>>,
    /// Transaction account include filters
    pub transaction_accounts_include: Option<Vec<String>>,
    /// Transaction account required filters
    pub transaction_accounts_required: Option<Vec<String>>,
}

/// Response body for filter operations
#[derive(Debug, Serialize)]
pub struct FilterResponse {
    /// Success status
    pub success: bool,
    /// Response message
    pub message: String,
    /// Current filter count
    pub filter_count: usize,
}

/// Get current filters
async fn get_filters(State(state): State<FilterApiState>) -> Json<HashMap<String, String>> {
    let filters = state.filters.read().unwrap();
    let filter_summary: HashMap<String, String> = filters
        .iter()
        .map(|(k, _v)| (k.clone(), "active".to_string()))
        .collect();
    Json(filter_summary)
}

/// Update a filter
async fn update_filter(
    State(state): State<FilterApiState>,
    Json(request): Json<UpdateFilterRequest>,
) -> Result<Json<FilterResponse>, StatusCode> {
    let parser_id = request.parser_id.clone();
    
    // Build new prefilter from request
    let mut builder = PrefilterBuilder::default();
    
    if let Some(accounts) = request.accounts {
        builder = builder.accounts(accounts.iter().map(|s| s.as_bytes()));
    }
    
    if let Some(owners) = request.account_owners {
        builder = builder.account_owners(owners.iter().map(|s| s.as_bytes()));
    }
    
    if let Some(tx_include) = request.transaction_accounts_include {
        builder = builder.transaction_accounts_include(tx_include.iter().map(|s| s.as_bytes()));
    }
    
    if let Some(tx_required) = request.transaction_accounts_required {
        builder = builder.transaction_accounts(tx_required.iter().map(|s| s.as_bytes()));
    }
    
    let prefilter = builder.build().map_err(|e| {
        error!(error = ?e, "Failed to build prefilter");
        StatusCode::BAD_REQUEST
    })?;
    
    // Update local filter state
    {
        let mut filters = state.filters.write().unwrap();
        filters.insert(parser_id.clone(), prefilter.clone());
    }
    
    // Send update to runtime
    let update = FilterUpdate {
        parser_id: parser_id.clone(),
        prefilter,
    };
    
    if let Err(e) = state.filter_update_tx.send(update) {
        error!(error = ?e, "Failed to send filter update to runtime");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let filter_count = state.filters.read().unwrap().len();
    Ok(Json(FilterResponse {
        success: true,
        message: format!("Filter updated for parser: {}", parser_id),
        filter_count,
    }))
}

/// Remove a filter
async fn remove_filter(
    State(state): State<FilterApiState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<FilterResponse>, StatusCode> {
    let parser_id = request["parser_id"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // Remove from local state
    let removed = {
        let mut filters = state.filters.write().unwrap();
        filters.remove(parser_id).is_some()
    };
    
    if !removed {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Send removal to runtime (empty prefilter)
    let update = FilterUpdate {
        parser_id: parser_id.to_string(),
        prefilter: Prefilter::default(),
    };
    
    if let Err(e) = state.filter_update_tx.send(update) {
        error!(error = ?e, "Failed to send filter removal to runtime");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let filter_count = state.filters.read().unwrap().len();
    Ok(Json(FilterResponse {
        success: true,
        message: format!("Filter removed for parser: {}", parser_id),
        filter_count,
    }))
}

/// Create and start the Filter API server
pub async fn start_filter_api_server(
    addr: SocketAddr,
    filter_update_tx: mpsc::UnboundedSender<FilterUpdate>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = FilterApiState {
        filters: Arc::new(RwLock::new(HashMap::new())),
        filter_update_tx: Arc::new(filter_update_tx),
    };
    
    let app = Router::new()
        .route("/filters", get(get_filters))
        .route("/filters/update", post(update_filter))
        .route("/filters/remove", post(remove_filter))
        .with_state(state);
    
    info!(addr = %addr, "Starting Filter API server");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
