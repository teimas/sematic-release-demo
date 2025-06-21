//! Semantic release domain implementing version management and release logic
//! 
//! This domain encapsulates all semantic versioning business logic and provides
//! clean abstractions for release operations without external dependencies.

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

// TODO: Implement semantic release domain
// Placeholder for now to enable compilation 