use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

use crate::{
    app::App,
    services::MondayClient,
    types::MondayTask,
};

pub trait TaskOperations {
    async fn search_monday_tasks(&self, query: &str) -> Result<Vec<MondayTask>>;
    fn update_task_selection(&mut self);
    fn toggle_task_selection(&mut self);
    fn confirm_task_selection(&mut self);
}

impl TaskOperations for App {
    async fn search_monday_tasks(&self, query: &str) -> Result<Vec<MondayTask>> {
        // Write debug to file
        let mut debug_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .unwrap_or_else(|_| std::fs::File::create("debug.log").unwrap());
            
        writeln!(debug_file, "DEBUG: search_monday_tasks called with query: '{}'", query).ok();
        
        let client = MondayClient::new(&self.config)?;
        writeln!(debug_file, "DEBUG: MondayClient created successfully").ok();
        
        let result = client.comprehensive_search(query).await;
        match &result {
            Ok(tasks) => {
                writeln!(debug_file, "DEBUG: Search returned {} tasks", tasks.len()).ok();
                for (i, task) in tasks.iter().enumerate().take(3) {
                    writeln!(debug_file, "DEBUG: Task {}: {} ({})", i, task.title, task.id).ok();
                }
            }
            Err(e) => {
                writeln!(debug_file, "DEBUG: Search failed with error: {}", e).ok();
            }
        }
        
        result
    }

    fn update_task_selection(&mut self) {
        // Update commit form with latest selections
        self.commit_form.selected_tasks = self.selected_tasks.clone();
        
        // Update scope with task IDs
        let task_ids: Vec<String> = self.selected_tasks.iter().map(|t| t.id.clone()).collect();
        self.commit_form.scope = if task_ids.is_empty() {
            String::new()
        } else {
            task_ids.join("|")
        };
    }

    fn toggle_task_selection(&mut self) {
        // Toggle task selection
        if let Some(task) = self.tasks.get(self.ui_state.selected_tab) {
            if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                self.selected_tasks.remove(pos);
            } else {
                self.selected_tasks.push(task.clone());
            }
        }
    }

    fn confirm_task_selection(&mut self) {
        // Confirm selection and return to commit screen
        self.commit_form.selected_tasks = self.selected_tasks.clone();
        
        // Generate scope from selected task IDs
        let task_ids: Vec<String> = self.selected_tasks.iter().map(|t| t.id.clone()).collect();
        if !task_ids.is_empty() {
            self.commit_form.scope = task_ids.join("|");
        }
        
        self.current_screen = crate::types::AppScreen::Commit;
    }
} 