//! Centralized State Management for Semantic Release TUI
//! 
//! This module implements a reactive state management system that centralizes
//! all application state and provides event-driven updates with proper change
//! detection and subscription mechanisms.

pub mod manager;
pub mod events;
pub mod reactive;
pub mod stores;

// Re-export main types for easy access
pub use manager::{StateManager, StateManagerError, StatePersistence, SerializedState, FilePersistence, MemoryPersistence};
pub use events::{StateEvent, StateChange, StateSubscription};
pub use reactive::{ReactiveState, StateObserver, ReactiveStateManager, LoggingObserver, PerformanceObserver};
pub use stores::{AppState, UiState, GitState, TaskState, AiState, AppMode, Screen, InputMode};

use serde::{Deserialize, Serialize};

/// Configuration for state management behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Enable debug logging for state changes
    pub debug_logging: bool,
    
    /// Maximum number of history entries to keep
    pub history_size: usize,
    
    /// Whether to automatically persist state changes
    pub auto_persist: bool,
    
    /// Interval in seconds for automatic persistence
    pub persist_interval_seconds: u64,
    
    /// Maximum time to wait for observers to process events (milliseconds)
    pub observer_timeout_ms: u64,
    
    /// Enable performance monitoring
    pub performance_monitoring: bool,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            debug_logging: false,
            history_size: 100,
            auto_persist: false,
            persist_interval_seconds: 60,
            observer_timeout_ms: 100,
            performance_monitoring: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_state_manager_initialization() {
        let config = StateConfig {
            debug_logging: true,
            performance_monitoring: true,
            ..Default::default()
        };
        
        let state_manager = StateManager::with_config(config);
        
        // Initialize the state manager
        assert!(state_manager.initialize().await.is_ok());
        
        // Check initial state
        let app_state = state_manager.app_state().await;
        assert_eq!(app_state.current_screen, Screen::Main);
        assert_eq!(app_state.mode, AppMode::Normal);
    }
    
    #[tokio::test]
    async fn test_reactive_state_updates() {
        let state_manager = StateManager::new();
        state_manager.initialize().await.unwrap();
        
        // Update app state
        state_manager.update_app_state(|state| {
            let previous_screen = state.current_screen.to_string();
            state.current_screen = Screen::Config;
            
            Some(StateEvent::ScreenChanged {
                previous: previous_screen,
                current: state.current_screen.to_string(),
                timestamp: chrono::Utc::now(),
            })
        }).await.unwrap();
        
        // Verify the change
        let app_state = state_manager.app_state().await;
        assert_eq!(app_state.current_screen, Screen::Config);
    }
    
    #[tokio::test]
    async fn test_ui_state_updates() {
        let state_manager = StateManager::new();
        state_manager.initialize().await.unwrap();
        
        // Update UI state
        state_manager.update_ui_state(|state| {
            state.input_mode = InputMode::Editing;
            state.focused_component = "text_input".to_string();
            
            Some(StateEvent::UiModeChanged {
                previous: "normal".to_string(),
                current: "editing".to_string(),
                timestamp: chrono::Utc::now(),
            })
        }).await.unwrap();
        
        // Verify the change
        let ui_state = state_manager.ui_state().await;
        assert_eq!(ui_state.input_mode, InputMode::Editing);
        assert_eq!(ui_state.focused_component, "text_input");
    }
    
    #[tokio::test]
    async fn test_event_subscription() {
        let state_manager = StateManager::new();
        state_manager.initialize().await.unwrap();
        
        // Subscribe to events
        let mut event_receiver = state_manager.subscribe();
        
        // Emit an event
        let test_event = StateEvent::ConfigUpdated {
            changes: vec!["test_setting".to_string()],
            timestamp: chrono::Utc::now(),
        };
        
        state_manager.emit_event(test_event.clone()).await;
        
        // Verify we received the event
        let received_event = event_receiver.try_recv().unwrap();
        match received_event {
            StateEvent::ConfigUpdated { changes, .. } => {
                assert_eq!(changes[0], "test_setting");
            }
            _ => panic!("Expected ConfigUpdated event"),
        }
    }
    
    #[tokio::test]
    async fn test_performance_observer() {
        let config = StateConfig {
            performance_monitoring: true,
            debug_logging: true,
            ..Default::default()
        };
        
        let state_manager = StateManager::with_config(config);
        state_manager.initialize().await.unwrap();
        
        // Generate some events through state updates to ensure they're captured
        for i in 0..5 {
            state_manager.update_app_state(|state| {
                state.status_message = format!("Update {}", i);
                Some(StateEvent::ConfigUpdated {
                    changes: vec![format!("setting_{}", i)],
                    timestamp: chrono::Utc::now(),
                })
            }).await.unwrap();
        }
        
        // Wait a bit longer for events to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Check performance metrics
        let metrics = state_manager.get_performance_metrics().await;
        // Since we're using state updates which trigger the observers, we should have events
        assert!(metrics.total_events > 0, "Expected some events to be recorded, got {}", metrics.total_events);
    }
    
    #[tokio::test]
    async fn test_memory_persistence() {
        let state_manager = StateManager::new()
            .with_persistence(Arc::new(MemoryPersistence::new()));
        
        state_manager.initialize().await.unwrap();
        
        // Update some state
        state_manager.update_app_state(|state| {
            state.current_screen = Screen::Help;
            state.status_message = "Help screen loaded".to_string();
            None
        }).await.unwrap();
        
        // Persist the state
        assert!(state_manager.persist_state().await.is_ok());
    }
} 