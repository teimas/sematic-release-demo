//! Keyboard Input Handling and Navigation
//! 
//! This module provides abstractions for keyboard input handling, navigation
//! management, and key action mapping for components.

use super::{ComponentId, ComponentEvent, NavigationDirection, ComponentResult, ComponentError};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::{debug, warn};

/// Key action that can be performed by components
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAction {
    // Navigation actions
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NavigateNext,
    NavigatePrevious,
    NavigateFirst,
    NavigateLast,
    
    // Focus actions
    FocusNext,
    FocusPrevious,
    FocusFirst,
    FocusLast,
    
    // Component actions
    Activate,
    Cancel,
    Submit,
    Escape,
    Enter,
    Space,
    Tab,
    BackTab,
    
    // Editing actions
    Insert(char),
    Delete,
    Backspace,
    Clear,
    SelectAll,
    Cut,
    Copy,
    Paste,
    
    // Application actions
    Quit,
    Help,
    Menu,
    Settings,
    Refresh,
    
    // Custom actions
    Custom(String),
}

impl KeyAction {
    /// Convert to navigation direction if applicable
    pub fn to_navigation_direction(&self) -> Option<NavigationDirection> {
        match self {
            KeyAction::NavigateUp => Some(NavigationDirection::Up),
            KeyAction::NavigateDown => Some(NavigationDirection::Down),
            KeyAction::NavigateLeft => Some(NavigationDirection::Left),
            KeyAction::NavigateRight => Some(NavigationDirection::Right),
            KeyAction::NavigateNext | KeyAction::FocusNext => Some(NavigationDirection::Next),
            KeyAction::NavigatePrevious | KeyAction::FocusPrevious => Some(NavigationDirection::Previous),
            KeyAction::NavigateFirst | KeyAction::FocusFirst => Some(NavigationDirection::First),
            KeyAction::NavigateLast | KeyAction::FocusLast => Some(NavigationDirection::Last),
            _ => None,
        }
    }

    /// Check if this is a navigation action
    pub fn is_navigation(&self) -> bool {
        self.to_navigation_direction().is_some()
    }

    /// Check if this is an editing action
    pub fn is_editing(&self) -> bool {
        matches!(self, 
            KeyAction::Insert(_) | 
            KeyAction::Delete | 
            KeyAction::Backspace | 
            KeyAction::Clear | 
            KeyAction::SelectAll | 
            KeyAction::Cut | 
            KeyAction::Copy | 
            KeyAction::Paste
        )
    }
}

/// Key binding that maps key events to actions (not serializable due to crossterm types)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub action: KeyAction,
}

impl KeyBinding {
    pub fn new(key: KeyCode, action: KeyAction) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::empty(),
            action,
        }
    }

    pub fn with_modifiers(key: KeyCode, modifiers: KeyModifiers, action: KeyAction) -> Self {
        Self {
            key,
            modifiers,
            action,
        }
    }

    /// Check if this binding matches a key event
    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.key == event.code && self.modifiers == event.modifiers
    }
}

/// Navigation context for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationContext {
    /// Current component in focus
    pub current_component: Option<ComponentId>,
    
    /// Navigation history
    pub history: Vec<ComponentId>,
    
    /// Maximum history size
    pub max_history: usize,
    
    /// Whether navigation wrapping is enabled
    pub wrap_navigation: bool,
    
    /// Navigation mode (normal, vim-style, etc.)
    pub navigation_mode: NavigationMode,
}

impl NavigationContext {
    pub fn new() -> Self {
        Self {
            current_component: None,
            history: Vec::new(),
            max_history: 50,
            wrap_navigation: true,
            navigation_mode: NavigationMode::Normal,
        }
    }

    /// Set current component and update history
    pub fn set_current(&mut self, component_id: ComponentId) {
        if let Some(current) = &self.current_component {
            if current != &component_id {
                self.add_to_history(current.clone());
            }
        }
        self.current_component = Some(component_id);
    }

    /// Go back to previous component
    pub fn go_back(&mut self) -> Option<ComponentId> {
        if let Some(previous) = self.history.pop() {
            self.current_component = Some(previous.clone());
            Some(previous)
        } else {
            None
        }
    }

