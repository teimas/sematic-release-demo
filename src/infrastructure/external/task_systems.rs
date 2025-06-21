//! Task System Integrations
//! 
//! This module provides integrations with external task management systems
//! like JIRA, Monday.com, Azure DevOps, etc.

#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use reqwest::Client;
#[cfg(feature = "new-domains")]
use serde_json::Value;

#[cfg(feature = "new-domains")]
use crate::domains::tasks::{
    repository::TaskSynchronizationPort,
    entities::Task,
    value_objects::{TaskId, TaskStatus, TaskPriority, ExternalSystemConfig, TaskSystem, TimeTracking},
    errors::TaskManagementDomainError,
};

/// JIRA adapter for task management
#[cfg(feature = "new-domains")]
pub struct JiraAdapter {
    base_url: String,
    username: String,
    api_token: String,
    client: Client,
}

#[cfg(feature = "new-domains")]
impl JiraAdapter {
    pub fn new(base_url: String, username: String, api_token: String) -> Self {
        Self {
            base_url,
            username,
            api_token,
            client: Client::new(),
        }
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl TaskSynchronizationPort for JiraAdapter {
    async fn sync_task_to_external(
        &self,
        task: &Task,
        _config: &ExternalSystemConfig,
    ) -> Result<Task, TaskManagementDomainError> {
        let issue_data = serde_json::json!({
            "fields": {
                "project": {"key": "PROJ"},
                "summary": task.title,
                "description": task.description,
                "issuetype": {"name": "Task"}
            }
        });

        let response = self
            .client
            .post(&format!("{}/rest/api/3/issue", self.base_url))
            .basic_auth(&self.username, Some(&self.api_token))
            .json(&issue_data)
            .send()
            .await
            .map_err(|e| TaskManagementDomainError::ExternalSystemApiError {
                system: "JIRA".to_string(),
                message: format!("Failed to create issue: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(TaskManagementDomainError::ExternalSystemApiError {
                system: "JIRA".to_string(),
                message: format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default()),
            });
        }

        // Return the task (in practice, would update with external ID)
        Ok(task.clone())
    }

    async fn fetch_task_from_external(
        &self,
        task_id: &TaskId,
        _config: &ExternalSystemConfig,
    ) -> Result<Option<Task>, TaskManagementDomainError> {
        let response = self
            .client
            .get(&format!("{}/rest/api/3/issue/{}", self.base_url, task_id.as_str()))
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await
            .map_err(|e| TaskManagementDomainError::ExternalSystemApiError {
                system: "JIRA".to_string(),
                message: format!("Failed to get issue: {}", e),
            })?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(TaskManagementDomainError::ExternalSystemApiError {
                system: "JIRA".to_string(),
                message: format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default()),
            });
        }

        let issue: Value = response
            .json()
            .await
            .map_err(|e| TaskManagementDomainError::ExternalSystemApiError {
                system: "JIRA".to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        // Convert JIRA issue to Task
        let task_id = TaskId::new(
            issue.get("key").and_then(|k| k.as_str()).unwrap_or("").to_string(),
            TaskSystem::Jira
        )?;
        
        let task = Task {
            id: task_id,
            title: issue
                .get("fields")
                .and_then(|f| f.get("summary"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string(),
            description: issue
                .get("fields")
                .and_then(|f| f.get("description"))
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
            status: TaskStatus::in_progress(TaskSystem::Jira), 
            priority: TaskPriority::Medium,
            assignee: None,
            reporter: None,
            labels: vec![],
            time_tracking: TimeTracking::new(),
            comments: vec![],
            dependencies: vec![],
            custom_fields: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            resolution_date: None,
            external_url: None,
        };

        Ok(Some(task))
    }

    async fn sync_all_tasks(
        &self,
        _system: &TaskSystem,
        _config: &ExternalSystemConfig,
    ) -> Result<crate::domains::tasks::repository::SyncResult, TaskManagementDomainError> {
        // Implementation would sync all tasks
        Ok(crate::domains::tasks::repository::SyncResult {
            tasks_created: 0,
            tasks_updated: 0,
            tasks_deleted: 0,
            errors: vec![],
            duration: std::time::Duration::from_millis(100),
        })
    }

    async fn get_sync_status(
        &self,
        system: &TaskSystem,
    ) -> Result<crate::domains::tasks::repository::SyncStatus, TaskManagementDomainError> {
        Ok(crate::domains::tasks::repository::SyncStatus {
            system: system.clone(),
            last_sync: None,
            is_syncing: false,
            sync_errors: vec![],
            next_sync: None,
        })
    }

    async fn force_full_sync(
        &self,
        system: &TaskSystem,
        config: &ExternalSystemConfig,
    ) -> Result<crate::domains::tasks::repository::SyncResult, TaskManagementDomainError> {
        self.sync_all_tasks(system, config).await
    }
}

/// Mock task system adapter for testing
#[cfg(feature = "new-domains")]
pub struct MockTaskSystemAdapter;

#[cfg(feature = "new-domains")]
impl MockTaskSystemAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl TaskSynchronizationPort for MockTaskSystemAdapter {
    async fn sync_task_to_external(
        &self,
        task: &Task,
        _config: &ExternalSystemConfig,
    ) -> Result<Task, TaskManagementDomainError> {
        Ok(task.clone())
    }

    async fn fetch_task_from_external(
        &self,
        task_id: &TaskId,
        _config: &ExternalSystemConfig,
    ) -> Result<Option<Task>, TaskManagementDomainError> {
        if task_id.as_str().starts_with("MOCK-") {
            Ok(Some(Task {
                id: task_id.clone(),
                title: "Mock Task".to_string(),
                description: Some("A mock task for testing".to_string()),
                status: TaskStatus::in_progress(TaskSystem::Generic),
                priority: TaskPriority::Medium,
                assignee: None,
                reporter: None,
                labels: vec![],
                time_tracking: TimeTracking::new(),
                comments: vec![],
                dependencies: vec![],
                custom_fields: std::collections::HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                due_date: None,
                resolution_date: None,
                external_url: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn sync_all_tasks(
        &self,
        _system: &TaskSystem,
        _config: &ExternalSystemConfig,
    ) -> Result<crate::domains::tasks::repository::SyncResult, TaskManagementDomainError> {
        Ok(crate::domains::tasks::repository::SyncResult {
            tasks_created: 5,
            tasks_updated: 3,
            tasks_deleted: 1,
            errors: vec![],
            duration: std::time::Duration::from_millis(100),
        })
    }

    async fn get_sync_status(
        &self,
        system: &TaskSystem,
    ) -> Result<crate::domains::tasks::repository::SyncStatus, TaskManagementDomainError> {
        Ok(crate::domains::tasks::repository::SyncStatus {
            system: system.clone(),
            last_sync: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            is_syncing: false,
            sync_errors: vec![],
            next_sync: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        })
    }

    async fn force_full_sync(
        &self,
        system: &TaskSystem,
        config: &ExternalSystemConfig,
    ) -> Result<crate::domains::tasks::repository::SyncResult, TaskManagementDomainError> {
        self.sync_all_tasks(system, config).await
    }
} 