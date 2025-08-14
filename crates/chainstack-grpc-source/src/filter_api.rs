//! Real-time filter configuration API for dynamic filter management
//!
//! This module provides a REST API interface for managing Yellowstone gRPC filters
//! in real-time without requiring service restarts. It enables dynamic adjustment
//! of filter rules for accounts, transactions, and programs.

use crate::config::ChainstackVixenConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use warp::{Filter, Reply};
use yellowstone_vixen_core::{Prefilter, AccountPrefilter, TransactionPrefilter, Pubkey};

/// Real-time filter management API server
#[derive(Debug)]
pub struct FilterApiServer {
    config: ChainstackVixenConfig,
    filter_manager: Arc<RwLock<FilterManager>>,
    port: u16,
    auth_token: Option<String>,
}

/// Dynamic filter manager for runtime filter updates
#[derive(Debug)]
pub struct FilterManager {
    active_filters: HashMap<String, DynamicFilter>,
    filter_templates: HashMap<String, FilterTemplate>,
    update_callbacks: Vec<FilterUpdateCallback>,
}

/// Dynamic filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicFilter {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub account_filter: Option<DynamicAccountFilter>,
    pub transaction_filter: Option<DynamicTransactionFilter>,
    pub program_filter: Option<DynamicProgramFilter>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: FilterMetadata,
}

/// Dynamic account filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAccountFilter {
    pub accounts: HashSet<String>,
    pub owners: HashSet<String>,
    pub data_size_range: Option<(u64, u64)>,
    pub lamports_range: Option<(u64, u64)>,
    pub exclude_accounts: HashSet<String>,
    pub include_executable: Option<bool>,
}

/// Dynamic transaction filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTransactionFilter {
    pub accounts_include: HashSet<String>,
    pub accounts_required: HashSet<String>,
    pub accounts_exclude: HashSet<String>,
    pub programs_include: HashSet<String>,
    pub programs_exclude: HashSet<String>,
    pub fee_range: Option<(u64, u64)>,
    pub include_failed: bool,
    pub signature_patterns: Vec<String>,
}

/// Dynamic program filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicProgramFilter {
    pub program_ids: HashSet<String>,
    pub instruction_patterns: Vec<InstructionPattern>,
    pub account_patterns: Vec<AccountPattern>,
    pub exclude_programs: HashSet<String>,
}

/// Instruction pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionPattern {
    pub program_id: String,
    pub instruction_data_prefix: Option<String>,
    pub accounts_count_range: Option<(usize, usize)>,
    pub description: String,
}

/// Account pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPattern {
    pub owner: String,
    pub data_size_range: Option<(u64, u64)>,
    pub data_prefix: Option<String>,
    pub description: String,
}

/// Filter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterMetadata {
    pub description: String,
    pub tags: HashMap<String, String>,
    pub priority: u32,
    pub use_case: String,
    pub created_by: String,
}

/// Filter template for common filter patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template: DynamicFilter,
    pub parameters: Vec<TemplateParameter>,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub parameter_type: ParameterType,
    pub description: String,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ParameterValidation>,
}

/// Parameter types for template parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    PublicKey,
    Range,
}

/// Parameter validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
}

/// Filter update callback type
type FilterUpdateCallback = Box<dyn Fn(&DynamicFilter) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;

