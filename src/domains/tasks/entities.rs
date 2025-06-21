//! Task management domain entities
//! 
//! Rich domain entities that encapsulate task management state and behavior.
//! These entities contain business logic and enforce task management rules.

use crate::domains::tasks::{
    errors::TaskManagementDomainError,
    value_objects::{
        TaskId, TaskStatus, TaskPriority, TaskAssignee, TimeTracking, TaskComment,
        TaskDependency, ExternalSystemConfig, TaskSystem, StatusCategory
    }
};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a task with rich domain behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee: Option<TaskAssignee>,
    pub reporter: Option<TaskAssignee>,
    pub labels: Vec<String>,
    pub time_tracking: TimeTracking,
    pub comments: Vec<TaskComment>,
    pub dependencies: Vec<TaskDependency>,
    pub custom_fields: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub resolution_date: Option<DateTime<Utc>>,
    pub external_url: Option<String>,
}

impl Task {
    /// Creates a new task
    pub fn new(id: TaskId, title: String, status: TaskStatus) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description: None,
            status,
            priority: TaskPriority::default(),
            assignee: None,
            reporter: None,
            labels: Vec::new(),
            time_tracking: TimeTracking::new(),
            comments: Vec::new(),
            dependencies: Vec::new(),
            custom_fields: HashMap::new(),
            created_at: now,
            updated_at: now,
            due_date: None,
            resolution_date: None,
            external_url: None,
        }
    }
    
    /// Updates the task status with validation
    pub fn update_status(&mut self, new_status: TaskStatus) -> Result<(), TaskManagementDomainError> {
        if !self.status.can_transition_to(&new_status) {
            return Err(TaskManagementDomainError::InvalidStatusTransition {
                from: self.status.name().to_string(),
                to: new_status.name().to_string(),
            });
        }
        
        // Set resolution date when moving to done
        if new_status.category().is_completed() && self.resolution_date.is_none() {
            self.resolution_date = Some(Utc::now());
        }
        
        // Clear resolution date when reopening
        if !new_status.category().is_completed() && self.resolution_date.is_some() {
            self.resolution_date = None;
        }
        
        self.status = new_status;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Assigns the task to a user
    pub fn assign_to(&mut self, assignee: TaskAssignee) -> Result<(), TaskManagementDomainError> {
        // Validate that assignee is from the same system
        if assignee.system() != self.id.system() {
            return Err(TaskManagementDomainError::TaskAssignmentFailed {
                reason: "Assignee must be from the same external system as the task".to_string(),
            });
        }
        
        self.assignee = Some(assignee);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Removes the current assignee
    pub fn unassign(&mut self) {
        self.assignee = None;
        self.updated_at = Utc::now();
    }
    
    /// Updates the task priority
    pub fn set_priority(&mut self, priority: TaskPriority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }
    
    /// Sets the due date
    pub fn set_due_date(&mut self, due_date: Option<DateTime<Utc>>) {
        self.due_date = due_date;
        self.updated_at = Utc::now();
    }
    
    /// Adds a comment to the task
    pub fn add_comment(&mut self, comment: TaskComment) -> Result<(), TaskManagementDomainError> {
        // Validate that comment author is from the same system
        if comment.author.system() != self.id.system() {
            return Err(TaskManagementDomainError::CommentOperationFailed {
                reason: "Comment author must be from the same external system as the task".to_string(),
            });
        }
        
        self.comments.push(comment);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Adds a dependency
    pub fn add_dependency(&mut self, dependency: TaskDependency) -> Result<(), TaskManagementDomainError> {
        // Check for circular dependencies
        if self.would_create_cycle(&dependency)? {
            return Err(TaskManagementDomainError::DependencyCycleDetected {
                tasks: format!("{} -> {}", dependency.from_task, dependency.to_task),
            });
        }
        
        self.dependencies.push(dependency);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Checks if adding a dependency would create a cycle
    fn would_create_cycle(&self, new_dependency: &TaskDependency) -> Result<bool, TaskManagementDomainError> {
        // Simple cycle detection: check if the new dependency would create a direct cycle
        // In a real implementation, this would traverse the entire dependency graph
        let creates_cycle = self.dependencies.iter().any(|dep| {
            dep.to_task == new_dependency.from_task && dep.from_task == new_dependency.to_task
        });
        
        Ok(creates_cycle)
    }
    
    /// Adds a label
    pub fn add_label(&mut self, label: String) {
        if !self.labels.contains(&label) {
            self.labels.push(label);
            self.updated_at = Utc::now();
        }
    }
    
    /// Removes a label
    pub fn remove_label(&mut self, label: &str) {
        if let Some(pos) = self.labels.iter().position(|l| l == label) {
            self.labels.remove(pos);
            self.updated_at = Utc::now();
        }
    }
    
    /// Sets a custom field
    pub fn set_custom_field(&mut self, key: String, value: String) {
        self.custom_fields.insert(key, value);
        self.updated_at = Utc::now();
    }
    
    /// Gets a custom field value
    pub fn get_custom_field(&self, key: &str) -> Option<&String> {
        self.custom_fields.get(key)
    }
    
    /// Checks if the task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            !self.status.category().is_completed() && Utc::now() > due_date
        } else {
            false
        }
    }
    
    /// Checks if the task is blocked by dependencies
    pub fn is_blocked_by_dependencies(&self) -> bool {
        // This would need to be implemented with access to other tasks
        // For now, just check if there are blocking dependencies
        self.dependencies.iter().any(|dep| {
            matches!(dep.dependency_type, crate::domains::tasks::value_objects::DependencyType::DependsOn)
        })
    }
    
    /// Gets the task age in days
    pub fn age_in_days(&self) -> i64 {
        (Utc::now() - self.created_at).num_days()
    }
    
    /// Gets completion percentage based on time tracking
    pub fn completion_percentage(&self) -> Option<f32> {
        self.time_tracking.completion_percentage()
    }
    
    /// Validates the task state
    pub fn validate(&self) -> Result<(), TaskManagementDomainError> {
        if self.title.trim().is_empty() {
            return Err(TaskManagementDomainError::FieldValidationFailed {
                field: "title".to_string(),
                reason: "Task title cannot be empty".to_string(),
            });
        }
        
        // Validate that all dependencies are valid
        for dependency in &self.dependencies {
            if dependency.from_task == dependency.to_task {
                return Err(TaskManagementDomainError::DependencyCycleDetected {
                    tasks: format!("Self-dependency: {}", dependency.from_task),
                });
            }
        }
        
        Ok(())
    }
}

/// Represents a task template for creating standardized tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub name: String,
    pub description: String,
    pub title_template: String,
    pub description_template: Option<String>,
    pub default_status: TaskStatus,
    pub default_priority: TaskPriority,
    pub default_labels: Vec<String>,
    pub required_fields: Vec<String>,
    pub custom_field_templates: HashMap<String, String>,
    pub system: TaskSystem,
}

