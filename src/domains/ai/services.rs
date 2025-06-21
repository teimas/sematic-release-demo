//! AI domain services
//! 
//! Domain services that orchestrate AI operations and enforce business rules.
//! These services coordinate between entities and repository ports.

use crate::domains::ai::{
    errors::AiDomainError,
    entities::{
        AnalysisResult, AnalysisStatus, ConversationSession, ConversationMessage,
        BatchAnalysisOperation, BatchStatus, MessageRole
    },
    value_objects::{
        AiProvider, AiModel, PromptTemplate, AnalysisRequest, AnalysisType,
        AnalysisOptions, ModelPreferences, UsageMetrics, FallbackStrategy,
        QualityPreference
    },
    repository::{
        AiProviderPort, PromptTemplatePort, AnalysisResultPort, ConversationPort,
        BatchOperationPort, AiCachePort, AiConfigurationPort, RateLimits,
        ProviderHealth
    }
};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Main AI domain service orchestrating all AI operations
pub struct AiDomainService {
    providers: HashMap<String, Arc<dyn AiProviderPort>>,
    template_port: Arc<dyn PromptTemplatePort>,
    result_port: Arc<dyn AnalysisResultPort>,
    conversation_port: Arc<dyn ConversationPort>,
    batch_port: Arc<dyn BatchOperationPort>,
    cache_port: Arc<dyn AiCachePort>,
    config_port: Arc<dyn AiConfigurationPort>,
}

impl AiDomainService {
    /// Creates a new AI domain service
    pub fn new(
        providers: HashMap<String, Arc<dyn AiProviderPort>>,
        template_port: Arc<dyn PromptTemplatePort>,
        result_port: Arc<dyn AnalysisResultPort>,
        conversation_port: Arc<dyn ConversationPort>,
        batch_port: Arc<dyn BatchOperationPort>,
        cache_port: Arc<dyn AiCachePort>,
        config_port: Arc<dyn AiConfigurationPort>,
    ) -> Self {
        Self {
            providers,
            template_port,
            result_port,
            conversation_port,
            batch_port,
            cache_port,
            config_port,
        }
    }
    
    /// Executes an AI analysis with caching and fallback logic
    pub async fn execute_analysis(&self, request: AnalysisRequest) -> Result<AnalysisResult, AiDomainError> {
        // Validate the request
        request.validate()?;
        
        // Generate cache key and check cache
        let cache_key = self.cache_port.generate_cache_key(&request);
        if request.options.cache_results {
            if let Ok(Some(cached_result)) = self.cache_port.get_cached_result(&cache_key).await {
                let mut result = cached_result;
                result.mark_cache_hit();
                return Ok(result);
            }
        }
        
        // Create analysis result
        let result_id = Uuid::new_v4().to_string();
        let mut result = AnalysisResult::new(result_id, request.analysis_type.clone());
        result.start_processing();
        
        // Select best provider based on preferences
        let provider = self.select_provider(&request.model_preferences).await?;
        
        // Execute the analysis
        match provider.execute_analysis(request.clone()).await {
            Ok(analysis_result) => {
                // Cache the result if caching is enabled
                if request.options.cache_results {
                    let ttl = Duration::hours(24); // Default TTL
                    let _ = self.cache_port.cache_result(&cache_key, analysis_result.clone(), ttl).await;
                }
                
                // Save the result
                self.result_port.save_result(analysis_result.clone()).await?;
                
                Ok(analysis_result)
            }
            Err(error) => {
                result.complete_with_failure(error.to_string());
                self.result_port.save_result(result.clone()).await?;
                Err(error)
            }
        }
    }
    
