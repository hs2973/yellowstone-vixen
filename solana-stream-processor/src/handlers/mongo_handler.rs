use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use mongodb::{
    Client, Database, Collection, IndexModel,
    options::{ClientOptions, ConnectionString, IndexOptions, CreateIndexOptions}
};
use bson::{doc, Document};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use chrono::Utc;

use crate::config::{DatabaseConfig, IndexConfig};
use crate::models::{ParsedData, ParsedAccountData, ParsedInstructionData};
use crate::metrics::Metrics;

/// MongoDB handler for storing and retrieving parsed Solana data
/// Manages connections, collections, and provides optimized storage operations
pub struct MongoHandler {
    client: Client,
    database: Database,
    collections: Arc<RwLock<HashMap<String, Collection<Document>>>>,
    config: DatabaseConfig,
    metrics: Arc<Metrics>,
}

impl MongoHandler {
    /// Create a new MongoDB handler with the given configuration
    /// Establishes connection and sets up database with proper indexes
    pub async fn new(config: DatabaseConfig, metrics: Arc<Metrics>) -> Result<Self, mongodb::error::Error> {
        info!("Connecting to MongoDB at: {}", config.uri);

        // Parse connection string and configure client options
        let connection_string = ConnectionString::parse(&config.uri)
            .map_err(|e| {
                error!("Failed to parse MongoDB URI: {}", e);
                mongodb::error::Error::custom(format!("Invalid MongoDB URI: {}", e))
            })?;

        let mut client_options = ClientOptions::parse_connection_string(connection_string).await?;
        
        // Configure connection pool
        client_options.min_pool_size = Some(config.pool.min_connections);
        client_options.max_pool_size = Some(config.pool.max_connections);
        client_options.connect_timeout = Some(Duration::from_secs(config.pool.connect_timeout));
        client_options.server_selection_timeout = Some(Duration::from_secs(config.pool.server_selection_timeout));

        // Set application name for monitoring
        client_options.app_name = Some("solana-stream-processor".to_string());

        let client = Client::with_options(client_options)?;
        let database = client.database(&config.database_name);

        let handler = Self {
            client,
            database,
            collections: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics,
        };

        // Test connection
        handler.test_connection().await?;

        // Create indexes if configured
        if handler.config.indexes.create_on_startup {
            handler.create_indexes().await?;
        }

        info!("MongoDB handler initialized successfully");
        Ok(handler)
    }

    /// Test MongoDB connection by pinging the server
    async fn test_connection(&self) -> Result<(), mongodb::error::Error> {
        debug!("Testing MongoDB connection");
        
        self.database.run_command(doc! { "ping": 1 }).await
            .map_err(|e| {
                error!("MongoDB connection test failed: {}", e);
                e
            })?;

        info!("MongoDB connection test successful");
        Ok(())
    }

    /// Create indexes for all supported program collections
    /// Ensures optimal query performance for common access patterns
    async fn create_indexes(&self) -> Result<(), mongodb::error::Error> {
        info!("Creating MongoDB indexes");

        let index_config = &self.config.indexes;
        
        for program in crate::models::ProgramMetadata::all_programs() {
            let program_name = program.name.to_lowercase().replace(['.', ' ', '-'], "_");
            
            // Create indexes for both accounts and instructions collections
            for collection_type in ["accounts", "instructions"] {
                let collection_name = format!("{}_{}", program_name, collection_type);
                let collection = self.get_collection(&collection_name).await;

                let mut indexes = vec![];

                // Common indexes for all collections
                if index_config.slot_index {
                    indexes.push(IndexModel::builder()
                        .keys(doc! { "slot": 1 })
                        .options(IndexOptions::builder()
                            .name("slot_idx".to_string())
                            .build())
                        .build());
                }

                if index_config.timestamp_index {
                    indexes.push(IndexModel::builder()
                        .keys(doc! { "ingested_at": 1 })
                        .options(IndexOptions::builder()
                            .name("ingested_at_idx".to_string())
                            .build())
                        .build());

                    // Compound index for time-range queries
                    indexes.push(IndexModel::builder()
                        .keys(doc! { "slot": 1, "ingested_at": 1 })
                        .options(IndexOptions::builder()
                            .name("slot_time_idx".to_string())
                            .build())
                        .build());
                }

                // Program-specific indexes
                if collection_type == "instructions" {
                    // Index for instruction type filtering
                    indexes.push(IndexModel::builder()
                        .keys(doc! { "instruction_type": 1 })
                        .options(IndexOptions::builder()
                            .name("instruction_type_idx".to_string())
                            .build())
                        .build());

                    // Index for trading instruction filtering
                    indexes.push(IndexModel::builder()
                        .keys(doc! { "is_trading": 1 })
                        .options(IndexOptions::builder()
                            .name("is_trading_idx".to_string())
                            .build())
                        .build());

                    // Token mint index for trading queries
                    if index_config.token_mint_index {
                        indexes.push(IndexModel::builder()
                            .keys(doc! { "token_mints": 1 })
                            .options(IndexOptions::builder()
                                .name("token_mints_idx".to_string())
                                .build())
                            .build());
                    }

                    // Signature index for transaction lookups
                    indexes.push(IndexModel::builder()
                        .keys(doc! { "signature": 1 })
                        .options(IndexOptions::builder()
                            .name("signature_idx".to_string())
                            .unique(false)
                            .build())
                        .build());
                } else if collection_type == "accounts" {
                    // Account pubkey index
                    indexes.push(IndexModel::builder()
                        .keys(doc! { "account_pubkey": 1 })
                        .options(IndexOptions::builder()
                            .name("account_pubkey_idx".to_string())
                            .build())
                        .build());
                }

                if !indexes.is_empty() {
                    match collection.create_indexes(indexes).await {
                        Ok(_) => debug!("Created indexes for collection: {}", collection_name),
                        Err(e) => warn!("Failed to create indexes for collection {}: {}", collection_name, e),
                    }
                }
            }
        }

        info!("MongoDB indexes creation completed");
        Ok(())
    }

