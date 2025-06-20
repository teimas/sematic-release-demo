use config::ConfigError;
use miette::Diagnostic;
use thiserror::Error;

/// Result type alias for the application
pub type Result<T> = miette::Result<T, SemanticReleaseError>;

/// Main application error type with rich diagnostic information
#[derive(Error, Diagnostic, Debug)]
pub enum SemanticReleaseError {
    #[error("Git repository error")]
    #[diagnostic(
        code(semantic_release::git_error),
        help("Make sure you're in a valid git repository and have proper permissions")
    )]
    GitError(git2::Error),

    #[error("Configuration error: {message}")]
    #[diagnostic(
        code(semantic_release::config_error),
        help("Check your configuration file at ~/.config/semantic-release-tui/config.toml")
    )]
    ConfigError { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },



    #[error("Monday.com API error")]
    #[diagnostic(
        code(semantic_release::monday_error),
        help("Verify your Monday.com API key, board ID, and account slug in the configuration")
    )]
    MondayError(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("JIRA API error")]
    #[diagnostic(
        code(semantic_release::jira_error), 
        help("Verify your JIRA URL, username, and API token in the configuration")
    )]
    JiraError(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("AI service error: {provider}")]
    #[diagnostic(
        code(semantic_release::ai_error),
        help("Check your AI provider configuration and API key")
    )]
    AiError {
        provider: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("I/O operation failed")]
    #[diagnostic(
        code(semantic_release::io_error),
        help("Check file permissions and disk space")
    )]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error")]
    #[diagnostic(
        code(semantic_release::json_error),
        help("The JSON data might be malformed or incomplete")
    )]
    JsonError(#[from] serde_json::Error),



    #[error("Command execution failed: {command}")]
    #[diagnostic(
        code(semantic_release::command_error),
        help("Ensure the command exists and you have proper permissions")
    )]
    CommandError {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },



    #[error("HTTP request failed")]
    #[diagnostic(code(semantic_release::http_error))]
    HttpError(#[from] reqwest::Error),

    #[error("User interaction failed")]
    #[diagnostic(code(semantic_release::user_interaction_error))]
    UserInteractionError(#[from] dialoguer::Error),

    #[error("Release operation failed: {operation}")]
    #[diagnostic(code(semantic_release::release_error))]
    ReleaseError {
        operation: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl SemanticReleaseError {
    /// Create a git error
    pub fn git_error(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::GitError(git2::Error::from_str(&source.to_string()))
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a configuration error with source
    pub fn config_error_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ConfigError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }



    /// Create a JIRA error
    pub fn jira_error(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::JiraError(Box::new(source))
    }

    /// Create a Monday.com error
    pub fn monday_error(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::MondayError(Box::new(source))
    }

    /// Create an AI service error
    pub fn ai_error(
        provider: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::AiError {
            provider: provider.into(),
            source: Box::new(source),
        }
    }



    /// Create a command error
    pub fn command_error(command: impl Into<String>, exit_code: Option<i32>, stderr: String) -> Self {
        Self::CommandError {
            command: command.into(),
            exit_code,
            stderr,
        }
    }



    /// Create a release operation error
    pub fn release_error(operation: impl Into<String>) -> Self {
        Self::ReleaseError {
            operation: operation.into(),
            source: None,
        }
    }


}

/// Convert from anyhow::Error for gradual migration
impl From<anyhow::Error> for SemanticReleaseError {
    fn from(err: anyhow::Error) -> Self {
        // For gradual migration, wrap anyhow errors as config errors
        Self::config_error(err.to_string())
    }
}

/// Convert config::ConfigError to our error type
impl From<ConfigError> for SemanticReleaseError {
    fn from(err: ConfigError) -> Self {
        Self::config_error_with_source("Configuration loading failed", err)
    }
}

impl From<git2::Error> for SemanticReleaseError {
    fn from(err: git2::Error) -> Self {
        Self::GitError(err)
    }
} 