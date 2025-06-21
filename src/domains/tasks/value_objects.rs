//! Task management domain value objects
//! 
//! Immutable value objects that encapsulate task management data with validation
//! and ensure business rules are maintained across different external systems.

use crate::domains::tasks::errors::TaskManagementDomainError;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

/// Represents a task identifier with validation for different systems
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId {
    id: String,
    system: TaskSystem,
}

impl TaskId {
    /// Creates a new task ID with validation
    pub fn new(id: String, system: TaskSystem) -> Result<Self, TaskManagementDomainError> {
        Self::validate_id_format(&id, &system)?;
        Ok(Self { id, system })
    }
    
    /// Creates a JIRA task ID
    pub fn jira(id: String) -> Result<Self, TaskManagementDomainError> {
        Self::new(id, TaskSystem::Jira)
    }
    
    /// Creates a Monday.com task ID
    pub fn monday(id: String) -> Result<Self, TaskManagementDomainError> {
        Self::new(id, TaskSystem::Monday)
    }
    
    /// Creates a generic task ID
    pub fn generic(id: String) -> Result<Self, TaskManagementDomainError> {
        Self::new(id, TaskSystem::Generic)
    }
    
    /// Gets the ID string
    pub fn as_str(&self) -> &str {
        &self.id
    }
    
    /// Gets the task system
    pub fn system(&self) -> &TaskSystem {
        &self.system
    }
    