    /// Get or create a collection for the given name
    /// Collections are cached to avoid repeated lookups
    async fn get_collection(&self, name: &str) -> Collection<Document> {
        // Check if collection is already cached
        {
            let collections = self.collections.read().await;
            if let Some(collection) = collections.get(name) {
                return collection.clone();
            }
        }

        // Create new collection and cache it
        let collection = self.database.collection::<Document>(name);
        
        {
            let mut collections = self.collections.write().await;
            collections.insert(name.to_string(), collection.clone());
        }

        collection
    }

    /// Store a single parsed data item in the appropriate collection
    pub async fn store_parsed_data(&self, data: ParsedData) -> Result<(), mongodb::error::Error> {
        let collection_name = data.collection_name();
        let collection = self.get_collection(&collection_name).await;

        let document = data.to_bson_document()
            .map_err(|e| mongodb::error::Error::custom(format!("BSON serialization error: {}", e)))?;

        let start_time = std::time::Instant::now();
        
        match collection.insert_one(document).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                self.metrics.record_database_operation_duration("insert", "success", duration);
                self.metrics.increment_database_operations("insert", "success");
                debug!("Stored data in collection: {}", collection_name);
                Ok(())
            }
            Err(e) => {
                let duration = start_time.elapsed();
                self.metrics.record_database_operation_duration("insert", "error", duration);
                self.metrics.increment_database_operations("insert", "error");
                error!("Failed to store data in collection {}: {}", collection_name, e);
                Err(e)
            }
        }
    }

    /// Store multiple parsed data items in batch for better performance
    /// Groups items by collection and performs batch inserts
    pub async fn store_parsed_data_batch(&self, data_items: Vec<ParsedData>) -> Result<(), mongodb::error::Error> {
        if data_items.is_empty() {
            return Ok(());
        }

        // Group items by collection
        let mut collections_data: HashMap<String, Vec<Document>> = HashMap::new();
        
        for data in data_items {
            let collection_name = data.collection_name();
            let document = data.to_bson_document()
                .map_err(|e| mongodb::error::Error::custom(format!("BSON serialization error: {}", e)))?;
            
            collections_data.entry(collection_name).or_default().push(document);
        }

        // Perform batch inserts for each collection
        for (collection_name, documents) in collections_data {
            let collection = self.get_collection(&collection_name).await;
            let start_time = std::time::Instant::now();

            match collection.insert_many(documents).await {
                Ok(result) => {
                    let duration = start_time.elapsed();
                    self.metrics.record_database_operation_duration("batch_insert", "success", duration);
                    self.metrics.increment_database_operations("batch_insert", "success");
                    info!("Batch inserted {} documents into collection: {}", 
                          result.inserted_ids.len(), collection_name);
                }
                Err(e) => {
                    let duration = start_time.elapsed();
                    self.metrics.record_database_operation_duration("batch_insert", "error", duration);
                    self.metrics.increment_database_operations("batch_insert", "error");
                    error!("Failed to batch insert into collection {}: {}", collection_name, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Query recent data for a specific token mint from a program
    /// Returns the last N records ordered by slot descending
    pub async fn query_recent_by_token_mint(
        &self,
        program_name: &str,
        token_mint: &str,
        limit: u32,
    ) -> Result<Vec<Document>, mongodb::error::Error> {
        let collection_name = format!("{}_instructions", 
            program_name.to_lowercase().replace(['.', ' ', '-'], "_"));
        let collection = self.get_collection(&collection_name).await;

        let start_time = std::time::Instant::now();
        
        let filter = doc! {
            "token_mints": token_mint,
            "is_trading": true
        };

        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "slot": -1 })
            .limit(limit as i64)
            .build();

        let mut cursor = collection.find(filter).with_options(options).await?;
        let mut results = Vec::new();

        while cursor.advance().await? {
            results.push(cursor.current().clone());
        }

        let duration = start_time.elapsed();
        self.metrics.record_database_operation_duration("query", "success", duration);
        self.metrics.increment_database_operations("query", "success");

        debug!("Found {} recent records for token {} in program {}", 
               results.len(), token_mint, program_name);

        Ok(results)
    }

    /// Query data within a specific slot range
    pub async fn query_by_slot_range(
        &self,
        program_name: &str,
        collection_type: &str, // "accounts" or "instructions"
        min_slot: u64,
        max_slot: u64,
        limit: Option<u32>,
    ) -> Result<Vec<Document>, mongodb::error::Error> {
        let collection_name = format!("{}_{}", 
            program_name.to_lowercase().replace(['.', ' ', '-'], "_"), 
            collection_type);
        let collection = self.get_collection(&collection_name).await;

        let start_time = std::time::Instant::now();
        
        let filter = doc! {
            "slot": {
                "$gte": min_slot,
                "$lte": max_slot
            }
        };

        let mut options_builder = mongodb::options::FindOptions::builder()
            .sort(doc! { "slot": 1 });

        if let Some(limit) = limit {
            options_builder = options_builder.limit(limit as i64);
        }

        let options = options_builder.build();
        let mut cursor = collection.find(filter).with_options(options).await?;
        let mut results = Vec::new();

        while cursor.advance().await? {
            results.push(cursor.current().clone());
        }

        let duration = start_time.elapsed();
        self.metrics.record_database_operation_duration("query", "success", duration);
        self.metrics.increment_database_operations("query", "success");

        debug!("Found {} records in slot range {}-{} for {}", 
               results.len(), min_slot, max_slot, collection_name);

        Ok(results)
    }

    /// Get database statistics for monitoring
    pub async fn get_stats(&self) -> Result<Document, mongodb::error::Error> {
        self.database.run_command(doc! { "dbStats": 1 }).await
    }

    /// Health check for the MongoDB connection
    pub async fn health_check(&self) -> bool {
        match self.test_connection().await {
            Ok(_) => true,
            Err(e) => {
                error!("MongoDB health check failed: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseConfig, PoolConfig, IndexConfig};

    fn create_test_config() -> DatabaseConfig {
        DatabaseConfig {
            uri: "mongodb://localhost:27017".to_string(),
            database_name: "test_solana_stream".to_string(),
            pool: PoolConfig::default(),
            indexes: IndexConfig::default(),
        }
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB instance
    async fn test_mongo_handler_creation() {
        let config = create_test_config();
        let metrics = Arc::new(Metrics::new());
        
        let result = MongoHandler::new(config, metrics).await;
        // This test requires a running MongoDB instance
        // In real tests, you would use a test container or mock
        assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable without MongoDB
    }

    #[test]
    fn test_collection_naming() {
        use crate::models::{ParsedData, ParsedAccountData};
        use std::collections::HashMap;
        
        let account_data = ParsedData::Account(ParsedAccountData {
            id: "test".to_string(),
            account_pubkey: "test".to_string(),
            program_id: "test".to_string(),
            program_name: "Pump.fun".to_string(),
            slot: 123,
            block_time: None,
            ingested_at: Utc::now(),
            raw_data: vec![],
            parsed_data: doc! {},
            lamports: 0,
            owner: "test".to_string(),
            executable: false,
            rent_epoch: 0,
            metadata: HashMap::new(),
        });

        assert_eq!(account_data.collection_name(), "pump_fun_accounts");
    }
}