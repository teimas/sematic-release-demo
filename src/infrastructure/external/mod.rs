//! External service integrations
//! 
//! This module contains adapters for external services like AI providers,
//! task management systems, and other third-party integrations.

pub mod ai_providers;
pub mod task_systems;
pub mod http_client;

// Re-export external service modules
pub use ai_providers::*;
pub use task_systems::*;
pub use http_client::*;

// TODO: Implement external services infrastructure
// Placeholder for now to enable compilation 