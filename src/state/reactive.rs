//! Reactive State Management
//! 
//! This module provides reactive patterns for state management, including
//! observers, computed values, and reactive state updates.

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use derive_more::Display;

use super::events::StateEvent;
use super::stores::{AppState, UiState, GitState, TaskState, AiState};

/// Trait for objects that can observe state changes
#[async_trait]
pub trait StateObserver: Send + Sync {
    /// Called when state changes
    async fn on_state_changed(&self, event: &StateEvent);
    
    /// Get the observer's name for debugging
    fn observer_name(&self) -> &str;
    
    /// Check if this observer should react to a specific event
    fn should_react_to(&self, _event: &StateEvent) -> bool {
        // Default: react to all events
        true
    }
}

/// A reactive state container that notifies observers of changes
pub struct ReactiveState<T> {
    state: Arc<RwLock<T>>,
    observers: Arc<RwLock<Vec<Arc<dyn StateObserver>>>>,
    event_tx: broadcast::Sender<StateEvent>,
    change_count: Arc<RwLock<u64>>,
}

impl<T> ReactiveState<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new reactive state with initial value
    pub fn new(initial_state: T) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            observers: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            change_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Get read access to the current state
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, T> {
        self.state.read().await
    }
    
    /// Get write access to the state (does not notify observers)
    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, T> {
        self.state.write().await
    }
    
    /// Update the state and notify observers
    pub async fn update<F>(&self, updater: F) -> Result<(), ReactiveError>
    where
        F: FnOnce(&mut T) -> Option<StateEvent> + Send,
    {
        let mut state = self.state.write().await;
        
        if let Some(event) = updater(&mut *state) {
            // Increment change count
            {
                let mut count = self.change_count.write().await;
                *count += 1;
            }
            
            // Notify observers
            self.notify_observers(&event).await;
            
            // Broadcast event
            let _ = self.event_tx.send(event);
        }
        
        Ok(())
    }
    
    /// Add an observer to be notified of state changes
    pub async fn add_observer(&self, observer: Arc<dyn StateObserver>) {
        let mut observers = self.observers.write().await;
        observers.push(observer);
    }
    
    /// Remove an observer by name
    pub async fn remove_observer(&self, observer_name: &str) {
        let mut observers = self.observers.write().await;
        observers.retain(|observer| observer.observer_name() != observer_name);
    }
    
    /// Subscribe to state change events
    pub fn subscribe(&self) -> broadcast::Receiver<StateEvent> {
        self.event_tx.subscribe()
    }
    
    /// Get the number of times the state has changed
    pub async fn change_count(&self) -> u64 {
        *self.change_count.read().await
    }
    
    /// Notify all observers of a state change
    async fn notify_observers(&self, event: &StateEvent) {
        let observers = self.observers.read().await;
        
        for observer in observers.iter() {
            if observer.should_react_to(event) {
                if let Err(e) = tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    observer.on_state_changed(event)
                ).await {
                    tracing::warn!(
                        "Observer '{}' timed out processing event: {:?}",
                        observer.observer_name(),
                        e
                    );
                }
            }
        }
    }
}

/// Errors that can occur in reactive state management
#[derive(Debug, thiserror::Error, Display)]
pub enum ReactiveError {
    #[display(fmt = "Observer timeout: {}", observer_name)]
    ObserverTimeout { observer_name: String },
    
    #[display(fmt = "State update failed: {}", reason)]
    UpdateFailed { reason: String },
    
    #[display(fmt = "Observer registration failed: {}", reason)]
    ObserverRegistrationFailed { reason: String },
}

/// Computed state that derives its value from other reactive states
pub struct ComputedState<T, F> {
    compute_fn: F,
    cached_value: Arc<RwLock<Option<T>>>,
    dependencies: Vec<String>, // Names of states this depends on
    is_dirty: Arc<RwLock<bool>>,
}

impl<T, F> ComputedState<T, F>
where
    T: Clone + Send + Sync + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    /// Create a new computed state
    pub fn new(compute_fn: F, dependencies: Vec<String>) -> Self {
        Self {
            compute_fn,
            cached_value: Arc::new(RwLock::new(None)),
            dependencies,
            is_dirty: Arc::new(RwLock::new(true)),
        }
    }
    
    /// Get the current computed value, recomputing if necessary
    pub async fn get(&self) -> T {
        let is_dirty = *self.is_dirty.read().await;
        
        if is_dirty {
            let new_value = (self.compute_fn)();
            {
                let mut cached = self.cached_value.write().await;
                *cached = Some(new_value.clone());
            }
            {
                let mut dirty = self.is_dirty.write().await;
                *dirty = false;
            }
            new_value
        } else {
            let cached = self.cached_value.read().await;
            cached.as_ref().unwrap().clone()
        }
    }
    
    /// Mark this computed state as dirty (needs recomputation)
    pub async fn invalidate(&self) {
        let mut dirty = self.is_dirty.write().await;
        *dirty = true;
    }
    
    /// Get the dependencies of this computed state
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}

