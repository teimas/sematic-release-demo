//! State Manager
//! 
//! This module provides the main StateManager that coordinates all state
//! management functionality, including reactive state, persistence, and
//! change history.

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use derive_more::Display;
use chrono::{DateTime, Utc};

use super::{
    StateConfig, 
    events::{StateEvent, StateChange},
    reactive::{ReactiveStateManager, ReactiveError, StateObserver, LoggingObserver, PerformanceObserver},
    stores::{AppState, UiState, GitState, TaskState, AiState},
};

/// Main state manager that coordinates all application state
pub struct StateManager {
    /// Reactive state manager for all domain states
    reactive_states: Arc<ReactiveStateManager>,
    
    /// Configuration for state management behavior
    config: StateConfig,
    
    /// Change history for undo/redo functionality
    change_history: Arc<RwLock<VecDeque<StateChange>>>,
    
    /// Current position in change history
    history_position: Arc<RwLock<usize>>,
    
    /// Global event broadcaster
    event_tx: broadcast::Sender<StateEvent>,
    
    /// State persistence handler
    persistence: Option<Arc<dyn StatePersistence>>,
    
    /// Performance tracking
    performance_observer: Arc<PerformanceObserver>,
    
    /// Whether manager is currently initialized
    initialized: Arc<RwLock<bool>>,
}

impl StateManager {
    /// Create a new state manager with default configuration
    pub fn new() -> Self {
        Self::with_config(StateConfig::default())
    }
    
    /// Create a new state manager with custom configuration
    pub fn with_config(config: StateConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        let reactive_states = Arc::new(ReactiveStateManager::new());
        let performance_observer = Arc::new(PerformanceObserver::new("global_performance"));
        
        let manager = Self {
            reactive_states,
            config,
            change_history: Arc::new(RwLock::new(VecDeque::new())),
            history_position: Arc::new(RwLock::new(0)),
            event_tx,
            persistence: None,
            performance_observer,
            initialized: Arc::new(RwLock::new(false)),
        };
        
        // Don't initialize automatically - let the caller do it
        manager
    }
    
