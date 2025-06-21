//! Task management domain services
//! 
//! Domain services that implement complex task management business logic
//! involving multiple entities and external system integrations.

use crate::domains::tasks::{
    entities::{Task, TaskTemplate, TaskAutomationRule, BulkTaskOperation, TaskEvent},
    errors::TaskManagementDomainError,
    repository::{
        TaskRepositoryPort, TaskSynchronizationPort, TaskTemplatePort, TaskAutomationPort,
        BulkTaskOperationsPort, TaskAnalyticsPort, SyncResult, SyncStatus, AutomationExecutionResult,
        TaskMetrics, AssigneeMetrics
    },
    value_objects::{
        TaskId, TaskStatus, TaskPriority, TaskAssignee, ExternalSystemConfig,
        TaskSystem, TaskComment, TaskDependency, StatusCategory
    }
};
use chrono::{DateTime, Utc, Duration};
use std::{collections::HashMap, sync::Arc};

/// Task management domain service providing high-level task operations
pub struct TaskManagementDomainService {
    task_repository: Arc<dyn TaskRepositoryPort>,
    sync_port: Arc<dyn TaskSynchronizationPort>,
    template_port: Arc<dyn TaskTemplatePort>,
    automation_port: Arc<dyn TaskAutomationPort>,
    bulk_operations_port: Arc<dyn BulkTaskOperationsPort>,
    analytics_port: Arc<dyn TaskAnalyticsPort>,
}

impl TaskManagementDomainService {
    /// Creates a new task management domain service
    pub fn new(
        task_repository: Arc<dyn TaskRepositoryPort>,
        sync_port: Arc<dyn TaskSynchronizationPort>,
        template_port: Arc<dyn TaskTemplatePort>,
        automation_port: Arc<dyn TaskAutomationPort>,
        bulk_operations_port: Arc<dyn BulkTaskOperationsPort>,
        analytics_port: Arc<dyn TaskAnalyticsPort>,
    ) -> Self {
        Self {
            task_repository,
            sync_port,
            template_port,
            automation_port,
            bulk_operations_port,
            analytics_port,
        }
    }
    
    /// Creates a new task with validation and automation
    pub async fn create_task(&self, mut task: Task) -> Result<Task, TaskManagementDomainError> {
        // Validate the task
        task.validate()?;
        
        // Store the task
        self.task_repository.create_task(&task).await?;
        
        // Trigger automation rules
        let event = TaskEvent::Created(task.clone());
        self.automation_port.execute_automation_rules(&event).await?;
        
        Ok(task)
    }
    
    /// Creates a task from a template
    pub async fn create_task_from_template(
        &self,
        template_name: &str,
        system: &TaskSystem,
        context: &HashMap<String, String>,
    ) -> Result<Task, TaskManagementDomainError> {
        let task = self.template_port
            .create_task_from_template(template_name, system, context)
            .await?;
        
        self.create_task(task).await
    }
    
    /// Updates a task with validation and automation
    pub async fn update_task(&self, mut task: Task) -> Result<Task, TaskManagementDomainError> {
        // Get the previous version for comparison
        let previous_task = self.task_repository
            .get_task(&task.id)
            .await?
            .ok_or_else(|| TaskManagementDomainError::TaskNotFound {
                task_id: task.id.to_string(),
            })?;
        
        // Validate the updated task
        task.validate()?;
        
        // Update the task
        self.task_repository.update_task(&task).await?;
        
        // Trigger automation rules for different types of changes
        let events = self.detect_task_changes(&previous_task, &task);
        for event in events {
            self.automation_port.execute_automation_rules(&event).await?;
        }
        
        Ok(task)
    }
    
