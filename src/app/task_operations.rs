use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

use crate::{
    app::App,
    services::{jira::JiraClient, monday::MondayClient},
    types::{JiraTask, MondayTask, TaskSystem},
    utils,
};

#[allow(async_fn_in_trait)]
pub trait TaskOperations {
    async fn search_monday_tasks(&self, query: &str) -> Result<Vec<MondayTask>>;
    async fn search_jira_tasks(&self, query: &str) -> Result<Vec<JiraTask>>;
    fn update_task_selection(&mut self);
}

impl TaskOperations for App {
    async fn search_monday_tasks(&self, query: &str) -> Result<Vec<MondayTask>> {
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
        writeln!(debug_file, "DEBUG: MondayClient created successfully").ok();

        let result = client.search_tasks(query).await;
        match &result {
            Ok(tasks) => {
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
                writeln!(debug_file, "DEBUG: Search failed with error: {}", e).ok();
            }
        }

        result
    }

    async fn search_jira_tasks(&self, query: &str) -> Result<Vec<JiraTask>> {
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
        writeln!(debug_file, "DEBUG: JiraClient created successfully").ok();

        let result = client.search_tasks(query).await;
        match &result {
            Ok(tasks) => {
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
                writeln!(debug_file, "DEBUG: JIRA Search failed with error: {}", e).ok();
            }
        }

        result
    }

    fn update_task_selection(&mut self) {
        use crate::types::TaskLike;

        // Update commit form with latest selections and sync unified interface
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => {
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
            }
            crate::types::TaskSystem::Jira => {
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
            }
            crate::types::TaskSystem::None => {
                self.commit_form.selected_tasks.clear();
                self.commit_form.selected_monday_tasks.clear();
                self.commit_form.selected_jira_tasks.clear();
                self.commit_form.scope.clear();
            }
        }
    }
}
