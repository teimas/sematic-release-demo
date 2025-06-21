//! AI domain repository ports
//! 
//! Port definitions for AI infrastructure dependencies.
//! These define contracts for external AI services and storage.

use crate::domains::ai::{
    errors::AiDomainError,
    entities::{
        AnalysisResult, ConversationSession, ConversationMessage, BatchAnalysisOperation
    },
    value_objects::{
        AiProvider, AiModel, PromptTemplate, AnalysisRequest, AnalysisType,
        UsageMetrics, CacheConfig, ModelPreferences
    }
};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// Port for AI provider integrations
#[async_trait]
pub trait AiProviderPort: Send + Sync {
    /// Executes an analysis request using the AI provider
    async fn execute_analysis(&self, request: AnalysisRequest) -> Result<AnalysisResult, AiDomainError>;
    
    /// Validates AI provider credentials and connectivity
    async fn validate_connection(&self) -> Result<(), AiDomainError>;
    
    /// Gets available models for the provider
    async fn get_available_models(&self) -> Result<Vec<AiModel>, AiDomainError>;
    
    /// Gets current usage metrics for the provider
    async fn get_usage_metrics(&self, period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Result<UsageMetrics, AiDomainError>;
    
    /// Tests a prompt template with sample data
    async fn test_prompt_template(&self, template: &PromptTemplate, variables: &HashMap<String, String>) -> Result<String, AiDomainError>;
    
    /// Estimates cost for a given request
    async fn estimate_cost(&self, request: &AnalysisRequest) -> Result<f64, AiDomainError>;
    
    /// Gets provider-specific rate limits
    async fn get_rate_limits(&self) -> Result<RateLimits, AiDomainError>;
    
    /// Checks provider health status
    async fn health_check(&self) -> Result<ProviderHealth, AiDomainError>;
}

/// Port for prompt template management
#[async_trait]
pub trait PromptTemplatePort: Send + Sync {
    /// Saves a prompt template
    async fn save_template(&self, template: PromptTemplate) -> Result<(), AiDomainError>;
    
    /// Gets a prompt template by name
    async fn get_template(&self, name: &str) -> Result<Option<PromptTemplate>, AiDomainError>;
    
    /// Lists all available templates
    async fn list_templates(&self) -> Result<Vec<PromptTemplate>, AiDomainError>;
    
    /// Lists templates by analysis type
    async fn list_templates_by_type(&self, analysis_type: AnalysisType) -> Result<Vec<PromptTemplate>, AiDomainError>;
    
    /// Updates an existing template
    async fn update_template(&self, template: PromptTemplate) -> Result<(), AiDomainError>;
    
    /// Deletes a template by name
    async fn delete_template(&self, name: &str) -> Result<(), AiDomainError>;
    
    /// Validates a template
    async fn validate_template(&self, template: &PromptTemplate) -> Result<(), AiDomainError>;
    
    /// Gets template usage statistics
    async fn get_template_usage_stats(&self, name: &str) -> Result<TemplateUsageStats, AiDomainError>;
}

/// Port for analysis result storage and retrieval
#[async_trait]
pub trait AnalysisResultPort: Send + Sync {
    /// Saves an analysis result
    async fn save_result(&self, result: AnalysisResult) -> Result<(), AiDomainError>;
    
    /// Gets an analysis result by ID
    async fn get_result(&self, id: &str) -> Result<Option<AnalysisResult>, AiDomainError>;
    
    /// Lists results by analysis type
    async fn list_results_by_type(&self, analysis_type: AnalysisType, limit: usize, offset: usize) -> Result<Vec<AnalysisResult>, AiDomainError>;
    
    /// Lists recent results
    async fn list_recent_results(&self, limit: usize) -> Result<Vec<AnalysisResult>, AiDomainError>;
    
    /// Searches results by content
    async fn search_results(&self, query: &str, limit: usize) -> Result<Vec<AnalysisResult>, AiDomainError>;
    
    /// Updates an analysis result
    async fn update_result(&self, result: AnalysisResult) -> Result<(), AiDomainError>;
    
    /// Deletes results older than the specified date
    async fn cleanup_old_results(&self, older_than: DateTime<Utc>) -> Result<usize, AiDomainError>;
    
    /// Gets analysis statistics
    async fn get_analysis_statistics(&self, period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Result<AnalysisStatistics, AiDomainError>;
}

/// Port for conversation management
#[async_trait]
pub trait ConversationPort: Send + Sync {
    /// Saves a conversation session
    async fn save_conversation(&self, conversation: ConversationSession) -> Result<(), AiDomainError>;
    
    /// Gets a conversation by ID
    async fn get_conversation(&self, id: &str) -> Result<Option<ConversationSession>, AiDomainError>;
    
    /// Lists recent conversations
    async fn list_recent_conversations(&self, limit: usize) -> Result<Vec<ConversationSession>, AiDomainError>;
    
    /// Updates a conversation
    async fn update_conversation(&self, conversation: ConversationSession) -> Result<(), AiDomainError>;
    
    /// Deletes a conversation
    async fn delete_conversation(&self, id: &str) -> Result<(), AiDomainError>;
    
    /// Adds a message to a conversation
    async fn add_message(&self, conversation_id: &str, message: ConversationMessage) -> Result<(), AiDomainError>;
    
    /// Gets conversation messages with pagination
    async fn get_messages(&self, conversation_id: &str, limit: usize, offset: usize) -> Result<Vec<ConversationMessage>, AiDomainError>;
    