    /// Add component to navigation history
    fn add_to_history(&mut self, component_id: ComponentId) {
        // Remove if already in history to avoid duplicates
        self.history.retain(|id| id != &component_id);
        
        // Add to end
        self.history.push(component_id);
        
        // Trim if exceeds max size
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Clear navigation history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for NavigationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigation mode for different navigation styles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationMode {
    /// Standard tab-based navigation
    Normal,
    /// Vim-style hjkl navigation
    Vim,
    /// Emacs-style navigation
    Emacs,
    /// Custom navigation scheme
    Custom,
}

/// Default key bindings for different navigation modes
pub fn default_key_bindings(mode: NavigationMode) -> Vec<KeyBinding> {
    match mode {
        NavigationMode::Normal => normal_key_bindings(),
        NavigationMode::Vim => vim_key_bindings(),
        NavigationMode::Emacs => emacs_key_bindings(),
        NavigationMode::Custom => Vec::new(), // User-defined
    }
}

/// Standard navigation key bindings
fn normal_key_bindings() -> Vec<KeyBinding> {
    vec![
        // Navigation
        KeyBinding::new(KeyCode::Up, KeyAction::NavigateUp),
        KeyBinding::new(KeyCode::Down, KeyAction::NavigateDown),
        KeyBinding::new(KeyCode::Left, KeyAction::NavigateLeft),
        KeyBinding::new(KeyCode::Right, KeyAction::NavigateRight),
        KeyBinding::new(KeyCode::Tab, KeyAction::FocusNext),
        KeyBinding::new(KeyCode::BackTab, KeyAction::FocusPrevious),
        
        // Actions
        KeyBinding::new(KeyCode::Enter, KeyAction::Activate),
        KeyBinding::new(KeyCode::Esc, KeyAction::Escape),
        KeyBinding::new(KeyCode::Delete, KeyAction::Delete),
        KeyBinding::new(KeyCode::Backspace, KeyAction::Backspace),
        
        // Application
        KeyBinding::with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL, KeyAction::Quit),
        KeyBinding::with_modifiers(KeyCode::Char('h'), KeyModifiers::CONTROL, KeyAction::Help),
        KeyBinding::with_modifiers(KeyCode::Char('r'), KeyModifiers::CONTROL, KeyAction::Refresh),
        KeyBinding::with_modifiers(KeyCode::Char('p'), KeyModifiers::CONTROL, KeyAction::Menu),
        
        // Editing
        KeyBinding::with_modifiers(KeyCode::Char('a'), KeyModifiers::CONTROL, KeyAction::SelectAll),
        KeyBinding::with_modifiers(KeyCode::Char('x'), KeyModifiers::CONTROL, KeyAction::Cut),
        KeyBinding::with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL, KeyAction::Copy),
        KeyBinding::with_modifiers(KeyCode::Char('v'), KeyModifiers::CONTROL, KeyAction::Paste),
    ]
}

/// Vim-style navigation key bindings
fn vim_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = normal_key_bindings();
    
    // Add vim-specific bindings
    bindings.extend(vec![
        // Vim navigation
        KeyBinding::new(KeyCode::Char('h'), KeyAction::NavigateLeft),
        KeyBinding::new(KeyCode::Char('j'), KeyAction::NavigateDown),
        KeyBinding::new(KeyCode::Char('k'), KeyAction::NavigateUp),
        KeyBinding::new(KeyCode::Char('l'), KeyAction::NavigateRight),
        KeyBinding::new(KeyCode::Char('g'), KeyAction::NavigateFirst),
        KeyBinding::with_modifiers(KeyCode::Char('g'), KeyModifiers::SHIFT, KeyAction::NavigateLast),
        
        // Vim actions
        KeyBinding::new(KeyCode::Char('i'), KeyAction::Custom("insert_mode".to_string())),
        KeyBinding::new(KeyCode::Char('a'), KeyAction::Custom("append_mode".to_string())),
        KeyBinding::new(KeyCode::Char('o'), KeyAction::Custom("open_line".to_string())),
        KeyBinding::new(KeyCode::Char('x'), KeyAction::Delete),
        KeyBinding::new(KeyCode::Char('d'), KeyAction::Custom("delete_line".to_string())),
        KeyBinding::new(KeyCode::Char('y'), KeyAction::Copy),
        KeyBinding::new(KeyCode::Char('p'), KeyAction::Paste),
    ]);
    
    bindings
}

