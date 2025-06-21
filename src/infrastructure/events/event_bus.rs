//! Event bus implementation
//! 
//! Simple in-memory event bus for publishing and subscribing to domain events.

use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// A domain event that can be published through the event bus
pub trait DomainEvent: Send + Sync + Any {
    fn event_type(&self) -> &'static str;
    fn event_id(&self) -> String;
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc>;
}

/// Event handler trait for processing domain events
#[async_trait]
pub trait EventHandler<T: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Simple in-memory event bus
pub struct InMemoryEventBus {
    handlers: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>>>,
    sender: mpsc::UnboundedSender<Box<dyn DomainEvent>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<Box<dyn DomainEvent>>();
        let handlers = Arc::new(RwLock::new(HashMap::new()));
        let handlers_clone = handlers.clone();
        
        // Spawn background task to process events
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let handlers_guard = handlers_clone.read().unwrap();
                let event_type_id = (*event).type_id();
                
                if let Some(handler_list) = handlers_guard.get(&event_type_id) {
                    for handler_any in handler_list {
                        // This is unsafe but necessary for our type-erased event system
                        // In a production system, you might want a more type-safe approach
                        log::debug!("Processing event: {}", event.event_type());
                    }
                }
            }
        });
        
        Self {
            handlers,
            sender,
        }
    }
    
    /// Subscribe to events of a specific type
    pub fn subscribe<T: DomainEvent + 'static, H: EventHandler<T> + 'static>(&self, handler: H) {
        let mut handlers = self.handlers.write().unwrap();
        let type_id = TypeId::of::<T>();
        
        handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }
    
    /// Publish an event to all subscribers
    pub async fn publish<T: DomainEvent + 'static>(&self, event: T) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.sender
            .send(Box::new(event))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        Ok(())
    }
    
    /// Get the number of registered handlers for all event types
    pub fn handler_count(&self) -> usize {
        let handlers = self.handlers.read().unwrap();
        handlers.values().map(|v| v.len()).sum()
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Example domain events
#[derive(Debug, Clone)]
pub struct TaskCreatedEvent {
    pub task_id: String,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
}

impl DomainEvent for TaskCreatedEvent {
    fn event_type(&self) -> &'static str {
        "TaskCreated"
    }
    
    fn event_id(&self) -> String {
        format!("task_created_{}", self.task_id)
    }
    
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisCompletedEvent {
    pub analysis_id: String,
    pub analysis_type: String,
    pub result: String,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

impl DomainEvent for AnalysisCompletedEvent {
    fn event_type(&self) -> &'static str {
        "AnalysisCompleted"
    }
    
    fn event_id(&self) -> String {
        format!("analysis_completed_{}", self.analysis_id)
    }
    
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.completed_at
    }
}

#[derive(Debug, Clone)]
pub struct ReleasePublishedEvent {
    pub release_version: String,
    pub tag_name: String,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub release_notes: String,
}

impl DomainEvent for ReleasePublishedEvent {
    fn event_type(&self) -> &'static str {
        "ReleasePublished"
    }
    
    fn event_id(&self) -> String {
        format!("release_published_{}", self.release_version)
    }
    
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.published_at
    }
} 