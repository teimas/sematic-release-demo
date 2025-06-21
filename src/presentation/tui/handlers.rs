//! TUI Input Handlers
//! 
//! This module provides specialized input handlers for different TUI screens.

#[cfg(feature = "new-domains")]
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
#[cfg(feature = "new-domains")]
use super::state::{UiAction, UiState};

/// Handle input for the main screen
#[cfg(feature = "new-domains")]
pub fn handle_main_screen_input(key: KeyEvent, _state: &UiState) -> Option<UiAction> {
    match key.code {
        KeyCode::Char('1') => Some(UiAction::NavigateToScreen("create_release".to_string())),
        KeyCode::Char('2') => Some(UiAction::NavigateToScreen("sync_tasks".to_string())),
        KeyCode::Char('3') => Some(UiAction::NavigateToScreen("generate_notes".to_string())),
        KeyCode::Char('4') => Some(UiAction::NavigateToScreen("settings".to_string())),
        _ => None,
    }
}

/// Handle input for the create release screen
#[cfg(feature = "new-domains")]
pub fn handle_create_release_input(key: KeyEvent, state: &UiState) -> Option<UiAction> {
    match key.code {
        KeyCode::Enter => Some(UiAction::SubmitForm),
        KeyCode::Esc => Some(UiAction::NavigateToScreen("main".to_string())),
        KeyCode::Char(c) => {
            let field = match state.focused_component.as_str() {
                "version_input" => "target_version",
                "notes_input" => "release_notes", 
                _ => return None,
            };
            
            let mut current_value = state.form_fields.get(field).cloned().unwrap_or_default();
            current_value.push(c);
            Some(UiAction::UpdateField(field.to_string(), current_value))
        }
        KeyCode::Backspace => {
            let field = match state.focused_component.as_str() {
                "version_input" => "target_version",
                "notes_input" => "release_notes",
                _ => return None,
            };
            
            let mut current_value = state.form_fields.get(field).cloned().unwrap_or_default();
            current_value.pop();
            Some(UiAction::UpdateField(field.to_string(), current_value))
        }
        _ => None,
    }
}

/// Handle input for the sync tasks screen
#[cfg(feature = "new-domains")]
pub fn handle_sync_tasks_input(key: KeyEvent, _state: &UiState) -> Option<UiAction> {
    match key.code {
        KeyCode::Enter => Some(UiAction::SubmitForm),
        KeyCode::Esc => Some(UiAction::NavigateToScreen("main".to_string())),
        _ => None,
    }
}

/// Handle input for the generate notes screen
#[cfg(feature = "new-domains")]
pub fn handle_generate_notes_input(key: KeyEvent, _state: &UiState) -> Option<UiAction> {
    match key.code {
        KeyCode::Enter => Some(UiAction::SubmitForm),
        KeyCode::Esc => Some(UiAction::NavigateToScreen("main".to_string())),
        _ => None,
    }
}

/// Main input router based on current screen
#[cfg(feature = "new-domains")]
pub fn route_input(key: KeyEvent, state: &UiState) -> Option<UiAction> {
    // Global shortcuts first
    match key.code {
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return None; // Let app handle quit
        }
        _ => {}
    }
    
    // Route to screen-specific handlers
    match state.current_screen.as_str() {
        "main" => handle_main_screen_input(key, state),
        "create_release" => handle_create_release_input(key, state),
        "sync_tasks" => handle_sync_tasks_input(key, state),
        "generate_notes" => handle_generate_notes_input(key, state),
        _ => None,
    }
}
