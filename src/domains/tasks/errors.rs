//! Task management domain error types
//! 
//! Domain-specific errors that provide rich context and diagnostic information
//! for task management operations with external systems.

use miette::Diagnostic;
use thiserror::Error;

/// Task management domain-specific errors with rich diagnostic information
#[derive(Error, Diagnostic, Debug)]
pub enum TaskManagementDomainError {
    #[error("Invalid task identifier: {task_id}")]
    #[diagnostic(
        code(tasks::invalid_task_id),
        help("Task IDs must follow the pattern: PROJECT-123 for JIRA or numeric ID for Monday.com")
    )]
    InvalidTaskId { task_id: String },

    #[error("Task {task_id} not found")]
    #[diagnostic(
        code(tasks::task_not_found),
        help("Check that the task exists and you have permission to access it")
    )]
    TaskNotFound { task_id: String },

    #[error("Invalid task status transition from {from} to {to}")]
    #[diagnostic(
        code(tasks::invalid_status_transition),
        help("Check the allowed status transitions in your project workflow configuration")
    )]
    InvalidStatusTransition { from: String, to: String },

    #[error("Task assignment failed: {reason}")]
    #[diagnostic(
        code(tasks::assignment_failed),
        help("Ensure the assignee exists and has permission to be assigned to this task")
    )]
    TaskAssignmentFailed { reason: String },

    #[error("External system authentication failed: {system}")]
    #[diagnostic(
        code(tasks::auth_failed),
        help("Check your API credentials and ensure they have the required permissions")
    )]
    ExternalSystemAuthFailed { system: String },

    #[error("External system API error: {system} - {message}")]
    #[diagnostic(
        code(tasks::api_error),
        help("Check the external system status and your API rate limits")
    )]
    ExternalSystemApiError { system: String, message: String },

    #[error("Task synchronization failed: {reason}")]
    #[diagnostic(
        code(tasks::sync_failed),
        help("Check network connectivity and external system availability")
    )]
    SynchronizationFailed { reason: String },

    #[error("Invalid task template: {template_name}")]
    #[diagnostic(
        code(tasks::invalid_template),
        help("Check that the template exists and contains all required fields")
    )]
    InvalidTaskTemplate { template_name: String },

    #[error("Task field validation failed: {field} - {reason}")]
    #[diagnostic(
        code(tasks::field_validation_failed),
        help("Ensure the field value meets the requirements defined in the task schema")
    )]
    FieldValidationFailed { field: String, reason: String },

    #[error("Workflow step execution failed: {step}")]
    #[diagnostic(
        code(tasks::workflow_step_failed),
        help("Check the step configuration and ensure all prerequisites are met")
    )]
    WorkflowStepFailed { step: String },

    #[error("Task automation rule failed: {rule_name}")]
    #[diagnostic(
        code(tasks::automation_failed),
        help("Review the automation rule configuration and trigger conditions")
    )]
    AutomationRuleFailed { rule_name: String },

    #[error("Bulk operation failed: {operation} - {failed_count} of {total_count} tasks")]
    #[diagnostic(
        code(tasks::bulk_operation_failed),
        help("Check the individual task errors and retry the failed operations")
    )]
    BulkOperationFailed {
        operation: String,
        failed_count: usize,
        total_count: usize,
    },

    #[error("Task dependency cycle detected: {tasks}")]
    #[diagnostic(
        code(tasks::dependency_cycle),
        help("Remove circular dependencies between tasks to resolve this issue")
    )]
    DependencyCycleDetected { tasks: String },

    #[error("Project configuration invalid: {reason}")]
    #[diagnostic(
        code(tasks::invalid_project_config),
        help("Check your project configuration file and ensure all required settings are present")
    )]
    InvalidProjectConfiguration { reason: String },

    #[error("Task comment operation failed: {reason}")]
    #[diagnostic(
        code(tasks::comment_failed),
        help("Ensure you have permission to add comments to this task")
    )]
    CommentOperationFailed { reason: String },

    #[error("Time tracking operation failed: {reason}")]
    #[diagnostic(
        code(tasks::time_tracking_failed),
        help("Check that time tracking is enabled for this project and task type")
    )]
    TimeTrackingFailed { reason: String },
} 