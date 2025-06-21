//! AI domain entities
//! 
//! Rich domain entities that encapsulate AI analysis state and behavior.
//! These entities contain business logic and enforce AI operation rules.

use crate::domains::ai::{
    errors::AiDomainError,
    value_objects::{
        AiProvider, AiModel, PromptTemplate, AnalysisRequest, AnalysisType,
        AnalysisOptions, ModelPreferences, UsageMetrics, CacheConfig,
        OutputFormat, ModelRequirements, AiCapability
    }
};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an AI analysis result with rich domain behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: String,
    pub analysis_type: AnalysisType,
    pub status: AnalysisStatus,
    pub result_data: Option<String>,
    pub confidence_score: Option<f32>,
    pub reasoning: Option<String>,
    pub metadata: AnalysisMetadata,
    pub errors: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cache_hit: bool,
}

impl AnalysisResult {
    /// Creates a new analysis result
    pub fn new(id: String, analysis_type: AnalysisType) -> Self {
        Self {
            id,
            analysis_type,
            status: AnalysisStatus::Pending,
            result_data: None,
            confidence_score: None,
            reasoning: None,
            metadata: AnalysisMetadata::new(),
            errors: Vec::new(),
            created_at: Utc::now(),
            completed_at: None,
            cache_hit: false,
        }
    }
    
    /// Marks the analysis as completed with success
    pub fn complete_with_success(
        &mut self,
        result_data: String,
        confidence_score: Option<f32>,
        reasoning: Option<String>,
    ) {
        self.status = AnalysisStatus::Completed;
        self.result_data = Some(result_data);
        self.confidence_score = confidence_score;
        self.reasoning = reasoning;
        self.completed_at = Some(Utc::now());
    }
    
    /// Marks the analysis as failed
    pub fn complete_with_failure(&mut self, error: String) {
        self.status = AnalysisStatus::Failed;
        self.errors.push(error);
        self.completed_at = Some(Utc::now());
    }
    
    /// Marks the analysis as cancelled
    pub fn cancel(&mut self) {
        self.status = AnalysisStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
    
    /// Marks the analysis as processing
    pub fn start_processing(&mut self) {
        self.status = AnalysisStatus::Processing;
    }
    
    /// Sets metadata
    pub fn set_metadata(&mut self, metadata: AnalysisMetadata) {
        self.metadata = metadata;
    }
    
    /// Adds an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
    
    /// Marks as cache hit
    pub fn mark_cache_hit(&mut self) {
        self.cache_hit = true;
    }
    
    /// Gets the processing duration
    pub fn processing_duration(&self) -> Option<Duration> {
        self.completed_at.map(|end| end - self.created_at)
    }
    
    /// Checks if the analysis was successful
    pub fn is_successful(&self) -> bool {
        matches!(self.status, AnalysisStatus::Completed) && self.result_data.is_some()
    }
    
    /// Checks if the analysis failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, AnalysisStatus::Failed)
    }
    
    /// Checks if the analysis is still in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self.status, AnalysisStatus::Processing | AnalysisStatus::Pending)
    }
    
    /// Gets the result data with validation
    pub fn get_result(&self) -> Result<&str, AiDomainError> {
        match &self.result_data {
            Some(data) => Ok(data),
            None => Err(AiDomainError::AnalysisFailed {
                reason: "Analysis has no result data".to_string(),
            }),
        }
    }
    
    /// Validates the result quality based on confidence
    pub fn validate_quality(&self, min_confidence: f32) -> Result<(), AiDomainError> {
        if let Some(confidence) = self.confidence_score {
            if confidence < min_confidence {
                return Err(AiDomainError::AnalysisFailed {
                    reason: format!(
                        "Analysis confidence {} is below minimum threshold {}",
                        confidence, min_confidence
                    ),
                });
            }
        }
        Ok(())
    }
}

/// Status of an AI analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl AnalysisStatus {
    /// Gets the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Processing => "Processing",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
    
    /// Checks if the status represents a final state
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Metadata for AI analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub model_used: Option<String>,
    pub provider_used: Option<String>,
    pub tokens_consumed: usize,
    pub request_duration: Option<Duration>,
    pub cost_estimate: Option<f64>,
    pub prompt_template_used: Option<String>,
    pub context_window_utilization: Option<f32>,
    pub custom_fields: HashMap<String, String>,
}

