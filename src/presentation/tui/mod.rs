//! TUI presentation layer
//! 
//! This module provides the TUI application coordinator that integrates with the CQRS
//! application layer through commands and queries.

#[cfg(feature = "new-domains")]
pub mod app;
#[cfg(feature = "new-domains")]
pub mod state;
#[cfg(feature = "new-domains")]
pub mod handlers;

// Re-exports
#[cfg(feature = "new-domains")]
pub use app::*;
#[cfg(feature = "new-domains")]
pub use state::*;
#[cfg(feature = "new-domains")]
pub use handlers::*;

// TODO: Implement TUI presentation layer
// Placeholder for now to enable compilation 