/// Emacs-style navigation key bindings
fn emacs_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = normal_key_bindings();
    
    // Add emacs-specific bindings
    bindings.extend(vec![
        // Emacs navigation
        KeyBinding::with_modifiers(KeyCode::Char('p'), KeyModifiers::CONTROL, KeyAction::NavigateUp),
        KeyBinding::with_modifiers(KeyCode::Char('n'), KeyModifiers::CONTROL, KeyAction::NavigateDown),
        KeyBinding::with_modifiers(KeyCode::Char('b'), KeyModifiers::CONTROL, KeyAction::NavigateLeft),
        KeyBinding::with_modifiers(KeyCode::Char('f'), KeyModifiers::CONTROL, KeyAction::NavigateRight),
        KeyBinding::with_modifiers(KeyCode::Char('a'), KeyModifiers::CONTROL, KeyAction::NavigateFirst),
        KeyBinding::with_modifiers(KeyCode::Char('e'), KeyModifiers::CONTROL, KeyAction::NavigateLast),
        
        // Emacs editing
        KeyBinding::with_modifiers(KeyCode::Char('d'), KeyModifiers::CONTROL, KeyAction::Delete),
        KeyBinding::with_modifiers(KeyCode::Char('k'), KeyModifiers::CONTROL, KeyAction::Custom("kill_line".to_string())),
        KeyBinding::with_modifiers(KeyCode::Char('w'), KeyModifiers::CONTROL, KeyAction::Cut),
        KeyBinding::with_modifiers(KeyCode::Char('y'), KeyModifiers::CONTROL, KeyAction::Paste),
    ]);
    
    bindings
}

/// Keyboard handler trait for components
pub trait KeyboardHandler {
    /// Handle key input and return resulting actions
    fn handle_key(&mut self, event: KeyEvent, context: &NavigationContext) -> ComponentResult<Vec<KeyAction>>;
    
    /// Get component-specific key bindings
    fn key_bindings(&self) -> Vec<KeyBinding>;
    
    /// Check if component can handle a specific key event
    fn can_handle_key(&self, event: &KeyEvent) -> bool {
        self.key_bindings().iter().any(|binding| binding.matches(event))
    }
}

/// Key mapper that converts key events to actions
#[derive(Debug)]
pub struct KeyMapper {
    bindings: HashMap<(KeyCode, KeyModifiers), KeyAction>,
    navigation_mode: NavigationMode,
}

impl KeyMapper {
    /// Create a new key mapper with default bindings for the specified mode
    pub fn new(mode: NavigationMode) -> Self {
        let bindings = default_key_bindings(mode.clone())
            .into_iter()
            .map(|binding| ((binding.key, binding.modifiers), binding.action))
            .collect();
        
        Self {
            bindings,
            navigation_mode: mode,
        }
    }

    /// Add a custom key binding
    pub fn add_binding(&mut self, binding: KeyBinding) {
        self.bindings.insert((binding.key, binding.modifiers), binding.action);
    }

    /// Remove a key binding
    pub fn remove_binding(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        self.bindings.remove(&(key, modifiers));
    }

    /// Map a key event to an action
    pub fn map_key(&self, event: &KeyEvent) -> Option<KeyAction> {
        // First check for exact match with modifiers
        if let Some(action) = self.bindings.get(&(event.code, event.modifiers)) {
            return Some(action.clone());
        }

        // For character input without modifiers, create Insert action
        if event.modifiers.is_empty() {
            if let KeyCode::Char(c) = event.code {
                return Some(KeyAction::Insert(c));
            }
        }

        None
    }

    /// Get all current bindings
    pub fn bindings(&self) -> &HashMap<(KeyCode, KeyModifiers), KeyAction> {
        &self.bindings
    }

    /// Change navigation mode and reload bindings
    pub fn set_navigation_mode(&mut self, mode: NavigationMode) {
        self.navigation_mode = mode.clone();
        
        // Clear current bindings and reload defaults
        self.bindings.clear();
        let new_bindings = default_key_bindings(mode)
            .into_iter()
            .map(|binding| ((binding.key, binding.modifiers), binding.action))
            .collect();
        self.bindings = new_bindings;
    }

    /// Get current navigation mode
    pub fn navigation_mode(&self) -> &NavigationMode {
        &self.navigation_mode
    }
}