impl AnalysisMetadata {
    /// Creates new analysis metadata
    pub fn new() -> Self {
        Self {
            model_used: None,
            provider_used: None,
            tokens_consumed: 0,
            request_duration: None,
            cost_estimate: None,
            prompt_template_used: None,
            context_window_utilization: None,
            custom_fields: HashMap::new(),
        }
    }
    
    /// Sets the model used
    pub fn with_model(mut self, model: String) -> Self {
        self.model_used = Some(model);
        self
    }
    
    /// Sets the provider used
    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider_used = Some(provider);
        self
    }
    
    /// Sets tokens consumed
    pub fn with_tokens_consumed(mut self, tokens: usize) -> Self {
        self.tokens_consumed = tokens;
        self
    }
    
    /// Sets request duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.request_duration = Some(duration);
        self
    }
    
    /// Sets cost estimate
    pub fn with_cost_estimate(mut self, cost: f64) -> Self {
        self.cost_estimate = Some(cost);
        self
    }
    
    /// Sets prompt template used
    pub fn with_prompt_template(mut self, template: String) -> Self {
        self.prompt_template_used = Some(template);
        self
    }
    
    /// Sets context window utilization
    pub fn with_context_utilization(mut self, utilization: f32) -> Self {
        self.context_window_utilization = Some(utilization);
        self
    }
    
    /// Adds a custom field
    pub fn with_custom_field(mut self, key: String, value: String) -> Self {
        self.custom_fields.insert(key, value);
        self
    }
}

impl Default for AnalysisMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an AI conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<ConversationMessage>,
    pub context: HashMap<String, String>,
    pub total_tokens_used: usize,
    pub total_cost: f64,
    pub provider: AiProvider,
    pub model: AiModel,
}

impl ConversationSession {
    /// Creates a new conversation session
    pub fn new(id: String, title: String, provider: AiProvider, model: AiModel) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            context: HashMap::new(),
            total_tokens_used: 0,
            total_cost: 0.0,
            provider,
            model,
        }
    }
    
    /// Adds a message to the conversation
    pub fn add_message(&mut self, message: ConversationMessage) {
        self.total_tokens_used += message.token_count;
        if let Some(cost) = message.cost {
            self.total_cost += cost;
        }
        self.messages.push(message);
        self.updated_at = Utc::now();
    }
    
    /// Gets messages within a token limit
    pub fn get_recent_messages(&self, max_tokens: usize) -> Vec<&ConversationMessage> {
        let mut selected_messages = Vec::new();
        let mut token_count = 0;
        
        // Start from the most recent messages
        for message in self.messages.iter().rev() {
            if token_count + message.token_count > max_tokens {
                break;
            }
            token_count += message.token_count;
            selected_messages.push(message);
        }
        
        // Reverse to maintain chronological order
        selected_messages.reverse();
        selected_messages
    }
    
    /// Sets context
    pub fn set_context(&mut self, key: String, value: String) {
        self.context.insert(key, value);
        self.updated_at = Utc::now();
    }
    
    /// Gets the conversation length in tokens
    pub fn total_tokens(&self) -> usize {
        self.total_tokens_used
    }
    
    /// Gets the average cost per message
    pub fn average_cost_per_message(&self) -> f64 {
        if self.messages.is_empty() {
            0.0
        } else {
            self.total_cost / self.messages.len() as f64
        }
    }
    
    /// Checks if the conversation fits within context window
    pub fn fits_in_context_window(&self) -> bool {
        self.total_tokens_used <= self.model.context_window()
    }
    
    /// Trims conversation to fit within context window
    pub fn trim_to_context_window(&mut self) {
        if self.fits_in_context_window() {
            return;
        }
        
        let context_window = self.model.context_window();
        let recent_messages = self.get_recent_messages(context_window);
        
        // Keep only the messages that fit
        self.messages = recent_messages.into_iter().cloned().collect();
        
        // Recalculate totals
        self.total_tokens_used = self.messages.iter().map(|m| m.token_count).sum();
        self.total_cost = self.messages.iter().filter_map(|m| m.cost).sum();
        self.updated_at = Utc::now();
    }
}

/// Represents a message in an AI conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: usize,
    pub cost: Option<f64>,
    pub metadata: HashMap<String, String>,
}

