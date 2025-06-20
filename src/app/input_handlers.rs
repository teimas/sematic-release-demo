use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::App,
    types::{AppScreen, AppState},
    ui::{CommitField, InputMode},
};

impl App {
    // Helper methods to work with the appropriate task collections based on configuration
    fn get_current_tasks_count(&self) -> usize {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => self.monday_tasks.len(),
            crate::types::TaskSystem::Jira => self.jira_tasks.len(),
            crate::types::TaskSystem::None => 0,
        }
    }

    fn get_current_selected_tasks_count(&self) -> usize {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => self.selected_monday_tasks.len(),
            crate::types::TaskSystem::Jira => self.selected_jira_tasks.len(),
            crate::types::TaskSystem::None => 0,
        }
    }

    fn get_current_task_id(&self, index: usize) -> Option<String> {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => self.monday_tasks.get(index).map(|t| t.id.clone()),
            crate::types::TaskSystem::Jira => self.jira_tasks.get(index).map(|t| t.id.clone()),
            crate::types::TaskSystem::None => None,
        }
    }

    fn is_task_selected(&self, task_id: &str) -> bool {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => {
                self.selected_monday_tasks.iter().any(|t| t.id == task_id)
            }
            crate::types::TaskSystem::Jira => {
                self.selected_jira_tasks.iter().any(|t| t.id == task_id)
            }
            crate::types::TaskSystem::None => false,
        }
    }

    fn add_task_to_selection(&mut self, index: usize) {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => {
                if let Some(task) = self.monday_tasks.get(index) {
                    self.selected_monday_tasks.push(task.clone());
                }
            }
            crate::types::TaskSystem::Jira => {
                if let Some(task) = self.jira_tasks.get(index) {
                    self.selected_jira_tasks.push(task.clone());
                }
            }
            crate::types::TaskSystem::None => {}
        }
    }

    fn remove_task_from_selection(&mut self, task_id: &str) {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => {
                if let Some(pos) = self
                    .selected_monday_tasks
                    .iter()
                    .position(|t| t.id == task_id)
                {
                    self.selected_monday_tasks.remove(pos);
                }
            }
            crate::types::TaskSystem::Jira => {
                if let Some(pos) = self
                    .selected_jira_tasks
                    .iter()
                    .position(|t| t.id == task_id)
                {
                    self.selected_jira_tasks.remove(pos);
                }
            }
            crate::types::TaskSystem::None => {}
        }
    }

    fn remove_selected_task_at_index(&mut self, index: usize) {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => {
                if index < self.selected_monday_tasks.len() {
                    self.selected_monday_tasks.remove(index);
                }
            }
            crate::types::TaskSystem::Jira => {
                if index < self.selected_jira_tasks.len() {
                    self.selected_jira_tasks.remove(index);
                }
            }
            crate::types::TaskSystem::None => {}
        }
    }

    fn clear_current_tasks(&mut self) {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => self.monday_tasks.clear(),
            crate::types::TaskSystem::Jira => self.jira_tasks.clear(),
            crate::types::TaskSystem::None => {}
        }
    }

    pub async fn handle_input_mode(&mut self, key: KeyEvent) -> Result<()> {
        // Handle different screens with their appropriate TextArea functions
        match self.current_screen {
            AppScreen::CommitPreview => {
                return self.handle_commit_preview_text_editing(key).await;
            }
            AppScreen::Commit => {
                return self.handle_commit_text_editing(key).await;
            }
            AppScreen::TaskSearch => {
                return self.handle_search_input_mode(key).await;
            }
            _ => {
                // Default behavior for other screens
                if key.code == KeyCode::Esc {
                    self.ui_state.input_mode = InputMode::Normal;
                }
            }
        }
        Ok(())
    }

    // Tab navigation functions moved to handle_tab_in_input_mode and handle_back_tab_in_input_mode in handle_commit_text_editing

    fn enter_edit_mode_if_text_field_input(&mut self) {
        if !matches!(
            self.ui_state.current_field,
            CommitField::Type | CommitField::SelectedTasks
        ) {
            // Load current form data into the appropriate textarea
            let current_field = self.ui_state.current_field.clone();
            if let Some(textarea) = self.ui_state.get_textarea_mut(&current_field) {
                let text = match current_field {
                    CommitField::Scope => &self.commit_form.scope,
                    CommitField::Title => &self.commit_form.title,
                    CommitField::Description => &self.commit_form.description,
                    CommitField::BreakingChange => &self.commit_form.breaking_change,
                    CommitField::TestDetails => &self.commit_form.test_details,
                    CommitField::Security => &self.commit_form.security,
                    CommitField::MigracionesLentas => &self.commit_form.migraciones_lentas,
                    CommitField::PartesAEjecutar => &self.commit_form.partes_a_ejecutar,
                    _ => "",
                };
                
                // Clear and set the textarea content
                textarea.select_all();
                textarea.delete_str(textarea.lines().join("\n").len());
                textarea.insert_str(text);
                
                self.ui_state.input_mode = InputMode::Editing;
            }
        } else {
            self.ui_state.input_mode = InputMode::Normal;
        }
    }

    pub async fn handle_commit_preview_text_editing(&mut self, key: KeyEvent) -> Result<()> {
        use crate::app::commit_operations::CommitOperations;

        // Check for Ctrl+C first (commit action)
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            let commit_message = self.ui_state.commit_preview_textarea.lines().join("\n");

            // Check if there are staged changes
            use crate::git::GitRepo;
            let git_repo = match GitRepo::new() {
                Ok(repo) => repo,
                Err(e) => {
                    self.current_state = AppState::Error(format!("Git repository error: {}", e));
                    return Ok(());
                }
            };

            let git_status = match git_repo.get_status() {
                Ok(status) => status,
                Err(e) => {
                    self.current_state =
                        AppState::Error(format!("Could not check git status: {}", e));
                    return Ok(());
                }
            };

            // If no staged changes but there are modified/untracked files, ask user to stage
            if git_status.staged.is_empty()
                && (!git_status.modified.is_empty() || !git_status.untracked.is_empty())
            {
                self.current_state = AppState::ConfirmingStageAll;
                self.message = Some(format!(
                    "No staged changes found. {} modified files and {} untracked files. Press 'y' to stage all (git add -A), 'n' to cancel.",
                    git_status.modified.len(),
                    git_status.untracked.len()
                ));
                return Ok(());
            }

            // If no staged changes and no other changes, show error
            if git_status.staged.is_empty() {
                self.current_state =
                    AppState::Error("No changes to commit. Make some changes first.".to_string());
                return Ok(());
            }

            // Proceed with commit if there are staged changes
            if let Err(e) = self.create_commit_with_message(&commit_message).await {
                self.current_state = AppState::Error(e.to_string());
            } else {
                self.message = Some("Commit created successfully!".to_string());
                self.current_screen = AppScreen::Main;
                self.ui_state.input_mode = InputMode::Normal;
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                self.current_screen = AppScreen::Commit;
                self.ui_state.input_mode = InputMode::Normal;
                self.message = Some("Commit cancelled".to_string());
            }
            KeyCode::Tab => {
                // Save current textarea content and move to next field
                self.save_current_textarea_to_form();
                self.ui_state.input_mode = InputMode::Normal;
                self.handle_tab_navigation();
                return Ok(());
            }
            _ => {
                // Pass all other inputs to textarea
                let input = Self::crossterm_key_to_textarea_input(key);
                self.ui_state.commit_preview_textarea.input(input);
            }
        }
        Ok(())
    }

    fn save_current_textarea_to_form(&mut self) {
        if let Some(textarea) = self.ui_state.get_textarea(&self.ui_state.current_field) {
            let content = textarea.lines().join("\n");
            match self.ui_state.current_field {
                CommitField::Scope => self.commit_form.scope = content,
                CommitField::Title => self.commit_form.title = content,
                CommitField::Description => self.commit_form.description = content,
                CommitField::BreakingChange => self.commit_form.breaking_change = content,
                CommitField::TestDetails => self.commit_form.test_details = content,
                CommitField::Security => self.commit_form.security = content,
                CommitField::MigracionesLentas => self.commit_form.migraciones_lentas = content,
                CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar = content,
                _ => {}
            }
        }
    }

    // Text manipulation methods - replaced with tui-textarea
    
    // Helper function to convert crossterm KeyEvent to tui_textarea Input
    fn crossterm_key_to_textarea_input(key: KeyEvent) -> tui_textarea::Input {
        use tui_textarea::{Input, Key};
        
        let tui_key = match key.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::Tab, // tui-textarea doesn't have BackTab, use Tab
            KeyCode::Delete => Key::Delete,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Insert => Key::Null, // tui-textarea doesn't have Insert, use Null
            KeyCode::Esc => Key::Esc,
            KeyCode::F(n) => Key::F(n),
            _ => Key::Null,
        };
        
        Input {
            key: tui_key,
            ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
            alt: key.modifiers.contains(KeyModifiers::ALT),
            shift: key.modifiers.contains(KeyModifiers::SHIFT),
        }
    }

    pub async fn handle_commit_text_editing(&mut self, key: KeyEvent) -> Result<()> {
        // Handle special keys that should exit edit mode or perform actions
        match key.code {
            KeyCode::Esc => {
                self.ui_state.input_mode = InputMode::Normal;
                return Ok(());
            }
            KeyCode::Tab => {
                // Save current textarea content and move to next field
                self.save_current_textarea_to_form();
                self.ui_state.input_mode = InputMode::Normal;
                self.handle_tab_navigation();
                return Ok(());
            }
            KeyCode::BackTab => {
                // Save current textarea content and move to previous field  
                self.save_current_textarea_to_form();
                self.ui_state.input_mode = InputMode::Normal;
                self.handle_back_tab_navigation();
                return Ok(());
            }
            _ => {}
        }

        // Pass input to the current textarea
        let current_field = self.ui_state.current_field.clone();
        if let Some(textarea) = self.ui_state.get_textarea_mut(&current_field) {
            // Convert crossterm KeyEvent to tui_textarea::Input
            let input = Self::crossterm_key_to_textarea_input(key);
            textarea.input(input);
        }

        Ok(())
    }

    pub fn handle_tab_navigation(&mut self) {
        // Navigate to next field
        self.ui_state.current_field = match self.ui_state.current_field {
            CommitField::Type => CommitField::Scope,
            CommitField::Scope => CommitField::Title,
            CommitField::Title => CommitField::Description,
            CommitField::Description => CommitField::BreakingChange,
            CommitField::BreakingChange => CommitField::TestDetails,
            CommitField::TestDetails => CommitField::Security,
            CommitField::Security => CommitField::MigracionesLentas,
            CommitField::MigracionesLentas => CommitField::PartesAEjecutar,
            CommitField::PartesAEjecutar => CommitField::SelectedTasks,
            CommitField::SelectedTasks => CommitField::Type,
        };

        self.enter_edit_mode_if_text_field_input();
    }

    pub fn handle_back_tab_navigation(&mut self) {
        // Navigate to previous field
        self.ui_state.current_field = match self.ui_state.current_field {
            CommitField::Type => CommitField::SelectedTasks,
            CommitField::Scope => CommitField::Type,
            CommitField::Title => CommitField::Scope,
            CommitField::Description => CommitField::Title,
            CommitField::BreakingChange => CommitField::Description,
            CommitField::TestDetails => CommitField::BreakingChange,
            CommitField::Security => CommitField::TestDetails,
            CommitField::MigracionesLentas => CommitField::Security,
            CommitField::PartesAEjecutar => CommitField::MigracionesLentas,
            CommitField::SelectedTasks => CommitField::PartesAEjecutar,
        };

        self.enter_edit_mode_if_text_field_input();
    }

    // All manual text manipulation methods removed - now handled by tui-textarea

    // Search input handling
    pub async fn handle_search_input_mode(&mut self, key: KeyEvent) -> Result<()> {
        use crate::app::task_operations::TaskOperations;

        match key.code {
            KeyCode::Esc => {
                self.ui_state.input_mode = InputMode::Normal;
                self.message = Some("Exited search mode".to_string());
            }
            KeyCode::Enter => {
                let search_query = self.ui_state.search_textarea.lines().join(" ");
                self.message = Some(format!(
                    "DEBUG: Starting search with query: '{}'",
                    search_query
                ));
                if !search_query.is_empty() {
                    self.current_state = AppState::Loading;

                    match self.config.get_task_system() {
                        crate::types::TaskSystem::Monday => {
                            match self.search_monday_tasks(&search_query).await {
                                Ok(tasks) => {
                                    self.monday_tasks = tasks;
                                    self.ui_state.selected_tab = 0;
                                    self.current_state = AppState::Normal;
                                    self.ui_state.input_mode = InputMode::Normal;
                                    self.message = Some(format!(
                                        "DEBUG: Monday search completed! Found {} tasks",
                                        self.monday_tasks.len()
                                    ));
                                }
                                Err(e) => {
                                    self.current_state = AppState::Error(format!(
                                        "DEBUG: Monday search failed: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        crate::types::TaskSystem::Jira => {
                            match self.search_jira_tasks(&search_query).await {
                                Ok(tasks) => {
                                    self.jira_tasks = tasks;
                                    self.ui_state.selected_tab = 0;
                                    self.current_state = AppState::Normal;
                                    self.ui_state.input_mode = InputMode::Normal;
                                    self.message = Some(format!(
                                        "DEBUG: JIRA search completed! Found {} tasks",
                                        self.jira_tasks.len()
                                    ));
                                }
                                Err(e) => {
                                    self.current_state = AppState::Error(format!(
                                        "DEBUG: JIRA search failed: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        crate::types::TaskSystem::None => {
                            self.current_state =
                                AppState::Error("No task management system configured".to_string());
                        }
                    }
                } else {
                    self.message = Some("DEBUG: Search query is empty".to_string());
                }
            }
            _ => {
                // Pass all other inputs to the search textarea
                let input = Self::crossterm_key_to_textarea_input(key);
                self.ui_state.search_textarea.input(input);
            }
        }
        Ok(())
    }

    pub async fn handle_search_navigation_mode(&mut self, key: KeyCode) -> Result<()> {
        use crate::app::task_operations::TaskOperations;

        match key {
            KeyCode::Char('q') => {
                self.current_screen = AppScreen::Commit;
            }
            KeyCode::Esc => {
                self.clear_current_tasks();
                self.ui_state.search_textarea.select_all();
                self.ui_state.search_textarea.delete_str(self.ui_state.search_textarea.lines().join("\n").len());
                self.ui_state.focused_search_index = 0;
                self.message = Some("Search cleared".to_string());
            }
            KeyCode::Char('i') | KeyCode::Char('/') => {
                self.ui_state.input_mode = InputMode::Editing;
                self.message = Some("DEBUG: Entered edit mode - you can now type".to_string());
            }
            KeyCode::Enter => {
                let search_query = self.ui_state.search_textarea.lines().join(" ");
                if !search_query.is_empty() {
                    self.current_state = AppState::Loading;

                    match self.config.get_task_system() {
                        crate::types::TaskSystem::Monday => {
                            match self.search_monday_tasks(&search_query).await {
                                Ok(tasks) => {
                                    self.monday_tasks = tasks;
                                    self.ui_state.selected_tab = 0;
                                    self.current_state = AppState::Normal;
                                    self.message = Some(format!(
                                        "Found {} Monday tasks",
                                        self.monday_tasks.len()
                                    ));
                                }
                                Err(e) => {
                                    self.current_state = AppState::Error(e.to_string());
                                }
                            }
                        }
                        crate::types::TaskSystem::Jira => {
                            match self.search_jira_tasks(&search_query).await {
                                Ok(tasks) => {
                                    self.jira_tasks = tasks;
                                    self.ui_state.selected_tab = 0;
                                    self.current_state = AppState::Normal;
                                    self.message =
                                        Some(format!("Found {} JIRA tasks", self.jira_tasks.len()));
                                }
                                Err(e) => {
                                    self.current_state = AppState::Error(e.to_string());
                                }
                            }
                        }
                        crate::types::TaskSystem::None => {
                            self.current_state =
                                AppState::Error("No task management system configured".to_string());
                        }
                    }
                } else {
                    self.ui_state.input_mode = InputMode::Editing;
                }
            }
            KeyCode::Up => {
                self.handle_search_up_navigation();
            }
            KeyCode::Down => {
                self.handle_search_down_navigation();
            }
            KeyCode::Delete | KeyCode::Char('r') => {
                self.handle_search_task_removal();
            }
            KeyCode::Char(' ') => {
                self.handle_search_task_toggle();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.handle_numeric_task_selection(c);
            }
            KeyCode::Backspace => {
                self.ui_state.search_textarea.select_all();
                self.ui_state.search_textarea.delete_str(self.ui_state.search_textarea.lines().join("\n").len());
                self.clear_current_tasks();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_up_navigation(&mut self) {
        if self.get_current_tasks_count() > 0 {
            if self.ui_state.focused_search_index > 0 {
                self.ui_state.focused_search_index -= 1;
            }
        } else if self.get_current_selected_tasks_count() > 0 && self.ui_state.selected_tab > 0 {
            self.ui_state.selected_tab -= 1;
        }
    }

    fn handle_search_down_navigation(&mut self) {
        if self.get_current_tasks_count() > 0 {
            if self.ui_state.focused_search_index < self.get_current_tasks_count().saturating_sub(1)
            {
                self.ui_state.focused_search_index += 1;
            }
        } else if self.get_current_selected_tasks_count() > 0
            && self.ui_state.selected_tab
                < self.get_current_selected_tasks_count().saturating_sub(1)
        {
            self.ui_state.selected_tab += 1;
        }
    }

    fn handle_search_task_removal(&mut self) {
        use crate::app::task_operations::TaskOperations;

        if self.get_current_tasks_count() > 0 {
            if let Some(task_id) = self.get_current_task_id(self.ui_state.focused_search_index) {
                if self.is_task_selected(&task_id) {
                    self.remove_task_from_selection(&task_id);
                    self.update_task_selection();
                    self.message = Some("Task removed from selection".to_string());
                } else {
                    self.message = Some("Task is not selected".to_string());
                }
            }
        } else {
            let selected_tab = self.ui_state.selected_tab;
            let selected_tasks_len = self.get_current_selected_tasks_count();

            if selected_tasks_len > 0 && selected_tab < selected_tasks_len {
                self.remove_selected_task_at_index(selected_tab);

                let new_len = self.get_current_selected_tasks_count();
                if selected_tab >= new_len && new_len > 0 {
                    self.ui_state.selected_tab = new_len - 1;
                }

                self.update_task_selection();
                self.message = Some("Task removed from selection".to_string());
            }
        }
    }

    fn handle_search_task_toggle(&mut self) {
        use crate::app::task_operations::TaskOperations;

        if self.get_current_tasks_count() > 0 {
            if let Some(task_id) = self.get_current_task_id(self.ui_state.focused_search_index) {
                if self.is_task_selected(&task_id) {
                    self.remove_task_from_selection(&task_id);
                    self.message = Some("Task deselected".to_string());
                } else {
                    self.add_task_to_selection(self.ui_state.focused_search_index);
                    self.message = Some("Task selected".to_string());
                }
                self.update_task_selection();
            }
        } else {
            let selected_tab = self.ui_state.selected_tab;
            let selected_tasks_len = self.get_current_selected_tasks_count();

            if selected_tasks_len > 0 && selected_tab < selected_tasks_len {
                self.remove_selected_task_at_index(selected_tab);

                let new_len = self.get_current_selected_tasks_count();
                if selected_tab >= new_len && new_len > 0 {
                    self.ui_state.selected_tab = new_len - 1;
                }

                self.update_task_selection();
                self.message = Some("Task removed from selection".to_string());
            }
        }
    }

    fn handle_numeric_task_selection(&mut self, c: char) {
        use crate::app::task_operations::TaskOperations;

        let index = if c == '0' {
            9
        } else {
            (c as usize) - ('1' as usize)
        };

        if let Some(task_id) = self.get_current_task_id(index) {
            if self.is_task_selected(&task_id) {
                self.remove_task_from_selection(&task_id);
            } else {
                self.add_task_to_selection(index);
            }
            self.update_task_selection();
        }
    }
}
