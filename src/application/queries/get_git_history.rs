/// Get Git History Query
/// 
/// This query retrieves git history with advanced filtering and analysis options.

#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use std::any::Any;
#[cfg(feature = "new-domains")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc};

#[cfg(feature = "new-domains")]
use super::Query;
#[cfg(feature = "new-domains")]
use crate::domains::git::value_objects::{CommitHash, BranchName};

/// Git history query options
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHistoryOptions {
    pub branch: Option<BranchName>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub max_count: Option<u32>,
    pub include_merges: bool,
    pub commit_range: Option<(CommitHash, CommitHash)>,
    pub file_path: Option<String>,
    pub grep_pattern: Option<String>,
}

#[cfg(feature = "new-domains")]
impl Default for GitHistoryOptions {
    fn default() -> Self {
        Self {
            branch: None,
            since: None,
            until: None,
            author: None,
            max_count: Some(100),
            include_merges: true,
            commit_range: None,
            file_path: None,
            grep_pattern: None,
        }
    }
}

/// Query to get git history with filtering options
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGitHistoryQuery {
    pub repository_path: String,
    pub options: GitHistoryOptions,
}

/// Result of git history query
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHistoryResult {
    pub commits: Vec<crate::domains::git::entities::GitCommit>,
    pub total_count: usize,
    pub branch: Option<BranchName>,
    pub summary: GitHistorySummary,
}

/// Summary statistics for git history
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHistorySummary {
    pub authors: HashMap<String, usize>,
    pub commit_types: HashMap<String, usize>,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub files_changed: usize,
}

#[cfg(feature = "new-domains")]
impl Query for GetGitHistoryQuery {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