/// Observer that logs all state changes
pub struct LoggingObserver {
    name: String,
}

impl LoggingObserver {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[async_trait]
impl StateObserver for LoggingObserver {
    async fn on_state_changed(&self, event: &StateEvent) {
        tracing::debug!(
            observer = %self.name,
            event = ?event,
            "State change observed"
        );
    }
    
    fn observer_name(&self) -> &str {
        &self.name
    }
}

/// Observer that tracks performance metrics
pub struct PerformanceObserver {
    name: String,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl PerformanceObserver {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }
    
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
}

#[async_trait]
impl StateObserver for PerformanceObserver {
    async fn on_state_changed(&self, event: &StateEvent) {
        let mut metrics = self.metrics.write().await;
        metrics.total_events += 1;
        metrics.last_event_time = chrono::Utc::now();
        
        // Track event type frequency
        let event_type = std::mem::discriminant(event);
        let event_name = format!("{:?}", event_type);
        *metrics.event_type_counts.entry(event_name).or_insert(0) += 1;
    }
    
    fn observer_name(&self) -> &str {
        &self.name
    }
}

/// Performance metrics for state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_events: u64,
    pub last_event_time: chrono::DateTime<chrono::Utc>,
    pub event_type_counts: HashMap<String, u64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_events: 0,
            last_event_time: chrono::Utc::now(),
            event_type_counts: HashMap::new(),
        }
    }
}

/// Observer that can filter events based on criteria
pub struct FilteringObserver<F> {
    name: String,
    filter: F,
    inner_observer: Arc<dyn StateObserver>,
}

impl<F> FilteringObserver<F>
where
    F: Fn(&StateEvent) -> bool + Send + Sync,
{
    pub fn new(
        name: impl Into<String>,
        filter: F,
        inner_observer: Arc<dyn StateObserver>,
    ) -> Self {
        Self {
            name: name.into(),
            filter,
            inner_observer,
        }
    }
}

#[async_trait]
impl<F> StateObserver for FilteringObserver<F>
where
    F: Fn(&StateEvent) -> bool + Send + Sync,
{
    async fn on_state_changed(&self, event: &StateEvent) {
        if (self.filter)(event) {
            self.inner_observer.on_state_changed(event).await;
        }
    }
    
    fn observer_name(&self) -> &str {
        &self.name
    }
    
    fn should_react_to(&self, event: &StateEvent) -> bool {
        (self.filter)(event)
    }
}

/// Reactive state manager that coordinates multiple reactive states
pub struct ReactiveStateManager {
    app_state: ReactiveState<AppState>,
    ui_state: ReactiveState<UiState>,
    git_state: ReactiveState<GitState>,
    task_state: ReactiveState<TaskState>,
    ai_state: ReactiveState<AiState>,
    global_observers: Arc<RwLock<Vec<Arc<dyn StateObserver>>>>,
}

impl ReactiveStateManager {
    /// Create a new reactive state manager with default states
    pub fn new() -> Self {
        Self {
            app_state: ReactiveState::new(AppState::default()),
            ui_state: ReactiveState::new(UiState::default()),
            git_state: ReactiveState::new(GitState::default()),
            task_state: ReactiveState::new(TaskState::default()),
            ai_state: ReactiveState::new(AiState::default()),
            global_observers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Get access to the app state
    pub fn app_state(&self) -> &ReactiveState<AppState> {
        &self.app_state
    }
    
    /// Get access to the UI state
    pub fn ui_state(&self) -> &ReactiveState<UiState> {
        &self.ui_state
    }
    
    /// Get access to the Git state
    pub fn git_state(&self) -> &ReactiveState<GitState> {
        &self.git_state
    }
    
    /// Get access to the Task state
    pub fn task_state(&self) -> &ReactiveState<TaskState> {
        &self.task_state
    }
    
    /// Get access to the AI state
    pub fn ai_state(&self) -> &ReactiveState<AiState> {
        &self.ai_state
    }
    
    /// Add a global observer that observes all state changes
    pub async fn add_global_observer(&self, observer: Arc<dyn StateObserver>) {
        // Add to global list
        {
            let mut observers = self.global_observers.write().await;
            observers.push(observer.clone());
        }
        
        // Add to all individual state managers
        self.app_state.add_observer(observer.clone()).await;
        self.ui_state.add_observer(observer.clone()).await;
        self.git_state.add_observer(observer.clone()).await;
        self.task_state.add_observer(observer.clone()).await;
        self.ai_state.add_observer(observer).await;
    }
    
    /// Subscribe to all state change events
    pub fn subscribe_to_all(&self) -> Vec<broadcast::Receiver<StateEvent>> {
        vec![
            self.app_state.subscribe(),
            self.ui_state.subscribe(),
            self.git_state.subscribe(),
            self.task_state.subscribe(),
            self.ai_state.subscribe(),
        ]
    }
}

impl Default for ReactiveStateManager {
    fn default() -> Self {
        Self::new()
    }
}