impl TaskTemplate {
    /// Creates a new task template
    pub fn new(
        name: String,
        title_template: String,
        default_status: TaskStatus,
        system: TaskSystem,
    ) -> Self {
        Self {
            name,
            description: String::new(),
            title_template,
            description_template: None,
            default_status,
            default_priority: TaskPriority::default(),
            default_labels: Vec::new(),
            required_fields: Vec::new(),
            custom_field_templates: HashMap::new(),
            system,
        }
    }
    
    /// Creates a task from this template
    pub fn create_task(
        &self,
        id: TaskId,
        context: &HashMap<String, String>,
    ) -> Result<Task, TaskManagementDomainError> {
        // Validate required fields are provided
        for field in &self.required_fields {
            if !context.contains_key(field) {
                return Err(TaskManagementDomainError::FieldValidationFailed {
                    field: field.clone(),
                    reason: "Required field not provided in context".to_string(),
                });
            }
        }
        
        // Render title template
        let title = self.render_template(&self.title_template, context)?;
        
        // Create the task
        let mut task = Task::new(id, title, self.default_status.clone());
        
        // Set description if template exists
        if let Some(ref desc_template) = self.description_template {
            let description = self.render_template(desc_template, context)?;
            task.description = Some(description);
        }
        
        // Set default values
        task.priority = self.default_priority.clone();
        task.labels = self.default_labels.clone();
        
        // Set custom fields from templates
        for (key, template) in &self.custom_field_templates {
            let value = self.render_template(template, context)?;
            task.set_custom_field(key.clone(), value);
        }
        
        task.validate()?;
        
        Ok(task)
    }
    
