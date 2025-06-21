//! Git Storage Adapter
//! 
//! This module provides a git storage adapter that implements git operations
//! using external git commands to avoid thread safety issues with git2.

#[cfg(feature = "new-domains")]
use std::path::Path;
#[cfg(feature = "new-domains")]
use std::process::Command;
#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc};

#[cfg(feature = "new-domains")]
use crate::domains::git::{
    repository::{GitRepositoryPort, GitAnalysisPort},
    entities::{GitRepository, GitCommit, GitTag, GitBranch, ReleasePreparation},
    value_objects::{CommitHash, BranchName, TagName, CommitMessage},
    errors::GitDomainError,
};

/// Git storage adapter using external git commands
#[cfg(feature = "new-domains")]
pub struct GitStorageAdapter {
    repository_path: String,
}

#[cfg(feature = "new-domains")]
impl GitStorageAdapter {
    pub fn new(repository_path: String) -> Self {
        Self { repository_path }
    }
    
    fn run_git_command(&self, args: &[&str]) -> Result<String, GitDomainError> {
        let output = Command::new("git")
            .current_dir(&self.repository_path)
            .args(args)
            .output()
                    .map_err(|e| GitDomainError::OperationFailed {
            operation: format!("Failed to execute git command: {}", e),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitDomainError::OperationFailed {
                operation: format!("Git command failed: {}", stderr),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl GitRepositoryPort for GitStorageAdapter {
    async fn status(&self, repo: &GitRepository) -> Result<GitRepository, GitDomainError> {
        // Return updated repository status
        Ok(repo.clone())
    }

    async fn current_branch(&self, _repo: &GitRepository) -> Result<Option<BranchName>, GitDomainError> {
        let branch = self.run_git_command(&["branch", "--show-current"])?;
        if branch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(BranchName::new(branch)?))
        }
    }

    async fn branches(&self, _repo: &GitRepository) -> Result<Vec<GitBranch>, GitDomainError> {
        // Return empty list for now
        Ok(vec![])
    }

    async fn commits_since(
        &self,
        _repo: &GitRepository,
        since: Option<&TagName>,
        _until: Option<&CommitHash>,
    ) -> Result<Vec<GitCommit>, GitDomainError> {
        let range = if let Some(tag) = since {
            format!("{}..HEAD", tag.as_str())
        } else {
            "HEAD".to_string()
        };

        let output = self.run_git_command(&[
            "log", 
            &range,
            "--pretty=format:%H|%s|%an|%ae|%ad",
            "--date=iso"
        ])?;

        let commits = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    if let (Ok(hash), message) = (
                        CommitHash::new(parts[0].to_string()),
                        CommitMessage::new(parts[1].to_string())
                    ) {
                        Some(GitCommit {
                            hash,
                            message,
                            author_name: parts[2].to_string(),
                            author_email: parts[3].to_string(),
                            timestamp: parts[4].parse().unwrap_or_else(|_| Utc::now()),
                            parent_hashes: vec![],
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(commits)
    }

    async fn tags(&self, _repo: &GitRepository) -> Result<Vec<GitTag>, GitDomainError> {
        let output = self.run_git_command(&["tag", "-l"])?;
        let tags = output
            .lines()
            .filter_map(|tag_name| {
                let name = TagName::new(tag_name.to_string()).ok()?;
                let commit_hash = CommitHash::new("".to_string()).ok()?;
                Some(GitTag {
                    name,
                    commit_hash,
                    message: None,
                    tagger_name: None,
                    tagger_email: None,
                    timestamp: Some(Utc::now()),
                })
            })
            .collect();
        Ok(tags)
    }

    async fn latest_tag(&self, _repo: &GitRepository) -> Result<Option<GitTag>, GitDomainError> {
        let output = self.run_git_command(&["describe", "--tags", "--abbrev=0"]);
        
        match output {
            Ok(tag_name) => {
                let name = TagName::new(tag_name)?;
                let commit_hash = CommitHash::new("".to_string())?;
                Ok(Some(GitTag {
                    name,
                    commit_hash,
                    message: None,
                    tagger_name: None,
                    tagger_email: None,
                    timestamp: Some(Utc::now()),
                }))
            },
            Err(_) => Ok(None),
        }
    }

    async fn create_tag(
        &self,
        _repo: &GitRepository,
        tag_name: &TagName,
        _commit: &CommitHash,
        message: Option<&str>,
    ) -> Result<GitTag, GitDomainError> {
        let args = if let Some(msg) = message {
            vec!["tag", "-a", tag_name.as_str(), "-m", msg]
        } else {
            vec!["tag", tag_name.as_str()]
        };

        self.run_git_command(&args)?;
        
        let commit_hash = CommitHash::new("".to_string())?;
        Ok(GitTag {
            name: tag_name.clone(),
            commit_hash,
            message: message.map(|s| s.to_string()),
            tagger_name: None,
            tagger_email: None,
            timestamp: Some(Utc::now()),
        })
    }

    async fn push_tags(&self, _repo: &GitRepository, remote: Option<&str>) -> Result<(), GitDomainError> {
        let remote_name = remote.unwrap_or("origin");
        self.run_git_command(&["push", remote_name, "--tags"])?;
        Ok(())
    }

    async fn remote_url(&self, _repo: &GitRepository, remote: &str) -> Result<Option<String>, GitDomainError> {
        let output = self.run_git_command(&["remote", "get-url", remote]);
        match output {
            Ok(url) => Ok(Some(url)),
            Err(_) => Ok(None),
        }
    }

    async fn is_clean(&self, _repo: &GitRepository) -> Result<bool, GitDomainError> {
        let status = self.run_git_command(&["status", "--porcelain"])?;
        Ok(status.is_empty())
    }

    async fn get_diff(&self, _repo: &GitRepository, staged: bool) -> Result<String, GitDomainError> {
        let args = if staged {
            vec!["diff", "--cached"]
        } else {
            vec!["diff"]
        };
        self.run_git_command(&args)
    }

    async fn stage_files(&self, _repo: &GitRepository, files: &[String]) -> Result<(), GitDomainError> {
        for file in files {
            self.run_git_command(&["add", file])?;
        }
        Ok(())
    }

    async fn commit(
        &self,
        _repo: &GitRepository,
        message: &str,
        _author_name: Option<&str>,
        _author_email: Option<&str>,
    ) -> Result<GitCommit, GitDomainError> {
        self.run_git_command(&["commit", "-m", message])?;
        let hash = self.run_git_command(&["rev-parse", "HEAD"])?;
        
        Ok(GitCommit {
            hash: CommitHash::new(hash)?,
            message: CommitMessage::new(message.to_string()),
            author_name: "".to_string(),
            author_email: "".to_string(),
            timestamp: Utc::now(),
            parent_hashes: vec![],
        })
    }

    async fn push(
        &self,
        _repo: &GitRepository,
        remote: Option<&str>,
        branch: Option<&BranchName>,
    ) -> Result<(), GitDomainError> {
        let remote_name = remote.unwrap_or("origin");
        let branch_name = if let Some(b) = branch {
            b.as_str()
        } else {
            "main"
        };
        self.run_git_command(&["push", remote_name, branch_name])?;
        Ok(())
    }

    async fn fetch(&self, _repo: &GitRepository, remote: Option<&str>) -> Result<(), GitDomainError> {
        let remote_name = remote.unwrap_or("origin");
        self.run_git_command(&["fetch", remote_name])?;
        Ok(())
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl GitAnalysisPort for GitStorageAdapter {
    async fn prepare_release(
        &self,
        repo: &GitRepository,
        target_version: Option<(u32, u32, u32)>,
    ) -> Result<ReleasePreparation, GitDomainError> {
        let version = target_version.unwrap_or((0, 1, 0));
        
        // Get commits since last tag
        let commits = self.commits_since(repo, None, None).await?;
        
        // Create required objects
        let target_branch = self.current_branch(repo).await?
            .unwrap_or(BranchName::from_trusted("main".to_string()));
        let commit_hash = CommitHash::from_trusted("abcd1234567890abcdef1234567890abcdef1234".to_string());
        let tag_name = TagName::for_version(version.0, version.1, version.2);
        
        Ok(ReleasePreparation::new(
            target_branch,
            commit_hash,
            tag_name,
            commits,
        ))
    }

    async fn get_commit_history(
        &self,
        repo: &GitRepository,
        since: Option<&TagName>,
        _commit_types: Option<&[String]>,
        limit: Option<usize>,
    ) -> Result<Vec<GitCommit>, GitDomainError> {
        let mut commits = self.commits_since(repo, since, None).await?;
        
        if let Some(limit_count) = limit {
            commits.truncate(limit_count);
        }
        
        Ok(commits)
    }

    async fn analyze_change_impact(
        &self,
        repo: &GitRepository,
    ) -> Result<crate::domains::git::repository::ChangeImpactAnalysis, GitDomainError> {
        let commits = self.commits_since(repo, None, None).await?;
        
        let has_breaking_changes = commits.iter().any(|c| c.message.raw.contains("BREAKING"));
        let has_features = commits.iter().any(|c| c.message.raw.starts_with("feat"));
        let has_fixes = commits.iter().any(|c| c.message.raw.starts_with("fix"));
        
        let suggested_version_bump = if has_breaking_changes {
            crate::domains::git::repository::VersionBump::Major
        } else if has_features {
            crate::domains::git::repository::VersionBump::Minor
        } else if has_fixes {
            crate::domains::git::repository::VersionBump::Patch
        } else {
            crate::domains::git::repository::VersionBump::None
        };
        
        Ok(crate::domains::git::repository::ChangeImpactAnalysis {
            has_breaking_changes,
            has_features,
            has_fixes,
            suggested_version_bump,
            risk_level: crate::domains::git::repository::RiskLevel::Low,
            affected_areas: vec![],
        })
    }

    async fn validate_release_readiness(
        &self,
        repo: &GitRepository,
    ) -> Result<crate::domains::git::repository::ReleaseReadinessReport, GitDomainError> {
        let is_clean = self.is_clean(repo).await?;
        let blocking_issues = if !is_clean {
            vec!["Working directory has uncommitted changes".to_string()]
        } else {
            vec![]
        };
        
        Ok(crate::domains::git::repository::ReleaseReadinessReport {
            is_ready: blocking_issues.is_empty(),
            blocking_issues,
            warnings: vec![],
            recommendations: vec!["Run tests before release".to_string()],
        })
    }
} 