    /// Executes a batch analysis operation
    pub async fn execute_batch_analysis(&self, requests: Vec<AnalysisRequest>, name: String) -> Result<BatchAnalysisOperation, AiDomainError> {
        let batch_id = Uuid::new_v4().to_string();
        let mut batch = BatchAnalysisOperation::new(batch_id, name, requests);
        
        // Save initial batch state
        self.batch_port.save_batch_operation(batch.clone()).await?;
        
        // Start processing
        batch.start();
        self.batch_port.update_batch_operation(batch.clone()).await?;
        
        // Process requests in parallel (respecting concurrency limits)
        let max_concurrent = batch.options.max_concurrent_requests;
        let requests = batch.requests.clone(); // Clone to avoid borrowing issues
        
        for chunk in requests.chunks(max_concurrent) {
            let chunk_futures: Vec<_> = chunk.iter()
                .map(|request| self.execute_analysis(request.clone()))
                .collect();
            
            let chunk_results = futures::future::join_all(chunk_futures).await;
            
            for result in chunk_results {
                match result {
                    Ok(analysis_result) => {
                        batch.add_result(analysis_result);
                    }
                    Err(error) => {
                        if batch.options.fail_fast {
                            batch.fail(error.to_string());
                            self.batch_port.update_batch_operation(batch.clone()).await?;
                            return Err(error);
                        }
                        // Create a failed result for non-fail-fast mode
                        let failed_result = AnalysisResult::new(
                            Uuid::new_v4().to_string(),
                            AnalysisType::TaskAnalysis, // Default type for failed results
                        );
                        batch.add_result(failed_result);
                    }
                }
            }
            
            // Update progress
            self.batch_port.update_batch_operation(batch.clone()).await?;
        }
        
        // Complete the batch
        batch.complete();
        self.batch_port.update_batch_operation(batch.clone()).await?;
        
        Ok(batch)
    }
    
    /// Creates and manages a conversation session
    pub async fn create_conversation(&self, title: String, provider: AiProvider, model: AiModel) -> Result<ConversationSession, AiDomainError> {
        let conversation_id = Uuid::new_v4().to_string();
        let conversation = ConversationSession::new(conversation_id, title, provider, model);
        
        self.conversation_port.save_conversation(conversation.clone()).await?;
        Ok(conversation)
    }
    
    /// Adds a message to a conversation and processes AI response
    pub async fn add_conversation_message(&self, conversation_id: String, user_message: String) -> Result<ConversationMessage, AiDomainError> {
        // Get the conversation
        let mut conversation = self.conversation_port
            .get_conversation(&conversation_id)
            .await?
            .ok_or_else(|| AiDomainError::InvalidAnalysisContext {
                reason: format!("Conversation {} not found", conversation_id),
            })?;
        
        // Add user message
        let user_msg_id = Uuid::new_v4().to_string();
        let token_count = conversation.model.estimate_tokens(&user_message);
        let user_msg = ConversationMessage::user(user_msg_id, user_message.clone(), token_count);
        
        conversation.add_message(user_msg);
        self.conversation_port.add_message(&conversation_id, conversation.messages.last().unwrap().clone()).await?;
        
        // Prepare AI analysis request
        let analysis_request = AnalysisRequest::new(
            AnalysisType::TaskAnalysis, // Default type for conversations
            user_message,
        )
        .with_context("conversation_id".to_string(), conversation_id.clone());
        
        // Get AI response
        let ai_result = self.execute_analysis(analysis_request).await?;
        let ai_response = ai_result.get_result()?.to_string();
        
        // Add AI response message
        let ai_msg_id = Uuid::new_v4().to_string();
        let ai_token_count = conversation.model.estimate_tokens(&ai_response);
        let ai_msg = ConversationMessage::assistant(ai_msg_id, ai_response, ai_token_count)
            .with_cost(ai_result.metadata.cost_estimate.unwrap_or(0.0));
        
        conversation.add_message(ai_msg.clone());
        self.conversation_port.add_message(&conversation_id, ai_msg.clone()).await?;
        
        // Update conversation
        self.conversation_port.update_conversation(conversation).await?;
        
        Ok(ai_msg)
    }
    
