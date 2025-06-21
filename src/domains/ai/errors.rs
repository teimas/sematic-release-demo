//! AI domain error types
//! 
//! Domain-specific errors that provide rich context and diagnostic information
//! for AI operations and integrations.

use miette::Diagnostic;
use thiserror::Error;

/// AI domain-specific errors with rich diagnostic information
#[derive(Error, Diagnostic, Debug)]
pub enum AiDomainError {
    #[error("AI provider authentication failed: {provider}")]
    #[diagnostic(
        code(ai::auth_failed),
        help("Check your API credentials and ensure they are valid and have sufficient quota")
    )]
    AuthenticationFailed { provider: String },

    #[error("AI provider API error: {provider} - {message}")]
    #[diagnostic(
        code(ai::api_error),
        help("Check the AI provider status and your API rate limits")
    )]
    ProviderApiError { provider: String, message: String },

    #[error("AI analysis failed: {reason}")]
    #[diagnostic(
        code(ai::analysis_failed),
        help("Review the input data and ensure it meets the analysis requirements")
    )]
    AnalysisFailed { reason: String },

    #[error("Invalid AI prompt template: {template_name}")]
    #[diagnostic(
        code(ai::invalid_prompt_template),
        help("Check that the template exists and contains valid placeholder syntax")
    )]
    InvalidPromptTemplate { template_name: String },

    #[error("AI response parsing failed: {reason}")]
    #[diagnostic(
        code(ai::response_parsing_failed),
        help("The AI response format was unexpected. Check the prompt template and retry")
    )]
    ResponseParsingFailed { reason: String },

    #[error("AI model not available: {model_name}")]
    #[diagnostic(
        code(ai::model_not_available),
        help("Check that the model name is correct and available in your AI provider plan")
    )]
    ModelNotAvailable { model_name: String },

    #[error("AI request timeout: {timeout_seconds}s")]
    #[diagnostic(
        code(ai::request_timeout),
        help("Consider increasing the timeout or simplifying the analysis request")
    )]
    RequestTimeout { timeout_seconds: u64 },

    #[error("AI quota exceeded: {provider}")]
    #[diagnostic(
        code(ai::quota_exceeded),
        help("Check your AI provider quota and billing settings")
    )]
    QuotaExceeded { provider: String },

    #[error("Invalid analysis context: {reason}")]
    #[diagnostic(
        code(ai::invalid_context),
        help("Ensure the analysis context contains all required fields and valid data")
    )]
    InvalidAnalysisContext { reason: String },

    #[error("AI suggestion generation failed: {task_type}")]
    #[diagnostic(
        code(ai::suggestion_failed),
        help("Review the input parameters and ensure they are complete and valid")
    )]
    SuggestionGenerationFailed { task_type: String },

    #[error("Unsupported analysis type: {analysis_type}")]
    #[diagnostic(
        code(ai::unsupported_analysis),
        help("Check the available analysis types for your AI provider configuration")
    )]
    UnsupportedAnalysisType { analysis_type: String },

    #[error("AI configuration invalid: {reason}")]
    #[diagnostic(
        code(ai::invalid_config),
        help("Review your AI provider configuration and ensure all required settings are present")
    )]
    InvalidConfiguration { reason: String },

    #[error("Context window exceeded: {tokens} tokens")]
    #[diagnostic(
        code(ai::context_window_exceeded),
        help("Reduce the input size or use a model with a larger context window")
    )]
    ContextWindowExceeded { tokens: usize },

    #[error("AI safety filter triggered: {reason}")]
    #[diagnostic(
        code(ai::safety_filter),
        help("Review the input content and ensure it complies with AI provider safety guidelines")
    )]
    SafetyFilterTriggered { reason: String },

    #[error("AI provider rate limit exceeded")]
    #[diagnostic(
        code(ai::rate_limit_exceeded),
        help("Wait before retrying or consider upgrading your AI provider plan")
    )]
    RateLimitExceeded,

    #[error("AI embedding generation failed: {reason}")]
    #[diagnostic(
        code(ai::embedding_failed),
        help("Check the input text length and format requirements")
    )]
    EmbeddingGenerationFailed { reason: String },

    #[error("AI cache error: {reason}")]
    #[diagnostic(
        code(ai::cache_error),
        help("Check the cache configuration and storage availability")
    )]
    CacheError { reason: String },
} 