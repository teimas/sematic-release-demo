//! Sync Tasks Command
//! 
//! This command synchronizes tasks between different external systems like JIRA,
//! Monday.com, Azure DevOps, etc. with configurable synchronization options.

#[cfg(feature = "new-domains")]
use super::{Command, CommandHandler};
#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use std::any::Any;
#[cfg(feature = "new-domains")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc};

#[cfg(feature = "new-domains")]
use crate::domains::tasks::{
    entities::Task,
    value_objects::{TaskSystem, TaskId, ExternalSystemConfig},
    errors::TaskManagementDomainError,
};

/// Command to synchronize tasks across multiple systems
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTasksCommand {
    pub systems: Vec<String>,
    pub direction: SyncDirection,
    pub filters: Option<TaskFilters>,
}

#[cfg(feature = "new-domains")]
impl Command for SyncTasksCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "new-domains")]
impl SyncTasksCommand {
    pub fn new(systems: Vec<String>) -> Self {
        Self {
            systems,
            direction: SyncDirection::Bidirectional,
            filters: None,
        }
    }
    
    pub fn from_external(mut self) -> Self {
        self.direction = SyncDirection::FromExternal;
        self
    }
    
    pub fn to_external(mut self) -> Self {
        self.direction = SyncDirection::ToExternal;
        self
    }
    
    pub fn with_filters(mut self, filters: TaskFilters) -> Self {
        self.filters = Some(filters);
        self
    }
}

/// Direction of synchronization
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDirection {
    /// Sync from external systems to local
    FromExternal,
    /// Sync from local to external systems
    ToExternal,
    /// Bidirectional sync
    Bidirectional,
}

/// Comprehensive task filtering options
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilters {
    pub project_keys: Vec<String>,
    pub assignees: Vec<String>,
    pub statuses: Vec<String>,
    pub labels: Vec<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub priority: Option<String>,
    pub epic_keys: Vec<String>,
    pub custom_fields: HashMap<String, String>,
}

#[cfg(feature = "new-domains")]
impl Default for TaskFilters {
    fn default() -> Self {
        Self {
            project_keys: Vec::new(),
            assignees: Vec::new(),
            statuses: Vec::new(),
            labels: Vec::new(),
            created_after: None,
            updated_after: None,
            priority: None,
            epic_keys: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}

/// Result of task synchronization
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTasksResult {
    pub synchronized_tasks: Vec<TaskSyncResult>,
    pub conflicts: Vec<TaskConflict>,
    pub errors: Vec<SyncError>,
    pub total_processed: usize,
    pub duration_ms: u64,
    pub warnings: Vec<String>,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSyncResult {
    pub task_id: String,
    pub action: SyncAction,
    pub source_system: String,
    pub target_system: String,
    pub changes: Vec<String>,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncAction {
    Created,
    Updated,
    Deleted,
    Skipped,
}

/// Task conflict resolution strategy
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConflict {
    pub task_id: String,
    pub field: String,
    pub local_value: String,
    pub external_value: String,
    pub last_modified: DateTime<Utc>,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepLocal,
    KeepExternal,
    Merge,
    Skip,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub task_id: String,
    pub system: String,
    pub operation: String,
    pub error_message: String,
}

/// Errors that can occur during task synchronization
#[cfg(feature = "new-domains")]
#[derive(Debug, thiserror::Error)]
pub enum SyncTasksError {
    #[error("System connection failed: {system}: {message}")]
    SystemConnectionFailed { system: String, message: String },
    
    #[error("Authentication failed: {system}: {message}")]
    AuthenticationFailed { system: String, message: String },
    
    #[error("Task mapping failed: {task_id}: {message}")]
    TaskMappingFailed { task_id: String, message: String },
    
    #[error("Conflict resolution failed: {task_id}: {message}")]
    ConflictResolutionFailed { task_id: String, message: String },
    
    #[error("Batch processing failed: {batch_size}: {message}")]
    BatchProcessingFailed { batch_size: usize, message: String },
    
    #[error("Validation failed: {field}: {message}")]
    ValidationFailed { field: String, message: String },
}

/// Handler for sync tasks command
#[cfg(feature = "new-domains")]
pub struct SyncTasksHandler {
    task_manager: Arc<dyn TaskManager>,
}

#[cfg(feature = "new-domains")]
impl SyncTasksHandler {
    pub fn new(task_manager: Arc<dyn TaskManager>) -> Self {
        Self { task_manager }
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl CommandHandler<SyncTasksCommand> for SyncTasksHandler {
    type Result = SyncTasksResult;
    type Error = SyncTasksError;
    
    async fn handle(&self, command: SyncTasksCommand) -> Result<Self::Result, Self::Error> {
        self.task_manager
            .sync_tasks(command)
            .await
            .map_err(|e| SyncTasksError::BatchProcessingFailed {
                batch_size: 0,
                message: e.to_string(),
            })
    }
}

/// Task manager trait for dependency injection
#[cfg(feature = "new-domains")]
#[async_trait]
pub trait TaskManager: Send + Sync {
    async fn sync_tasks(&self, command: SyncTasksCommand) -> Result<SyncTasksResult, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_task(&self, id: &TaskId) -> Result<Option<Task>, TaskManagementDomainError>;
    async fn create_task(&self, task: &Task) -> Result<(), TaskManagementDomainError>;
    async fn update_task(&self, task: &Task) -> Result<(), TaskManagementDomainError>;
    async fn delete_task(&self, id: &TaskId) -> Result<(), TaskManagementDomainError>;
    
    // Query methods (these should ideally be in separate query handlers)
    async fn list_tasks(&self, filters: Option<crate::application::queries::TaskQueryFilters>, pagination: Option<crate::application::queries::Pagination>, sort: Option<crate::application::queries::TaskSorting>) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>>;
} 