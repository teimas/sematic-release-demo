//! Application commands (CQRS)
//! 
//! This module provides command handlers for write operations.

#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use std::any::Any;

// Command infrastructure
#[cfg(feature = "new-domains")]
pub trait Command: Send + Sync + 'static {
    /// Get the command as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "new-domains")]
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    type Result: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn handle(&self, command: C) -> Result<Self::Result, Self::Error>;
}

#[cfg(feature = "new-domains")]
#[async_trait]
pub trait CommandBus: Send + Sync {
    async fn execute(&self, command: Box<dyn Command>) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

// Command definitions
pub mod create_release;
pub mod sync_tasks;
pub mod generate_notes;

// Re-exports
#[cfg(feature = "new-domains")]
pub use create_release::*;
#[cfg(feature = "new-domains")]
pub use sync_tasks::*;
#[cfg(feature = "new-domains")]
pub use generate_notes::*;

/// Simple implementation of CommandBus
#[cfg(feature = "new-domains")]
pub struct SimpleCommandBus {
    release_orchestrator: Arc<dyn crate::application::services::ReleaseOrchestrator>,
    task_manager: Arc<dyn crate::application::services::TaskManager>,
    ai_coordinator: Arc<dyn crate::application::services::AiCoordinator>,
}

#[cfg(feature = "new-domains")]
impl SimpleCommandBus {
    pub fn new(
        release_orchestrator: Arc<dyn crate::application::services::ReleaseOrchestrator>,
        task_manager: Arc<dyn crate::application::services::TaskManager>,
        ai_coordinator: Arc<dyn crate::application::services::AiCoordinator>,
    ) -> Self {
        Self {
            release_orchestrator,
            task_manager,
            ai_coordinator,
        }
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl CommandBus for SimpleCommandBus {
    async fn execute(&self, command: Box<dyn Command>) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // Try to downcast to known command types
        let any_command = command.as_any();
        
        if let Some(cmd) = any_command.downcast_ref::<CreateReleaseCommand>() {
            let result = self.release_orchestrator.execute_release(
                cmd.clone(),
            ).await?;
            
            return Ok(Box::new(result) as Box<dyn Any + Send + Sync>);
        }
        
        if let Some(cmd) = any_command.downcast_ref::<SyncTasksCommand>() {
            let result = self.task_manager.sync_tasks(
                cmd.clone(),
            ).await?;
            
            return Ok(Box::new(result) as Box<dyn Any + Send + Sync>);
        }
        
        if let Some(cmd) = any_command.downcast_ref::<GenerateNotesCommand>() {
            let result = self.ai_coordinator.generate_release_notes(
                cmd.clone(),
            ).await?;
            
            return Ok(Box::new(result) as Box<dyn Any + Send + Sync>);
        }
        
        Err("Unknown command type".into())
    }
} 