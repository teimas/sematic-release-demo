use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::semantic_release_operations::SemanticReleaseOperations,
    app::App,
    types::{AppScreen, AppState, CommitType},
    ui::{CommitField, InputMode},
};

pub trait EventHandlers {
    async fn handle_key_event_impl(&mut self, key: KeyEvent) -> Result<()>;
}

impl EventHandlers for App {
    async fn handle_key_event_impl(&mut self, key: KeyEvent) -> Result<()> {
        // Clear error state on any key press
        if matches!(self.current_state, AppState::Error(_)) {
            self.current_state = AppState::Normal;
            return Ok(());
        }

        // Handle confirmation dialog for staging all files
        if matches!(self.current_state, AppState::ConfirmingStageAll) {
            return self.handle_stage_confirmation(key.code).await;
        }

        match (&self.current_screen, &self.ui_state.input_mode) {
            (_, InputMode::Editing) => {
                self.handle_input_mode(key).await?;
            }
            (AppScreen::Main, _) => {
                self.handle_main_screen(key.code).await?;
            }
            (AppScreen::Config, _) => {
                self.handle_config_screen(key.code).await?;
            }
            (AppScreen::Commit, _) => {
                self.handle_commit_screen(key.code).await?;
            }
            (AppScreen::CommitPreview, _) => {
                self.handle_commit_preview_screen(key.code).await?;
            }
            (AppScreen::ReleaseNotes, _) => {
                self.handle_release_notes_screen(key.code).await?;
            }
            (AppScreen::SemanticRelease, _) => {
                self.handle_semantic_release_screen(key.code).await?;
            }
            (AppScreen::TaskSearch, _) => {
                self.handle_task_search_screen(key.code).await?;
            }
        }

        Ok(())
    }
}

