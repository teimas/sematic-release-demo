//! Git domain error types
//! 
//! Domain-specific errors that provide rich context and diagnostic information
//! for git-related operations.

use miette::Diagnostic;
use thiserror::Error;

/// Git domain-specific errors with rich diagnostic information
#[derive(Error, Diagnostic, Debug)]
pub enum GitDomainError {
    #[error("Working directory is dirty")]
    #[diagnostic(
        code(git::working_directory_dirty),
        help("Commit or stash your changes before proceeding with the release")
    )]
    WorkingDirectoryDirty,

    #[error("Invalid commit hash: {hash}")]
    #[diagnostic(
        code(git::invalid_commit_hash),
        help("Commit hash must be a 40-character hexadecimal string")
    )]
    InvalidCommitHash { hash: String },

    #[error("Invalid branch name: {name}")]
    #[diagnostic(
        code(git::invalid_branch_name),
        help("Branch names cannot contain spaces or special characters")
    )]
    InvalidBranchName { name: String },

    #[error("Invalid tag name: {name}")]
    #[diagnostic(
        code(git::invalid_tag_name),
        help("Tag names must follow semantic versioning pattern (e.g., v1.2.3)")
    )]
    InvalidTagName { name: String },

    #[error("Repository not found at path: {path}")]
    #[diagnostic(
        code(git::repository_not_found),
        help("Ensure you're in a git repository or initialize one with 'git init'")
    )]
    RepositoryNotFound { path: String },

    #[error("Branch {branch} does not exist")]
    #[diagnostic(
        code(git::branch_not_found),
        help("Check available branches with 'git branch -a'")
    )]
    BranchNotFound { branch: String },

    #[error("Remote {remote} is not configured")]
    #[diagnostic(
        code(git::remote_not_configured),
        help("Add a remote with 'git remote add origin <url>'")
    )]
    RemoteNotConfigured { remote: String },

    #[error("Git operation failed: {operation}")]
    #[diagnostic(
        code(git::operation_failed),
        help("Check git status and ensure repository is in a clean state")
    )]
    OperationFailed { operation: String },

    #[error("Merge conflict detected")]
    #[diagnostic(
        code(git::merge_conflict),
        help("Resolve merge conflicts and try again")
    )]
    MergeConflict,

    #[error("Permission denied for git operation")]
    #[diagnostic(
        code(git::permission_denied),
        help("Check file permissions and git configuration")
    )]
    PermissionDenied,

    #[error("Network error during git operation: {details}")]
    #[diagnostic(
        code(git::network_error),
        help("Check your internet connection and repository URL")
    )]
    NetworkError { details: String },
} 