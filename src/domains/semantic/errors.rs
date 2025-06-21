//! Semantic release domain error types
//! 
//! Domain-specific errors that provide rich context and diagnostic information
//! for semantic release operations.

use miette::Diagnostic;
use thiserror::Error;

/// Semantic release domain-specific errors with rich diagnostic information
#[derive(Error, Diagnostic, Debug)]
pub enum SemanticReleaseDomainError {
    #[error("Invalid semantic version: {version}")]
    #[diagnostic(
        code(semantic::invalid_version),
        help("Semantic versions must follow the format MAJOR.MINOR.PATCH (e.g., 1.2.3)")
    )]
    InvalidSemanticVersion { version: String },

    #[error("Version {version} already exists")]
    #[diagnostic(
        code(semantic::version_exists),
        help("Choose a different version number or use --force to override")
    )]
    VersionAlreadyExists { version: String },

    #[error("Cannot downgrade from version {current} to {target}")]
    #[diagnostic(
        code(semantic::version_downgrade),
        help("Semantic versioning only allows version increments, not downgrades")
    )]
    VersionDowngrade { current: String, target: String },

    #[error("No commits found since last release")]
    #[diagnostic(
        code(semantic::no_changes),
        help("Add some commits with conventional commit messages before creating a release")
    )]
    NoChangesForRelease,

    #[error("Breaking changes detected but version bump is not major")]
    #[diagnostic(
        code(semantic::breaking_changes_require_major),
        help("Breaking changes require a major version bump (e.g., 1.0.0 -> 2.0.0)")
    )]
    BreakingChangesRequireMajor { suggested_version: String },

    #[error("Release configuration is invalid: {reason}")]
    #[diagnostic(
        code(semantic::invalid_config),
        help("Check your release configuration file and ensure all required fields are present")
    )]
    InvalidReleaseConfiguration { reason: String },

    #[error("Pre-release version format is invalid: {version}")]
    #[diagnostic(
        code(semantic::invalid_prerelease),
        help("Pre-release versions must follow the format: 1.2.3-alpha.1, 1.2.3-beta.2, etc.")
    )]
    InvalidPreReleaseVersion { version: String },

    #[error("Release workflow failed at step: {step}")]
    #[diagnostic(
        code(semantic::workflow_failed),
        help("Check the logs for the specific step that failed and resolve any issues")
    )]
    WorkflowStepFailed { step: String },

    #[error("Release notes template is invalid: {reason}")]
    #[diagnostic(
        code(semantic::invalid_template),
        help("Check your release notes template syntax and ensure all variables are defined")
    )]
    InvalidReleaseNotesTemplate { reason: String },

    #[error("Dependency update required before release: {dependency}")]
    #[diagnostic(
        code(semantic::dependency_outdated),
        help("Update the specified dependency to meet the minimum version requirements")
    )]
    DependencyUpdateRequired { dependency: String },

    #[error("Release channel {channel} is not configured")]
    #[diagnostic(
        code(semantic::invalid_channel),
        help("Configure the release channel in your semantic release configuration")
    )]
    InvalidReleaseChannel { channel: String },
} 