impl Default for KeyMapper {
    fn default() -> Self {
        Self::new(NavigationMode::Normal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_action_navigation() {
        assert_eq!(KeyAction::NavigateUp.to_navigation_direction(), Some(NavigationDirection::Up));
        assert_eq!(KeyAction::FocusNext.to_navigation_direction(), Some(NavigationDirection::Next));
        assert_eq!(KeyAction::Activate.to_navigation_direction(), None);
        
        assert!(KeyAction::NavigateUp.is_navigation());
        assert!(!KeyAction::Activate.is_navigation());
        
        assert!(KeyAction::Insert('a').is_editing());
        assert!(!KeyAction::NavigateUp.is_editing());
    }

    #[test]
    fn test_key_binding() {
        let binding = KeyBinding::new(KeyCode::Enter, KeyAction::Activate);
        
        let matching_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let non_matching_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        
        assert!(binding.matches(&matching_event));
        assert!(!binding.matches(&non_matching_event));
    }

    #[test]
    fn test_key_binding_with_modifiers() {
        let binding = KeyBinding::with_modifiers(
            KeyCode::Char('c'), 
            KeyModifiers::CONTROL, 
            KeyAction::Quit
        );
        
        let matching_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let non_matching_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty());
        
        assert!(binding.matches(&matching_event));
        assert!(!binding.matches(&non_matching_event));
    }

    #[test]
    fn test_navigation_context() {
        let mut context = NavigationContext::new();
        
        assert!(context.current_component.is_none());
        assert!(context.history.is_empty());
        
        let component1 = ComponentId::new("comp1");
        let component2 = ComponentId::new("comp2");
        
        context.set_current(component1.clone());
        assert_eq!(context.current_component, Some(component1.clone()));
        
        context.set_current(component2.clone());
        assert_eq!(context.current_component, Some(component2));
        assert_eq!(context.history, vec![component1.clone()]);
        
        let previous = context.go_back();
        assert_eq!(previous, Some(component1.clone()));
        assert_eq!(context.current_component, Some(component1));
    }

    #[test]
    fn test_key_mapper() {
        let mut mapper = KeyMapper::new(NavigationMode::Normal);
        
        // Test default binding
        let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(mapper.map_key(&enter_event), Some(KeyAction::Activate));
        
        // Test character insertion
        let char_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        assert_eq!(mapper.map_key(&char_event), Some(KeyAction::Insert('a')));
        
        // Test custom binding
        let custom_binding = KeyBinding::new(KeyCode::Char('z'), KeyAction::Custom("test".to_string()));
        mapper.add_binding(custom_binding);
        
        let z_event = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty());
        assert_eq!(mapper.map_key(&z_event), Some(KeyAction::Custom("test".to_string())));
        
        // Test removal
        mapper.remove_binding(KeyCode::Char('z'), KeyModifiers::empty());
        assert_eq!(mapper.map_key(&z_event), Some(KeyAction::Insert('z')));  // Falls back to Insert
    }

    #[test]
    fn test_vim_key_bindings() {
        let bindings = vim_key_bindings();
        
        // Check that vim-specific bindings are included
        assert!(bindings.iter().any(|b| b.key == KeyCode::Char('h') && b.action == KeyAction::NavigateLeft));
        assert!(bindings.iter().any(|b| b.key == KeyCode::Char('j') && b.action == KeyAction::NavigateDown));
        assert!(bindings.iter().any(|b| b.key == KeyCode::Char('k') && b.action == KeyAction::NavigateUp));
        assert!(bindings.iter().any(|b| b.key == KeyCode::Char('l') && b.action == KeyAction::NavigateRight));
    }

    #[test]
    fn test_emacs_key_bindings() {
        let bindings = emacs_key_bindings();
        
        // Check that emacs-specific bindings are included
        assert!(bindings.iter().any(|b| 
            b.key == KeyCode::Char('p') && 
            b.modifiers == KeyModifiers::CONTROL && 
            b.action == KeyAction::NavigateUp
        ));
        assert!(bindings.iter().any(|b| 
            b.key == KeyCode::Char('n') && 
            b.modifiers == KeyModifiers::CONTROL && 
            b.action == KeyAction::NavigateDown
        ));
    }

    #[test]
    fn test_navigation_mode_switching() {
        let mut mapper = KeyMapper::new(NavigationMode::Normal);
        assert_eq!(mapper.navigation_mode(), &NavigationMode::Normal);
        
        mapper.set_navigation_mode(NavigationMode::Vim);
        assert_eq!(mapper.navigation_mode(), &NavigationMode::Vim);
        
        // Check that vim bindings are now active
        let h_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty());
        assert_eq!(mapper.map_key(&h_event), Some(KeyAction::NavigateLeft));
    }
} 