//! Storage infrastructure implementations
//! 
//! Concrete implementations of repository ports for data persistence,
//! including file system, in-memory, and database storage adapters.

pub mod file_system;
pub mod memory;
pub mod database;
pub mod git_storage;
pub mod cache;

// Re-export storage implementations
pub use file_system::*;
pub use memory::*;
pub use database::*;
pub use git_storage::*;
pub use cache::*;

// TODO: Implement storage infrastructure
// Placeholder for now to enable compilation 