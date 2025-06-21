//! Base Component Trait and State Integration
//! 
//! This module defines the core Component trait that all UI components implement,
//! along with reactive state binding capabilities that allow components to
//! automatically update when application state changes.

use super::{ComponentId, ComponentEvent, FocusState, VisibilityState, ValidationState, ComponentResult, ComponentError};
use crate::state::{StateEvent, StateManager};
use async_trait::async_trait;
use ratatui::{Frame, layout::Rect, widgets::Widget};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Core trait that all components must implement
#[async_trait]
pub trait Component: Send + Sync {
    type Props: ComponentProps;
    type State: ComponentState;

    /// Unique identifier for this component
    fn id(&self) -> &ComponentId;

    /// Current component state
    fn state(&self) -> &Self::State;

    /// Mutable access to component state
    fn state_mut(&mut self) -> &mut Self::State;

    /// Component properties (configuration)
    fn props(&self) -> &Self::Props;

    /// Handle keyboard input and return events
    async fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> ComponentResult<Vec<ComponentEvent>>;

    /// Handle component events (focus, value changes, etc.)
    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>>;

    /// Render the component to a frame
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Update component state from application state changes
    async fn update_from_state(&mut self, state_event: &StateEvent) -> ComponentResult<bool>;

    /// Validate component state and return validation result
    fn validate(&self) -> ValidationState;

    /// Get the component's current focus state
    fn focus_state(&self) -> FocusState {
        self.state().common().focus_state
    }

    /// Set the component's focus state
    fn set_focus_state(&mut self, state: FocusState) {
        self.state_mut().common_mut().focus_state = state;
    }

    /// Get the component's visibility state
    fn visibility_state(&self) -> VisibilityState {
        self.state().common().visibility_state
    }

    /// Set the component's visibility state
    fn set_visibility_state(&mut self, state: VisibilityState) {
        self.state_mut().common_mut().visibility_state = state;
    }

    /// Check if component can receive focus
    fn can_focus(&self) -> bool {
        matches!(self.visibility_state(), VisibilityState::Visible) &&
        !matches!(self.focus_state(), FocusState::Disabled)
    }

    /// Check if component is currently focused
    fn is_focused(&self) -> bool {
        matches!(self.focus_state(), FocusState::Focused)
    }

    /// Check if component is visible
    fn is_visible(&self) -> bool {
        matches!(self.visibility_state(), VisibilityState::Visible)
    }
}

/// Trait for component properties (immutable configuration)
pub trait ComponentProps: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> {
    fn default_props() -> Self;
}

/// Trait for component state (mutable runtime state)
pub trait ComponentState: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// Access to common component state
    fn common(&self) -> &CommonComponentState;
    
    /// Mutable access to common component state
    fn common_mut(&mut self) -> &mut CommonComponentState;
}

/// Common state shared by all components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommonComponentState {
    pub focus_state: FocusState,
    pub visibility_state: VisibilityState,
    pub validation_state: ValidationState,
    pub dirty: bool,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

impl Default for CommonComponentState {
    fn default() -> Self {
        Self {
            focus_state: FocusState::Unfocused,
            visibility_state: VisibilityState::Visible,
            validation_state: ValidationState::Valid,
            dirty: false,
            last_update: chrono::Utc::now(),
        }
    }
}

impl CommonComponentState {
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.last_update = chrono::Utc::now();
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Reactive component that automatically updates with state changes
#[async_trait]
pub trait ReactiveComponent: Component {
    /// Subscribe to state changes and automatically update component
    async fn bind_to_state(&mut self, state_manager: Arc<StateManager>) -> ComponentResult<()> {
        // Subscribe to relevant state events
        let mut event_receiver = state_manager.subscribe();
        let component_id = self.id().clone();
        
        // Spawn a task to listen for state changes
        tokio::spawn(async move {
            while let Ok(state_event) = event_receiver.recv().await {
                // This would be handled by the component manager in practice
                tracing::debug!("Component {} received state event: {:?}", component_id, state_event);
            }
        });

        Ok(())
    }

    /// Check if this component should react to a specific state event
    fn should_react_to(&self, event: &StateEvent) -> bool;

    /// Get component dependencies - other components this component depends on
    fn dependencies(&self) -> Vec<ComponentId> {
        Vec::new()
    }
}

/// Builder trait for creating components with proper configuration
pub trait ComponentBuilder<C: Component> {
    type Error;

    /// Set component ID
    fn id(self, id: ComponentId) -> Self;

    /// Set component properties
    fn props(self, props: C::Props) -> Self;

    /// Set initial state
    fn state(self, state: C::State) -> Self;

