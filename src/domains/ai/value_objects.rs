//! AI domain value objects
//! 
//! Immutable value objects that encapsulate AI-related data with validation
//! and ensure AI operations maintain their invariants.

use crate::domains::ai::errors::AiDomainError;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
use url::Url;

/// Represents an AI provider with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AiProvider {
    name: String,
    provider_type: AiProviderType,
    base_url: Option<Url>,
}

impl AiProvider {
    /// Creates a new AI provider
    pub fn new(name: String, provider_type: AiProviderType, base_url: Option<Url>) -> Self {
        Self {
            name,
            provider_type,
            base_url,
        }
    }
    
    /// Creates a Gemini provider
    pub fn gemini() -> Self {
        Self::new(
            "Google Gemini".to_string(),
            AiProviderType::Gemini,
            Some("https://generativelanguage.googleapis.com".parse().unwrap()),
        )
    }
    
    /// Creates an OpenAI provider
    pub fn openai() -> Self {
        Self::new(
            "OpenAI".to_string(),
            AiProviderType::OpenAI,
            Some("https://api.openai.com".parse().unwrap()),
        )
    }
    
    /// Creates an Anthropic provider
    pub fn anthropic() -> Self {
        Self::new(
            "Anthropic".to_string(),
            AiProviderType::Anthropic,
            Some("https://api.anthropic.com".parse().unwrap()),
        )
    }
    
    /// Gets the provider name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Gets the provider type
    pub fn provider_type(&self) -> &AiProviderType {
        &self.provider_type
    }
    
    /// Gets the base URL
    pub fn base_url(&self) -> Option<&Url> {
        self.base_url.as_ref()
    }
    
    /// Checks if the provider supports a specific capability
    pub fn supports_capability(&self, capability: AiCapability) -> bool {
        match self.provider_type {
            AiProviderType::Gemini => matches!(capability, 
                AiCapability::TextGeneration | 
                AiCapability::CodeAnalysis | 
                AiCapability::Summarization |
                AiCapability::Translation
            ),
            AiProviderType::OpenAI => matches!(capability,
                AiCapability::TextGeneration |
                AiCapability::CodeAnalysis |
                AiCapability::Summarization |
                AiCapability::Translation |
                AiCapability::Embeddings
            ),
            AiProviderType::Anthropic => matches!(capability,
                AiCapability::TextGeneration |
                AiCapability::CodeAnalysis |
                AiCapability::Summarization
            ),
            AiProviderType::Custom => true, // Assume custom providers can be configured for any capability
        }
    }
}

impl fmt::Display for AiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Types of AI providers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiProviderType {
    Gemini,
    OpenAI,
    Anthropic,
    Custom,
}

impl AiProviderType {
    /// Gets the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gemini => "Google Gemini",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Custom => "Custom",
        }
    }
    
    /// Gets default models for the provider
    pub fn default_models(&self) -> Vec<AiModel> {
        match self {
            Self::Gemini => vec![
                AiModel::new("gemini-1.5-flash".to_string(), ModelCapability::TextGeneration, 1_000_000),
                AiModel::new("gemini-1.5-pro".to_string(), ModelCapability::TextGeneration, 2_000_000),
            ],
            Self::OpenAI => vec![
                AiModel::new("gpt-4o-mini".to_string(), ModelCapability::TextGeneration, 128_000),
                AiModel::new("gpt-4o".to_string(), ModelCapability::TextGeneration, 128_000),
                AiModel::new("text-embedding-3-small".to_string(), ModelCapability::Embeddings, 8_192),
            ],
            Self::Anthropic => vec![
                AiModel::new("claude-3-haiku-20240307".to_string(), ModelCapability::TextGeneration, 200_000),
                AiModel::new("claude-3-sonnet-20240229".to_string(), ModelCapability::TextGeneration, 200_000),
            ],
            Self::Custom => vec![],
        }
    }
}

/// AI capabilities that can be provided
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiCapability {
    TextGeneration,
    CodeAnalysis,
    Summarization,
    Translation,
    Embeddings,
    ImageAnalysis,
    SpeechToText,
    TextToSpeech,
}

/// Represents an AI model with its capabilities and limits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiModel {
    name: String,
    capability: ModelCapability,
    context_window: usize,
    max_output_tokens: Option<usize>,
    cost_per_token: Option<f64>,
}

impl AiModel {
    /// Creates a new AI model
    pub fn new(name: String, capability: ModelCapability, context_window: usize) -> Self {
        Self {
            name,
            capability,
            context_window,
            max_output_tokens: None,
            cost_per_token: None,
        }
    }
    
    /// Sets the maximum output tokens
    pub fn with_max_output_tokens(mut self, max_tokens: usize) -> Self {
        self.max_output_tokens = Some(max_tokens);
        self
    }
    
    /// Sets the cost per token
    pub fn with_cost_per_token(mut self, cost: f64) -> Self {
        self.cost_per_token = Some(cost);
        self
    }
    
    /// Gets the model name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Gets the model capability
    pub fn capability(&self) -> &ModelCapability {
        &self.capability
    }
    