    /// Updates task status with business logic
    pub async fn update_task_status(
        &self,
        task_id: &TaskId,
        new_status: TaskStatus,
    ) -> Result<Task, TaskManagementDomainError> {
        let mut task = self.task_repository
            .get_task(task_id)
            .await?
            .ok_or_else(|| TaskManagementDomainError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;
        
        let previous_status = task.status.clone();
        task.update_status(new_status.clone())?;
        
        self.task_repository.update_task(&task).await?;
        
        // Trigger status change automation
        let event = TaskEvent::StatusChanged(task.clone(), previous_status);
        self.automation_port.execute_automation_rules(&event).await?;
        
        Ok(task)
    }
    
    /// Assigns a task to a user
    pub async fn assign_task(
        &self,
        task_id: &TaskId,
        assignee: TaskAssignee,
    ) -> Result<Task, TaskManagementDomainError> {
        let mut task = self.task_repository
            .get_task(task_id)
            .await?
            .ok_or_else(|| TaskManagementDomainError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;
        
        let previous_assignee = task.assignee.clone();
        task.assign_to(assignee.clone())?;
        
        self.task_repository.update_task(&task).await?;
        
        // Trigger assignee change automation
        let event = TaskEvent::AssigneeChanged(task.clone(), previous_assignee);
        self.automation_port.execute_automation_rules(&event).await?;
        
        Ok(task)
    }
    
    /// Synchronizes tasks with external system
    pub async fn sync_with_external_system(
        &self,
        system: &TaskSystem,
        config: &ExternalSystemConfig,
        force_full_sync: bool,
    ) -> Result<SyncResult, TaskManagementDomainError> {
        config.validate()?;
        
        if force_full_sync {
            self.sync_port.force_full_sync(system, config).await
        } else {
            self.sync_port.sync_all_tasks(system, config).await
        }
    }
    
    /// Gets task metrics and analytics
    pub async fn get_task_analytics(
        &self,
        system: &TaskSystem,
        date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Result<TaskMetrics, TaskManagementDomainError> {
        self.analytics_port.get_task_metrics(system, date_range).await
    }
    
    /// Gets productivity metrics for an assignee
    pub async fn get_assignee_productivity(
        &self,
        assignee: &TaskAssignee,
        date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Result<AssigneeMetrics, TaskManagementDomainError> {
        self.analytics_port.get_assignee_metrics(assignee, date_range).await
    }
    
    /// Detects what changed between two task versions
    fn detect_task_changes(&self, previous: &Task, current: &Task) -> Vec<TaskEvent> {
        let mut events = Vec::new();
        
        // Always add the general update event
        events.push(TaskEvent::Updated(current.clone()));
        
        // Check for specific changes
        if previous.status != current.status {
            events.push(TaskEvent::StatusChanged(current.clone(), previous.status.clone()));
        }
        
        if previous.assignee != current.assignee {
            events.push(TaskEvent::AssigneeChanged(current.clone(), previous.assignee.clone()));
        }
        
        if previous.priority != current.priority {
            events.push(TaskEvent::PriorityChanged(current.clone(), previous.priority.clone()));
        }
        
        if previous.comments.len() < current.comments.len() {
            events.push(TaskEvent::CommentAdded(current.clone()));
        }
        
        events
    }
}

/// Service for task workflow management
pub struct TaskWorkflowService;

impl TaskWorkflowService {
    /// Gets valid next statuses for a task
    pub fn get_valid_transitions(current_status: &TaskStatus, system: &TaskSystem) -> Vec<TaskStatus> {
        use StatusCategory::*;
        
        let mut valid_statuses = Vec::new();
        
        match current_status.category() {
            Todo => {
                valid_statuses.push(TaskStatus::in_progress(system.clone()));
                valid_statuses.push(TaskStatus::blocked(system.clone()));
                valid_statuses.push(TaskStatus::done(system.clone()));
            }
            InProgress => {
                valid_statuses.push(TaskStatus::todo(system.clone()));
                valid_statuses.push(TaskStatus::done(system.clone()));
                valid_statuses.push(TaskStatus::blocked(system.clone()));
            }
            Review => {
                valid_statuses.push(TaskStatus::in_progress(system.clone()));
                valid_statuses.push(TaskStatus::done(system.clone()));
            }
            Done => {
                valid_statuses.push(TaskStatus::todo(system.clone()));
                valid_statuses.push(TaskStatus::in_progress(system.clone()));
            }
            Blocked => {
                valid_statuses.push(TaskStatus::todo(system.clone()));
                valid_statuses.push(TaskStatus::in_progress(system.clone()));
            }
            Cancelled => {
                valid_statuses.push(TaskStatus::todo(system.clone()));
            }
        }
        
        valid_statuses
    }
    
    /// Calculates task priority based on various factors
    pub fn calculate_priority(
        due_date: Option<DateTime<Utc>>,
        assignee_workload: usize,
        dependencies: &[TaskDependency],
        labels: &[String],
    ) -> TaskPriority {
        let mut score = 0;
        
        // Due date urgency
        if let Some(due_date) = due_date {
            let days_until_due = (due_date - Utc::now()).num_days();
            match days_until_due {
                ..=0 => score += 6, // Overdue - Critical
                1..=3 => score += 5, // Due soon - Highest
                4..=7 => score += 4, // Due this week - High
                8..=14 => score += 3, // Due next week - Medium
                _ => score += 2, // Due later - Low
            }
        }
        
        // Assignee workload factor
        match assignee_workload {
            0..=5 => score += 1,
            6..=10 => score += 0,
            _ => score -= 1, // High workload reduces priority
        }
        
        // Dependencies factor
        if !dependencies.is_empty() {
            score += 1; // Tasks with dependencies are slightly more important
        }
        
        // Label-based priority
        for label in labels {
            match label.to_lowercase().as_str() {
                "urgent" | "critical" | "hotfix" => score += 3,
                "important" | "high-priority" => score += 2,
                "nice-to-have" | "low-priority" => score -= 1,
                _ => {}
            }
        }
        
        // Convert score to priority
        match score {
            ..=0 => TaskPriority::Lowest,
            1..=2 => TaskPriority::Low,
            3..=4 => TaskPriority::Medium,
            5..=6 => TaskPriority::High,
            7..=8 => TaskPriority::Highest,
            9.. => TaskPriority::Critical,
        }
    }
    
    /// Estimates task completion time based on historical data
    pub fn estimate_completion_time(
        task: &Task,
        historical_data: &[Task],
    ) -> Option<Duration> {
        // Filter similar tasks (same labels, similar complexity)
        let similar_tasks: Vec<&Task> = historical_data
            .iter()
            .filter(|t| {
                t.status.category().is_completed() 
                    && t.resolution_date.is_some()
                    && has_similar_characteristics(task, t)
            })
            .collect();
        
        if similar_tasks.is_empty() {
            return None;
        }
        
        // Calculate average completion time
        let total_duration: i64 = similar_tasks
            .iter()
            .filter_map(|t| {
                t.resolution_date.map(|end| (end - t.created_at).num_hours())
            })
            .sum();
        
        let average_hours = total_duration / similar_tasks.len() as i64;
        Some(Duration::hours(average_hours))
    }
}

/// Helper function to check if tasks have similar characteristics
fn has_similar_characteristics(task1: &Task, task2: &Task) -> bool {
    // Check for common labels
    let common_labels = task1.labels.iter()
        .any(|label| task2.labels.contains(label));
    
    // Check priority similarity
    let priority_similar = (task1.priority.numeric_value() as i32 - task2.priority.numeric_value() as i32).abs() <= 1;
    
    common_labels || priority_similar
}

/// Service for task dependency management
pub struct TaskDependencyService;

impl TaskDependencyService {
    /// Validates that adding a dependency won't create cycles
    pub async fn validate_dependency(
        task_repository: &dyn TaskRepositoryPort,
        from_task_id: &TaskId,
        to_task_id: &TaskId,
    ) -> Result<bool, TaskManagementDomainError> {
        // Get all tasks to build dependency graph
        let from_task = task_repository.get_task(from_task_id).await?
            .ok_or_else(|| TaskManagementDomainError::TaskNotFound {
                task_id: from_task_id.to_string(),
            })?;
        
        let to_task = task_repository.get_task(to_task_id).await?
            .ok_or_else(|| TaskManagementDomainError::TaskNotFound {
                task_id: to_task_id.to_string(),
            })?;
        
        // Simple cycle detection - in a real implementation, this would be more sophisticated
        let would_create_cycle = from_task.dependencies.iter().any(|dep| {
            dep.to_task == *from_task_id && dep.from_task == *to_task_id
        });
        
        Ok(!would_create_cycle)
    }
    
    /// Gets the dependency chain for a task
    pub async fn get_dependency_chain(
        task_repository: &dyn TaskRepositoryPort,
        task_id: &TaskId,
    ) -> Result<Vec<TaskId>, TaskManagementDomainError> {
        let task = task_repository.get_task(task_id).await?
            .ok_or_else(|| TaskManagementDomainError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;
        
        let dependencies: Vec<TaskId> = task.dependencies
            .iter()
            .filter(|dep| matches!(dep.dependency_type, crate::domains::tasks::value_objects::DependencyType::DependsOn))
            .map(|dep| dep.to_task.clone())
            .collect();
        
        Ok(dependencies)
    }
    
    /// Checks if a task is blocked by incomplete dependencies
    pub async fn is_blocked_by_dependencies(
        task_repository: &dyn TaskRepositoryPort,
        task_id: &TaskId,
    ) -> Result<bool, TaskManagementDomainError> {
        let dependencies = Self::get_dependency_chain(task_repository, task_id).await?;
        
        for dep_id in dependencies {
            if let Some(dep_task) = task_repository.get_task(&dep_id).await? {
                if !dep_task.status.category().is_completed() {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
}

/// Service for task time tracking and estimation
pub struct TaskTimeTrackingService;

impl TaskTimeTrackingService {
    /// Calculates burndown rate for a sprint
    pub fn calculate_burndown_rate(
        tasks: &[Task],
        sprint_start: DateTime<Utc>,
        sprint_end: DateTime<Utc>,
    ) -> f32 {
        let total_tasks = tasks.len() as f32;
        let completed_tasks = tasks.iter()
            .filter(|t| t.status.category().is_completed())
            .count() as f32;
        
        let sprint_duration = (sprint_end - sprint_start).num_days() as f32;
        let elapsed_days = (Utc::now() - sprint_start).num_days() as f32;
        
        if elapsed_days <= 0.0 {
            return 0.0;
        }
        
        let expected_completion_rate = elapsed_days / sprint_duration;
        let actual_completion_rate = completed_tasks / total_tasks;
        
        actual_completion_rate / expected_completion_rate
    }
    
    /// Predicts task completion date based on current progress
    pub fn predict_completion_date(task: &Task, similar_tasks: &[Task]) -> Option<DateTime<Utc>> {
        if let Some(estimated_duration) = TaskWorkflowService::estimate_completion_time(task, similar_tasks) {
            Some(task.created_at + estimated_duration)
        } else {
            None
        }
    }
    
    /// Calculates team velocity based on completed tasks
    pub fn calculate_team_velocity(
        completed_tasks: &[Task],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> f32 {
        let completed_in_period = completed_tasks.iter()
            .filter(|t| {
                t.resolution_date
                    .map(|date| date >= period_start && date <= period_end)
                    .unwrap_or(false)
            })
            .count();
        
        let period_days = (period_end - period_start).num_days() as f32;
        
        if period_days > 0.0 {
            completed_in_period as f32 / period_days
        } else {
            0.0
        }
    }
} 