impl ConversationMessage {
    /// Creates a new conversation message
    pub fn new(id: String, role: MessageRole, content: String, token_count: usize) -> Self {
        Self {
            id,
            role,
            content,
            timestamp: Utc::now(),
            token_count,
            cost: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Creates a user message
    pub fn user(id: String, content: String, token_count: usize) -> Self {
        Self::new(id, MessageRole::User, content, token_count)
    }
    
    /// Creates an assistant message
    pub fn assistant(id: String, content: String, token_count: usize) -> Self {
        Self::new(id, MessageRole::Assistant, content, token_count)
    }
    
    /// Creates a system message
    pub fn system(id: String, content: String, token_count: usize) -> Self {
        Self::new(id, MessageRole::System, content, token_count)
    }
    
    /// Sets the cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = Some(cost);
        self
    }
    
    /// Adds metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Role of a message in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    /// Gets the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::User => "User",
            Self::Assistant => "Assistant",
        }
    }
}

/// Represents a batch AI operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisOperation {
    pub id: String,
    pub name: String,
    pub requests: Vec<AnalysisRequest>,
    pub results: Vec<AnalysisResult>,
    pub status: BatchStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_cost: f64,
    pub total_tokens: usize,
    pub options: BatchOptions,
}

impl BatchAnalysisOperation {
    /// Creates a new batch operation
    pub fn new(id: String, name: String, requests: Vec<AnalysisRequest>) -> Self {
        Self {
            id,
            name,
            requests,
            results: Vec::new(),
            status: BatchStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            total_cost: 0.0,
            total_tokens: 0,
            options: BatchOptions::default(),
        }
    }
    
    /// Starts the batch operation
    pub fn start(&mut self) {
        self.status = BatchStatus::Processing;
        self.started_at = Some(Utc::now());
    }
    
    /// Adds a result to the batch
    pub fn add_result(&mut self, result: AnalysisResult) {
        self.total_tokens += result.metadata.tokens_consumed;
        if let Some(cost) = result.metadata.cost_estimate {
            self.total_cost += cost;
        }
        self.results.push(result);
    }
    
    /// Completes the batch operation
    pub fn complete(&mut self) {
        self.status = BatchStatus::Completed;
        self.completed_at = Some(Utc::now());
    }
    
    /// Fails the batch operation
    pub fn fail(&mut self, error: String) {
        self.status = BatchStatus::Failed(error);
        self.completed_at = Some(Utc::now());
    }
    
    /// Gets the success rate
    pub fn success_rate(&self) -> f32 {
        if self.results.is_empty() {
            return 0.0;
        }
        
        let successful = self.results.iter().filter(|r| r.is_successful()).count();
        successful as f32 / self.results.len() as f32
    }
    
    /// Gets the completion percentage
    pub fn completion_percentage(&self) -> f32 {
        if self.requests.is_empty() {
            return 100.0;
        }
        
        (self.results.len() as f32 / self.requests.len() as f32) * 100.0
    }
    
    /// Gets the estimated remaining time
    pub fn estimated_remaining_time(&self) -> Option<Duration> {
        if let Some(started) = self.started_at {
            let elapsed = Utc::now() - started;
            let completed = self.results.len();
            let total = self.requests.len();
            
            if completed > 0 && completed < total {
                let avg_time_per_request = elapsed / completed as i32;
                let remaining_requests = total - completed;
                return Some(avg_time_per_request * remaining_requests as i32);
            }
        }
        None
    }
}

/// Status of a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Cancelled,
}

impl BatchStatus {
    /// Checks if the status is final
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed(_) | Self::Cancelled)
    }
    
    /// Gets the display name
    pub fn display_name(&self) -> String {
        match self {
            Self::Pending => "Pending".to_string(),
            Self::Processing => "Processing".to_string(),
            Self::Completed => "Completed".to_string(),
            Self::Failed(error) => format!("Failed: {}", error),
            Self::Cancelled => "Cancelled".to_string(),
        }
    }
}

/// Options for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    pub max_concurrent_requests: usize,
    pub retry_failed_requests: bool,
    pub max_retries: usize,
    pub fail_fast: bool,
    pub priority: BatchPriority,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 5,
            retry_failed_requests: true,
            max_retries: 3,
            fail_fast: false,
            priority: BatchPriority::Normal,
        }
    }
}

/// Priority levels for batch operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl BatchPriority {
    /// Gets the numeric value for sorting
    pub fn numeric_value(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Normal => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
} 