    /// Gets the context window size
    pub fn context_window(&self) -> usize {
        self.context_window
    }
    
    /// Gets the maximum output tokens
    pub fn max_output_tokens(&self) -> Option<usize> {
        self.max_output_tokens
    }
    
    /// Estimates token count for text (rough estimation)
    pub fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimation: ~4 characters per token for English text
        (text.len() + 3) / 4
    }
    
    /// Checks if text fits within context window
    pub fn can_fit_context(&self, text: &str) -> bool {
        self.estimate_tokens(text) <= self.context_window
    }
}

/// Model capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    TextGeneration,
    Embeddings,
    ImageGeneration,
    SpeechGeneration,
    MultiModal,
}

/// Represents an AI prompt template with placeholders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    name: String,
    description: String,
    template: String,
    required_variables: Vec<String>,
    optional_variables: Vec<String>,
    model_requirements: ModelRequirements,
    expected_output_format: OutputFormat,
}

impl PromptTemplate {
    /// Creates a new prompt template
    pub fn new(
        name: String,
        description: String,
        template: String,
        required_variables: Vec<String>,
    ) -> Self {
        Self {
            name,
            description,
            template,
            required_variables,
            optional_variables: Vec::new(),
            model_requirements: ModelRequirements::default(),
            expected_output_format: OutputFormat::Text,
        }
    }
    
    /// Sets optional variables
    pub fn with_optional_variables(mut self, variables: Vec<String>) -> Self {
        self.optional_variables = variables;
        self
    }
    
    /// Sets model requirements
    pub fn with_model_requirements(mut self, requirements: ModelRequirements) -> Self {
        self.model_requirements = requirements;
        self
    }
    
    /// Sets expected output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.expected_output_format = format;
        self
    }
    
    /// Renders the template with provided variables
    pub fn render(&self, variables: &HashMap<String, String>) -> Result<String, AiDomainError> {
        // Check required variables
        for required_var in &self.required_variables {
            if !variables.contains_key(required_var) {
                return Err(AiDomainError::InvalidPromptTemplate {
                    template_name: format!("{} - missing required variable: {}", self.name, required_var),
                });
            }
        }
        
        let mut rendered = self.template.clone();
        
        // Replace variables in the template
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }
        
        // Check if any unreplaced placeholders remain
        if rendered.contains('{') && rendered.contains('}') {
            return Err(AiDomainError::InvalidPromptTemplate {
                template_name: format!("{} - unreplaced placeholders found", self.name),
            });
        }
        
        Ok(rendered)
    }
    
    /// Validates the template structure
    pub fn validate(&self) -> Result<(), AiDomainError> {
        if self.name.trim().is_empty() {
            return Err(AiDomainError::InvalidPromptTemplate {
                template_name: "empty name".to_string(),
            });
        }
        
        if self.template.trim().is_empty() {
            return Err(AiDomainError::InvalidPromptTemplate {
                template_name: format!("{} - empty template", self.name),
            });
        }
        
        // Check that all required variables have placeholders in the template
        for required_var in &self.required_variables {
            let placeholder = format!("{{{}}}", required_var);
            if !self.template.contains(&placeholder) {
                return Err(AiDomainError::InvalidPromptTemplate {
                    template_name: format!("{} - required variable {} not found in template", self.name, required_var),
                });
            }
        }
        
        Ok(())
    }
    
    /// Gets the template name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Gets the template description
    pub fn description(&self) -> &str {
        &self.description
    }
    
    /// Gets required variables
    pub fn required_variables(&self) -> &[String] {
        &self.required_variables
    }
    
    /// Gets optional variables
    pub fn optional_variables(&self) -> &[String] {
        &self.optional_variables
    }
    
    /// Gets model requirements
    pub fn model_requirements(&self) -> &ModelRequirements {
        &self.model_requirements
    }
    
    /// Gets expected output format
    pub fn expected_output_format(&self) -> &OutputFormat {
        &self.expected_output_format
    }
}

/// Model requirements for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub min_context_window: usize,
    pub required_capabilities: Vec<AiCapability>,
    pub preferred_providers: Vec<AiProviderType>,
}

impl Default for ModelRequirements {
    fn default() -> Self {
        Self {
            min_context_window: 4096,
            required_capabilities: vec![AiCapability::TextGeneration],
            preferred_providers: vec![],
        }
    }
}

/// Expected output formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
    Code,
    Structured { schema: String },
}

/// Represents an AI analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub analysis_type: AnalysisType,
    pub input_data: String,
    pub context: HashMap<String, String>,
    pub options: AnalysisOptions,
    pub model_preferences: ModelPreferences,
}

impl AnalysisRequest {
    /// Creates a new analysis request
    pub fn new(analysis_type: AnalysisType, input_data: String) -> Self {
        Self {
            analysis_type,
            input_data,
            context: HashMap::new(),
            options: AnalysisOptions::default(),
            model_preferences: ModelPreferences::default(),
        }
    }
    
    /// Adds context to the request
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
    
    /// Sets analysis options
    pub fn with_options(mut self, options: AnalysisOptions) -> Self {
        self.options = options;
        self
    }
    