    /// Simple template rendering (placeholder implementation)
    fn render_template(
        &self,
        template: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, TaskManagementDomainError> {
        let mut result = template.to_string();
        
        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        // Check if any unreplaced placeholders remain
        if result.contains('{') && result.contains('}') {
            return Err(TaskManagementDomainError::InvalidTaskTemplate {
                template_name: self.name.clone(),
            });
        }
        
        Ok(result)
    }
    
    /// Validates the template
    pub fn validate(&self) -> Result<(), TaskManagementDomainError> {
        if self.name.trim().is_empty() {
            return Err(TaskManagementDomainError::InvalidTaskTemplate {
                template_name: "empty name".to_string(),
            });
        }
        
        if self.title_template.trim().is_empty() {
            return Err(TaskManagementDomainError::InvalidTaskTemplate {
                template_name: self.name.clone(),
            });
        }
        
        Ok(())
    }
}

/// Represents a task automation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAutomationRule {
    pub name: String,
    pub description: String,
    pub trigger: AutomationTrigger,
    pub conditions: Vec<AutomationCondition>,
    pub actions: Vec<AutomationAction>,
    pub enabled: bool,
    pub system: TaskSystem,
}

impl TaskAutomationRule {
    /// Creates a new automation rule
    pub fn new(
        name: String,
        trigger: AutomationTrigger,
        system: TaskSystem,
    ) -> Self {
        Self {
            name,
            description: String::new(),
            trigger,
            conditions: Vec::new(),
            actions: Vec::new(),
            enabled: true,
            system,
        }
    }
    
    /// Checks if the rule should be triggered for a task event
    pub fn should_trigger(&self, event: &TaskEvent) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check if trigger matches
        if !self.trigger.matches(event) {
            return false;
        }
        
        // Check all conditions
        self.conditions.iter().all(|condition| condition.evaluate(event))
    }
    
    /// Executes the rule actions
    pub fn execute(&self, task: &mut Task) -> Result<Vec<String>, TaskManagementDomainError> {
        let mut executed_actions = Vec::new();
        
        for action in &self.actions {
            let result = action.execute(task)?;
            executed_actions.push(result);
        }
        
        Ok(executed_actions)
    }
}

/// Represents automation triggers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationTrigger {
    TaskCreated,
    TaskUpdated,
    StatusChanged,
    AssigneeChanged,
    PriorityChanged,
    CommentAdded,
    DueDateApproaching,
}

impl AutomationTrigger {
    /// Checks if the trigger matches a task event
    pub fn matches(&self, event: &TaskEvent) -> bool {
        match (self, event) {
            (Self::TaskCreated, TaskEvent::Created(_)) => true,
            (Self::TaskUpdated, TaskEvent::Updated(_)) => true,
            (Self::StatusChanged, TaskEvent::StatusChanged(_, _)) => true,
            (Self::AssigneeChanged, TaskEvent::AssigneeChanged(_, _)) => true,
            (Self::PriorityChanged, TaskEvent::PriorityChanged(_, _)) => true,
            (Self::CommentAdded, TaskEvent::CommentAdded(_)) => true,
            _ => false,
        }
    }
}

/// Represents automation conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationCondition {
    StatusEquals(String),
    PriorityEquals(TaskPriority),
    HasLabel(String),
    AssigneeEquals(String),
    FieldEquals(String, String),
}

