//! Git domain entities
//! 
//! Rich domain entities that encapsulate git repository state and behavior.
//! These entities contain domain logic and enforce business rules.

use crate::domains::git::{
    errors::GitDomainError,
    value_objects::{BranchName, CommitHash, CommitMessage, GitRemoteUrl, TagName}
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a git repository with rich domain behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    pub path: PathBuf,
    pub current_branch: Option<BranchName>,
    pub head_commit: Option<CommitHash>,
    pub is_dirty: bool,
    pub remote_url: Option<GitRemoteUrl>,
    pub last_tag: Option<TagName>,
    pub untracked_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub staged_files: Vec<String>,
}

impl GitRepository {
    /// Creates a new git repository entity
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            current_branch: None,
            head_commit: None,
            is_dirty: false,
            remote_url: None,
            last_tag: None,
            untracked_files: Vec::new(),
            modified_files: Vec::new(),
            staged_files: Vec::new(),
        }
    }
    
    /// Checks if the repository is ready for a release
    pub fn is_ready_for_release(&self) -> Result<(), GitDomainError> {
        if self.is_dirty {
            return Err(GitDomainError::WorkingDirectoryDirty);
        }
        
        if self.current_branch.is_none() {
            return Err(GitDomainError::OperationFailed {
                operation: "get current branch".to_string(),
            });
        }
        
        if self.remote_url.is_none() {
            return Err(GitDomainError::RemoteNotConfigured {
                remote: "origin".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Checks if currently on main branch
    pub fn is_on_main_branch(&self) -> bool {
        self.current_branch
            .as_ref()
            .map(|branch| branch.is_main_branch())
            .unwrap_or(false)
    }
    
    /// Gets the working directory status summary
    pub fn status_summary(&self) -> String {
        let mut parts = Vec::new();
        
        if !self.staged_files.is_empty() {
            parts.push(format!("{} staged", self.staged_files.len()));
        }
        
        if !self.modified_files.is_empty() {
            parts.push(format!("{} modified", self.modified_files.len()));
        }
        
        if !self.untracked_files.is_empty() {
            parts.push(format!("{} untracked", self.untracked_files.len()));
        }
        
        if parts.is_empty() {
            "clean".to_string()
        } else {
            parts.join(", ")
        }
    }
    
    /// Updates the repository dirty state based on file changes
    pub fn update_dirty_state(&mut self) {
        self.is_dirty = !self.staged_files.is_empty() 
            || !self.modified_files.is_empty() 
            || !self.untracked_files.is_empty();
    }
}

/// Represents a git commit with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: CommitHash,
    pub message: CommitMessage,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: DateTime<Utc>,
    pub parent_hashes: Vec<CommitHash>,
}

impl GitCommit {
    /// Creates a new git commit entity
    pub fn new(
        hash: CommitHash,
        message: CommitMessage,
        author_name: String,
        author_email: String,
        timestamp: DateTime<Utc>,
        parent_hashes: Vec<CommitHash>,
    ) -> Self {
        Self {
            hash,
            message,
            author_name,
            author_email,
            timestamp,
            parent_hashes,
        }
    }
    
    /// Checks if this commit is a merge commit
    pub fn is_merge_commit(&self) -> bool {
        self.parent_hashes.len() > 1
    }
    
    /// Gets the commit type for release notes categorization
    pub fn commit_type(&self) -> CommitType {
        if self.message.is_breaking_change() {
            CommitType::BreakingChange
        } else if self.message.is_feature() {
            CommitType::Feature
        } else if self.message.is_fix() {
            CommitType::Fix
        } else {
            CommitType::Other
        }
    }
    
    /// Gets a short summary of the commit
    pub fn summary(&self) -> String {
        format!(
            "{} - {} ({})",
            self.hash.short(),
            self.message.subject,
            self.author_name
        )
    }
}

/// Represents a git branch with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: BranchName,
    pub head_commit: CommitHash,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
}

impl GitBranch {
    /// Creates a new git branch entity
    pub fn new(
        name: BranchName,
        head_commit: CommitHash,
        is_current: bool,
        is_remote: bool,
    ) -> Self {
        Self {
            name,
            head_commit,
            is_current,
            is_remote,
            upstream: None,
        }
    }
    
    /// Checks if this branch is protected (main/master)
    pub fn is_protected(&self) -> bool {
        self.name.is_main_branch()
    }
    
    /// Gets branch type for display purposes
    pub fn branch_type(&self) -> &'static str {
        if self.is_remote {
            "remote"
        } else if self.is_current {
            "current"
        } else {
            "local"
        }
    }
}