/// API request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFilterRequest {
    pub filter: DynamicFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFilterRequest {
    pub filter: DynamicFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterListResponse {
    pub filters: Vec<DynamicFilter>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterResponse {
    pub filter: DynamicFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateListResponse {
    pub templates: Vec<FilterTemplate>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u32,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl FilterApiServer {
    /// Create a new filter API server
    pub fn new(config: ChainstackVixenConfig, port: u16, auth_token: Option<String>) -> Self {
        let filter_manager = Arc::new(RwLock::new(FilterManager::new()));
        
        Self {
            config,
            filter_manager,
            port,
            auth_token,
        }
    }

    /// Start the filter API server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(port = self.port, "Starting filter API server");

        let filter_manager = Arc::clone(&self.filter_manager);
        let auth_token = self.auth_token.clone();

        // Create API routes
        let routes = self.create_routes(filter_manager, auth_token);

        // Start the server
        warp::serve(routes)
            .run(([0, 0, 0, 0], self.port))
            .await;

        Ok(())
    }

    /// Create API routes
    fn create_routes(
        &self,
        filter_manager: Arc<RwLock<FilterManager>>,
        auth_token: Option<String>,
    ) -> impl Filter<Extract = impl Reply> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        // Authentication filter
        let auth_filter = warp::header::optional::<String>("authorization")
            .and_then(move |token: Option<String>| {
                let auth_token = auth_token.clone();
                async move {
                    if let Some(required_token) = auth_token {
                        if let Some(provided_token) = token {
                            if provided_token == format!("Bearer {}", required_token) {
                                Ok(())
                            } else {
                                Err(warp::reject::custom(UnauthorizedError))
                            }
                        } else {
                            Err(warp::reject::custom(UnauthorizedError))
                        }
                    } else {
                        Ok(())
                    }
                }
            });

        // Health check endpoint
        let health = warp::path("health")
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

        // List filters endpoint
        let list_filters = warp::path("filters")
            .and(warp::get())
            .and(auth_filter.clone())
            .and(warp::query::<HashMap<String, String>>())
            .and_then({
                let filter_manager = Arc::clone(&filter_manager);
                move |_: (), query: HashMap<String, String>| {
                    let filter_manager = Arc::clone(&filter_manager);
                    async move {
                        let page = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
                        let per_page = query.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20);
                        
                        let manager = filter_manager.read().await;
                        let filters: Vec<DynamicFilter> = manager.active_filters.values().cloned().collect();
                        let total = filters.len();
                        
                        let start = ((page - 1) * per_page).min(total);
                        let end = (start + per_page).min(total);
                        let page_filters = filters[start..end].to_vec();
                        
                        let response = FilterListResponse {
                            filters: page_filters,
                            total,
                            page,
                            per_page,
                        };
                        
                        Ok::<_, warp::Rejection>(warp::reply::json(&response))
                    }
                }
            });

        // Get filter endpoint
        let get_filter = warp::path!("filters" / String)
            .and(warp::get())
            .and(auth_filter.clone())
            .and_then({
                let filter_manager = Arc::clone(&filter_manager);
                move |filter_id: String, _: ()| {
                    let filter_manager = Arc::clone(&filter_manager);
                    async move {
                        let manager = filter_manager.read().await;
                        if let Some(filter) = manager.active_filters.get(&filter_id) {
                            Ok(warp::reply::json(&FilterResponse { filter: filter.clone() }))
                        } else {
                            Err(warp::reject::not_found())
                        }
                    }
                }
            });

        // Create filter endpoint
        let create_filter = warp::path("filters")
            .and(warp::post())
            .and(auth_filter.clone())
            .and(warp::body::json())
            .and_then({
                let filter_manager = Arc::clone(&filter_manager);
                move |_: (), request: CreateFilterRequest| {
                    let filter_manager = Arc::clone(&filter_manager);
                    async move {
                        let mut manager = filter_manager.write().await;
                        let mut filter = request.filter;
                        filter.created_at = chrono::Utc::now();
                        filter.updated_at = chrono::Utc::now();
                        
                        if let Err(e) = manager.add_filter(filter.clone()).await {
                            error!(error = ?e, "Failed to add filter");
                            return Err(warp::reject::custom(InternalServerError));
                        }
                        
                        Ok::<_, warp::Rejection>(warp::reply::json(&FilterResponse { filter }))
                    }
                }
            });

        // Update filter endpoint
        let update_filter = warp::path!("filters" / String)
            .and(warp::put())
            .and(auth_filter.clone())
            .and(warp::body::json())
            .and_then({
                let filter_manager = Arc::clone(&filter_manager);
                move |filter_id: String, _: (), request: UpdateFilterRequest| {
                    let filter_manager = Arc::clone(&filter_manager);
                    async move {
                        let mut manager = filter_manager.write().await;
                        let mut filter = request.filter;
                        filter.id = filter_id.clone();
                        filter.updated_at = chrono::Utc::now();
                        
                        if let Err(e) = manager.update_filter(filter.clone()).await {
                            error!(error = ?e, "Failed to update filter");
                            return Err(warp::reject::custom(InternalServerError));
                        }
                        
                        Ok::<_, warp::Rejection>(warp::reply::json(&FilterResponse { filter }))
                    }
                }
            });

        // Delete filter endpoint
        let delete_filter = warp::path!("filters" / String)
            .and(warp::delete())
            .and(auth_filter.clone())
            .and_then({
                let filter_manager = Arc::clone(&filter_manager);
                move |filter_id: String, _: ()| {
                    let filter_manager = Arc::clone(&filter_manager);
                    async move {
                        let mut manager = filter_manager.write().await;
                        if manager.active_filters.remove(&filter_id).is_some() {
                            if let Err(e) = manager.remove_filter(&filter_id).await {
                                error!(error = ?e, "Failed to remove filter");
                            }
                            Ok(warp::reply::with_status("", warp::http::StatusCode::NO_CONTENT))
                        } else {
                            Err(warp::reject::not_found())
                        }
                    }
                }
            });

        // List templates endpoint
        let list_templates = warp::path("templates")
            .and(warp::get())
            .and(auth_filter.clone())
            .and_then({
                let filter_manager = Arc::clone(&filter_manager);
                move |_: ()| {
                    let filter_manager = Arc::clone(&filter_manager);
                    async move {
                        let manager = filter_manager.read().await;
                        let templates: Vec<FilterTemplate> = manager.filter_templates.values().cloned().collect();
                        let response = TemplateListResponse {
                            total: templates.len(),
                            templates,
                        };
                        
                        Ok::<_, warp::Rejection>(warp::reply::json(&response))
                    }
                }
            });

        health
            .or(list_filters)
            .or(get_filter)
            .or(create_filter)
            .or(update_filter)
            .or(delete_filter)
            .or(list_templates)
            .with(cors)
            .recover(handle_rejection)
    }
}

impl FilterManager {
    fn new() -> Self {
        let mut manager = Self {
            active_filters: HashMap::new(),
            filter_templates: HashMap::new(),
            update_callbacks: Vec::new(),
        };

        // Initialize with default templates
        manager.initialize_default_templates();
        
        manager
    }

    /// Add a new filter
    async fn add_filter(&mut self, filter: DynamicFilter) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(filter_id = %filter.id, "Adding new filter");
        
        // Validate filter
        self.validate_filter(&filter)?;
        
        // Add to active filters
        self.active_filters.insert(filter.id.clone(), filter.clone());
        
        // Notify callbacks
        self.notify_callbacks(&filter)?;
        
        Ok(())
    }

    /// Update an existing filter
    async fn update_filter(&mut self, filter: DynamicFilter) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(filter_id = %filter.id, "Updating filter");
        
        // Validate filter
        self.validate_filter(&filter)?;
        
        // Update active filters
        self.active_filters.insert(filter.id.clone(), filter.clone());
        
        // Notify callbacks
        self.notify_callbacks(&filter)?;
        
        Ok(())
    }

    /// Remove a filter
    async fn remove_filter(&mut self, filter_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(filter_id = %filter_id, "Removing filter");
        
        // TODO: Notify about filter removal
        
        Ok(())
    }

    /// Convert dynamic filter to Vixen prefilter
    pub fn to_prefilter(&self, dynamic_filter: &DynamicFilter) -> Result<Prefilter, Box<dyn std::error::Error + Send + Sync>> {
        let mut builder = Prefilter::builder();

        // Convert account filter
        if let Some(ref account_filter) = dynamic_filter.account_filter {
            let accounts: Result<Vec<Pubkey>, _> = account_filter.accounts
                .iter()
                .map(|s| s.parse())
                .collect();
            
            let owners: Result<Vec<Pubkey>, _> = account_filter.owners
                .iter()
                .map(|s| s.parse())
                .collect();

            if !account_filter.accounts.is_empty() {
                builder = builder.accounts(accounts?);
            }
            
            if !account_filter.owners.is_empty() {
                builder = builder.account_owners(owners?);
            }
        }

        // Convert transaction filter
        if let Some(ref tx_filter) = dynamic_filter.transaction_filter {
            let accounts_include: Result<Vec<Pubkey>, _> = tx_filter.accounts_include
                .iter()
                .map(|s| s.parse())
                .collect();
            
            let accounts_required: Result<Vec<Pubkey>, _> = tx_filter.accounts_required
                .iter()
                .map(|s| s.parse())
                .collect();

            if !tx_filter.accounts_include.is_empty() {
                builder = builder.transaction_accounts_include(accounts_include?);
            }
            
            if !tx_filter.accounts_required.is_empty() {
                builder = builder.transaction_accounts(accounts_required?);
            }
        }

        Ok(builder.build()?)
    }

    fn validate_filter(&self, filter: &DynamicFilter) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Validate filter ID
        if filter.id.is_empty() {
            return Err("Filter ID cannot be empty".into());
        }

        // Validate account filter
        if let Some(ref account_filter) = filter.account_filter {
            for account in &account_filter.accounts {
                account.parse::<Pubkey>()?;
            }
            for owner in &account_filter.owners {
                owner.parse::<Pubkey>()?;
            }
        }

        // Validate transaction filter
        if let Some(ref tx_filter) = filter.transaction_filter {
            for account in &tx_filter.accounts_include {
                account.parse::<Pubkey>()?;
            }
            for account in &tx_filter.accounts_required {
                account.parse::<Pubkey>()?;
            }
        }

        Ok(())
    }

    fn notify_callbacks(&self, filter: &DynamicFilter) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for callback in &self.update_callbacks {
            if let Err(e) = callback(filter) {
                warn!(error = ?e, "Filter update callback failed");
            }
        }
        Ok(())
    }

    fn initialize_default_templates(&mut self) {
        // Token trading template
        let token_trading_template = FilterTemplate {
            id: "token_trading".to_string(),
            name: "Token Trading".to_string(),
            description: "Monitor token trading activities".to_string(),
            template: DynamicFilter {
                id: "template_token_trading".to_string(),
                name: "Token Trading Template".to_string(),
                enabled: true,
                account_filter: None,
                transaction_filter: Some(DynamicTransactionFilter {
                    accounts_include: HashSet::new(),
                    accounts_required: HashSet::new(),
                    accounts_exclude: HashSet::new(),
                    programs_include: [
                        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(), // Token Program
                        "11111111111111111111111111111111".to_string(), // System Program
                    ].into_iter().collect(),
                    programs_exclude: HashSet::new(),
                    fee_range: None,
                    include_failed: false,
                    signature_patterns: Vec::new(),
                }),
                program_filter: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: FilterMetadata {
                    description: "Template for monitoring token trading activities".to_string(),
                    tags: [("category".to_string(), "trading".to_string())].into_iter().collect(),
                    priority: 100,
                    use_case: "trading".to_string(),
                    created_by: "system".to_string(),
                },
            },
            parameters: vec![
                TemplateParameter {
                    name: "token_mint".to_string(),
                    parameter_type: ParameterType::PublicKey,
                    description: "Token mint address to monitor".to_string(),
                    default_value: None,
                    validation: Some(ParameterValidation {
                        required: true,
                        min_length: None,
                        max_length: None,
                        min_value: None,
                        max_value: None,
                        pattern: None,
                    }),
                },
            ],
        };

        self.filter_templates.insert("token_trading".to_string(), token_trading_template);

        // More templates can be added here...
    }
}

// Error types for API
#[derive(Debug)]
struct UnauthorizedError;
impl warp::reject::Reject for UnauthorizedError {}

#[derive(Debug)]
struct InternalServerError;
impl warp::reject::Reject for InternalServerError {}

/// Handle API rejections
async fn handle_rejection(err: warp::Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "Not found".to_string();
    } else if err.find::<UnauthorizedError>().is_some() {
        code = warp::http::StatusCode::UNAUTHORIZED;
        message = "Unauthorized".to_string();
    } else if err.find::<InternalServerError>().is_some() {
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal server error".to_string();
    } else {
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal server error".to_string();
    }

    let error_response = ErrorResponse {
        error: message,
        code: code.as_u16() as u32,
        details: None,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&error_response),
        code,
    ))
}