impl AutomationCondition {
    /// Evaluates the condition against a task event
    pub fn evaluate(&self, event: &TaskEvent) -> bool {
        let task = event.task();
        
        match self {
            Self::StatusEquals(status) => task.status.name() == status,
            Self::PriorityEquals(priority) => &task.priority == priority,
            Self::HasLabel(label) => task.labels.contains(label),
            Self::AssigneeEquals(assignee) => {
                task.assignee.as_ref()
                    .map(|a| a.display_name() == assignee)
                    .unwrap_or(false)
            }
            Self::FieldEquals(field, value) => {
                task.custom_fields.get(field)
                    .map(|v| v == value)
                    .unwrap_or(false)
            }
        }
    }
}

/// Represents automation actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationAction {
    SetStatus(String),
    SetPriority(TaskPriority),
    AddLabel(String),
    RemoveLabel(String),
    SetAssignee(String),
    AddComment(String),
    SetCustomField(String, String),
}

impl AutomationAction {
    /// Executes the action on a task
    pub fn execute(&self, task: &mut Task) -> Result<String, TaskManagementDomainError> {
        match self {
            Self::SetStatus(status_name) => {
                let new_status = TaskStatus::new(
                    status_name.clone(),
                    StatusCategory::Todo, // This would need proper mapping
                    task.id.system().clone(),
                );
                task.update_status(new_status)?;
                Ok(format!("Set status to {}", status_name))
            }
            Self::SetPriority(priority) => {
                task.set_priority(priority.clone());
                Ok(format!("Set priority to {}", priority.display_name()))
            }
            Self::AddLabel(label) => {
                task.add_label(label.clone());
                Ok(format!("Added label: {}", label))
            }
            Self::RemoveLabel(label) => {
                task.remove_label(label);
                Ok(format!("Removed label: {}", label))
            }
            Self::SetAssignee(_assignee) => {
                // This would need proper assignee resolution
                Ok("Set assignee".to_string())
            }
            Self::AddComment(content) => {
                // This would need proper comment creation with author
                Ok(format!("Added comment: {}", content))
            }
            Self::SetCustomField(field, value) => {
                task.set_custom_field(field.clone(), value.clone());
                Ok(format!("Set {} to {}", field, value))
            }
        }
    }
}

/// Represents task events for automation
#[derive(Debug, Clone)]
pub enum TaskEvent {
    Created(Task),
    Updated(Task),
    StatusChanged(Task, TaskStatus),
    AssigneeChanged(Task, Option<TaskAssignee>),
    PriorityChanged(Task, TaskPriority),
    CommentAdded(Task),
}

impl TaskEvent {
    /// Gets the task from the event
    pub fn task(&self) -> &Task {
        match self {
            Self::Created(task) 
            | Self::Updated(task)
            | Self::StatusChanged(task, _)
            | Self::AssigneeChanged(task, _)
            | Self::PriorityChanged(task, _)
            | Self::CommentAdded(task) => task,
        }
    }
}

/// Represents a bulk operation on tasks
#[derive(Debug, Clone)]
pub struct BulkTaskOperation {
    pub operation_type: BulkOperationType,
    pub task_ids: Vec<TaskId>,
    pub parameters: HashMap<String, String>,
    pub results: Vec<BulkOperationResult>,
}

impl BulkTaskOperation {
    /// Creates a new bulk operation
    pub fn new(operation_type: BulkOperationType, task_ids: Vec<TaskId>) -> Self {
        Self {
            operation_type,
            task_ids,
            parameters: HashMap::new(),
            results: Vec::new(),
        }
    }
    
    /// Adds a parameter to the operation
    pub fn with_parameter(mut self, key: String, value: String) -> Self {
        self.parameters.insert(key, value);
        self
    }
    
    /// Records the result of an operation on a task
    pub fn record_result(&mut self, task_id: TaskId, success: bool, message: String) {
        self.results.push(BulkOperationResult {
            task_id,
            success,
            message,
        });
    }
    
    /// Gets the number of successful operations
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }
    
    /// Gets the number of failed operations
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }
}

/// Types of bulk operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulkOperationType {
    UpdateStatus,
    UpdateAssignee,
    UpdatePriority,
    AddLabel,
    RemoveLabel,
    Delete,
}

/// Result of a bulk operation on a single task
#[derive(Debug, Clone)]
pub struct BulkOperationResult {
    pub task_id: TaskId,
    pub success: bool,
    pub message: String,
} 