    /// Initialize the state manager and set up observers
    pub async fn initialize(&self) -> Result<(), StateManagerError> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }
        
        // Set up global observers
        if self.config.debug_logging {
            let logging_observer = Arc::new(LoggingObserver::new("global_logging"));
            self.reactive_states.add_global_observer(logging_observer).await;
        }
        
        // Add performance observer
        self.reactive_states.add_global_observer(self.performance_observer.clone()).await;
        
        // Load persisted state if available
        if let Some(persistence) = &self.persistence {
            if let Ok(persisted_state) = persistence.load_state().await {
                self.restore_state(persisted_state).await?;
            }
        }
        
        *initialized = true;
        tracing::info!("State manager initialized successfully");
        Ok(())
    }
    
    /// Get access to the reactive state manager
    pub fn reactive_states(&self) -> &Arc<ReactiveStateManager> {
        &self.reactive_states
    }
    
    /// Get read access to app state
    pub async fn app_state(&self) -> tokio::sync::RwLockReadGuard<'_, AppState> {
        self.reactive_states.app_state().read().await
    }
    
    /// Get read access to UI state
    pub async fn ui_state(&self) -> tokio::sync::RwLockReadGuard<'_, UiState> {
        self.reactive_states.ui_state().read().await
    }
    
    /// Get read access to Git state
    pub async fn git_state(&self) -> tokio::sync::RwLockReadGuard<'_, GitState> {
        self.reactive_states.git_state().read().await
    }
    
    /// Get read access to Task state
    pub async fn task_state(&self) -> tokio::sync::RwLockReadGuard<'_, TaskState> {
        self.reactive_states.task_state().read().await
    }
    
    /// Get read access to AI state
    pub async fn ai_state(&self) -> tokio::sync::RwLockReadGuard<'_, AiState> {
        self.reactive_states.ai_state().read().await
    }
    
    /// Update app state and emit event
    pub async fn update_app_state<F>(&self, updater: F) -> Result<(), StateManagerError>
    where
        F: FnOnce(&mut AppState) -> Option<StateEvent> + Send,
    {
        self.reactive_states.app_state().update(updater).await
            .map_err(StateManagerError::ReactiveError)?;
        Ok(())
    }
    
    /// Update UI state and emit event
    pub async fn update_ui_state<F>(&self, updater: F) -> Result<(), StateManagerError>
    where
        F: FnOnce(&mut UiState) -> Option<StateEvent> + Send,
    {
        self.reactive_states.ui_state().update(updater).await
            .map_err(StateManagerError::ReactiveError)?;
        Ok(())
    }
    
    /// Update Git state and emit event
    pub async fn update_git_state<F>(&self, updater: F) -> Result<(), StateManagerError>
    where
        F: FnOnce(&mut GitState) -> Option<StateEvent> + Send,
    {
        self.reactive_states.git_state().update(updater).await
            .map_err(StateManagerError::ReactiveError)?;
        Ok(())
    }
    
    /// Update Task state and emit event
    pub async fn update_task_state<F>(&self, updater: F) -> Result<(), StateManagerError>
    where
        F: FnOnce(&mut TaskState) -> Option<StateEvent> + Send,
    {
        self.reactive_states.task_state().update(updater).await
            .map_err(StateManagerError::ReactiveError)?;
        Ok(())
    }
    
    /// Update AI state and emit event
    pub async fn update_ai_state<F>(&self, updater: F) -> Result<(), StateManagerError>
    where
        F: FnOnce(&mut AiState) -> Option<StateEvent> + Send,
    {
        self.reactive_states.ai_state().update(updater).await
            .map_err(StateManagerError::ReactiveError)?;
        Ok(())
    }
    
    /// Emit a global state event
    pub async fn emit_event(&self, event: StateEvent) {
        let change = StateChange::new(event.clone(), "state_manager");
        
        // Add to history
        self.add_to_history(change).await;
        
        // Broadcast event
        let _ = self.event_tx.send(event);
    }
    
    /// Subscribe to all state change events
    pub fn subscribe(&self) -> broadcast::Receiver<StateEvent> {
        self.event_tx.subscribe()
    }
    
    /// Add an observer to all state changes
    pub async fn add_global_observer(&self, observer: Arc<dyn StateObserver>) {
        self.reactive_states.add_global_observer(observer).await;
    }
    
    /// Get the current change history
    pub async fn get_change_history(&self) -> Vec<StateChange> {
        let history = self.change_history.read().await;
        history.iter().cloned().collect()
    }
    
    /// Undo the last state change
    pub async fn undo(&self) -> Result<bool, StateManagerError> {
        let mut position = self.history_position.write().await;
        let history = self.change_history.read().await;
        
        if *position > 0 {
            *position -= 1;
            
            // Emit undo event
            if let Some(_change) = history.get(*position) {
                let undo_event = StateEvent::ConfigUpdated {
                    changes: vec!["undo".to_string()],
                    timestamp: Utc::now(),
                };
                let _ = self.event_tx.send(undo_event);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
    
    /// Redo the next state change
    pub async fn redo(&self) -> Result<bool, StateManagerError> {
        let mut position = self.history_position.write().await;
        let history = self.change_history.read().await;
        
        if *position < history.len() {
            *position += 1;
            
            // Emit redo event
            if let Some(_change) = history.get(*position - 1) {
                let redo_event = StateEvent::ConfigUpdated {
                    changes: vec!["redo".to_string()],
                    timestamp: Utc::now(),
                };
                let _ = self.event_tx.send(redo_event);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
    
    /// Set up state persistence
    pub fn with_persistence(mut self, persistence: Arc<dyn StatePersistence>) -> Self {
        self.persistence = Some(persistence);
        self
    }
    
    /// Persist current state
    pub async fn persist_state(&self) -> Result<(), StateManagerError> {
        if let Some(persistence) = &self.persistence {
            let serialized_state = SerializedState {
                app_state: self.reactive_states.app_state().read().await.clone(),
                ui_state: self.reactive_states.ui_state().read().await.clone(),
                git_state: self.reactive_states.git_state().read().await.clone(),
                task_state: self.reactive_states.task_state().read().await.clone(),
                ai_state: self.reactive_states.ai_state().read().await.clone(),
                timestamp: Utc::now(),
            };
            
            persistence.save_state(&serialized_state).await
                .map_err(|error| StateManagerError::PersistenceError { error })?;
        }
        Ok(())
    }
    
    /// Restore state from persistence
    async fn restore_state(&self, state: SerializedState) -> Result<(), StateManagerError> {
        // Update all states
        {
            let mut app_state = self.reactive_states.app_state().write().await;
            *app_state = state.app_state;
        }
        {
            let mut ui_state = self.reactive_states.ui_state().write().await;
            *ui_state = state.ui_state;
        }
        {
            let mut git_state = self.reactive_states.git_state().write().await;
            *git_state = state.git_state;
        }
        {
            let mut task_state = self.reactive_states.task_state().write().await;
            *task_state = state.task_state;
        }
        {
            let mut ai_state = self.reactive_states.ai_state().write().await;
            *ai_state = state.ai_state;
        }
        
        // Emit restoration event
        let event = StateEvent::ConfigUpdated {
            changes: vec!["state_restored".to_string()],
            timestamp: Utc::now(),
        };
        self.emit_event(event).await;
        
        Ok(())
    }
    
    /// Add a state change to history
    async fn add_to_history(&self, change: StateChange) {
        let mut history = self.change_history.write().await;
        let mut position = self.history_position.write().await;
        
        // Remove any history after current position (for branch history)
        history.truncate(*position);
        
        // Add new change
        history.push_back(change);
        *position = history.len();
        
        // Keep history size bounded
        while history.len() > self.config.history_size {
            history.pop_front();
            if *position > 0 {
                *position -= 1;
            }
        }
    }
    
    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> super::reactive::PerformanceMetrics {
        self.performance_observer.get_metrics().await
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur in state management
#[derive(Debug, thiserror::Error, Display)]
pub enum StateManagerError {
    #[display(fmt = "Reactive state error: {}", _0)]
    ReactiveError(ReactiveError),
    
    #[display(fmt = "Persistence error: {}", error)]
    PersistenceError { error: String },
    
    #[display(fmt = "Initialization error: {}", reason)]
    InitializationError { reason: String },
    
    #[display(fmt = "History operation failed: {}", reason)]
    HistoryError { reason: String },
    
    #[display(fmt = "State validation failed: {}", reason)]
    ValidationError { reason: String },
}

/// Trait for state persistence implementations
#[async_trait::async_trait]
pub trait StatePersistence: Send + Sync {
    /// Save the current state
    async fn save_state(&self, state: &SerializedState) -> Result<(), String>;
    
    /// Load previously saved state
    async fn load_state(&self) -> Result<SerializedState, String>;
    
    /// Check if persistence is available
    async fn is_available(&self) -> bool;
}

/// Serialized state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedState {
    pub app_state: AppState,
    pub ui_state: UiState,
    pub git_state: GitState,
    pub task_state: TaskState,
    pub ai_state: AiState,
    pub timestamp: DateTime<Utc>,
}

/// File-based state persistence implementation
pub struct FilePersistence {
    file_path: std::path::PathBuf,
}

impl FilePersistence {
    pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait::async_trait]
impl StatePersistence for FilePersistence {
    async fn save_state(&self, state: &SerializedState) -> Result<(), String> {
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        
        tokio::fs::write(&self.file_path, json).await
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        Ok(())
    }
    
    async fn load_state(&self) -> Result<SerializedState, String> {
        let content = tokio::fs::read_to_string(&self.file_path).await
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let state = serde_json::from_str(&content)
            .map_err(|e| format!("Deserialization failed: {}", e))?;
        
        Ok(state)
    }
    
    async fn is_available(&self) -> bool {
        self.file_path.exists()
    }
}

/// In-memory state persistence for testing
pub struct MemoryPersistence {
    state: Arc<RwLock<Option<SerializedState>>>,
}

impl MemoryPersistence {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for MemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StatePersistence for MemoryPersistence {
    async fn save_state(&self, state: &SerializedState) -> Result<(), String> {
        let mut stored_state = self.state.write().await;
        *stored_state = Some(state.clone());
        Ok(())
    }
    
    async fn load_state(&self) -> Result<SerializedState, String> {
        let stored_state = self.state.read().await;
        stored_state.clone().ok_or_else(|| "No state stored".to_string())
    }
    
    async fn is_available(&self) -> bool {
        let stored_state = self.state.read().await;
        stored_state.is_some()
    }
}
