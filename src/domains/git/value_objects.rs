//! Git domain value objects
//! 
//! Immutable value objects that encapsulate git-specific data with validation
//! and ensure invariants are maintained throughout the domain.

use crate::domains::git::errors::GitDomainError;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

/// Represents a git commit hash with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitHash(String);

impl CommitHash {
    /// Creates a new commit hash with validation
    pub fn new(hash: String) -> Result<Self, GitDomainError> {
        if hash.len() != 40 {
            return Err(GitDomainError::InvalidCommitHash { hash });
        }
        
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GitDomainError::InvalidCommitHash { hash });
        }
        
        Ok(Self(hash.to_lowercase()))
    }
    
    /// Creates a commit hash from a trusted source (e.g., git2 library)
    pub fn from_trusted(hash: String) -> Self {
        Self(hash.to_lowercase())
    }
    
    /// Returns the hash as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Returns the short form of the hash (first 7 characters)
    pub fn short(&self) -> String {
        self.0[..7].to_string()
    }
}

impl fmt::Display for CommitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a git branch name with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchName(String);

impl BranchName {
    /// Creates a new branch name with validation
    pub fn new(name: String) -> Result<Self, GitDomainError> {
        if name.is_empty() {
            return Err(GitDomainError::InvalidBranchName { name });
        }
        
        // Git branch name validation rules
        if name.contains(' ') || name.starts_with('-') || name.contains("..") {
            return Err(GitDomainError::InvalidBranchName { name });
        }
        
        Ok(Self(name))
    }
    
    /// Creates a branch name from a trusted source
    pub fn from_trusted(name: String) -> Self {
        Self(name)
    }
    
    /// Returns the branch name as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Checks if this is the main/master branch
    pub fn is_main_branch(&self) -> bool {
        matches!(self.0.as_str(), "main" | "master")
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a git tag name with semantic version validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagName(String);

impl TagName {
    /// Creates a new tag name with validation
    pub fn new(name: String) -> Result<Self, GitDomainError> {
        if name.is_empty() {
            return Err(GitDomainError::InvalidTagName { name });
        }
        
        // Basic tag name validation (can be extended for semantic versioning)
        if name.contains(' ') || name.starts_with('-') {
            return Err(GitDomainError::InvalidTagName { name });
        }
        
        Ok(Self(name))
    }
    
    /// Creates a tag name for a semantic version
    pub fn for_version(major: u32, minor: u32, patch: u32) -> Self {
        Self(format!("v{}.{}.{}", major, minor, patch))
    }
    
    /// Creates a tag name from a trusted source
    pub fn from_trusted(name: String) -> Self {
        Self(name)
    }
    
    /// Returns the tag name as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Checks if this is a semantic version tag
    pub fn is_semantic_version(&self) -> bool {
        self.0.starts_with('v') && 
        self.0[1..].chars().all(|c| c.is_ascii_digit() || c == '.')
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a git commit message with conventional commit support
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMessage {
    pub raw: String,
    pub commit_type: Option<String>,
    pub scope: Option<String>,
    pub subject: String,
    pub body: Option<String>,
    pub breaking_change: bool,
}

impl CommitMessage {
    /// Creates a new commit message with conventional commit parsing
    pub fn new(message: String) -> Self {
        let lines: Vec<&str> = message.lines().collect();
        let first_line = lines.first().unwrap_or(&"").to_string();
        
        // Parse conventional commit format: type(scope): subject
        let (commit_type, scope, subject, breaking_change) = 
            Self::parse_conventional_commit(&first_line);
        
        let body = if lines.len() > 2 {
            Some(lines[2..].join("\n"))
        } else {
            None
        };
        
        Self {
            raw: message,
            commit_type,
            scope,
            subject,
            body,
            breaking_change,
        }
    }
    
    /// Parses conventional commit format
    fn parse_conventional_commit(line: &str) -> (Option<String>, Option<String>, String, bool) {
        // Check for breaking change indicator
        let breaking_change = line.contains("BREAKING CHANGE") || line.contains('!');
        
        // Try to parse type(scope): subject format
        if let Some(colon_pos) = line.find(':') {
            let prefix = &line[..colon_pos];
            let subject = line[colon_pos + 1..].trim().to_string();
            
            if let Some(paren_pos) = prefix.find('(') {
                if let Some(close_paren) = prefix.find(')') {
                    let commit_type = prefix[..paren_pos].trim().to_string();
                    let scope = prefix[paren_pos + 1..close_paren].trim().to_string();
                    return (Some(commit_type), Some(scope), subject, breaking_change);
                }
            } else {
                let commit_type = prefix.trim().to_string();
                return (Some(commit_type), None, subject, breaking_change);
            }
        }
        
        (None, None, line.to_string(), breaking_change)
    }
    
    /// Checks if this is a feature commit
    pub fn is_feature(&self) -> bool {
        self.commit_type.as_ref().map(|t| t == "feat").unwrap_or(false)
    }
    
    /// Checks if this is a fix commit
    pub fn is_fix(&self) -> bool {
        self.commit_type.as_ref().map(|t| t == "fix").unwrap_or(false)
    }
    
    /// Checks if this commit introduces breaking changes
    pub fn is_breaking_change(&self) -> bool {
        self.breaking_change
    }
}

/// Represents a git remote URL
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitRemoteUrl(Url);

impl GitRemoteUrl {
    /// Creates a new git remote URL with validation
    pub fn new(url: String) -> Result<Self, GitDomainError> {
        let parsed_url = Url::parse(&url)
            .map_err(|_| GitDomainError::OperationFailed { 
                operation: format!("parse remote URL: {}", url) 
            })?;
        
        Ok(Self(parsed_url))
    }
    
    /// Returns the URL as a string
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    
    /// Extracts the repository name from the URL
    pub fn repository_name(&self) -> Option<String> {
        self.0.path_segments()
            .and_then(|segments| segments.last())
            .map(|name| name.trim_end_matches(".git").to_string())
    }
} 