    /// Sets model preferences
    pub fn with_model_preferences(mut self, preferences: ModelPreferences) -> Self {
        self.model_preferences = preferences;
        self
    }
    
    /// Validates the request
    pub fn validate(&self) -> Result<(), AiDomainError> {
        if self.input_data.trim().is_empty() {
            return Err(AiDomainError::InvalidAnalysisContext {
                reason: "Input data cannot be empty".to_string(),
            });
        }
        
        // Validate context based on analysis type
        match &self.analysis_type {
            AnalysisType::SemanticRelease => {
                if !self.context.contains_key("repository") {
                    return Err(AiDomainError::InvalidAnalysisContext {
                        reason: "Semantic release analysis requires 'repository' context".to_string(),
                    });
                }
            }
            AnalysisType::TaskAnalysis => {
                if !self.context.contains_key("system") {
                    return Err(AiDomainError::InvalidAnalysisContext {
                        reason: "Task analysis requires 'system' context".to_string(),
                    });
                }
            }
            _ => {} // Other types may not require specific context
        }
        
        Ok(())
    }
}

/// Types of AI analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalysisType {
    SemanticRelease,
    TaskAnalysis,
    CodeReview,
    CommitMessageGeneration,
    ReleaseNotesGeneration,
    TaskSuggestion,
    ProjectSummary,
    TrendAnalysis,
}

impl AnalysisType {
    /// Gets the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SemanticRelease => "Semantic Release Analysis",
            Self::TaskAnalysis => "Task Analysis",
            Self::CodeReview => "Code Review",
            Self::CommitMessageGeneration => "Commit Message Generation",
            Self::ReleaseNotesGeneration => "Release Notes Generation",
            Self::TaskSuggestion => "Task Suggestion",
            Self::ProjectSummary => "Project Summary",
            Self::TrendAnalysis => "Trend Analysis",
        }
    }
    
    /// Gets the default prompt template name for this analysis type
    pub fn default_template_name(&self) -> &'static str {
        match self {
            Self::SemanticRelease => "semantic_release_analysis",
            Self::TaskAnalysis => "task_analysis",
            Self::CodeReview => "code_review",
            Self::CommitMessageGeneration => "commit_message_generation",
            Self::ReleaseNotesGeneration => "release_notes_generation",
            Self::TaskSuggestion => "task_suggestion",
            Self::ProjectSummary => "project_summary",
            Self::TrendAnalysis => "trend_analysis",
        }
    }
}

/// Analysis options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub include_confidence: bool,
    pub include_reasoning: bool,
    pub cache_results: bool,
    pub timeout_seconds: u64,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            max_tokens: None,
            temperature: Some(0.7),
            include_confidence: true,
            include_reasoning: true,
            cache_results: true,
            timeout_seconds: 30,
        }
    }
}

/// Model preferences for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    pub preferred_providers: Vec<AiProviderType>,
    pub preferred_models: Vec<String>,
    pub fallback_strategy: FallbackStrategy,
    pub quality_preference: QualityPreference,
}

impl Default for ModelPreferences {
    fn default() -> Self {
        Self {
            preferred_providers: vec![AiProviderType::Gemini, AiProviderType::OpenAI],
            preferred_models: vec![],
            fallback_strategy: FallbackStrategy::BestAvailable,
            quality_preference: QualityPreference::Balanced,
        }
    }
}

/// Fallback strategies when preferred models are unavailable
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    BestAvailable,
    CheapestAvailable,
    FastestAvailable,
    Fail,
}

/// Quality preferences for model selection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreference {
    Speed,
    Balanced,
    Quality,
    Cost,
}

/// Represents caching configuration for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl: Duration,
    pub max_entries: usize,
    pub strategy: CacheStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: Duration::hours(24),
            max_entries: 10000,
            strategy: CacheStrategy::LRU,
        }
    }
}

/// Cache eviction strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheStrategy {
    LRU,
    FIFO,
    TTL,
}

/// Represents usage metrics for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub tokens_consumed: usize,
    pub requests_made: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub errors: usize,
    pub total_cost: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

impl UsageMetrics {
    /// Creates new usage metrics
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            tokens_consumed: 0,
            requests_made: 0,
            cache_hits: 0,
            cache_misses: 0,
            errors: 0,
            total_cost: 0.0,
            period_start,
            period_end,
        }
    }
    
    /// Calculates cache hit ratio
    pub fn cache_hit_ratio(&self) -> f32 {
        let total_cache_operations = self.cache_hits + self.cache_misses;
        if total_cache_operations > 0 {
            self.cache_hits as f32 / total_cache_operations as f32
        } else {
            0.0
        }
    }
    
    /// Calculates error rate
    pub fn error_rate(&self) -> f32 {
        if self.requests_made > 0 {
            self.errors as f32 / self.requests_made as f32
        } else {
            0.0
        }
    }
    
    /// Calculates average cost per request
    pub fn average_cost_per_request(&self) -> f64 {
        if self.requests_made > 0 {
            self.total_cost / self.requests_made as f64
        } else {
            0.0
        }
    }
} 