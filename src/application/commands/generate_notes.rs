//! Generate Notes Command
//! 
//! This command generates release notes using AI analysis of git commits,
//! with various output formats and customization options.

#[cfg(feature = "new-domains")]
use super::{Command, CommandHandler};
#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use std::any::Any;
#[cfg(feature = "new-domains")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc};

#[cfg(feature = "new-domains")]
use crate::domains::ai::value_objects::{AnalysisType, AiProvider};
#[cfg(feature = "new-domains")]
use crate::domains::git::entities::GitCommit;

/// Command to generate release notes
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateNotesCommand {
    pub repository_path: String,
    pub version_range: VersionRange,
    pub options: GenerateNotesOptions,
}

#[cfg(feature = "new-domains")]
impl Command for GenerateNotesCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "new-domains")]
impl GenerateNotesCommand {
    pub fn new(repository_path: String) -> Self {
        Self {
            repository_path,
            version_range: VersionRange::SinceLastTag,
            options: GenerateNotesOptions::default(),
        }
    }
    
    pub fn with_version_range(mut self, range: VersionRange) -> Self {
        self.version_range = range;
        self
    }
    
    pub fn with_options(mut self, options: GenerateNotesOptions) -> Self {
        self.options = options;
        self
    }
}

/// Version range for release notes generation
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionRange {
    /// Since the last git tag
    SinceLastTag,
    /// Between specific tags
    BetweenTags { from: String, to: String },
    /// Since a specific commit
    SinceCommit(String),
    /// All commits
    All,
}

/// Options for generating release notes
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateNotesOptions {
    pub output_format: OutputFormat,
    pub sections: Vec<NotesSection>,
    pub include_contributors: bool,
    pub include_pull_requests: bool,
    pub group_by_type: bool,
    pub ai_enhancement: bool,
    pub custom_template: Option<String>,
    pub language: String,
    pub custom_fields: HashMap<String, String>,
}

#[cfg(feature = "new-domains")]
impl Default for GenerateNotesOptions {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Markdown,
            sections: vec![
                NotesSection::BreakingChanges,
                NotesSection::Features,
                NotesSection::BugFixes,
                NotesSection::Documentation,
            ],
            include_contributors: true,
            include_pull_requests: true,
            group_by_type: true,
            ai_enhancement: true,
            custom_template: None,
            language: "en".to_string(),
            custom_fields: HashMap::new(),
        }
    }
}

/// Output format for release notes
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Markdown,
    Html,
    PlainText,
    Json,
    Custom(String),
}

/// Sections to include in release notes
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotesSection {
    BreakingChanges,
    Features,
    BugFixes,
    Performance,
    Security,
    Documentation,
    Deprecations,
    Dependencies,
    Chores,
    Custom(String),
}

/// Result of generating release notes
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateNotesResult {
    pub content: String,
    pub format: OutputFormat,
    pub commit_count: usize,
    pub contributors: Vec<String>,
    pub sections: HashMap<String, Vec<String>>,
    pub generation_time: DateTime<Utc>,
    pub ai_enhanced: bool,
    pub warnings: Vec<String>,
}

/// Errors that can occur during release notes generation
#[cfg(feature = "new-domains")]
#[derive(Debug, thiserror::Error)]
pub enum GenerateNotesError {
    #[error("Repository analysis failed: {message}")]
    RepositoryAnalysisFailed { message: String },
    
    #[error("Git history access failed: {message}")]
    GitHistoryFailed { message: String },
    
    #[error("AI enhancement failed: {message}")]
    AiEnhancementFailed { message: String },
    
    #[error("Template processing failed: {template}: {message}")]
    TemplateProcessingFailed { template: String, message: String },
    
    #[error("Output formatting failed: {format:?}: {message}")]
    OutputFormattingFailed { format: OutputFormat, message: String },
    
    #[error("Validation failed: {field}: {message}")]
    ValidationFailed { field: String, message: String },
}

/// Handler for generate notes command
#[cfg(feature = "new-domains")]
pub struct GenerateNotesHandler {
    ai_coordinator: Arc<dyn AiCoordinator>,
}

#[cfg(feature = "new-domains")]
impl GenerateNotesHandler {
    pub fn new(ai_coordinator: Arc<dyn AiCoordinator>) -> Self {
        Self { ai_coordinator }
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl CommandHandler<GenerateNotesCommand> for GenerateNotesHandler {
    type Result = GenerateNotesResult;
    type Error = GenerateNotesError;
    
    async fn handle(&self, command: GenerateNotesCommand) -> Result<Self::Result, Self::Error> {
        self.ai_coordinator
            .generate_release_notes(command)
            .await
            .map_err(|e| GenerateNotesError::AiEnhancementFailed {
                message: e.to_string(),
            })
    }
}

/// AI coordinator trait for dependency injection
#[cfg(feature = "new-domains")]
#[async_trait]
pub trait AiCoordinator: Send + Sync {
    async fn generate_release_notes(&self, command: GenerateNotesCommand) -> Result<GenerateNotesResult, Box<dyn std::error::Error + Send + Sync>>;
    async fn analyze_commits(&self, commits: Vec<GitCommit>) -> Result<Vec<CommitAnalysis>, Box<dyn std::error::Error + Send + Sync>>;
    async fn suggest_version_bump(&self, commits: Vec<GitCommit>) -> Result<VersionSuggestion, Box<dyn std::error::Error + Send + Sync>>;
}

/// Analysis result for a single commit
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub struct CommitAnalysis {
    pub commit_hash: String,
    pub commit_type: CommitType,
    pub breaking_change: bool,
    pub scope: Option<String>,
    pub description: String,
    pub confidence: f32,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub enum CommitType {
    Feature,
    Fix,
    Documentation,
    Style,
    Refactor,
    Performance,
    Test,
    Chore,
    BreakingChange,
    Unknown,
}

/// Version bump suggestion from AI analysis
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub struct VersionSuggestion {
    pub current_version: Option<(u32, u32, u32)>,
    pub suggested_version: (u32, u32, u32),
    pub bump_type: BumpType,
    pub reasoning: String,
    pub confidence: f32,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
    None,
} 