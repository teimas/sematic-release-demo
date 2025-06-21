//! Semantic release domain value objects
//! 
//! Immutable value objects that encapsulate semantic versioning data with validation
//! and ensure semantic versioning invariants are maintained.

use crate::domains::semantic::errors::SemanticReleaseDomainError;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt};

/// Represents a semantic version with validation and comparison
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

impl SemanticVersion {
    /// Creates a new semantic version with validation
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }
    
    /// Creates a semantic version from a string with validation
    pub fn parse(version: &str) -> Result<Self, SemanticReleaseDomainError> {
        let version = version.strip_prefix('v').unwrap_or(version);
        
        // Split by '+' to separate build metadata
        let (version_part, build_metadata) = if let Some(plus_pos) = version.find('+') {
            let build = Some(version[plus_pos + 1..].to_string());
            (&version[..plus_pos], build)
        } else {
            (version, None)
        };
        
        // Split by '-' to separate pre-release
        let (core_version, pre_release) = if let Some(dash_pos) = version_part.find('-') {
            let pre = Some(version_part[dash_pos + 1..].to_string());
            (&version_part[..dash_pos], pre)
        } else {
            (version_part, None)
        };
        
        // Parse core version (MAJOR.MINOR.PATCH)
        let parts: Vec<&str> = core_version.split('.').collect();
        if parts.len() != 3 {
            return Err(SemanticReleaseDomainError::InvalidSemanticVersion {
                version: version.to_string(),
            });
        }
        
        let major = parts[0].parse().map_err(|_| {
            SemanticReleaseDomainError::InvalidSemanticVersion {
                version: version.to_string(),
            }
        })?;
        
        let minor = parts[1].parse().map_err(|_| {
            SemanticReleaseDomainError::InvalidSemanticVersion {
                version: version.to_string(),
            }
        })?;
        
        let patch = parts[2].parse().map_err(|_| {
            SemanticReleaseDomainError::InvalidSemanticVersion {
                version: version.to_string(),
            }
        })?;
        
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
            build_metadata,
        })
    }
    
    /// Creates a pre-release version
    pub fn with_pre_release(mut self, pre_release: String) -> Result<Self, SemanticReleaseDomainError> {
        // Validate pre-release format (alphanumeric, dots, and hyphens only)
        if pre_release.is_empty() || !pre_release.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
            return Err(SemanticReleaseDomainError::InvalidPreReleaseVersion {
                version: format!("{}-{}", self, pre_release),
            });
        }
        
        self.pre_release = Some(pre_release);
        Ok(self)
    }
    
    /// Creates a version with build metadata
    pub fn with_build_metadata(mut self, build_metadata: String) -> Self {
        self.build_metadata = Some(build_metadata);
        self
    }
    
    /// Increments the major version (resets minor and patch to 0)
    pub fn increment_major(self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }
    
    /// Increments the minor version (resets patch to 0)
    pub fn increment_minor(self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }
    
    /// Increments the patch version
    pub fn increment_patch(self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
    
    /// Checks if this is a pre-release version
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }
    
    /// Checks if this is a stable release
    pub fn is_stable(&self) -> bool {
        !self.is_pre_release()
    }
    
    /// Gets the core version without pre-release or build metadata
    pub fn core_version(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
    
    /// Checks if this version is compatible with another version (same major)
    pub fn is_compatible_with(&self, other: &SemanticVersion) -> bool {
        self.major == other.major && self.major > 0
    }
    
    /// Checks if this version has breaking changes compared to another
    pub fn has_breaking_changes_from(&self, other: &SemanticVersion) -> bool {
        self.major > other.major
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        
        if let Some(pre_release) = &self.pre_release {
            write!(f, "-{}", pre_release)?;
        }
        
        if let Some(build_metadata) = &self.build_metadata {
            write!(f, "+{}", build_metadata)?;
        }
        
        Ok(())
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare core version first
        match (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)) {
            Ordering::Equal => {
                // Pre-release versions have lower precedence than normal versions
                match (&self.pre_release, &other.pre_release) {
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(a), Some(b)) => a.cmp(b),
                    (None, None) => Ordering::Equal,
                }
            }
            other => other,
        }
    }
}

