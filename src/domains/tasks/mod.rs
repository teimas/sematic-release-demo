//! Task management domain implementing JIRA, Monday.com integration and task automation
//! 
//! This domain encapsulates all task management business logic and provides
//! clean abstractions for external task management systems without coupling.

pub mod entities;
pub mod value_objects;
pub mod repository;
pub mod services;
pub mod errors;

// Re-export public interface
pub use entities::*;
pub use value_objects::*;
pub use repository::*;
pub use services::*;
pub use errors::*;

// TODO: Implement task management domain
// Placeholder for now to enable compilation 