    /// Validates ID format based on system
    fn validate_id_format(id: &str, system: &TaskSystem) -> Result<(), TaskManagementDomainError> {
        match system {
            TaskSystem::Jira => {
                // JIRA format: PROJECT-123
                if !id.contains('-') || id.split('-').count() != 2 {
                    return Err(TaskManagementDomainError::InvalidTaskId {
                        task_id: id.to_string(),
                    });
                }
                let parts: Vec<&str> = id.split('-').collect();
                if parts[0].is_empty() || !parts[1].chars().all(|c| c.is_ascii_digit()) {
                    return Err(TaskManagementDomainError::InvalidTaskId {
                        task_id: id.to_string(),
                    });
                }
            }
            TaskSystem::Monday => {
                // Monday.com format: numeric ID
                if !id.chars().all(|c| c.is_ascii_digit()) || id.is_empty() {
                    return Err(TaskManagementDomainError::InvalidTaskId {
                        task_id: id.to_string(),
                    });
                }
            }
            TaskSystem::Generic => {
                // Generic: just check it's not empty
                if id.is_empty() {
                    return Err(TaskManagementDomainError::InvalidTaskId {
                        task_id: id.to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Represents the external task management system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskSystem {
    Jira,
    Monday,
    Generic,
}

impl TaskSystem {
    /// Gets the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Jira => "JIRA",
            Self::Monday => "Monday.com",
            Self::Generic => "Generic",
        }
    }
    
    /// Checks if the system supports time tracking
    pub fn supports_time_tracking(&self) -> bool {
        matches!(self, Self::Jira | Self::Monday)
    }
    
    /// Checks if the system supports custom fields
    pub fn supports_custom_fields(&self) -> bool {
        matches!(self, Self::Jira | Self::Monday)
    }
}

/// Represents a task status with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskStatus {
    name: String,
    category: StatusCategory,
    system: TaskSystem,
}

impl TaskStatus {
    /// Creates a new task status
    pub fn new(name: String, category: StatusCategory, system: TaskSystem) -> Self {
        Self { name, category, system }
    }
    
    /// Creates common statuses
    pub fn todo(system: TaskSystem) -> Self {
        Self::new("To Do".to_string(), StatusCategory::Todo, system)
    }
    
    pub fn in_progress(system: TaskSystem) -> Self {
        Self::new("In Progress".to_string(), StatusCategory::InProgress, system)
    }
    
    pub fn done(system: TaskSystem) -> Self {
        Self::new("Done".to_string(), StatusCategory::Done, system)
    }
    
    pub fn blocked(system: TaskSystem) -> Self {
        Self::new("Blocked".to_string(), StatusCategory::Blocked, system)
    }
    
    /// Gets the status name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Gets the status category
    pub fn category(&self) -> &StatusCategory {
        &self.category
    }
    
    /// Gets the task system
    pub fn system(&self) -> &TaskSystem {
        &self.system
    }
    
    /// Checks if transition to another status is valid
    pub fn can_transition_to(&self, other: &TaskStatus) -> bool {
        use StatusCategory::*;
        
        // Same system check
        if self.system != other.system {
            return false;
        }
        
        // Basic transition rules
        match (&self.category, &other.category) {
            // Can always transition to blocked
            (_, Blocked) => true,
            // Can unblock to previous state categories
            (Blocked, Todo | InProgress | Review) => true,
            // Normal progression
            (Todo, InProgress | Review | Done) => true,
            (InProgress, Review | Done | Todo) => true,
            (Review, Done | InProgress) => true,
            // Can reopen from done
            (Done, Todo | InProgress) => true,
            // Same status is allowed
            (a, b) if a == b => true,
            _ => false,
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Categories of task statuses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusCategory {
    Todo,
    InProgress,
    Review,
    Done,
    Blocked,
    Cancelled,
}

impl StatusCategory {
    /// Checks if the status represents an active state
    pub fn is_active(&self) -> bool {
        matches!(self, Self::InProgress | Self::Review)
    }
    
    /// Checks if the status represents a completed state
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }
    
    /// Checks if the status represents a blocked state
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked)
    }
}

/// Represents task priority with validation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Lowest,
    Low,
    Medium,
    High,
    Highest,
    Critical,
}

impl TaskPriority {
    /// Gets the numeric value for sorting
    pub fn numeric_value(&self) -> u8 {
        match self {
            Self::Lowest => 1,
            Self::Low => 2,
            Self::Medium => 3,
            Self::High => 4,
            Self::Highest => 5,
            Self::Critical => 6,
        }
    }
    
    /// Gets the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Lowest => "Lowest",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Highest => "Highest",
            Self::Critical => "Critical",
        }
    }
    
    /// Gets the color associated with the priority
    pub fn color(&self) -> &'static str {
        match self {
            Self::Lowest => "gray",
            Self::Low => "blue",
            Self::Medium => "yellow",
            Self::High => "orange",
            Self::Highest => "red",
            Self::Critical => "purple",
        }
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Represents a task assignee with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskAssignee {
    user_id: String,
    display_name: String,
    email: Option<String>,
    system: TaskSystem,
}

impl TaskAssignee {
    /// Creates a new task assignee
    pub fn new(
        user_id: String,
        display_name: String,
        email: Option<String>,
        system: TaskSystem,
    ) -> Result<Self, TaskManagementDomainError> {
        if user_id.is_empty() || display_name.is_empty() {
            return Err(TaskManagementDomainError::FieldValidationFailed {
                field: "assignee".to_string(),
                reason: "User ID and display name cannot be empty".to_string(),
            });
        }
        
        // Validate email if provided
        if let Some(ref email) = email {
            if !email.contains('@') {
                return Err(TaskManagementDomainError::FieldValidationFailed {
                    field: "email".to_string(),
                    reason: "Invalid email format".to_string(),
                });
            }
        }
        
        Ok(Self {
            user_id,
            display_name,
            email,
            system,
        })
    }
    
    /// Gets the user ID
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    
    /// Gets the display name
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    
    /// Gets the email
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }
    
    /// Gets the system
    pub fn system(&self) -> &TaskSystem {
        &self.system
    }
}

impl fmt::Display for TaskAssignee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

/// Represents time tracking information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeTracking {
    pub original_estimate: Option<Duration>,
    pub remaining_estimate: Option<Duration>,
    pub time_spent: Duration,
    pub work_logs: Vec<WorkLog>,
}

impl TimeTracking {
    /// Creates new time tracking
    pub fn new() -> Self {
        Self {
            original_estimate: None,
            remaining_estimate: None,
            time_spent: Duration::zero(),
            work_logs: Vec::new(),
        }
    }
    
    /// Sets the original estimate
    pub fn with_original_estimate(mut self, estimate: Duration) -> Self {
        self.original_estimate = Some(estimate);
        self.remaining_estimate = Some(estimate);
        self
    }
    
    /// Adds a work log entry
    pub fn add_work_log(&mut self, work_log: WorkLog) {
        self.time_spent = self.time_spent + work_log.duration;
        
        // Update remaining estimate
        if let Some(remaining) = self.remaining_estimate {
            self.remaining_estimate = Some((remaining - work_log.duration).max(Duration::zero()));
        }
        
        self.work_logs.push(work_log);
    }
    
