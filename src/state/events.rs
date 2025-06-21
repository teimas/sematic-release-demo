//! State Event System
//! 
//! This module defines all state change events and provides mechanisms for
//! subscribing to and reacting to state changes throughout the application.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// All possible state change events in the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateEvent {
    // Application-level events
    AppStateChanged { 
        previous: String, 
        current: String,
        timestamp: DateTime<Utc>,
    },
    ScreenChanged { 
        previous: String, 
        current: String,
        timestamp: DateTime<Utc>,
    },
    ConfigUpdated {
        changes: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    
    // UI state events
    UiModeChanged {
        previous: String,
        current: String,
        timestamp: DateTime<Utc>,
    },
    FocusChanged {
        previous: Option<String>,
        current: String,
        timestamp: DateTime<Utc>,
    },
    FormFieldUpdated {
        field: String,
        value: String,
        timestamp: DateTime<Utc>,
    },
    ErrorDisplayed {
        error: String,
        context: Option<String>,
        timestamp: DateTime<Utc>,
    },
    ErrorCleared {
        timestamp: DateTime<Utc>,
    },
    
    // Task management events
    TasksLoaded {
        source: String,
        count: usize,
        timestamp: DateTime<Utc>,
    },
    TaskSelected {
        task_id: String,
        task_title: String,
        source: String,
        timestamp: DateTime<Utc>,
    },
    
    // AI events
    AiAnalysisStarted {
        analysis_type: String,
        input_summary: String,
        timestamp: DateTime<Utc>,
    },
    AiAnalysisCompleted {
        analysis_id: String,
        analysis_type: String,
        result_summary: String,
        confidence: f32,
        timestamp: DateTime<Utc>,
    },
    
    // Release events
    ReleaseStarted {
        version: String,
        release_type: String,
        dry_run: bool,
        timestamp: DateTime<Utc>,
    },
    ReleaseCompleted {
        version: String,
        success: bool,
        release_notes: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl StateEvent {
    /// Get the timestamp of when this event occurred
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            StateEvent::AppStateChanged { timestamp, .. } => *timestamp,
            StateEvent::ScreenChanged { timestamp, .. } => *timestamp,
            StateEvent::ConfigUpdated { timestamp, .. } => *timestamp,
            StateEvent::UiModeChanged { timestamp, .. } => *timestamp,
            StateEvent::FocusChanged { timestamp, .. } => *timestamp,
            StateEvent::FormFieldUpdated { timestamp, .. } => *timestamp,
            StateEvent::ErrorDisplayed { timestamp, .. } => *timestamp,
            StateEvent::ErrorCleared { timestamp } => *timestamp,
            StateEvent::TasksLoaded { timestamp, .. } => *timestamp,
            StateEvent::TaskSelected { timestamp, .. } => *timestamp,
            StateEvent::AiAnalysisStarted { timestamp, .. } => *timestamp,
            StateEvent::AiAnalysisCompleted { timestamp, .. } => *timestamp,
            StateEvent::ReleaseStarted { timestamp, .. } => *timestamp,
            StateEvent::ReleaseCompleted { timestamp, .. } => *timestamp,
        }
    }
    
    /// Get a human-readable description of this event
    pub fn description(&self) -> String {
        match self {
            StateEvent::AppStateChanged { previous, current, .. } => {
                format!("App state changed from {} to {}", previous, current)
            }
            StateEvent::ScreenChanged { previous, current, .. } => {
                format!("Screen changed from {} to {}", previous, current)
            }
            StateEvent::ConfigUpdated { changes, .. } => {
                format!("Configuration updated: {}", changes.join(", "))
            }
            StateEvent::UiModeChanged { previous, current, .. } => {
                format!("UI mode changed from {} to {}", previous, current)
            }
            StateEvent::FocusChanged { current, .. } => {
                format!("Focus changed to {}", current)
            }
            StateEvent::FormFieldUpdated { field, .. } => {
                format!("Form field '{}' updated", field)
            }
            StateEvent::ErrorDisplayed { error, .. } => {
                format!("Error displayed: {}", error)
            }
            StateEvent::ErrorCleared { .. } => {
                "Error cleared".to_string()
            }
            StateEvent::TasksLoaded { source, count, .. } => {
                format!("Loaded {} tasks from {}", count, source)
            }
            StateEvent::TaskSelected { task_title, source, .. } => {
                format!("Selected task '{}' from {}", task_title, source)
            }
            StateEvent::AiAnalysisStarted { analysis_type, .. } => {
                format!("AI analysis started: {}", analysis_type)
            }
            StateEvent::AiAnalysisCompleted { analysis_type, confidence, .. } => {
                format!("AI analysis completed: {} (confidence: {:.1}%)", analysis_type, confidence * 100.0)
            }
            StateEvent::ReleaseStarted { version, release_type, dry_run, .. } => {
                format!("Release {} started: {} {}", 
                    version, 
                    release_type, 
                    if *dry_run { "(dry run)" } else { "" })
            }
            StateEvent::ReleaseCompleted { version, success, .. } => {
                format!("Release {} {}", version, if *success { "completed successfully" } else { "failed" })
            }
        }
    }
}

/// Represents a logical change to application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub event: StateEvent,
    pub source: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl StateChange {
    pub fn new(event: StateEvent, source: impl Into<String>) -> Self {
        Self {
            event,
            source: source.into(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Subscription handle for state events
#[derive(Debug)]
pub struct StateSubscription {
    receiver: tokio::sync::broadcast::Receiver<StateEvent>,
}

impl StateSubscription {
    pub fn new(receiver: tokio::sync::broadcast::Receiver<StateEvent>) -> Self {
        Self { receiver }
    }
    
    /// Try to receive the next state event without blocking
    pub fn try_recv(&mut self) -> Result<StateEvent, tokio::sync::broadcast::error::TryRecvError> {
        self.receiver.try_recv()
    }
    
    /// Wait for the next state event
    pub async fn recv(&mut self) -> Result<StateEvent, tokio::sync::broadcast::error::RecvError> {
        self.receiver.recv().await
    }
}

/// Helper trait for creating state events with timestamps
pub trait StateEventBuilder {
    fn with_timestamp(self) -> StateEvent;
}

// Implement for all state event variants that need timestamps
// This would normally be done with a macro, but keeping it simple for now 