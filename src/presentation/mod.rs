//! Presentation layer handling user interfaces
//! 
//! This layer contains TUI components, CLI handlers, and reusable UI components
//! that interact with the application layer through commands and queries.

#[cfg(feature = "new-domains")]
pub mod tui;

#[cfg(feature = "new-domains")]
pub mod cli;

#[cfg(feature = "new-domains")]
pub mod components;

#[cfg(feature = "new-domains")]
pub mod container;

// Phase 2.3.3: Theme and Styling System
#[cfg(feature = "new-components")]
pub mod theme;

// Phase 3: Performance & UX Enhancement - Rendering Optimization
#[cfg(feature = "new-components")]
pub mod rendering;

// Re-exports for easier access
#[cfg(feature = "new-domains")]
pub use tui::*;

#[cfg(feature = "new-domains")]
pub use cli::*;

#[cfg(feature = "new-domains")]
pub use components::*;

#[cfg(feature = "new-domains")]
pub use container::*;

#[cfg(feature = "new-components")]
pub use theme::*;

#[cfg(feature = "new-components")]
pub use rendering::*; 