/// Represents the type of version bump required
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionBumpType {
    Major,
    Minor,
    Patch,
    PreRelease(String),
    None,
}

impl VersionBumpType {
    /// Applies the version bump to a semantic version
    pub fn apply(&self, version: &SemanticVersion) -> SemanticVersion {
        match self {
            Self::Major => version.clone().increment_major(),
            Self::Minor => version.clone().increment_minor(),
            Self::Patch => version.clone().increment_patch(),
            Self::PreRelease(identifier) => {
                version.clone()
                    .with_pre_release(identifier.clone())
                    .unwrap_or_else(|_| version.clone())
            }
            Self::None => version.clone(),
        }
    }
    
    /// Gets the string representation of the bump type
    pub fn as_str(&self) -> &str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::PreRelease(_) => "pre-release",
            Self::None => "none",
        }
    }
}

/// Represents a release channel with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReleaseChannel(String);

impl ReleaseChannel {
    /// Creates a new release channel with validation
    pub fn new(channel: String) -> Result<Self, SemanticReleaseDomainError> {
        if channel.is_empty() {
            return Err(SemanticReleaseDomainError::InvalidReleaseChannel { channel });
        }
        
        // Validate channel name (alphanumeric and hyphens only)
        if !channel.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(SemanticReleaseDomainError::InvalidReleaseChannel { channel });
        }
        
        Ok(Self(channel))
    }
    
    /// Creates a stable release channel
    pub fn stable() -> Self {
        Self("stable".to_string())
    }
    
    /// Creates a beta release channel
    pub fn beta() -> Self {
        Self("beta".to_string())
    }
    
    /// Creates an alpha release channel
    pub fn alpha() -> Self {
        Self("alpha".to_string())
    }
    
    /// Returns the channel name as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Checks if this is the stable channel
    pub fn is_stable(&self) -> bool {
        self.0 == "stable"
    }
    
    /// Checks if this is a pre-release channel
    pub fn is_pre_release(&self) -> bool {
        !self.is_stable()
    }
}

impl fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents release configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfiguration {
    pub default_channel: ReleaseChannel,
    pub auto_increment: bool,
    pub require_clean_working_directory: bool,
    pub create_git_tag: bool,
    pub push_to_remote: bool,
    pub generate_release_notes: bool,
    pub allowed_channels: Vec<ReleaseChannel>,
    pub version_prefix: String,
}

impl ReleaseConfiguration {
    /// Creates a new release configuration with defaults
    pub fn new() -> Self {
        Self {
            default_channel: ReleaseChannel::stable(),
            auto_increment: true,
            require_clean_working_directory: true,
            create_git_tag: true,
            push_to_remote: true,
            generate_release_notes: true,
            allowed_channels: vec![
                ReleaseChannel::alpha(),
                ReleaseChannel::beta(),
                ReleaseChannel::stable(),
            ],
            version_prefix: "v".to_string(),
        }
    }
    
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), SemanticReleaseDomainError> {
        if self.allowed_channels.is_empty() {
            return Err(SemanticReleaseDomainError::InvalidReleaseConfiguration {
                reason: "At least one release channel must be configured".to_string(),
            });
        }
        
        if !self.allowed_channels.contains(&self.default_channel) {
            return Err(SemanticReleaseDomainError::InvalidReleaseConfiguration {
                reason: "Default channel must be in the list of allowed channels".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Checks if a channel is allowed
    pub fn is_channel_allowed(&self, channel: &ReleaseChannel) -> bool {
        self.allowed_channels.contains(channel)
    }
}

impl Default for ReleaseConfiguration {
    fn default() -> Self {
        Self::new()
    }
} 