    /// Gets the completion percentage
    pub fn completion_percentage(&self) -> Option<f32> {
        if let Some(original) = self.original_estimate {
            if original.num_seconds() > 0 {
                let spent_seconds = self.time_spent.num_seconds() as f32;
                let original_seconds = original.num_seconds() as f32;
                Some((spent_seconds / original_seconds * 100.0).min(100.0))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for TimeTracking {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a work log entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkLog {
    pub duration: Duration,
    pub description: Option<String>,
    pub logged_by: String,
    pub logged_at: DateTime<Utc>,
}

impl WorkLog {
    /// Creates a new work log entry
    pub fn new(duration: Duration, logged_by: String) -> Self {
        Self {
            duration,
            description: None,
            logged_by,
            logged_at: Utc::now(),
        }
    }
    
    /// Adds a description to the work log
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Represents a task comment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskComment {
    pub id: String,
    pub content: String,
    pub author: TaskAssignee,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_internal: bool,
}

impl TaskComment {
    /// Creates a new comment
    pub fn new(id: String, content: String, author: TaskAssignee) -> Self {
        Self {
            id,
            content,
            author,
            created_at: Utc::now(),
            updated_at: None,
            is_internal: false,
        }
    }
    
    /// Marks the comment as internal
    pub fn as_internal(mut self) -> Self {
        self.is_internal = true;
        self
    }
    
    /// Updates the comment content
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.updated_at = Some(Utc::now());
    }
}

/// Represents task dependency relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskDependency {
    pub from_task: TaskId,
    pub to_task: TaskId,
    pub dependency_type: DependencyType,
}

impl TaskDependency {
    /// Creates a new task dependency
    pub fn new(from_task: TaskId, to_task: TaskId, dependency_type: DependencyType) -> Self {
        Self {
            from_task,
            to_task,
            dependency_type,
        }
    }
    
    /// Creates a "blocks" dependency
    pub fn blocks(blocking_task: TaskId, blocked_task: TaskId) -> Self {
        Self::new(blocking_task, blocked_task, DependencyType::Blocks)
    }
    
    /// Creates a "depends on" dependency
    pub fn depends_on(dependent_task: TaskId, dependency_task: TaskId) -> Self {
        Self::new(dependent_task, dependency_task, DependencyType::DependsOn)
    }
}

/// Types of task dependencies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    Blocks,
    DependsOn,
    Related,
    Duplicate,
    Subtask,
}

impl DependencyType {
    /// Gets the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Blocks => "Blocks",
            Self::DependsOn => "Depends On",
            Self::Related => "Related",
            Self::Duplicate => "Duplicate",
            Self::Subtask => "Subtask",
        }
    }
}

/// Represents external system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSystemConfig {
    pub system: TaskSystem,
    pub base_url: Url,
    pub project_key: Option<String>,
    pub board_id: Option<String>,
    pub default_assignee: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
    pub sync_enabled: bool,
    pub sync_interval_minutes: u32,
}

impl ExternalSystemConfig {
    /// Creates a new external system configuration
    pub fn new(system: TaskSystem, base_url: Url) -> Self {
        Self {
            system,
            base_url,
            project_key: None,
            board_id: None,
            default_assignee: None,
            custom_fields: std::collections::HashMap::new(),
            sync_enabled: true,
            sync_interval_minutes: 15,
        }
    }
    
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), TaskManagementDomainError> {
        match self.system {
            TaskSystem::Jira => {
                if self.project_key.is_none() {
                    return Err(TaskManagementDomainError::InvalidProjectConfiguration {
                        reason: "JIRA configuration requires a project key".to_string(),
                    });
                }
            }
            TaskSystem::Monday => {
                if self.board_id.is_none() {
                    return Err(TaskManagementDomainError::InvalidProjectConfiguration {
                        reason: "Monday.com configuration requires a board ID".to_string(),
                    });
                }
            }
            TaskSystem::Generic => {
                // No specific validation for generic systems
            }
        }
        
        if self.sync_interval_minutes == 0 {
            return Err(TaskManagementDomainError::InvalidProjectConfiguration {
                reason: "Sync interval must be greater than 0".to_string(),
            });
        }
        
        Ok(())
    }
} 