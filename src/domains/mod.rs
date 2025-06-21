//! Domain modules implementing Domain-Driven Design architecture
//! 
//! Each domain is completely independent and communicates through well-defined ports.
//! Feature flags control gradual migration from legacy code.

#[cfg(feature = "new-domains")]
pub mod git;

#[cfg(feature = "new-domains")]
pub mod semantic;

#[cfg(feature = "new-domains")]
pub mod tasks;

#[cfg(feature = "new-domains")]
pub mod ai;

#[cfg(feature = "new-domains")]
pub mod releases;

// Re-exports for easier access
#[cfg(feature = "new-domains")]
pub use git::*;

#[cfg(feature = "new-domains")]
pub use semantic::*;

#[cfg(feature = "new-domains")]
pub use tasks::*;

#[cfg(feature = "new-domains")]
pub use ai::*;

#[cfg(feature = "new-domains")]
pub use releases::*; 