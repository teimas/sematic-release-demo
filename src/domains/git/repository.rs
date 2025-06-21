//! Git repository port interface
//! 
//! This module defines the port (interface) for git repository operations.
//! The actual implementation will be provided by the infrastructure layer.

use crate::domains::git::{
    entities::{GitRepository, GitCommit, GitBranch, GitTag, ReleasePreparation},
    value_objects::{CommitHash, BranchName, TagName, CommitMessage},
    errors::GitDomainError,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Port for git repository factory operations
/// 
/// This trait handles repository creation and opening operations.
#[async_trait]
pub trait GitRepositoryFactory: Send + Sync {
    /// Opens an existing git repository
    async fn open(&self, path: &str) -> Result<GitRepository, GitDomainError>;
    
    /// Initializes a new git repository
    async fn init(&self, path: &str) -> Result<GitRepository, GitDomainError>;
}

/// Port for git repository operations
/// 
/// This trait defines all operations that can be performed on a git repository.
/// It will be implemented by infrastructure adapters (git2, CLI, etc.)
#[async_trait]
pub trait GitRepositoryPort: Send + Sync {
    
    /// Gets the current status of the repository
    async fn status(&self, repo: &GitRepository) -> Result<GitRepository, GitDomainError>;
    
    /// Gets the current branch
    async fn current_branch(&self, repo: &GitRepository) -> Result<Option<BranchName>, GitDomainError>;
    
    /// Gets all branches (local and remote)
    async fn branches(&self, repo: &GitRepository) -> Result<Vec<GitBranch>, GitDomainError>;
    
    /// Gets commits since a specific tag or commit
    async fn commits_since(
        &self,
        repo: &GitRepository,
        since: Option<&TagName>,
        until: Option<&CommitHash>,
    ) -> Result<Vec<GitCommit>, GitDomainError>;
    
    /// Gets all tags in the repository
    async fn tags(&self, repo: &GitRepository) -> Result<Vec<GitTag>, GitDomainError>;
    
    /// Gets the latest tag
    async fn latest_tag(&self, repo: &GitRepository) -> Result<Option<GitTag>, GitDomainError>;
    
    /// Creates a new tag
    async fn create_tag(
        &self,
        repo: &GitRepository,
        tag_name: &TagName,
        commit: &CommitHash,
        message: Option<&str>,
    ) -> Result<GitTag, GitDomainError>;
    
    /// Pushes tags to remote
    async fn push_tags(
        &self,
        repo: &GitRepository,
        remote: Option<&str>,
    ) -> Result<(), GitDomainError>;
    
    /// Gets the remote URL for a specific remote
    async fn remote_url(
        &self,
        repo: &GitRepository,
        remote: &str,
    ) -> Result<Option<String>, GitDomainError>;
    
    /// Checks if the working directory is clean
    async fn is_clean(&self, repo: &GitRepository) -> Result<bool, GitDomainError>;
    
    /// Gets detailed diff information
    async fn get_diff(
        &self,
        repo: &GitRepository,
        staged: bool,
    ) -> Result<String, GitDomainError>;
    
    /// Stages files for commit
    async fn stage_files(
        &self,
        repo: &GitRepository,
        files: &[String],
    ) -> Result<(), GitDomainError>;
    
    /// Creates a commit
    async fn commit(
        &self,
        repo: &GitRepository,
        message: &str,
        author_name: Option<&str>,
        author_email: Option<&str>,
    ) -> Result<GitCommit, GitDomainError>;
    
    /// Pushes commits to remote
    async fn push(
        &self,
        repo: &GitRepository,
        remote: Option<&str>,
        branch: Option<&BranchName>,
    ) -> Result<(), GitDomainError>;
    
    /// Fetches from remote
    async fn fetch(
        &self,
        repo: &GitRepository,
        remote: Option<&str>,
    ) -> Result<(), GitDomainError>;
}

/// Port for git analysis operations
/// 
/// This trait provides higher-level git analysis operations that build
/// on top of the basic repository operations.
#[async_trait]
pub trait GitAnalysisPort: Send + Sync {
    /// Analyzes changes for release preparation
    async fn prepare_release(
        &self,
        repo: &GitRepository,
        target_version: Option<(u32, u32, u32)>,
    ) -> Result<ReleasePreparation, GitDomainError>;
    
    /// Gets commit history with filtering options
    async fn get_commit_history(
        &self,
        repo: &GitRepository,
        since: Option<&TagName>,
        commit_types: Option<&[String]>,
        limit: Option<usize>,
    ) -> Result<Vec<GitCommit>, GitDomainError>;
    
    /// Analyzes the impact of pending changes
    async fn analyze_change_impact(
        &self,
        repo: &GitRepository,
    ) -> Result<ChangeImpactAnalysis, GitDomainError>;
    
    /// Validates repository for release readiness
    async fn validate_release_readiness(
        &self,
        repo: &GitRepository,
    ) -> Result<ReleaseReadinessReport, GitDomainError>;
}

/// Result of change impact analysis
#[derive(Debug, Clone)]
pub struct ChangeImpactAnalysis {
    pub has_breaking_changes: bool,
    pub has_features: bool,
    pub has_fixes: bool,
    pub suggested_version_bump: VersionBump,
    pub risk_level: RiskLevel,
    pub affected_areas: Vec<String>,
}

/// Result of release readiness validation
#[derive(Debug, Clone)]
pub struct ReleaseReadinessReport {
    pub is_ready: bool,
    pub blocking_issues: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Version bump recommendation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionBump {
    None,
    Patch,
    Minor,
    Major,
}

/// Risk level assessment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl VersionBump {
    /// Applies the version bump to a semantic version
    pub fn apply(&self, major: u32, minor: u32, patch: u32) -> (u32, u32, u32) {
        match self {
            Self::None => (major, minor, patch),
            Self::Patch => (major, minor, patch + 1),
            Self::Minor => (major, minor + 1, 0),
            Self::Major => (major + 1, 0, 0),
        }
    }
}
 