//! Semantic release domain services
//! 
//! Domain services that implement complex semantic release business logic
//! involving multiple entities and value objects.

use crate::domains::{
    git::entities::GitCommit,
    semantic::{
        entities::{SemanticRelease, VersionBumpAnalysis, ReleaseWorkflow},
        errors::SemanticReleaseDomainError,
        repository::{
            SemanticReleasePort, VersionAnalysisPort, ReleaseWorkflowPort, 
            VersionValidationResult, ValidationSeverity
        },
        value_objects::{
            SemanticVersion, VersionBumpType, ReleaseChannel, ReleaseConfiguration
        }
    }
};
use std::sync::Arc;

/// Semantic release domain service providing high-level release operations
pub struct SemanticReleaseDomainService {
    release_port: Arc<dyn SemanticReleasePort>,
    analysis_port: Arc<dyn VersionAnalysisPort>,
    workflow_port: Arc<dyn ReleaseWorkflowPort>,
}

impl SemanticReleaseDomainService {
    /// Creates a new semantic release domain service
    pub fn new(
        release_port: Arc<dyn SemanticReleasePort>,
        analysis_port: Arc<dyn VersionAnalysisPort>,
        workflow_port: Arc<dyn ReleaseWorkflowPort>,
    ) -> Self {
        Self {
            release_port,
            analysis_port,
            workflow_port,
        }
    }
    
    /// Analyzes changes and suggests the next version
    pub async fn analyze_next_version(
        &self,
        commits: &[GitCommit],
        channel: &ReleaseChannel,
    ) -> Result<VersionBumpAnalysis, SemanticReleaseDomainError> {
        let current_version = self.release_port.get_current_version().await?;
        self.analysis_port
            .analyze_version_bump(current_version.as_ref(), commits)
            .await
    }
    
    /// Creates a new release with validation
    pub async fn create_release(
        &self,
        version: SemanticVersion,
        channel: ReleaseChannel,
        commits: Vec<GitCommit>,
    ) -> Result<SemanticRelease, SemanticReleaseDomainError> {
        // Check if version already exists
        if self.release_port.version_exists(&version).await? {
            return Err(SemanticReleaseDomainError::VersionAlreadyExists {
                version: version.to_string(),
            });
        }
        
        // Validate the version against commits
        let current_version = self.release_port.get_current_version().await?;
        let validation_result = self.analysis_port
            .validate_version(current_version.as_ref(), &version, &commits)
            .await?;
        
        if !validation_result.is_valid {
            let error_messages: Vec<String> = validation_result.issues
                .into_iter()
                .filter(|issue| issue.severity == ValidationSeverity::Error)
                .map(|issue| issue.message)
                .collect();
            
            return Err(SemanticReleaseDomainError::InvalidReleaseConfiguration {
                reason: error_messages.join("; "),
            });
        }
        
        // Create the release
        let release = SemanticRelease::new(version, channel, commits);
        
        // Validate the release itself
        release.validate()?;
        
        // Store the release
        self.release_port.create_release(&release).await?;
        
        Ok(release)
    }
    
    /// Starts a release workflow
    pub async fn start_release_workflow(
        &self,
        release: SemanticRelease,
    ) -> Result<ReleaseWorkflow, SemanticReleaseDomainError> {
        let config = self.release_port.get_configuration().await?;
        self.workflow_port.start_workflow(release, config).await
    }
    
    /// Gets the current version
    pub async fn get_current_version(&self) -> Result<Option<SemanticVersion>, SemanticReleaseDomainError> {
        self.release_port.get_current_version().await
    }
    
    /// Gets all releases for a channel
    pub async fn get_releases_for_channel(
        &self,
        channel: &ReleaseChannel,
    ) -> Result<Vec<SemanticRelease>, SemanticReleaseDomainError> {
        self.release_port.get_releases_for_channel(channel).await
    }
    
    /// Checks if changes warrant a new release
    pub async fn should_create_release(
        &self,
        commits: &[GitCommit],
    ) -> Result<bool, SemanticReleaseDomainError> {
        self.analysis_port.should_release(commits).await
    }
}

/// Service for semantic version management and comparison
pub struct SemanticVersionService;

impl SemanticVersionService {
    /// Compares two semantic versions
    pub fn compare_versions(a: &SemanticVersion, b: &SemanticVersion) -> std::cmp::Ordering {
        a.cmp(b)
    }
    
    /// Checks if a version is greater than another
    pub fn is_greater_than(a: &SemanticVersion, b: &SemanticVersion) -> bool {
        a > b
    }
    
    /// Checks if a version is compatible with another (same major version)
    pub fn is_compatible(a: &SemanticVersion, b: &SemanticVersion) -> bool {
        a.is_compatible_with(b)
    }
    
    /// Gets the latest version from a list
    pub fn get_latest_version(versions: &[SemanticVersion]) -> Option<&SemanticVersion> {
        versions.iter().max()
    }
    
    /// Filters versions by stability (stable vs pre-release)
    pub fn filter_stable_versions(versions: &[SemanticVersion]) -> Vec<&SemanticVersion> {
        versions.iter().filter(|v| v.is_stable()).collect()
    }
    
    /// Filters versions by pre-release
    pub fn filter_pre_release_versions(versions: &[SemanticVersion]) -> Vec<&SemanticVersion> {
        versions.iter().filter(|v| v.is_pre_release()).collect()
    }
    
