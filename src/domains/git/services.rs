//! Git domain services
//! 
//! Domain services that implement complex business logic involving
//! multiple entities and value objects.

use crate::domains::git::{
    entities::{CommitType, GitCommit, GitRepository, ReleasePreparation},
    errors::GitDomainError,
    repository::{
        ChangeImpactAnalysis, GitAnalysisPort, GitRepositoryFactory, GitRepositoryPort, 
        ReleaseReadinessReport, RiskLevel, VersionBump
    },
    value_objects::TagName
};
use std::sync::Arc;

/// Git domain service providing high-level repository operations
pub struct GitDomainService {
    repository_factory: Arc<dyn GitRepositoryFactory>,
    repository_port: Arc<dyn GitRepositoryPort>,
    analysis_port: Arc<dyn GitAnalysisPort>,
}

impl GitDomainService {
    /// Creates a new git domain service
    pub fn new(
        repository_factory: Arc<dyn GitRepositoryFactory>,
        repository_port: Arc<dyn GitRepositoryPort>,
        analysis_port: Arc<dyn GitAnalysisPort>,
    ) -> Self {
        Self {
            repository_factory,
            repository_port,
            analysis_port,
        }
    }
    
    /// Validates if a repository is ready for release
    pub async fn validate_release_readiness(
        &self,
        repo: &GitRepository,
    ) -> Result<ReleaseReadinessReport, GitDomainError> {
        self.analysis_port.validate_release_readiness(repo).await
    }
    
    /// Prepares a release by analyzing changes and determining version bump
    pub async fn prepare_release(
        &self,
        repo: &GitRepository,
        target_version: Option<(u32, u32, u32)>,
    ) -> Result<ReleasePreparation, GitDomainError> {
        // Validate repository is ready for release
        repo.is_ready_for_release()?;
        
        // Prepare the release
        self.analysis_port.prepare_release(repo, target_version).await
    }
    
    /// Gets commit history for release notes generation
    pub async fn get_release_commits(
        &self,
        repo: &GitRepository,
        since_tag: Option<&TagName>,
        commit_types: Option<&[String]>,
    ) -> Result<Vec<GitCommit>, GitDomainError> {
        self.analysis_port
            .get_commit_history(repo, since_tag, commit_types, None)
            .await
    }
    
    /// Analyzes the impact of current changes
    pub async fn analyze_changes(
        &self,
        repo: &GitRepository,
    ) -> Result<ChangeImpactAnalysis, GitDomainError> {
        self.analysis_port.analyze_change_impact(repo).await
    }
}

/// Service for conventional commit parsing and categorization
pub struct ConventionalCommitService;

impl ConventionalCommitService {
    /// Categorizes commits by type for release notes
    pub fn categorize_commits(commits: &[GitCommit]) -> CommitCategories {
        let mut categories = CommitCategories::default();
        
        for commit in commits {
            match commit.commit_type() {
                CommitType::Feature => categories.features.push(commit.clone()),
                CommitType::Fix => categories.fixes.push(commit.clone()),
                CommitType::BreakingChange => categories.breaking_changes.push(commit.clone()),
                CommitType::Documentation => categories.documentation.push(commit.clone()),
                CommitType::Style => categories.style.push(commit.clone()),
                CommitType::Refactor => categories.refactor.push(commit.clone()),
                CommitType::Performance => categories.performance.push(commit.clone()),
                CommitType::Test => categories.tests.push(commit.clone()),
                CommitType::Chore => categories.chores.push(commit.clone()),
                CommitType::Other => categories.other.push(commit.clone()),
            }
        }
        
        categories
    }
    
    /// Determines the appropriate version bump based on commits
    pub fn determine_version_bump(commits: &[GitCommit]) -> VersionBump {
        let has_breaking = commits.iter().any(|c| c.commit_type() == CommitType::BreakingChange);
        let has_features = commits.iter().any(|c| c.commit_type() == CommitType::Feature);
        let has_fixes = commits.iter().any(|c| c.commit_type() == CommitType::Fix);
        
        if has_breaking {
            VersionBump::Major
        } else if has_features {
            VersionBump::Minor
        } else if has_fixes {
            VersionBump::Patch
        } else {
            VersionBump::None
        }
    }
    
