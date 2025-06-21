//! Task management repository port interface
//! 
//! This module defines the port (interface) for task management operations.
//! The actual implementation will be provided by the infrastructure layer.

use crate::domains::tasks::{
    entities::{Task, TaskTemplate, TaskAutomationRule, BulkTaskOperation, TaskEvent},
    errors::TaskManagementDomainError,
    value_objects::{
        TaskId, TaskStatus, TaskPriority, TaskAssignee, ExternalSystemConfig,
        TaskSystem, TaskComment, TaskDependency
    }
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Port for basic task operations
/// 
/// This trait defines all CRUD operations for tasks.
#[async_trait]
pub trait TaskRepositoryPort: Send + Sync {
    /// Creates a new task
    async fn create_task(&self, task: &Task) -> Result<(), TaskManagementDomainError>;
    
    /// Gets a task by ID
    async fn get_task(&self, id: &TaskId) -> Result<Option<Task>, TaskManagementDomainError>;
    
    /// Updates an existing task
    async fn update_task(&self, task: &Task) -> Result<(), TaskManagementDomainError>;
    
    /// Deletes a task
    async fn delete_task(&self, id: &TaskId) -> Result<(), TaskManagementDomainError>;
    
    /// Gets all tasks for a system
    async fn get_tasks_by_system(
        &self,
        system: &TaskSystem,
    ) -> Result<Vec<Task>, TaskManagementDomainError>;
    
    /// Gets tasks by status
    async fn get_tasks_by_status(
        &self,
        status: &TaskStatus,
    ) -> Result<Vec<Task>, TaskManagementDomainError>;
    
    /// Gets tasks assigned to a user
    async fn get_tasks_by_assignee(
        &self,
        assignee: &TaskAssignee,
    ) -> Result<Vec<Task>, TaskManagementDomainError>;
    
    /// Gets tasks with a specific label
    async fn get_tasks_by_label(
        &self,
        label: &str,
    ) -> Result<Vec<Task>, TaskManagementDomainError>;
    
    /// Gets overdue tasks
    async fn get_overdue_tasks(&self) -> Result<Vec<Task>, TaskManagementDomainError>;
    
    /// Gets tasks updated since a specific date
    async fn get_tasks_updated_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<Task>, TaskManagementDomainError>;
    
    /// Searches tasks by text query
    async fn search_tasks(
        &self,
        query: &str,
        system: Option<&TaskSystem>,
    ) -> Result<Vec<Task>, TaskManagementDomainError>;
}

/// Port for external system synchronization
/// 
/// This trait handles synchronization with external task management systems.
#[async_trait]
pub trait TaskSynchronizationPort: Send + Sync {
    /// Synchronizes a task with external system
    async fn sync_task_to_external(
        &self,
        task: &Task,
        config: &ExternalSystemConfig,
    ) -> Result<Task, TaskManagementDomainError>;
    
    /// Fetches a task from external system
    async fn fetch_task_from_external(
        &self,
        id: &TaskId,
        config: &ExternalSystemConfig,
    ) -> Result<Option<Task>, TaskManagementDomainError>;
    
    /// Synchronizes all tasks for a system
    async fn sync_all_tasks(
        &self,
        system: &TaskSystem,
        config: &ExternalSystemConfig,
    ) -> Result<SyncResult, TaskManagementDomainError>;
    
    /// Gets synchronization status
    async fn get_sync_status(
        &self,
        system: &TaskSystem,
    ) -> Result<SyncStatus, TaskManagementDomainError>;
    
    /// Forces a full resync
    async fn force_full_sync(
        &self,
        system: &TaskSystem,
        config: &ExternalSystemConfig,
    ) -> Result<SyncResult, TaskManagementDomainError>;
}

/// Port for task template operations
/// 
/// This trait handles task template management.
#[async_trait]
pub trait TaskTemplatePort: Send + Sync {
    /// Creates a new task template
    async fn create_template(
        &self,
        template: &TaskTemplate,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Gets a template by name
    async fn get_template(
        &self,
        name: &str,
        system: &TaskSystem,
    ) -> Result<Option<TaskTemplate>, TaskManagementDomainError>;
    
    /// Updates a template
    async fn update_template(
        &self,
        template: &TaskTemplate,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Deletes a template
    async fn delete_template(
        &self,
        name: &str,
        system: &TaskSystem,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Gets all templates for a system
    async fn get_templates_by_system(
        &self,
        system: &TaskSystem,
    ) -> Result<Vec<TaskTemplate>, TaskManagementDomainError>;
    
    /// Creates a task from a template
    async fn create_task_from_template(
        &self,
        template_name: &str,
        system: &TaskSystem,
        context: &HashMap<String, String>,
    ) -> Result<Task, TaskManagementDomainError>;
}

/// Port for task automation operations
/// 
/// This trait handles automation rules and their execution.
#[async_trait]
pub trait TaskAutomationPort: Send + Sync {
    /// Creates a new automation rule
    async fn create_automation_rule(
        &self,
        rule: &TaskAutomationRule,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Gets an automation rule by name
    async fn get_automation_rule(
        &self,
        name: &str,
        system: &TaskSystem,
    ) -> Result<Option<TaskAutomationRule>, TaskManagementDomainError>;
    
    /// Updates an automation rule
    async fn update_automation_rule(
        &self,
        rule: &TaskAutomationRule,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Deletes an automation rule
    async fn delete_automation_rule(
        &self,
        name: &str,
        system: &TaskSystem,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Gets all automation rules for a system
    async fn get_automation_rules_by_system(
        &self,
        system: &TaskSystem,
    ) -> Result<Vec<TaskAutomationRule>, TaskManagementDomainError>;
    
    /// Executes automation rules for a task event
    async fn execute_automation_rules(
        &self,
        event: &TaskEvent,
    ) -> Result<Vec<AutomationExecutionResult>, TaskManagementDomainError>;
    
    /// Gets automation execution history
    async fn get_automation_history(
        &self,
        task_id: &TaskId,
        limit: Option<usize>,
    ) -> Result<Vec<AutomationExecutionResult>, TaskManagementDomainError>;
}

/// Port for bulk task operations
/// 
/// This trait handles operations on multiple tasks.
#[async_trait]
pub trait BulkTaskOperationsPort: Send + Sync {
    /// Executes a bulk operation
    async fn execute_bulk_operation(
        &self,
        operation: &mut BulkTaskOperation,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Gets the status of a bulk operation
    async fn get_bulk_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<Option<BulkOperationStatus>, TaskManagementDomainError>;
    
    /// Cancels a bulk operation
    async fn cancel_bulk_operation(
        &self,
        operation_id: &str,
    ) -> Result<(), TaskManagementDomainError>;
    
    /// Gets history of bulk operations
    async fn get_bulk_operation_history(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<BulkOperationStatus>, TaskManagementDomainError>;
}

/// Port for task analytics and reporting
/// 
/// This trait provides analytics and reporting capabilities.
#[async_trait]
pub trait TaskAnalyticsPort: Send + Sync {
    /// Gets task metrics for a system
    async fn get_task_metrics(
        &self,
        system: &TaskSystem,
        date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Result<TaskMetrics, TaskManagementDomainError>;
    
    /// Gets productivity metrics for an assignee
    async fn get_assignee_metrics(
        &self,
        assignee: &TaskAssignee,
        date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Result<AssigneeMetrics, TaskManagementDomainError>;
    
    /// Gets task cycle time analysis
    async fn get_cycle_time_analysis(
        &self,
        system: &TaskSystem,
        date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Result<CycleTimeAnalysis, TaskManagementDomainError>;
    
    /// Gets task burndown data
    async fn get_burndown_data(
        &self,
        system: &TaskSystem,
        sprint_dates: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<BurndownData, TaskManagementDomainError>;
}

/// Result of synchronization operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub tasks_created: usize,
    pub tasks_updated: usize,
    pub tasks_deleted: usize,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
}

/// Synchronization status
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub system: TaskSystem,
    pub last_sync: Option<DateTime<Utc>>,
    pub is_syncing: bool,
    pub sync_errors: Vec<String>,
    pub next_sync: Option<DateTime<Utc>>,
}

/// Result of automation rule execution
#[derive(Debug, Clone)]
pub struct AutomationExecutionResult {
    pub rule_name: String,
    pub task_id: TaskId,
    pub actions_executed: Vec<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
}

/// Status of bulk operation
#[derive(Debug, Clone)]
pub struct BulkOperationStatus {
    pub operation_id: String,
    pub operation_type: String,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub is_cancelled: bool,
}

/// Task metrics for analytics
#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub overdue_tasks: usize,
    pub average_completion_time: Option<std::time::Duration>,
    pub tasks_by_priority: HashMap<TaskPriority, usize>,
    pub tasks_by_status: HashMap<String, usize>,
}

/// Assignee productivity metrics
#[derive(Debug, Clone)]
pub struct AssigneeMetrics {
    pub assignee: TaskAssignee,
    pub tasks_completed: usize,
    pub tasks_in_progress: usize,
    pub average_completion_time: Option<std::time::Duration>,
    pub time_logged: std::time::Duration,
    pub efficiency_score: f32,
}

/// Cycle time analysis data
#[derive(Debug, Clone)]
pub struct CycleTimeAnalysis {
    pub average_cycle_time: std::time::Duration,
    pub median_cycle_time: std::time::Duration,
    pub cycle_times_by_priority: HashMap<TaskPriority, std::time::Duration>,
    pub bottlenecks: Vec<String>,
}

/// Burndown chart data
#[derive(Debug, Clone)]
pub struct BurndownData {
    pub sprint_start: DateTime<Utc>,
    pub sprint_end: DateTime<Utc>,
    pub ideal_burndown: Vec<(DateTime<Utc>, usize)>,
    pub actual_burndown: Vec<(DateTime<Utc>, usize)>,
    pub scope_changes: Vec<(DateTime<Utc>, i32)>,
} 