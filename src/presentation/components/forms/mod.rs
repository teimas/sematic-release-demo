//! Form Components
//! 
//! This module provides form components with validation, auto-completion,
//! and reactive state integration for building robust user interfaces.

#[cfg(feature = "new-components")]
pub mod text_input;
#[cfg(feature = "new-components")]
pub mod text_input_demo;
#[cfg(feature = "new-components")]
pub mod select;
#[cfg(feature = "new-components")]
pub mod checkbox;
#[cfg(feature = "new-components")]
pub mod radio;
#[cfg(feature = "new-components")]
pub mod button;
#[cfg(feature = "new-components")]
pub mod form_builder;

// Re-exports
#[cfg(feature = "new-components")]
pub use text_input::*;
#[cfg(feature = "new-components")]
pub use text_input_demo::*;
#[cfg(feature = "new-components")]
pub use select::*;
#[cfg(feature = "new-components")]
pub use checkbox::*;
#[cfg(feature = "new-components")]
pub use radio::*;
#[cfg(feature = "new-components")]
pub use button::*;
#[cfg(feature = "new-components")]
pub use form_builder::*; 