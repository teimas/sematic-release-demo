//! TUI Application Coordinator
//! 
//! This module provides the main TUI application that coordinates between
//! user input, the CQRS application layer, and UI updates.

#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use tokio::sync::RwLock;
#[cfg(feature = "new-domains")]
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};

#[cfg(feature = "new-domains")]
use crate::application::commands::CommandBus;
#[cfg(feature = "new-domains")]
use crate::application::queries::QueryBus;
#[cfg(feature = "new-domains")]
use super::state::UiState;

/// Main TUI application coordinator
#[cfg(feature = "new-domains")]
pub struct TuiApplication {
    command_bus: Arc<dyn CommandBus>,
    query_bus: Arc<dyn QueryBus>,
    ui_state: Arc<RwLock<UiState>>,
    should_quit: bool,
}

#[cfg(feature = "new-domains")]
impl TuiApplication {
    pub fn new(
        command_bus: Arc<dyn CommandBus>,
        query_bus: Arc<dyn QueryBus>,
    ) -> Self {
        Self {
            command_bus,
            query_bus,
            ui_state: Arc::new(RwLock::new(UiState::default())),
            should_quit: false,
        }
    }
    
    /// Handle keyboard input events
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<(), TuiError> {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('r') => {
                // Handle create release - placeholder for now
                let mut state = self.ui_state.write().await;
                state.set_status("Creating release...".to_string());
            }
            _ => {
                // Handle other key events
            }
        }
        Ok(())
    }
    
    /// Check if application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    
    /// Get current UI state for rendering
    pub async fn get_ui_state(&self) -> UiState {
        self.ui_state.read().await.clone()
    }
}

/// TUI-specific errors
#[cfg(feature = "new-domains")]
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    
    #[error("UI state error: {0}")]
    StateError(String),
    
    #[error("Input handling error: {0}")]
    InputError(String),
}
