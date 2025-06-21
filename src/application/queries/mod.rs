//! Application queries (CQRS)
//! 
//! This module provides query handlers for read operations.

#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use std::any::Any;

// Query infrastructure
#[cfg(feature = "new-domains")]
pub trait Query: Send + Sync + 'static {
    /// Get the query as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "new-domains")]
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    type Result: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn handle(&self, query: Q) -> Result<Self::Result, Self::Error>;
}

#[cfg(feature = "new-domains")]
#[async_trait]
pub trait QueryBus: Send + Sync {
    async fn execute(&self, query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

// Query definitions
pub mod get_release_status;
pub mod list_tasks;
pub mod get_git_history;

// Re-exports
#[cfg(feature = "new-domains")]
pub use get_release_status::*;
#[cfg(feature = "new-domains")]
pub use list_tasks::*;
#[cfg(feature = "new-domains")]
pub use get_git_history::*;

/// Simple implementation of QueryBus
#[cfg(feature = "new-domains")]
pub struct SimpleQueryBus {
    release_orchestrator: Arc<dyn crate::application::services::ReleaseOrchestrator>,
    task_manager: Arc<dyn crate::application::services::TaskManager>,
}

#[cfg(feature = "new-domains")]
impl SimpleQueryBus {
    pub fn new(
        release_orchestrator: Arc<dyn crate::application::services::ReleaseOrchestrator>,
        task_manager: Arc<dyn crate::application::services::TaskManager>,
    ) -> Self {
        Self {
            release_orchestrator,
            task_manager,
        }
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl QueryBus for SimpleQueryBus {
    async fn execute(&self, query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // Try to downcast to known query types
        let any_query = query.as_any();
        
        if let Some(q) = any_query.downcast_ref::<GetReleaseStatusQuery>() {
            let result = self.release_orchestrator.get_release_status(
                q.repository_path.clone(),
            ).await?;
            
            return Ok(Box::new(result) as Box<dyn Any + Send + Sync>);
        }
        
        if let Some(q) = any_query.downcast_ref::<ListTasksQuery>() {
            let result = self.task_manager.list_tasks(
                q.filters.clone(),
                q.pagination.clone(),
                q.sort.clone(),
            ).await?;
            
            return Ok(Box::new(result) as Box<dyn Any + Send + Sync>);
        }
        
        if let Some(q) = any_query.downcast_ref::<GetGitHistoryQuery>() {
            let result = self.release_orchestrator.get_git_history(
                q.repository_path.clone(),
                q.options.clone(),
            ).await?;
            
            return Ok(Box::new(result) as Box<dyn Any + Send + Sync>);
        }
        
        Err("Unknown query type".into())
    }
} 