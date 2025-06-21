//! TUI State Management
//! 
//! This module manages UI-specific state that's separate from domain state.

#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use crossterm::event::KeyEvent;

/// UI-specific state for the TUI application
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub struct UiState {
    pub current_screen: String,
    pub current_repository_path: String,
    pub status_message: String,
    pub is_loading: bool,
    pub loading_message: String,
    pub focused_component: String,
    pub form_fields: HashMap<String, String>,
    pub error_message: Option<String>,
}

#[cfg(feature = "new-domains")]
impl Default for UiState {
    fn default() -> Self {
        Self {
            current_screen: "main".to_string(),
            current_repository_path: ".".to_string(),
            status_message: "Ready".to_string(),
            is_loading: false,
            loading_message: String::new(),
            focused_component: "main_menu".to_string(),
            form_fields: HashMap::new(),
            error_message: None,
        }
    }
}

#[cfg(feature = "new-domains")]
impl UiState {
    /// Set the current status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.error_message = None;
    }
    
    /// Set loading state
    pub fn set_loading(&mut self, loading: bool, message: String) {
        self.is_loading = loading;
        self.loading_message = message;
    }
    
    /// Set error message
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.is_loading = false;
    }
    
    /// Navigate to a different screen
    pub fn navigate_to(&mut self, screen: String) {
        self.current_screen = screen;
        self.error_message = None;
    }
    
    /// Move focus to next component
    pub fn next_focus(&mut self) {
        match self.focused_component.as_str() {
            "main_menu" => self.focused_component = "repository_path".to_string(),
            "repository_path" => self.focused_component = "actions".to_string(),
            "actions" => self.focused_component = "main_menu".to_string(),
            _ => self.focused_component = "main_menu".to_string(),
        }
    }
    
    /// Move focus to previous component
    pub fn previous_focus(&mut self) {
        match self.focused_component.as_str() {
            "main_menu" => self.focused_component = "actions".to_string(),
            "repository_path" => self.focused_component = "main_menu".to_string(),
            "actions" => self.focused_component = "repository_path".to_string(),
            _ => self.focused_component = "main_menu".to_string(),
        }
    }
    
    /// Update a form field
    pub fn update_field(&mut self, field: String, value: String) {
        self.form_fields.insert(field, value);
    }
    
    /// Get action for focused component with key input
    pub fn get_focused_action(&self, _key: KeyEvent) -> Option<UiAction> {
        // Placeholder - would implement actual key mappings
        None
    }
    
    /// Handle UI events
    pub fn handle_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::ReleaseCreated => {
                self.set_status("Release created successfully!".to_string());
            }
            UiEvent::TasksSynced => {
                self.set_status("Tasks synchronized successfully!".to_string());
            }
            UiEvent::NotesGenerated => {
                self.set_status("Release notes generated successfully!".to_string());
            }
            UiEvent::StatusUpdated(_status) => {
                self.set_status("Status updated".to_string());
            }
            UiEvent::Error(error) => {
                self.set_error(error);
            }
        }
    }
}

/// UI events that can occur
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub enum UiEvent {
    ReleaseCreated,
    TasksSynced,  
    NotesGenerated,
    StatusUpdated(String), // Simplified for now
    Error(String),
}

/// UI actions that can be triggered
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub enum UiAction {
    UpdateField(String, String),
    SubmitForm,
    NavigateToScreen(String),
}