    /// Build the component
    fn build(self) -> Result<C, Self::Error>;
}

/// Wrapper for components that provides additional functionality
pub struct ComponentWrapper<C: Component> {
    component: C,
    event_sender: broadcast::Sender<ComponentEvent>,
    last_render_hash: Option<u64>,
}

impl<C: Component> ComponentWrapper<C> {
    pub fn new(component: C) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            component,
            event_sender,
            last_render_hash: None,
        }
    }

    pub fn component(&self) -> &C {
        &self.component
    }

    pub fn component_mut(&mut self) -> &mut C {
        &mut self.component
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ComponentEvent> {
        self.event_sender.subscribe()
    }

    pub async fn emit_event(&self, event: ComponentEvent) {
        let _ = self.event_sender.send(event);
    }

    /// Render with change detection
    pub fn render_if_changed(&mut self, frame: &mut Frame, area: Rect) {
        // Calculate a simple hash of the component state for change detection
        let current_hash = self.calculate_state_hash();
        
        if self.last_render_hash != Some(current_hash) {
            self.component.render(frame, area);
            self.last_render_hash = Some(current_hash);
            self.component.state_mut().common_mut().clear_dirty();
        }
    }

    fn calculate_state_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Hash basic component state
        self.component.id().hash(&mut hasher);
        self.component.focus_state().hash(&mut hasher);
        self.component.visibility_state().hash(&mut hasher);
        self.component.state().common().dirty.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock component for testing
    #[derive(Debug)]
    struct MockComponent {
        id: ComponentId,
        props: MockProps,
        state: MockState,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct MockProps {
        label: String,
    }

    impl ComponentProps for MockProps {
        fn default_props() -> Self {
            Self {
                label: "Mock Component".to_string(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MockState {
        common: CommonComponentState,
        value: String,
    }

    impl ComponentState for MockState {
        fn common(&self) -> &CommonComponentState {
            &self.common
        }

        fn common_mut(&mut self) -> &mut CommonComponentState {
            &mut self.common
        }
    }

    #[async_trait]
    impl Component for MockComponent {
        type Props = MockProps;
        type State = MockState;

        fn id(&self) -> &ComponentId {
            &self.id
        }

        fn state(&self) -> &Self::State {
            &self.state
        }

        fn state_mut(&mut self) -> &mut Self::State {
            &mut self.state
        }

        fn props(&self) -> &Self::Props {
            &self.props
        }

        async fn handle_key(&mut self, _key: crossterm::event::KeyEvent) -> ComponentResult<Vec<ComponentEvent>> {
            Ok(vec![])
        }

        async fn handle_event(&mut self, _event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
            Ok(vec![])
        }

        fn render(&self, _frame: &mut Frame, _area: Rect) {
            // Mock render - do nothing
        }

        async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
            Ok(false)
        }

        fn validate(&self) -> ValidationState {
            ValidationState::Valid
        }
    }

    #[tokio::test]
    async fn test_component_focus_state() {
        let mut component = MockComponent {
            id: ComponentId::new("test"),
            props: MockProps::default_props(),
            state: MockState {
                common: CommonComponentState::default(),
                value: "test".to_string(),
            },
        };

        assert_eq!(component.focus_state(), FocusState::Unfocused);
        assert!(!component.is_focused());
        assert!(component.can_focus());

        component.set_focus_state(FocusState::Focused);
        assert_eq!(component.focus_state(), FocusState::Focused);
        assert!(component.is_focused());

        component.set_focus_state(FocusState::Disabled);
        assert!(!component.can_focus());
    }

    #[tokio::test]
    async fn test_component_visibility_state() {
        let mut component = MockComponent {
            id: ComponentId::new("test"),
            props: MockProps::default_props(),
            state: MockState {
                common: CommonComponentState::default(),
                value: "test".to_string(),
            },
        };

        assert!(component.is_visible());

        component.set_visibility_state(VisibilityState::Hidden);
        assert!(!component.is_visible());
        assert!(!component.can_focus());
    }

    #[tokio::test]
    async fn test_component_wrapper() {
        let component = MockComponent {
            id: ComponentId::new("test"),
            props: MockProps::default_props(),
            state: MockState {
                common: CommonComponentState::default(),
                value: "test".to_string(),
            },
        };

        let mut wrapper = ComponentWrapper::new(component);
        let mut receiver = wrapper.subscribe_to_events();

        let event = ComponentEvent::Activated {
            component_id: ComponentId::new("test"),
        };

        wrapper.emit_event(event.clone()).await;

        // Check that event was received
        let received = receiver.recv().await.unwrap();
        match received {
            ComponentEvent::Activated { component_id } => {
                assert_eq!(component_id.0, "test");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_common_component_state() {
        let mut state = CommonComponentState::default();
        assert!(!state.is_dirty());

        state.mark_dirty();
        assert!(state.is_dirty());

        state.clear_dirty();
        assert!(!state.is_dirty());
    }
} 