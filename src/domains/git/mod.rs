//! Git domain implementing repository operations and version control logic
//! 
//! This domain encapsulates all git-related business logic and provides
//! clean abstractions for git operations without external dependencies.

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