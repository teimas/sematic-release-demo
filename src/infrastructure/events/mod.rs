//! Event infrastructure
//! 
//! Event publishing and handling infrastructure for domain events.

pub mod event_bus;
pub mod handlers;

// Re-export event modules
pub use event_bus::*;
pub use handlers::*;

// TODO: Implement event bus infrastructure
// Placeholder for now to enable compilation 