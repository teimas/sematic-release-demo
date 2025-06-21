//! Semantic release domain entities
//! 
//! Rich domain entities that encapsulate semantic release state and behavior.
//! These entities contain domain logic and enforce semantic versioning rules.

use crate::domains::{
    git::entities::{GitCommit, CommitType},
    semantic::{
        errors::SemanticReleaseDomainError,
        value_objects::{SemanticVersion, VersionBumpType, ReleaseChannel, ReleaseConfiguration}
    }
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a semantic release with rich domain behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelease {
    pub version: SemanticVersion,
    pub channel: ReleaseChannel,
    pub commits: Vec<GitCommit>,
    pub release_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub is_draft: bool,
    pub tag_name: String,
}

impl SemanticRelease {
    /// Creates a new semantic release
    pub fn new(
        version: SemanticVersion,
        channel: ReleaseChannel,
        commits: Vec<GitCommit>,
    ) -> Self {
        let tag_name = format!("v{}", version);
        
        Self {
            version,
            channel,
            commits,
            release_notes: None,
            created_at: Utc::now(),
            published_at: None,
            is_published: false,
            is_draft: true,
            tag_name,
        }
    }
    
    /// Publishes the release
    pub fn publish(&mut self) -> Result<(), SemanticReleaseDomainError> {
        if self.is_published {
            return Err(SemanticReleaseDomainError::WorkflowStepFailed {
                step: "publish".to_string(),
            });
        }
        
        self.is_published = true;
        self.is_draft = false;
        self.published_at = Some(Utc::now());
        
        Ok(())
    }
    
    /// Sets the release notes
    pub fn set_release_notes(&mut self, notes: String) {
        self.release_notes = Some(notes);
    }
    
    /// Checks if this release contains breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        self.commits.iter().any(|commit| {
            matches!(commit.commit_type(), CommitType::BreakingChange)
        })
    }
    
    /// Gets the number of features in this release
    pub fn feature_count(&self) -> usize {
        self.commits.iter()
            .filter(|commit| matches!(commit.commit_type(), CommitType::Feature))
            .count()
    }
    
    /// Gets the number of fixes in this release
    pub fn fix_count(&self) -> usize {
        self.commits.iter()
            .filter(|commit| matches!(commit.commit_type(), CommitType::Fix))
            .count()
    }
    
    /// Gets a summary of changes in this release
    pub fn change_summary(&self) -> String {
        let breaking_count = self.commits.iter()
            .filter(|c| matches!(c.commit_type(), CommitType::BreakingChange))
            .count();
        
        let feature_count = self.feature_count();
        let fix_count = self.fix_count();
        
        let mut parts = Vec::new();
        
        if breaking_count > 0 {
            parts.push(format!("{} breaking change(s)", breaking_count));
        }
        
        if feature_count > 0 {
            parts.push(format!("{} feature(s)", feature_count));
        }
        
        if fix_count > 0 {
            parts.push(format!("{} fix(es)", fix_count));
        }
        
        if parts.is_empty() {
            "No significant changes".to_string()
        } else {
            parts.join(", ")
        }
    }
    
    /// Validates the release before publishing
    pub fn validate(&self) -> Result<(), SemanticReleaseDomainError> {
        if self.commits.is_empty() {
            return Err(SemanticReleaseDomainError::NoChangesForRelease);
        }
        
        // Check if breaking changes require major version bump
        if self.has_breaking_changes() && self.version.major == 0 {
            // For 0.x.x versions, breaking changes can be minor bumps
            // This follows semantic versioning specification for initial development
        } else if self.has_breaking_changes() {
            // For stable versions (1.x.x+), breaking changes must be major bumps
            let expected_version = if let Some(previous_version) = self.get_previous_version() {
                previous_version.increment_major()
            } else {
                SemanticVersion::new(1, 0, 0)
            };
            
            if self.version.major <= expected_version.major {
                return Err(SemanticReleaseDomainError::BreakingChangesRequireMajor {
                    suggested_version: expected_version.to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Gets the previous version (placeholder for now)
    fn get_previous_version(&self) -> Option<SemanticVersion> {
        // This would be implemented to get the actual previous version
        // For now, returning None as placeholder
        None
    }
}

/// Represents a version bump analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBumpAnalysis {
    pub current_version: Option<SemanticVersion>,
    pub suggested_version: SemanticVersion,
    pub bump_type: VersionBumpType,
    pub reason: String,
    pub commits_analyzed: Vec<GitCommit>,
    pub has_breaking_changes: bool,
    pub has_features: bool,
    pub has_fixes: bool,
}

impl VersionBumpAnalysis {
    /// Creates a new version bump analysis
    pub fn new(
        current_version: Option<SemanticVersion>,
        commits: Vec<GitCommit>,
    ) -> Self {
        let (bump_type, reason) = Self::analyze_commits(&commits);
        
        let suggested_version = if let Some(ref current) = current_version {
            bump_type.apply(current)
        } else {
            // If no current version, start with 1.0.0 or 0.1.0 based on breaking changes
            let has_breaking = commits.iter().any(|c| matches!(c.commit_type(), CommitType::BreakingChange));
            if has_breaking {
                SemanticVersion::new(1, 0, 0)
            } else {
                SemanticVersion::new(0, 1, 0)
            }
        };
        
        let has_breaking_changes = commits.iter().any(|c| matches!(c.commit_type(), CommitType::BreakingChange));
        let has_features = commits.iter().any(|c| matches!(c.commit_type(), CommitType::Feature));
        let has_fixes = commits.iter().any(|c| matches!(c.commit_type(), CommitType::Fix));
        
        Self {
            current_version,
            suggested_version,
            bump_type,
            reason,
            commits_analyzed: commits,
            has_breaking_changes,
            has_features,
            has_fixes,
        }
    }
    
    /// Analyzes commits to determine the appropriate version bump
    fn analyze_commits(commits: &[GitCommit]) -> (VersionBumpType, String) {
        let has_breaking = commits.iter().any(|c| matches!(c.commit_type(), CommitType::BreakingChange));
        let has_features = commits.iter().any(|c| matches!(c.commit_type(), CommitType::Feature));
        let has_fixes = commits.iter().any(|c| matches!(c.commit_type(), CommitType::Fix));
        
        if has_breaking {
            (VersionBumpType::Major, "Breaking changes detected".to_string())
        } else if has_features {
            (VersionBumpType::Minor, "New features added".to_string())
        } else if has_fixes {
            (VersionBumpType::Patch, "Bug fixes applied".to_string())
        } else {
            (VersionBumpType::None, "No significant changes".to_string())
        }
    }
    
    /// Checks if a version bump is needed
    pub fn needs_version_bump(&self) -> bool {
        !matches!(self.bump_type, VersionBumpType::None)
    }
}

/// Represents the release workflow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseWorkflow {
    pub release: SemanticRelease,
    pub configuration: ReleaseConfiguration,
    pub current_step: WorkflowStep,
    pub completed_steps: Vec<WorkflowStep>,
    pub failed_steps: Vec<(WorkflowStep, String)>,
}

impl ReleaseWorkflow {
    /// Creates a new release workflow
    pub fn new(release: SemanticRelease, configuration: ReleaseConfiguration) -> Self {
        Self {
            release,
            configuration,
            current_step: WorkflowStep::Validate,
            completed_steps: Vec::new(),
            failed_steps: Vec::new(),
        }
    }
    
    /// Executes the next step in the workflow
    pub fn execute_next_step(&mut self) -> Result<WorkflowStepResult, SemanticReleaseDomainError> {
        match self.current_step {
            WorkflowStep::Validate => self.execute_validate_step(),
            WorkflowStep::GenerateReleaseNotes => self.execute_generate_notes_step(),
            WorkflowStep::CreateGitTag => self.execute_create_tag_step(),
            WorkflowStep::PushToRemote => self.execute_push_step(),
            WorkflowStep::Publish => self.execute_publish_step(),
            WorkflowStep::Complete => Ok(WorkflowStepResult::AlreadyComplete),
        }
    }
    
    /// Executes the validation step
    fn execute_validate_step(&mut self) -> Result<WorkflowStepResult, SemanticReleaseDomainError> {
        self.release.validate()?;
        self.configuration.validate()?;
        
        self.completed_steps.push(WorkflowStep::Validate);
        self.current_step = WorkflowStep::GenerateReleaseNotes;
        
        Ok(WorkflowStepResult::Success {
            message: "Release validation completed successfully".to_string(),
        })
    }
    
    /// Executes the release notes generation step
    fn execute_generate_notes_step(&mut self) -> Result<WorkflowStepResult, SemanticReleaseDomainError> {
        if self.configuration.generate_release_notes {
            // Generate release notes (placeholder implementation)
            let notes = format!(
                "## Release {}\n\n{}",
                self.release.version,
                self.release.change_summary()
            );
            self.release.set_release_notes(notes);
        }
        
        self.completed_steps.push(WorkflowStep::GenerateReleaseNotes);
        self.current_step = WorkflowStep::CreateGitTag;
        
        Ok(WorkflowStepResult::Success {
            message: "Release notes generated successfully".to_string(),
        })
    }
    
    /// Executes the git tag creation step
    fn execute_create_tag_step(&mut self) -> Result<WorkflowStepResult, SemanticReleaseDomainError> {
        if !self.configuration.create_git_tag {
            self.completed_steps.push(WorkflowStep::CreateGitTag);
            self.current_step = WorkflowStep::PushToRemote;
            return Ok(WorkflowStepResult::Skipped {
                reason: "Git tag creation disabled in configuration".to_string(),
            });
        }
        
        // Create git tag (this would be handled by infrastructure layer)
        self.completed_steps.push(WorkflowStep::CreateGitTag);
        self.current_step = WorkflowStep::PushToRemote;
        
        Ok(WorkflowStepResult::Success {
            message: format!("Git tag {} created successfully", self.release.tag_name),
        })
    }
    
    /// Executes the push to remote step
    fn execute_push_step(&mut self) -> Result<WorkflowStepResult, SemanticReleaseDomainError> {
        if !self.configuration.push_to_remote {
            self.completed_steps.push(WorkflowStep::PushToRemote);
            self.current_step = WorkflowStep::Publish;
            return Ok(WorkflowStepResult::Skipped {
                reason: "Push to remote disabled in configuration".to_string(),
            });
        }
        
        // Push to remote (this would be handled by infrastructure layer)
        self.completed_steps.push(WorkflowStep::PushToRemote);
        self.current_step = WorkflowStep::Publish;
        
        Ok(WorkflowStepResult::Success {
            message: "Successfully pushed to remote repository".to_string(),
        })
    }
    
    /// Executes the publish step
    fn execute_publish_step(&mut self) -> Result<WorkflowStepResult, SemanticReleaseDomainError> {
        self.release.publish()?;
        
        self.completed_steps.push(WorkflowStep::Publish);
        self.current_step = WorkflowStep::Complete;
        
        Ok(WorkflowStepResult::Success {
            message: format!("Release {} published successfully", self.release.version),
        })
    }
    
    /// Checks if the workflow is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.current_step, WorkflowStep::Complete)
    }
    
    /// Gets the progress percentage
    pub fn progress_percentage(&self) -> f32 {
        let total_steps = 5;
        let completed = self.completed_steps.len() as f32;
        (completed / total_steps as f32) * 100.0
    }
}

/// Represents a workflow step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStep {
    Validate,
    GenerateReleaseNotes,
    CreateGitTag,
    PushToRemote,
    Publish,
    Complete,
}

impl WorkflowStep {
    /// Gets the display name for the step
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Validate => "Validate Release",
            Self::GenerateReleaseNotes => "Generate Release Notes",
            Self::CreateGitTag => "Create Git Tag",
            Self::PushToRemote => "Push to Remote",
            Self::Publish => "Publish Release",
            Self::Complete => "Complete",
        }
    }
}

/// Result of executing a workflow step
#[derive(Debug, Clone)]
pub enum WorkflowStepResult {
    Success { message: String },
    Skipped { reason: String },
    AlreadyComplete,
} 