    /// Searches conversations
    async fn search_conversations(&self, query: &str, limit: usize) -> Result<Vec<ConversationSession>, AiDomainError>;
}

/// Port for batch operation management
#[async_trait]
pub trait BatchOperationPort: Send + Sync {
    /// Saves a batch operation
    async fn save_batch_operation(&self, operation: BatchAnalysisOperation) -> Result<(), AiDomainError>;
    
    /// Gets a batch operation by ID
    async fn get_batch_operation(&self, id: &str) -> Result<Option<BatchAnalysisOperation>, AiDomainError>;
    
    /// Lists active batch operations
    async fn list_active_operations(&self) -> Result<Vec<BatchAnalysisOperation>, AiDomainError>;
    
    /// Lists batch operations by status
    async fn list_operations_by_status(&self, status: &str) -> Result<Vec<BatchAnalysisOperation>, AiDomainError>;
    
    /// Updates a batch operation
    async fn update_batch_operation(&self, operation: BatchAnalysisOperation) -> Result<(), AiDomainError>;
    
    /// Deletes a batch operation
    async fn delete_batch_operation(&self, id: &str) -> Result<(), AiDomainError>;
    
    /// Gets batch operation statistics
    async fn get_batch_statistics(&self) -> Result<BatchStatistics, AiDomainError>;
}

/// Port for AI caching operations
#[async_trait]
pub trait AiCachePort: Send + Sync {
    /// Gets a cached analysis result
    async fn get_cached_result(&self, cache_key: &str) -> Result<Option<AnalysisResult>, AiDomainError>;
    
    /// Caches an analysis result
    async fn cache_result(&self, cache_key: &str, result: AnalysisResult, ttl: Duration) -> Result<(), AiDomainError>;
    
    /// Invalidates cached results by pattern
    async fn invalidate_cache(&self, pattern: &str) -> Result<usize, AiDomainError>;
    
    /// Gets cache statistics
    async fn get_cache_statistics(&self) -> Result<CacheStatistics, AiDomainError>;
    
    /// Clears expired cache entries
    async fn cleanup_expired_entries(&self) -> Result<usize, AiDomainError>;
    
    /// Generates cache key for an analysis request
    fn generate_cache_key(&self, request: &AnalysisRequest) -> String;
}

/// Port for AI configuration management
#[async_trait]
pub trait AiConfigurationPort: Send + Sync {
    /// Gets AI provider configurations
    async fn get_provider_configs(&self) -> Result<Vec<ProviderConfig>, AiDomainError>;
    
    /// Saves provider configuration
    async fn save_provider_config(&self, config: ProviderConfig) -> Result<(), AiDomainError>;
    
    /// Gets cache configuration
    async fn get_cache_config(&self) -> Result<CacheConfig, AiDomainError>;
    
    /// Updates cache configuration
    async fn update_cache_config(&self, config: CacheConfig) -> Result<(), AiDomainError>;
    
    /// Gets model preferences
    async fn get_model_preferences(&self) -> Result<ModelPreferences, AiDomainError>;
    
    /// Updates model preferences
    async fn update_model_preferences(&self, preferences: ModelPreferences) -> Result<(), AiDomainError>;
    
    /// Validates configuration
    async fn validate_configuration(&self) -> Result<ConfigurationValidation, AiDomainError>;
}

/// Provider rate limits information
#[derive(Debug, Clone)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub tokens_per_minute: u32,
    pub tokens_per_hour: u32,
    pub tokens_per_day: u32,
    pub concurrent_requests: u32,
}

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub error_rate: f32,
    pub last_checked: DateTime<Utc>,
    pub status_message: String,
}

/// Template usage statistics
#[derive(Debug, Clone)]
pub struct TemplateUsageStats {
    pub template_name: String,
    pub total_uses: u64,
    pub successful_uses: u64,
    pub average_confidence: f32,
    pub average_cost: f64,
    pub last_used: Option<DateTime<Utc>>,
}

/// Analysis statistics
#[derive(Debug, Clone)]
pub struct AnalysisStatistics {
    pub total_analyses: u64,
    pub successful_analyses: u64,
    pub failed_analyses: u64,
    pub average_confidence: f32,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub average_response_time: Duration,
    pub analyses_by_type: HashMap<AnalysisType, u64>,
}

/// Batch operation statistics
#[derive(Debug, Clone)]
pub struct BatchStatistics {
    pub total_operations: u64,
    pub active_operations: u64,
    pub completed_operations: u64,
    pub failed_operations: u64,
    pub average_completion_time: Duration,
    pub total_requests_processed: u64,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_entries: u64,
    pub hit_ratio: f32,
    pub miss_ratio: f32,
    pub expired_entries: u64,
    pub cache_size_bytes: u64,
    pub oldest_entry_age: Option<Duration>,
}

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider: AiProvider,
    pub api_key: String,
    pub api_endpoint: Option<String>,
    pub default_model: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub enabled: bool,
    pub priority: u8,
}

/// Configuration validation results
#[derive(Debug, Clone)]
pub struct ConfigurationValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub provider_validations: HashMap<String, ProviderValidation>,
}

/// Provider-specific validation results
#[derive(Debug, Clone)]
pub struct ProviderValidation {
    pub provider_name: String,
    pub is_available: bool,
    pub connection_status: ConnectionStatus,
    pub model_availability: HashMap<String, bool>,
    pub rate_limit_status: RateLimitStatus,
}

/// Connection status for providers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    AuthenticationFailed,
    RateLimited,
    UnknownError(String),
}

/// Rate limit status
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub within_limits: bool,
    pub current_usage: u32,
    pub limit: u32,
    pub reset_time: Option<DateTime<Utc>>,
} 