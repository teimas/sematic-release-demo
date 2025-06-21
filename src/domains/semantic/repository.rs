//! Semantic release repository port interface
//! 
//! This module defines the port (interface) for semantic release operations.
//! The actual implementation will be provided by the infrastructure layer.

use crate::domains::{
    git::entities::GitCommit,
    semantic::{
        entities::{SemanticRelease, VersionBumpAnalysis, ReleaseWorkflow},
        errors::SemanticReleaseDomainError,
        value_objects::{SemanticVersion, ReleaseChannel, ReleaseConfiguration}
    }
};
use async_trait::async_trait;

/// Port for semantic release operations
/// 
/// This trait defines all operations for managing semantic releases.
/// It will be implemented by infrastructure adapters.
#[async_trait]
pub trait SemanticReleasePort: Send + Sync {
    /// Gets the current version from the repository
    async fn get_current_version(&self) -> Result<Option<SemanticVersion>, SemanticReleaseDomainError>;
    
    /// Gets all releases in the repository
    async fn get_all_releases(&self) -> Result<Vec<SemanticRelease>, SemanticReleaseDomainError>;
    
    /// Gets releases for a specific channel
    async fn get_releases_for_channel(
        &self,
        channel: &ReleaseChannel,
    ) -> Result<Vec<SemanticRelease>, SemanticReleaseDomainError>;
    
    /// Creates a new release
    async fn create_release(
        &self,
        release: &SemanticRelease,
    ) -> Result<(), SemanticReleaseDomainError>;
    
    /// Updates an existing release
    async fn update_release(
        &self,
        release: &SemanticRelease,
    ) -> Result<(), SemanticReleaseDomainError>;
    
    /// Deletes a release
    async fn delete_release(
        &self,
        version: &SemanticVersion,
    ) -> Result<(), SemanticReleaseDomainError>;
    
    /// Checks if a version already exists
    async fn version_exists(
        &self,
        version: &SemanticVersion,
    ) -> Result<bool, SemanticReleaseDomainError>;
    
    /// Gets the latest release for a channel
    async fn get_latest_release(
        &self,
        channel: &ReleaseChannel,
    ) -> Result<Option<SemanticRelease>, SemanticReleaseDomainError>;
    
    /// Gets the release configuration
    async fn get_configuration(&self) -> Result<ReleaseConfiguration, SemanticReleaseDomainError>;
    
    /// Updates the release configuration
    async fn update_configuration(
        &self,
        config: &ReleaseConfiguration,
    ) -> Result<(), SemanticReleaseDomainError>;
}

/// Port for version analysis operations
/// 
/// This trait provides operations for analyzing commits and determining version bumps.
#[async_trait]
pub trait VersionAnalysisPort: Send + Sync {
    /// Analyzes commits to determine the appropriate version bump
    async fn analyze_version_bump(
        &self,
        current_version: Option<&SemanticVersion>,
        commits: &[GitCommit],
    ) -> Result<VersionBumpAnalysis, SemanticReleaseDomainError>;
    
    /// Validates a proposed version against the current state
    async fn validate_version(
        &self,
        current_version: Option<&SemanticVersion>,
        proposed_version: &SemanticVersion,
        commits: &[GitCommit],
    ) -> Result<VersionValidationResult, SemanticReleaseDomainError>;
    
    /// Gets the next suggested version
    async fn get_next_version(
        &self,
        current_version: Option<&SemanticVersion>,
        commits: &[GitCommit],
        channel: &ReleaseChannel,
    ) -> Result<SemanticVersion, SemanticReleaseDomainError>;
    
    /// Checks if changes warrant a release
    async fn should_release(
        &self,
        commits: &[GitCommit],
    ) -> Result<bool, SemanticReleaseDomainError>;
}

/// Port for release workflow operations
/// 
/// This trait handles the execution of release workflows.
#[async_trait]
pub trait ReleaseWorkflowPort: Send + Sync {
    /// Starts a new release workflow
    async fn start_workflow(
        &self,
        release: SemanticRelease,
        config: ReleaseConfiguration,
    ) -> Result<ReleaseWorkflow, SemanticReleaseDomainError>;
    
    /// Executes a workflow step
    async fn execute_workflow_step(
        &self,
        workflow: &mut ReleaseWorkflow,
    ) -> Result<WorkflowStepResult, SemanticReleaseDomainError>;
    
    /// Gets the status of a workflow
    async fn get_workflow_status(
        &self,
        workflow_id: &str,
    ) -> Result<Option<ReleaseWorkflow>, SemanticReleaseDomainError>;
    
    /// Cancels a workflow
    async fn cancel_workflow(
        &self,
        workflow_id: &str,
    ) -> Result<(), SemanticReleaseDomainError>;
    
    /// Gets all active workflows
    async fn get_active_workflows(&self) -> Result<Vec<ReleaseWorkflow>, SemanticReleaseDomainError>;
}

/// Result of version validation
#[derive(Debug, Clone)]
pub struct VersionValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub recommendations: Vec<String>,
}

/// A validation issue found during version validation
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
    pub suggested_fix: Option<String>,
}

/// Severity level of a validation issue
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Result of executing a workflow step
#[derive(Debug, Clone)]
pub enum WorkflowStepResult {
    Success { message: String },
    Failure { error: String },
    Skipped { reason: String },
    InProgress { progress: f32 },
}

impl VersionValidationResult {
    /// Creates a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }
    
    /// Adds an error issue
    pub fn add_error(&mut self, message: String, suggested_fix: Option<String>) {
        self.is_valid = false;
        self.issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            message,
            suggested_fix,
        });
    }
    
    /// Adds a warning issue
    pub fn add_warning(&mut self, message: String, suggested_fix: Option<String>) {
        self.issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message,
            suggested_fix,
        });
    }
    
    /// Adds an info issue
    pub fn add_info(&mut self, message: String) {
        self.issues.push(ValidationIssue {
            severity: ValidationSeverity::Info,
            message,
            suggested_fix: None,
        });
    }
    
    /// Adds a recommendation
    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
    
    /// Checks if there are any errors
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|issue| issue.severity == ValidationSeverity::Error)
    }
    
    /// Checks if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|issue| issue.severity == ValidationSeverity::Warning)
    }
} 