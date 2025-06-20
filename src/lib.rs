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

// Re-export key types for backward compatibility
pub use error::{Result, SemanticReleaseError};
pub use types::*;
