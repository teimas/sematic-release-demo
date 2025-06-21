//! CLI presentation layer
//! 
//! This module provides CLI command handlers that use the CQRS application layer.

#[cfg(feature = "new-domains")]
pub mod command_handlers;
#[cfg(feature = "new-domains")]
pub mod output_formatters;

// Re-exports
#[cfg(feature = "new-domains")]
pub use command_handlers::*;
#[cfg(feature = "new-domains")]
pub use output_formatters::*;

// TODO: Implement CLI presentation layer
// Placeholder for now to enable compilation 