/// List Tasks Query
/// 
/// This query provides advanced task listing with filtering, pagination,
/// and sorting capabilities across multiple task systems.

#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use std::any::Any;
#[cfg(feature = "new-domains")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc};

#[cfg(feature = "new-domains")]
use super::Query;
#[cfg(feature = "new-domains")]
use crate::domains::tasks::value_objects::{TaskId, TaskStatus, TaskPriority};

/// Task query filters for advanced searching
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueryFilters {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee: Option<String>,
    pub project: Option<String>,
    pub labels: Option<Vec<String>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
    pub search_text: Option<String>,
}

/// Pagination configuration
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub max_results: Option<u32>,
}

#[cfg(feature = "new-domains")]
impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 50,
            max_results: Some(1000),
        }
    }
}

/// Task sorting options
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSorting {
    pub field: TaskSortField,
    pub direction: SortDirection,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSortField {
    CreatedAt,
    UpdatedAt,
    Priority,
    Status,
    Title,
    Assignee,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[cfg(feature = "new-domains")]
impl Default for TaskSorting {
    fn default() -> Self {
        Self {
            field: TaskSortField::UpdatedAt,
            direction: SortDirection::Descending,
        }
    }
}

/// Query to list tasks with advanced filtering
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksQuery {
    pub filters: Option<TaskQueryFilters>,
    pub pagination: Option<Pagination>,
    pub sort: Option<TaskSorting>,
}

#[cfg(feature = "new-domains")]
impl Query for ListTasksQuery {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
 