    /// Gets usage metrics across all providers
    pub async fn get_usage_metrics(&self, period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Result<HashMap<String, UsageMetrics>, AiDomainError> {
        let mut metrics = HashMap::new();
        
        for (provider_name, provider) in &self.providers {
            match provider.get_usage_metrics(period_start, period_end).await {
                Ok(provider_metrics) => {
                    metrics.insert(provider_name.clone(), provider_metrics);
                }
                Err(error) => {
                    // Log error but continue with other providers
                    eprintln!("Failed to get metrics for provider {}: {}", provider_name, error);
                }
            }
        }
        
        Ok(metrics)
    }
    
    /// Validates all provider connections
    pub async fn validate_all_providers(&self) -> Result<HashMap<String, ProviderHealth>, AiDomainError> {
        let mut health_status = HashMap::new();
        
        for (_provider_name, provider) in &self.providers {
            let health = provider.health_check().await.unwrap_or_else(|_| ProviderHealth {
                is_healthy: false,
                response_time_ms: 0,
                error_rate: 1.0,
                last_checked: Utc::now(),
                status_message: "Health check failed".to_string(),
            });
            
            health_status.insert(_provider_name.clone(), health);
        }
        
        Ok(health_status)
    }
    
    /// Selects the best provider based on model preferences
    async fn select_provider(&self, preferences: &ModelPreferences) -> Result<Arc<dyn AiProviderPort>, AiDomainError> {
        // Try preferred providers first
        for provider_type in &preferences.preferred_providers {
            let provider_name = provider_type.display_name();
            if let Some(provider) = self.providers.get(provider_name) {
                // Validate provider health
                if let Ok(health) = provider.health_check().await {
                    if health.is_healthy {
                        return Ok(provider.clone());
                    }
                }
            }
        }
        
        // Fallback strategy
        match preferences.fallback_strategy {
            FallbackStrategy::BestAvailable => {
                self.select_best_available_provider().await
            }
            FallbackStrategy::CheapestAvailable => {
                self.select_cheapest_provider().await
            }
            FallbackStrategy::FastestAvailable => {
                self.select_fastest_provider().await
            }
            FallbackStrategy::Fail => {
                Err(AiDomainError::ModelNotAvailable {
                    model_name: "No preferred providers available".to_string(),
                })
            }
        }
    }
    
    /// Selects the best available provider based on health and performance
    async fn select_best_available_provider(&self) -> Result<Arc<dyn AiProviderPort>, AiDomainError> {
        let mut best_provider = None;
        let mut best_score = 0.0;
        
        for (_provider_name, provider) in &self.providers {
            if let Ok(health) = provider.health_check().await {
                if health.is_healthy {
                    // Score based on response time and error rate
                    let score = 1000.0 / (health.response_time_ms as f32 + 1.0) * (1.0 - health.error_rate);
                    if score > best_score {
                        best_score = score;
                        best_provider = Some(provider.clone());
                    }
                }
            }
        }
        
        best_provider.ok_or_else(|| AiDomainError::ModelNotAvailable {
            model_name: "No healthy providers available".to_string(),
        })
    }
    
    /// Selects the cheapest available provider
    async fn select_cheapest_provider(&self) -> Result<Arc<dyn AiProviderPort>, AiDomainError> {
        // For now, return the first available provider
        // In a real implementation, you would compare pricing
        self.select_best_available_provider().await
    }
    
    /// Selects the fastest available provider
    async fn select_fastest_provider(&self) -> Result<Arc<dyn AiProviderPort>, AiDomainError> {
        let mut fastest_provider = None;
        let mut best_response_time = u64::MAX;
        
        for (_provider_name, provider) in &self.providers {
            if let Ok(health) = provider.health_check().await {
                if health.is_healthy && health.response_time_ms < best_response_time {
                    best_response_time = health.response_time_ms;
                    fastest_provider = Some(provider.clone());
                }
            }
        }
        
        fastest_provider.ok_or_else(|| AiDomainError::ModelNotAvailable {
            model_name: "No fast providers available".to_string(),
        })
    }
}

/// Service for managing AI prompt templates
pub struct PromptTemplateService {
    template_port: Arc<dyn PromptTemplatePort>,
}

impl PromptTemplateService {
    /// Creates a new prompt template service
    pub fn new(template_port: Arc<dyn PromptTemplatePort>) -> Self {
        Self { template_port }
    }
    