/// Represents a git tag with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitTag {
    pub name: TagName,
    pub commit_hash: CommitHash,
    pub message: Option<String>,
    pub tagger_name: Option<String>,
    pub tagger_email: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

impl GitTag {
    /// Creates a new git tag entity
    pub fn new(name: TagName, commit_hash: CommitHash) -> Self {
        Self {
            name,
            commit_hash,
            message: None,
            tagger_name: None,
            tagger_email: None,
            timestamp: None,
        }
    }
    
    /// Checks if this is a release tag
    pub fn is_release_tag(&self) -> bool {
        self.name.is_semantic_version()
    }
}

/// Enumeration of commit types for release categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitType {
    Feature,
    Fix,
    BreakingChange,
    Documentation,
    Style,
    Refactor,
    Performance,
    Test,
    Chore,
    Other,
}

impl CommitType {
    /// Gets the release impact of this commit type
    pub fn release_impact(&self) -> ReleaseImpact {
        match self {
            Self::BreakingChange => ReleaseImpact::Major,
            Self::Feature => ReleaseImpact::Minor,
            Self::Fix => ReleaseImpact::Patch,
            _ => ReleaseImpact::None,
        }
    }
    
    /// Gets the display name for this commit type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Feature => "Features",
            Self::Fix => "Bug Fixes",
            Self::BreakingChange => "Breaking Changes",
            Self::Documentation => "Documentation",
            Self::Style => "Code Style",
            Self::Refactor => "Refactoring",
            Self::Performance => "Performance",
            Self::Test => "Tests",
            Self::Chore => "Chores",
            Self::Other => "Other",
        }
    }
}

/// Enumeration of release impact levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReleaseImpact {
    None,
    Patch,
    Minor,
    Major,
}

/// Represents the preparation state for a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasePreparation {
    pub target_branch: BranchName,
    pub commit_hash: CommitHash,
    pub tag_name: TagName,
    pub commits_since_last_tag: Vec<GitCommit>,
    pub release_impact: ReleaseImpact,
}

impl ReleasePreparation {
    /// Creates a new release preparation
    pub fn new(
        target_branch: BranchName,
        commit_hash: CommitHash,
        tag_name: TagName,
        commits_since_last_tag: Vec<GitCommit>,
    ) -> Self {
        let release_impact = commits_since_last_tag
            .iter()
            .map(|commit| commit.commit_type().release_impact())
            .max()
            .unwrap_or(ReleaseImpact::None);
        
        Self {
            target_branch,
            commit_hash,
            tag_name,
            commits_since_last_tag,
            release_impact,
        }
    }
    
    /// Checks if this release contains breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        self.release_impact == ReleaseImpact::Major
    }
    
    /// Gets a summary of changes in this release
    pub fn change_summary(&self) -> String {
        let feature_count = self.commits_since_last_tag
            .iter()
            .filter(|c| c.commit_type() == CommitType::Feature)
            .count();
        
        let fix_count = self.commits_since_last_tag
            .iter()
            .filter(|c| c.commit_type() == CommitType::Fix)
            .count();
        
        let breaking_count = self.commits_since_last_tag
            .iter()
            .filter(|c| c.commit_type() == CommitType::BreakingChange)
            .count();
        
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
} 