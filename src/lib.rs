// Legacy modules (maintained for backward compatibility during migration)
pub mod app;
pub mod config;
pub mod error;
pub mod git;
pub mod observability;
pub mod services;
pub mod types;
pub mod ui;
pub mod utils;

// Modern State Management (Phase 2.2: State Management Revolution)
pub mod state;

// New Domain-Driven Design architecture (feature-flagged)
#[cfg(feature = "new-domains")]
pub mod domains;

#[cfg(feature = "new-domains")]
pub mod infrastructure;

#[cfg(feature = "new-domains")]
pub mod application;

#[cfg(feature = "new-domains")]
pub mod presentation;

// Re-export key types for backward compatibility
pub use error::{Result, SemanticReleaseError};
pub use types::*;