    /// Creates and validates a new prompt template
    pub async fn create_template(&self, template: PromptTemplate) -> Result<(), AiDomainError> {
        // Validate the template
        template.validate()?;
        
        // Check if template with same name already exists
        if let Ok(Some(_)) = self.template_port.get_template(template.name()).await {
            return Err(AiDomainError::InvalidPromptTemplate {
                template_name: format!("Template '{}' already exists", template.name()),
            });
        }
        
        // Save the template
        self.template_port.save_template(template).await
    }
    
    /// Updates an existing template
    pub async fn update_template(&self, template: PromptTemplate) -> Result<(), AiDomainError> {
        // Validate the template
        template.validate()?;
        
        // Check if template exists
        if self.template_port.get_template(template.name()).await?.is_none() {
            return Err(AiDomainError::InvalidPromptTemplate {
                template_name: format!("Template '{}' does not exist", template.name()),
            });
        }
        
        // Update the template
        self.template_port.update_template(template).await
    }
    
    /// Gets a template with default fallbacks
    pub async fn get_template_with_fallback(&self, name: &str, analysis_type: AnalysisType) -> Result<PromptTemplate, AiDomainError> {
        // Try to get the specific template
        if let Ok(Some(template)) = self.template_port.get_template(name).await {
            return Ok(template);
        }
        
        // Fallback to default template for the analysis type
        let default_name = analysis_type.default_template_name();
        if let Ok(Some(template)) = self.template_port.get_template(default_name).await {
            return Ok(template);
        }
        
        // Create a basic default template
        self.create_default_template(analysis_type).await
    }
    
    /// Creates a default template for an analysis type
    async fn create_default_template(&self, analysis_type: AnalysisType) -> Result<PromptTemplate, AiDomainError> {
        let (template_content, required_vars) = match analysis_type {
            AnalysisType::SemanticRelease => (
                "Analyze the following git changes for semantic release:\n\n{changes}\n\nRepository: {repository}\n\nProvide a semantic version bump recommendation (major, minor, patch) and explanation.",
                vec!["changes".to_string(), "repository".to_string()]
            ),
            AnalysisType::TaskAnalysis => (
                "Analyze the following task request:\n\n{task_description}\n\nSystem: {system}\n\nProvide task suggestions and recommendations.",
                vec!["task_description".to_string(), "system".to_string()]
            ),
            AnalysisType::CommitMessageGeneration => (
                "Generate a conventional commit message for the following changes:\n\n{changes}\n\nFormat: <type>(<scope>): <description>",
                vec!["changes".to_string()]
            ),
            _ => (
                "Analyze the following:\n\n{input}\n\nProvide analysis and recommendations.",
                vec!["input".to_string()]
            ),
        };
        
        let template = PromptTemplate::new(
            analysis_type.default_template_name().to_string(),
            format!("Default template for {}", analysis_type.display_name()),
            template_content.to_string(),
            required_vars,
        );
        
        // Save the default template for future use
        self.template_port.save_template(template.clone()).await?;
        
        Ok(template)
    }
}

/// Service for AI caching operations
pub struct AiCacheService {
    cache_port: Arc<dyn AiCachePort>,
}

impl AiCacheService {
    /// Creates a new AI cache service
    pub fn new(cache_port: Arc<dyn AiCachePort>) -> Self {
        Self { cache_port }
    }
    
    /// Performs cache maintenance operations
    pub async fn perform_maintenance(&self) -> Result<CacheMaintenanceResult, AiDomainError> {
        let expired_cleaned = self.cache_port.cleanup_expired_entries().await?;
        let stats = self.cache_port.get_cache_statistics().await?;
        
        Ok(CacheMaintenanceResult {
            expired_entries_cleaned: expired_cleaned,
            current_cache_size: stats.cache_size_bytes,
            hit_ratio: stats.hit_ratio,
        })
    }
    
    /// Invalidates cache entries matching a pattern
    pub async fn invalidate_cache_pattern(&self, pattern: &str) -> Result<usize, AiDomainError> {
        self.cache_port.invalidate_cache(pattern).await
    }
}

/// Result of cache maintenance operations
#[derive(Debug, Clone)]
pub struct CacheMaintenanceResult {
    pub expired_entries_cleaned: usize,
    pub current_cache_size: u64,
    pub hit_ratio: f32,
}

// Add these use statements at the top of the file
use futures;
use uuid; 