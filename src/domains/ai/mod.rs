//! AI domain implementing Gemini integration and AI-driven analysis
//! 
//! This domain encapsulates all AI-related business logic and provides
//! clean abstractions for AI services without coupling to specific providers.

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

// TODO: Implement AI domain
// Placeholder for now to enable compilation 