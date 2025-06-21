//! Core Component Framework
//! 
//! This module provides the foundation for the component-based UI architecture,
//! including base traits, lifecycle management, and reactive state integration.

#[cfg(feature = "new-components")]
pub mod base;
#[cfg(feature = "new-components")]
pub mod lifecycle;
#[cfg(feature = "new-components")]
pub mod keyboard;

// Re-exports for easy access
#[cfg(feature = "new-components")]
pub use base::{Component, ComponentState, ComponentProps, ReactiveComponent, CommonComponentState};
#[cfg(feature = "new-components")]
pub use lifecycle::{ComponentManager, ComponentRegistry, ComponentLifecycle, MountResult};
#[cfg(feature = "new-components")]
pub use keyboard::{KeyboardHandler, KeyAction, NavigationContext};

// Core types are defined in this module and automatically available

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Unique identifier for components
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub String);

impl ComponentId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
    
    pub fn with_suffix(&self, suffix: &str) -> Self {
        Self(format!("{}_{}", self.0, suffix))
    }
}

impl From<&str> for ComponentId {
    fn from(s: &str) -> Self {
        ComponentId::new(s)
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Component focus state for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FocusState {
    Focused,
    Unfocused,
    Disabled,
}

/// Component visibility state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VisibilityState {
    Visible,
    Hidden,
    Collapsed,
}

/// Component validation state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationState {
    Valid,
    Invalid(String),
    Pending,
}

/// Common component events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentEvent {
    /// Component gained focus
    FocusGained { component_id: ComponentId },
    /// Component lost focus
    FocusLost { component_id: ComponentId },
    /// Component value changed
    ValueChanged { component_id: ComponentId, old_value: String, new_value: String },
    /// Component validation state changed
    ValidationChanged { component_id: ComponentId, state: ValidationState },
    /// Component was clicked/activated
    Activated { component_id: ComponentId },
    /// Button was clicked with click count
    ButtonClicked { component_id: ComponentId, button_text: String, click_count: u32 },
    /// Component requests navigation
    NavigationRequested { component_id: ComponentId, direction: NavigationDirection },
}

/// Navigation directions for focus traversal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
    Next,
    Previous,
    First,
    Last,
}

/// Result type for component operations
pub type ComponentResult<T> = Result<T, ComponentError>;

/// Component-specific errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ComponentError {
    #[error("Component not found: {id}")]
    NotFound { id: ComponentId },
    
    #[error("Component validation failed: {message}")]
    ValidationFailed { message: String },
    
    #[error("Component state error: {message}")]
    StateError { message: String },
    
    #[error("Component rendering error: {message}")]
    RenderError { message: String },
    
    #[error("Component lifecycle error: {message}")]
    LifecycleError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id_creation() {
        let id = ComponentId::new("test_button");
        assert_eq!(id.0, "test_button");
        
        let suffixed = id.with_suffix("main");
        assert_eq!(suffixed.0, "test_button_main");
    }

    #[test]
    fn test_component_id_from_str() {
        let id: ComponentId = "form_input".into();
        assert_eq!(id.0, "form_input");
    }

    #[test]
    fn test_component_id_display() {
        let id = ComponentId::new("display_test");
        assert_eq!(format!("{}", id), "display_test");
    }

    #[test]
    fn test_focus_state_serialization() {
        let focused = FocusState::Focused;
        let json = serde_json::to_string(&focused).unwrap();
        let deserialized: FocusState = serde_json::from_str(&json).unwrap();
        assert_eq!(focused, deserialized);
    }

    #[test]
    fn test_component_event_serialization() {
        let event = ComponentEvent::ValueChanged {
            component_id: ComponentId::new("test"),
            old_value: "old".to_string(),
            new_value: "new".to_string(),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ComponentEvent = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            ComponentEvent::ValueChanged { component_id, old_value, new_value } => {
                assert_eq!(component_id.0, "test");
                assert_eq!(old_value, "old");
                assert_eq!(new_value, "new");
            }
            _ => panic!("Wrong event type"),
        }
    }
} 