    /// Gets the next version based on bump type
    pub fn calculate_next_version(
        current: &SemanticVersion,
        bump_type: &VersionBumpType,
    ) -> SemanticVersion {
        bump_type.apply(current)
    }
    
    /// Validates a version string format
    pub fn validate_version_string(version: &str) -> Result<(), SemanticReleaseDomainError> {
        SemanticVersion::parse(version).map(|_| ())
    }
    
    /// Parses multiple version strings
    pub fn parse_versions(version_strings: &[String]) -> Result<Vec<SemanticVersion>, SemanticReleaseDomainError> {
        version_strings
            .iter()
            .map(|v| SemanticVersion::parse(v))
            .collect()
    }
}

/// Service for release channel management
pub struct ReleaseChannelService;

impl ReleaseChannelService {
    /// Gets the appropriate channel for a version
    pub fn get_channel_for_version(version: &SemanticVersion) -> ReleaseChannel {
        if version.is_pre_release() {
            if let Some(pre_release) = &version.pre_release {
                if pre_release.starts_with("alpha") {
                    ReleaseChannel::alpha()
                } else if pre_release.starts_with("beta") {
                    ReleaseChannel::beta()
                } else {
                    ReleaseChannel::beta() // Default pre-release channel
                }
            } else {
                ReleaseChannel::beta()
            }
        } else {
            ReleaseChannel::stable()
        }
    }
    
    /// Validates that a version is appropriate for a channel
    pub fn validate_version_for_channel(
        version: &SemanticVersion,
        channel: &ReleaseChannel,
    ) -> Result<(), SemanticReleaseDomainError> {
        match (version.is_pre_release(), channel.is_stable()) {
            (true, true) => Err(SemanticReleaseDomainError::InvalidReleaseChannel {
                channel: channel.to_string(),
            }),
            (false, false) => {
                if !channel.is_stable() {
                    Err(SemanticReleaseDomainError::InvalidReleaseChannel {
                        channel: channel.to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    
    /// Gets the promotion path for channels (alpha -> beta -> stable)
    pub fn get_promotion_path(from: &ReleaseChannel) -> Vec<ReleaseChannel> {
        match from.as_str() {
            "alpha" => vec![ReleaseChannel::beta(), ReleaseChannel::stable()],
            "beta" => vec![ReleaseChannel::stable()],
            "stable" => vec![], // No promotion from stable
            _ => vec![ReleaseChannel::stable()], // Default to stable
        }
    }
    
    /// Checks if a channel can be promoted to another
    pub fn can_promote_to(from: &ReleaseChannel, to: &ReleaseChannel) -> bool {
        let promotion_path = Self::get_promotion_path(from);
        promotion_path.contains(to)
    }
}

/// Service for release workflow management
pub struct ReleaseWorkflowService;

impl ReleaseWorkflowService {
    /// Validates a release workflow configuration
    pub fn validate_workflow_config(
        config: &ReleaseConfiguration,
    ) -> Result<(), SemanticReleaseDomainError> {
        config.validate()
    }
    
    /// Estimates workflow duration based on configuration
    pub fn estimate_workflow_duration(config: &ReleaseConfiguration) -> std::time::Duration {
        let mut duration = std::time::Duration::from_secs(30); // Base validation time
        
        if config.generate_release_notes {
            duration += std::time::Duration::from_secs(60); // Release notes generation
        }
        
        if config.create_git_tag {
            duration += std::time::Duration::from_secs(30); // Git tag creation
        }
        
        if config.push_to_remote {
            duration += std::time::Duration::from_secs(120); // Push to remote
        }
        
        duration += std::time::Duration::from_secs(60); // Publishing
        
        duration
    }
    
    /// Gets workflow steps based on configuration
    pub fn get_workflow_steps(config: &ReleaseConfiguration) -> Vec<&'static str> {
        let mut steps = vec!["Validate"];
        
        if config.generate_release_notes {
            steps.push("Generate Release Notes");
        }
        
        if config.create_git_tag {
            steps.push("Create Git Tag");
        }
        
        if config.push_to_remote {
            steps.push("Push to Remote");
        }
        
        steps.push("Publish");
        steps
    }
    
    /// Calculates risk level for a workflow
    pub fn calculate_workflow_risk(
        release: &SemanticRelease,
        config: &ReleaseConfiguration,
    ) -> WorkflowRiskLevel {
        let mut risk_factors = 0;
        
        if release.has_breaking_changes() {
            risk_factors += 3;
        }
        
        if release.commits.len() > 20 {
            risk_factors += 2;
        }
        
        if config.push_to_remote {
            risk_factors += 1;
        }
        
        if !config.require_clean_working_directory {
            risk_factors += 2;
        }
        
        match risk_factors {
            0..=2 => WorkflowRiskLevel::Low,
            3..=5 => WorkflowRiskLevel::Medium,
            6..=8 => WorkflowRiskLevel::High,
            _ => WorkflowRiskLevel::Critical,
        }
    }
}

/// Risk level for release workflows
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl WorkflowRiskLevel {
    /// Gets the display name for the risk level
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "Low Risk",
            Self::Medium => "Medium Risk",
            Self::High => "High Risk",
            Self::Critical => "Critical Risk",
        }
    }
    
    /// Gets the color associated with the risk level
    pub fn color(&self) -> &'static str {
        match self {
            Self::Low => "green",
            Self::Medium => "yellow",
            Self::High => "orange",
            Self::Critical => "red",
        }
    }
} 