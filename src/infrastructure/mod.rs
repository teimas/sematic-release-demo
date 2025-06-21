//! Infrastructure layer implementing repository ports and external integrations
//! 
//! This layer provides concrete implementations of domain repository ports,
//! handles external service integrations, and manages technical concerns.

pub mod storage;
pub mod external;
pub mod events;

// Re-export infrastructure modules
pub use storage::*;
pub use external::*;
pub use events::*; 