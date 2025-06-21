//! Application layer coordinating domain operations
//! 
//! This layer implements the CQRS pattern with commands, queries, and services
//! that orchestrate multiple domains to fulfill business use cases.

#[cfg(feature = "new-domains")]
pub mod services;

#[cfg(feature = "new-domains")]
pub mod commands;

#[cfg(feature = "new-domains")]
pub mod queries;

// Re-exports for easier access
#[cfg(feature = "new-domains")]
pub use services::*;

#[cfg(feature = "new-domains")]
pub use commands::*;

#[cfg(feature = "new-domains")]
pub use queries::*; 