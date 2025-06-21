//! Reusable UI Components
//! 
//! This module provides reusable UI components for both TUI and CLI interfaces
//! with a modern component-based architecture using reactive state management.

// Core component framework (Phase 2.3)
#[cfg(feature = "new-components")]
pub mod core;

// Component categories (Phase 2.3)
#[cfg(feature = "new-components")]
pub mod forms;
#[cfg(feature = "new-components")]
pub mod lists;
#[cfg(feature = "new-components")]
pub mod dialogs;
#[cfg(feature = "new-components")]
pub mod layout;

// Legacy component support (Phase 2.1 - new-domains)
#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub mod forms;
#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub mod lists;
#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub mod dialogs;

// Re-exports for component framework
#[cfg(feature = "new-components")]
pub use core::{
    ComponentId, ComponentEvent, ComponentResult, ComponentError,
    FocusState, VisibilityState, ValidationState, NavigationDirection,
    Component, ComponentProps, ComponentState, ReactiveComponent,
    ComponentManager, ComponentRegistry, ComponentLifecycle,
    KeyboardHandler, KeyAction, NavigationContext,
};

// Re-exports for keyboard and navigation 
#[cfg(feature = "new-components")]
pub use core::keyboard::{NavigationMode, KeyMapper};

// Re-exports for base components
#[cfg(feature = "new-components")]
pub use core::base::{CommonComponentState, ComponentWrapper};

// Re-exports for component categories
#[cfg(feature = "new-components")]
pub use forms::*;
#[cfg(feature = "new-components")]
pub use lists::*;
#[cfg(feature = "new-components")]
pub use dialogs::*;
#[cfg(feature = "new-components")]
pub use layout::*;

// Legacy re-exports (backwards compatibility)
#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub use forms::*;
#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub use lists::*;
#[cfg(all(feature = "new-domains", not(feature = "new-components")))]
pub use dialogs::*;

// TODO: Implement reusable UI components for forms, lists, and dialogs 