    /// Calculates risk level based on changes
    pub fn calculate_risk_level(commits: &[GitCommit]) -> RiskLevel {
        let breaking_count = commits.iter()
            .filter(|c| c.commit_type() == CommitType::BreakingChange)
            .count();
        
        let feature_count = commits.iter()
            .filter(|c| c.commit_type() == CommitType::Feature)
            .count();
        
        let total_commits = commits.len();
        
        if breaking_count > 0 {
            RiskLevel::High
        } else if feature_count > 5 || total_commits > 20 {
            RiskLevel::Medium
        } else if feature_count > 0 || total_commits > 5 {
            RiskLevel::Low
        } else {
            RiskLevel::Low
        }
    }
}

/// Service for semantic version management
pub struct SemanticVersionService;

impl SemanticVersionService {
    /// Parses a semantic version from a tag name
    pub fn parse_version(tag: &TagName) -> Option<(u32, u32, u32)> {
        let tag_str = tag.as_str();
        let version_str = tag_str.strip_prefix('v').unwrap_or(tag_str);
        
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        
        Some((major, minor, patch))
    }
    
    /// Creates a tag name from version components
    pub fn create_tag_name(major: u32, minor: u32, patch: u32) -> TagName {
        TagName::for_version(major, minor, patch)
    }
    
    /// Compares two semantic versions
    pub fn compare_versions(
        a: (u32, u32, u32),
        b: (u32, u32, u32),
    ) -> std::cmp::Ordering {
        a.cmp(&b)
    }
    
    /// Gets the next version based on current version and bump type
    pub fn next_version(
        current: (u32, u32, u32),
        bump: VersionBump,
    ) -> (u32, u32, u32) {
        bump.apply(current.0, current.1, current.2)
    }
}

/// Categorized commits for release notes generation
#[derive(Debug, Default, Clone)]
pub struct CommitCategories {
    pub breaking_changes: Vec<GitCommit>,
    pub features: Vec<GitCommit>,
    pub fixes: Vec<GitCommit>,
    pub documentation: Vec<GitCommit>,
    pub style: Vec<GitCommit>,
    pub refactor: Vec<GitCommit>,
    pub performance: Vec<GitCommit>,
    pub tests: Vec<GitCommit>,
    pub chores: Vec<GitCommit>,
    pub other: Vec<GitCommit>,
}

impl CommitCategories {
    /// Gets all commits in order of importance
    pub fn all_commits(&self) -> Vec<&GitCommit> {
        let mut all = Vec::new();
        all.extend(&self.breaking_changes);
        all.extend(&self.features);
        all.extend(&self.fixes);
        all.extend(&self.performance);
        all.extend(&self.refactor);
        all.extend(&self.documentation);
        all.extend(&self.style);
        all.extend(&self.tests);
        all.extend(&self.chores);
        all.extend(&self.other);
        all
    }
    
    /// Checks if there are any significant changes
    pub fn has_significant_changes(&self) -> bool {
        !self.breaking_changes.is_empty() 
            || !self.features.is_empty() 
            || !self.fixes.is_empty()
    }
    
    /// Gets a summary of changes
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        if !self.breaking_changes.is_empty() {
            parts.push(format!("{} breaking change(s)", self.breaking_changes.len()));
        }
        
        if !self.features.is_empty() {
            parts.push(format!("{} feature(s)", self.features.len()));
        }
        
        if !self.fixes.is_empty() {
            parts.push(format!("{} fix(es)", self.fixes.len()));
        }
        
        if parts.is_empty() {
            "No significant changes".to_string()
        } else {
            parts.join(", ")
        }
    }
} 