impl App {
    // Helper methods to work with the appropriate task collections based on configuration
    fn get_selected_tasks_count(&self) -> usize {
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => self.selected_monday_tasks.len(),
            crate::types::TaskSystem::Jira => self.selected_jira_tasks.len(),
            crate::types::TaskSystem::None => 0,
        }
    }

    fn remove_selected_task_by_index(&mut self, index: usize) {
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

    async fn handle_main_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.ui_state.selected_tab = (self.ui_state.selected_tab + 1) % 5;
            }
            KeyCode::BackTab => {
                self.ui_state.selected_tab = if self.ui_state.selected_tab == 0 {
                    4
                } else {
                    self.ui_state.selected_tab - 1
                };
            }
            KeyCode::Enter => {
                match self.ui_state.selected_tab {
                    0 => self.current_screen = AppScreen::Commit,
                    1 => self.current_screen = AppScreen::ReleaseNotes,
                    2 => self.current_screen = AppScreen::SemanticRelease,
                    3 => self.current_screen = AppScreen::Config,
                    4 => {} // Help - stay here
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_config_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_commit_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            KeyCode::Char('s') => {
                match self.config.get_task_system() {
                    crate::types::TaskSystem::Monday => {
                        self.handle_monday_search();
                    }
                    crate::types::TaskSystem::Jira => {
                        self.message =
                            Some("JIRA is configured. Use 'j' to search JIRA tasks.".to_string());
                    }
                    crate::types::TaskSystem::None => {
                        self.message = Some("No task system configured. Configure Monday.com or JIRA in config screen.".to_string());
                    }
                }
            }
            KeyCode::Char('j') => {
                match self.config.get_task_system() {
                    crate::types::TaskSystem::Jira => {
                        self.handle_jira_search();
                    }
                    crate::types::TaskSystem::Monday => {
                        self.message = Some(
                            "Monday.com is configured. Use 's' to search Monday.com tasks."
                                .to_string(),
                        );
                    }
                    crate::types::TaskSystem::None => {
                        self.message = Some("No task system configured. Configure Monday.com or JIRA in config screen.".to_string());
                    }
                }
            }
            KeyCode::Char('c') => {
                self.handle_commit_preview();
            }
            KeyCode::Char('t') => {
                if !matches!(self.current_state, AppState::Loading)
                    && self.comprehensive_analysis_state.is_none()
                {
                    use crate::app::background_operations::BackgroundOperations;
                    self.start_comprehensive_analysis_wrapper().await;
                }
            }
            KeyCode::Char('m') => {
                self.handle_task_management_toggle();
            }
            KeyCode::Tab => {
                self.handle_tab_navigation();
            }
            KeyCode::BackTab => {
                self.handle_back_tab_navigation();
            }
            KeyCode::Up => {
                self.handle_up_navigation();
            }
            KeyCode::Down => {
                self.handle_down_navigation();
            }
            KeyCode::Enter => {
                self.handle_enter_in_commit();
            }
            KeyCode::Delete | KeyCode::Char(' ') => {
                self.handle_task_deletion();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_commit_preview_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Cancel commit and go back to commit screen
                self.current_screen = AppScreen::Commit;
                self.ui_state.input_mode = InputMode::Normal;
                self.ui_state.current_input.clear();
                self.message = Some("Commit cancelled".to_string());
            }
            _ => {
                // Handle normal text editing in the preview
                // The input mode handling will take care of character input
            }
        }
        Ok(())
    }

    async fn handle_release_notes_screen(&mut self, key: KeyCode) -> Result<()> {
        use crate::app::release_notes::ReleaseNotesOperations;

        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            KeyCode::Enter => {
                self.handle_release_notes_generation().await?;
            }
            KeyCode::Char('o') => {
                self.generate_release_notes_with_npm_wrapper().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_semantic_release_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            KeyCode::Char('r') => {
                // Clear results and go back to normal view
                self.semantic_release_state = None;
                self.ui_state.scroll_offset = 0;
                self.message = Some("Results cleared".to_string());
            }
            KeyCode::Up => {
                // Scroll up in results if we have results
                if self.semantic_release_state.is_some() && self.ui_state.scroll_offset > 0 {
                    self.ui_state.scroll_offset -= 1;
                }
            }
            KeyCode::Down => {
                // Scroll down in results if we have results
                if let Some(state) = &self.semantic_release_state {
                    if let Ok(result) = state.result.lock() {
                        let line_count = result.lines().count();
                        if self.ui_state.scroll_offset < line_count.saturating_sub(10) {
                            self.ui_state.scroll_offset += 1;
                        }
                    }
                }
            }
            KeyCode::Tab => {
                self.ui_state.selected_tab = (self.ui_state.selected_tab + 1) % 5;
            }
            KeyCode::BackTab => {
                self.ui_state.selected_tab = if self.ui_state.selected_tab == 0 {
                    4
                } else {
                    self.ui_state.selected_tab - 1
                };
            }
            KeyCode::Enter => {
                match self.ui_state.selected_tab {
                    0 => {
                        // Dry run - check what would be released
                        self.execute_semantic_release(true).await?;
                    }
                    1 => {
                        // Execute semantic release
                        self.execute_semantic_release(false).await?;
                    }
                    2 => {
                        // Get detailed version info
                        self.get_detailed_version_info().await?;
                    }
                    3 => {
                        // View last release info
                        self.view_last_release_info().await?;
                    }
                    4 => {
                        // View semantic-release config
                        self.view_semantic_release_config().await?;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_task_search_screen(&mut self, key: KeyCode) -> Result<()> {
        // Check if we're in search input mode (typing in the search box)
        let in_search_input = self.ui_state.input_mode == InputMode::Editing;

        // Debug: log key events and current mode
        self.message = Some(format!(
            "DEBUG: Key: {:?}, Mode: {:?}, Input: '{}'",
            key, self.ui_state.input_mode, self.ui_state.current_input
        ));

        if in_search_input {
            self.handle_search_input_mode(key).await?;
        } else {
            self.handle_search_navigation_mode(key).await?;
        }
        Ok(())
    }

    // Helper methods for commit screen handling
    fn handle_monday_search(&mut self) {
        self.current_screen = AppScreen::TaskSearch;
        self.ui_state.current_input.clear();
        self.monday_tasks.clear();
        self.ui_state.selected_tab = 0;
        self.message = Some("Monday.com Search - Press 'i' or '/' to start typing".to_string());
    }

    fn handle_jira_search(&mut self) {
        self.current_screen = AppScreen::TaskSearch;
        self.ui_state.current_input.clear();
        self.jira_tasks.clear();
        self.ui_state.selected_tab = 0;
        self.message = Some("JIRA Search - Press 'i' or '/' to start typing".to_string());
    }

    fn handle_commit_preview(&mut self) {
        use crate::app::commit_operations::CommitOperations;
        self.preview_commit_message = self.build_commit_message();
        self.current_screen = AppScreen::CommitPreview;
        self.ui_state.input_mode = InputMode::Editing;
        self.ui_state.current_input = self.preview_commit_message.clone();
        self.ui_state.cursor_position = self.preview_commit_message.len();
        self.message = Some(
            "Review and edit your commit message. Press Ctrl+C to commit, Esc to cancel"
                .to_string(),
        );
    }

    fn handle_task_management_toggle(&mut self) {
        self.ui_state.task_management_mode = !self.ui_state.task_management_mode;
        if self.ui_state.task_management_mode {
            self.ui_state.selected_tab = 0;
            self.message = Some(
                "Task management mode ON - Use ↑↓ to navigate, Delete/Space to remove tasks"
                    .to_string(),
            );
        } else {
            self.message = Some("Task management mode OFF".to_string());
        }
    }

    fn handle_tab_navigation(&mut self) {
        if self.ui_state.input_mode == InputMode::Editing {
            self.save_current_field();
        }

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

        self.enter_edit_mode_if_text_field();
    }

    fn handle_back_tab_navigation(&mut self) {
        if self.ui_state.input_mode == InputMode::Editing {
            self.save_current_field();
        }

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

        self.enter_edit_mode_if_text_field();
    }

    fn handle_up_navigation(&mut self) {
        if self.ui_state.task_management_mode {
            if self.get_selected_tasks_count() > 0 && self.ui_state.selected_tab > 0 {
                self.ui_state.selected_tab -= 1;
            }
        } else {
            match self.ui_state.current_field {
                CommitField::Type => {
                    if self.ui_state.selected_commit_type > 0 {
                        self.ui_state.selected_commit_type -= 1;
                    }
                }
                CommitField::SelectedTasks => {
                    if self.get_selected_tasks_count() > 0 && self.ui_state.selected_tab > 0 {
                        self.ui_state.selected_tab -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_down_navigation(&mut self) {
        if self.ui_state.task_management_mode {
            if self.get_selected_tasks_count() > 0
                && self.ui_state.selected_tab < self.get_selected_tasks_count().saturating_sub(1)
            {
                self.ui_state.selected_tab += 1;
            }
        } else {
            match self.ui_state.current_field {
                CommitField::Type => {
                    let max_types = CommitType::all().len();
                    if self.ui_state.selected_commit_type < max_types - 1 {
                        self.ui_state.selected_commit_type += 1;
                    }
                }
                CommitField::SelectedTasks => {
                    if self.get_selected_tasks_count() > 0
                        && self.ui_state.selected_tab
                            < self.get_selected_tasks_count().saturating_sub(1)
                    {
                        self.ui_state.selected_tab += 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_enter_in_commit(&mut self) {
        match self.ui_state.current_field {
            CommitField::Type => {
                let commit_types = CommitType::all();
                if let Some(selected_type) = commit_types.get(self.ui_state.selected_commit_type) {
                    self.commit_form.commit_type = Some(selected_type.clone());
                }
            }
            CommitField::SelectedTasks => {
                self.ui_state.task_management_mode = true;
                self.message = Some("Task management mode ON. Use Up/Down to navigate tasks, Delete/r/Space to remove".to_string());
            }
            _ => {
                self.ui_state.input_mode = InputMode::Editing;
                self.load_current_field_content();
                self.ui_state.cursor_position = self.ui_state.current_input.len();
            }
        }
    }

    fn handle_task_deletion(&mut self) {
        let selected_tab = self.ui_state.selected_tab;
        let selected_tasks_len = self.get_selected_tasks_count();
        let should_delete = (self.ui_state.task_management_mode
            || self.ui_state.current_field == CommitField::SelectedTasks)
            && selected_tasks_len > 0
            && selected_tab < selected_tasks_len;

        if should_delete {
            self.remove_selected_task_by_index(selected_tab);

            let new_len = self.get_selected_tasks_count();
            if selected_tab >= new_len && new_len > 0 {
                self.ui_state.selected_tab = new_len - 1;
            }

            use crate::app::task_operations::TaskOperations;
            self.update_task_selection();
            self.message = Some("Task removed from selection".to_string());
        }
    }

    fn enter_edit_mode_if_text_field(&mut self) {
        match self.ui_state.current_field {
            CommitField::Type | CommitField::SelectedTasks => {
                self.ui_state.input_mode = InputMode::Normal;
            }
            _ => {
                self.ui_state.input_mode = InputMode::Editing;
                self.load_current_field_content();
                self.ui_state.cursor_position = self.ui_state.current_input.len();
            }
        }
    }

    fn load_current_field_content(&mut self) {
        self.ui_state.current_input = match self.ui_state.current_field {
            CommitField::Type => String::new(),
            CommitField::Scope => self.commit_form.scope.clone(),
            CommitField::Title => self.commit_form.title.clone(),
            CommitField::Description => self.commit_form.description.clone(),
            CommitField::BreakingChange => self.commit_form.breaking_change.clone(),
            CommitField::TestDetails => self.commit_form.test_details.clone(),
            CommitField::Security => self.commit_form.security.clone(),
            CommitField::MigracionesLentas => self.commit_form.migraciones_lentas.clone(),
            CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar.clone(),
            CommitField::SelectedTasks => String::new(),
        };
    }

    pub fn save_current_field(&mut self) {
        match self.ui_state.current_field {
            CommitField::Type => {}
            CommitField::Scope => self.commit_form.scope = self.ui_state.current_input.clone(),
            CommitField::Title => self.commit_form.title = self.ui_state.current_input.clone(),
            CommitField::Description => {
                self.commit_form.description = self.ui_state.current_input.clone()
            }
            CommitField::BreakingChange => {
                self.commit_form.breaking_change = self.ui_state.current_input.clone()
            }
            CommitField::TestDetails => {
                self.commit_form.test_details = self.ui_state.current_input.clone()
            }
            CommitField::Security => {
                self.commit_form.security = self.ui_state.current_input.clone()
            }
            CommitField::MigracionesLentas => {
                self.commit_form.migraciones_lentas = self.ui_state.current_input.clone()
            }
            CommitField::PartesAEjecutar => {
                self.commit_form.partes_a_ejecutar = self.ui_state.current_input.clone()
            }
            CommitField::SelectedTasks => {}
        }
    }

    async fn handle_stage_confirmation(&mut self, key: KeyCode) -> Result<()> {
        use crate::app::commit_operations::CommitOperations;
        use crate::git::GitRepo;

        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // User confirmed, stage all changes
                let git_repo = match GitRepo::new() {
                    Ok(repo) => repo,
                    Err(e) => {
                        self.current_state =
                            AppState::Error(format!("Git repository error: {}", e));
                        return Ok(());
                    }
                };

                match git_repo.stage_all() {
                    Ok(_) => {
                        // Successfully staged, now proceed with commit
                        if let Err(e) = self
                            .create_commit_with_message(&self.preview_commit_message)
                            .await
                        {
                            self.current_state = AppState::Error(e.to_string());
                        } else {
                            self.message = Some(
                                "All changes staged and commit created successfully!".to_string(),
                            );
                            self.current_screen = AppScreen::Main;
                            self.ui_state.input_mode = InputMode::Normal;
                            self.ui_state.current_input.clear();
                            self.current_state = AppState::Normal;
                        }
                    }
                    Err(e) => {
                        self.current_state =
                            AppState::Error(format!("Failed to stage changes: {}", e));
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                // User declined or cancelled
                self.current_state = AppState::Normal;
                self.message = Some("Commit cancelled. No changes were staged.".to_string());
            }
            _ => {
                // Ignore other keys, keep waiting for y/n
            }
        }
        Ok(())
    }
}
