use crate::error::Result;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{debug, error, instrument};

use crate::{
    app::App,
    services::{jira::JiraClient, monday::MondayClient},
    types::{JiraTask, MondayTask},
};

#[allow(async_fn_in_trait)]
pub trait TaskOperations {
    async fn search_monday_tasks(&self, query: &str) -> Result<Vec<MondayTask>>;
    async fn search_jira_tasks(&self, query: &str) -> Result<Vec<JiraTask>>;
    fn update_task_selection(&mut self);
}

impl TaskOperations for App {
    #[instrument(skip(self), fields(query = %query))]
    async fn search_monday_tasks(&self, query: &str) -> Result<Vec<MondayTask>> {
        debug!("Starting Monday.com task search");

        // Write debug to file
        let mut debug_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .unwrap_or_else(|_| std::fs::File::create("debug.log").unwrap());

        writeln!(
            debug_file,
            "DEBUG: search_monday_tasks called with query: '{}'",
            query
        )
        .ok();

        let client = MondayClient::new(&self.config)?;
        debug!("Monday.com client created successfully");
        writeln!(debug_file, "DEBUG: MondayClient created successfully").ok();

        let result = client.search_tasks(query).await;
        match &result {
            Ok(tasks) => {
                debug!("Search returned {} tasks", tasks.len());
                writeln!(debug_file, "DEBUG: Search returned {} tasks", tasks.len()).ok();
                for (i, task) in tasks.iter().enumerate().take(3) {
                    writeln!(
                        debug_file,
                        "DEBUG: Task {}: {} ({})",
                        i, task.title, task.id
                    )
                    .ok();
                }
            }
            Err(e) => {
                error!("Search failed with error: {}", e);
                writeln!(debug_file, "DEBUG: Search failed with error: {}", e).ok();
            }
        }

        result
    }

    #[instrument(skip(self), fields(query = %query))]
    async fn search_jira_tasks(&self, query: &str) -> Result<Vec<JiraTask>> {
        debug!("Starting JIRA task search");

        // Write debug to file
        let mut debug_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .unwrap_or_else(|_| std::fs::File::create("debug.log").unwrap());

        writeln!(
            debug_file,
            "DEBUG: search_jira_tasks called with query: '{}'",
            query
        )
        .ok();

        let client = JiraClient::new(&self.config)?;
        debug!("JIRA client created successfully");
        writeln!(debug_file, "DEBUG: JiraClient created successfully").ok();

        let result = client.search_tasks(query).await;
        match &result {
            Ok(tasks) => {
                debug!("JIRA search returned {} tasks", tasks.len());
                writeln!(
                    debug_file,
                    "DEBUG: JIRA Search returned {} tasks",
                    tasks.len()
                )
                .ok();
                for (i, task) in tasks.iter().enumerate().take(3) {
                    writeln!(
                        debug_file,
                        "DEBUG: JIRA Task {}: {} ({})",
                        i, task.summary, task.key
                    )
                    .ok();
                }
            }
            Err(e) => {
                error!("JIRA search failed with error: {}", e);
                writeln!(debug_file, "DEBUG: JIRA Search failed with error: {}", e).ok();
            }
        }

        result
    }

    #[instrument(skip(self))]
    fn update_task_selection(&mut self) {
        use crate::types::TaskLike;

        debug!("Updating task selection based on current task system");

        // Update commit form with latest selections and sync unified interface
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => {
                debug!("Updating selection for Monday.com tasks");
                self.commit_form.selected_tasks = self.selected_monday_tasks.clone();
                self.commit_form.selected_monday_tasks = self.selected_monday_tasks.clone();
                self.commit_form.selected_jira_tasks.clear();

                // Update scope with Monday task IDs
                let task_ids: Vec<String> = self
                    .selected_monday_tasks
                    .iter()
                    .map(|t| t.get_id().to_string())
                    .collect();
                self.commit_form.scope = if task_ids.is_empty() {
                    String::new()
                } else {
                    task_ids.join("|")
                };
                debug!("Updated scope with {} Monday task IDs", task_ids.len());
            }
            crate::types::TaskSystem::Jira => {
                debug!("Updating selection for JIRA tasks");
                // For JIRA, we need to convert to Monday tasks for the unified interface
                // This is a temporary workaround until we implement a proper unified task type
                self.commit_form.selected_tasks.clear(); // Clear Monday tasks when using JIRA
                self.commit_form.selected_monday_tasks.clear();
                self.commit_form.selected_jira_tasks = self.selected_jira_tasks.clone();

                // Update scope with JIRA task IDs (use key for JIRA)
                let task_ids: Vec<String> = self
                    .selected_jira_tasks
                    .iter()
                    .map(|t| t.key.clone())
                    .collect();
                self.commit_form.scope = if task_ids.is_empty() {
                    String::new()
                } else {
                    task_ids.join("|")
                };
                debug!("Updated scope with {} JIRA task IDs", task_ids.len());
            }
            crate::types::TaskSystem::None => {
                debug!("Clearing all task selections (task system set to None)");
                self.commit_form.selected_tasks.clear();
                self.commit_form.selected_monday_tasks.clear();
                self.commit_form.selected_jira_tasks.clear();
                self.commit_form.scope.clear();
            }
        }
    }
}
