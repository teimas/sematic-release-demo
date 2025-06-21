//! Create Release Command
//! 
//! This command orchestrates the creation of a new release including version calculation,
//! changelog generation, git operations, and task system updates.

#[cfg(feature = "new-domains")]
use super::{Command, CommandHandler};
#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc};
#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use std::any::Any;
#[cfg(feature = "new-domains")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "new-domains")]
use crate::domains::semantic::entities::SemanticRelease;
#[cfg(feature = "new-domains")]
use crate::domains::git::entities::GitRepository;
#[cfg(feature = "new-domains")]
use crate::domains::tasks::value_objects::TaskSystem;

/// Command to create a new release
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReleaseCommand {
    pub repository_path: String,
    pub options: CreateReleaseOptions,
}

#[cfg(feature = "new-domains")]
impl Command for CreateReleaseCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Options for creating a release
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReleaseOptions {
    pub target_version: Option<String>,
    pub release_notes: Option<String>,
    pub task_systems: Vec<String>,
    pub dry_run: bool,
    pub auto_push: bool,
    pub generate_changelog: bool,
    pub update_tasks: bool,
    pub custom_fields: HashMap<String, String>,
    pub scheduled_time: Option<DateTime<Utc>>,
}

#[cfg(feature = "new-domains")]
impl Default for CreateReleaseOptions {
    fn default() -> Self {
        Self {
            target_version: None,
            release_notes: None,
            task_systems: Vec::new(),
            dry_run: false,
            auto_push: true,
            generate_changelog: true,
            update_tasks: false,
            custom_fields: HashMap::new(),
            scheduled_time: None,
        }
    }
}

#[cfg(feature = "new-domains")]
impl CreateReleaseCommand {
    pub fn new(repository_path: String) -> Self {
        Self {
            repository_path,
            options: CreateReleaseOptions::default(),
        }
    }
    
    /// Set target version for the release
    pub fn with_target_version(mut self, version: String) -> Self {
        self.options.target_version = Some(version);
        self
    }
    
    /// Set release notes
    pub fn with_release_notes(mut self, notes: String) -> Self {
        self.options.release_notes = Some(notes);
        self
    }
    
    /// Set task systems to update
    pub fn with_task_systems(mut self, systems: Vec<String>) -> Self {
        self.options.task_systems = systems;
        self
    }
    
    /// Set dry run mode
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.options.dry_run = dry_run;
        self
    }
    
    /// Set auto push
    pub fn with_auto_push(mut self, auto_push: bool) -> Self {
        self.options.auto_push = auto_push;
        self
    }
    
    /// Set changelog generation
    pub fn with_changelog_generation(mut self, generate: bool) -> Self {
        self.options.generate_changelog = generate;
        self
    }
    
    /// Set task updates
    pub fn with_task_updates(mut self, update: bool) -> Self {
        self.options.update_tasks = update;
        self
    }
    
    /// Add custom field
    pub fn with_custom_field(mut self, key: String, value: String) -> Self {
        self.options.custom_fields.insert(key, value);
        self
    }
    
    /// Set scheduled time
    pub fn with_scheduled_time(mut self, time: DateTime<Utc>) -> Self {
        self.options.scheduled_time = Some(time);
        self
    }
}

/// Result of creating a release
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReleaseResult {
    pub version: String,
    pub git_tag: String,
    pub changelog_generated: bool,
    pub tasks_updated: Vec<String>,
    pub release_url: Option<String>,
    pub duration_ms: u64,
    pub warnings: Vec<String>,
}

/// Errors that can occur during release creation
#[cfg(feature = "new-domains")]
#[derive(Debug, thiserror::Error)]
pub enum CreateReleaseError {
    #[error("Repository validation failed: {message}")]
    RepositoryValidationFailed { message: String },
    
    #[error("Version calculation failed: {message}")]
    VersionCalculationFailed { message: String },
    
    #[error("Git operation failed: {operation}: {message}")]
    GitOperationFailed { operation: String, message: String },
    
    #[error("Task system integration failed: {system}: {message}")]
    TaskSystemFailed { system: String, message: String },
    
    #[error("Changelog generation failed: {message}")]
    ChangelogGenerationFailed { message: String },
    
    #[error("Release scheduling failed: {message}")]
    SchedulingFailed { message: String },
    
    #[error("Validation failed: {field}: {message}")]
    ValidationFailed { field: String, message: String },
    
    #[error("Dependency error: {service}: {reason}")]
    DependencyError { service: String, reason: String },
}

/// Handler for create release command
#[cfg(feature = "new-domains")]
pub struct CreateReleaseHandler {
    release_orchestrator: Arc<dyn ReleaseOrchestrator>,
}

#[cfg(feature = "new-domains")]
impl CreateReleaseHandler {
    pub fn new(release_orchestrator: Arc<dyn ReleaseOrchestrator>) -> Self {
        Self { release_orchestrator }
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl CommandHandler<CreateReleaseCommand> for CreateReleaseHandler {
    type Result = CreateReleaseResult;
    type Error = CreateReleaseError;
    
    async fn handle(&self, command: CreateReleaseCommand) -> Result<Self::Result, Self::Error> {
        self.release_orchestrator
            .execute_release(command)
            .await
            .map_err(|e| CreateReleaseError::DependencyError {
                service: "ReleaseOrchestrator".to_string(),
                reason: e.to_string(),
            })
    }
}

/// Release orchestrator trait for dependency injection
#[cfg(feature = "new-domains")]
#[async_trait]
pub trait ReleaseOrchestrator: Send + Sync {
    async fn execute_release(&self, command: CreateReleaseCommand) -> Result<CreateReleaseResult, Box<dyn std::error::Error + Send + Sync>>;
    
    // Query methods (these should ideally be in separate query handlers)
    async fn get_release_status(&self, repository_path: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_git_history(&self, repository_path: String, options: crate::application::queries::GitHistoryOptions) -> Result<crate::application::queries::GitHistoryResult, Box<dyn std::error::Error + Send + Sync>>;
} 