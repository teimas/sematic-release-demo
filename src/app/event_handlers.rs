use crate::error::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::semantic_release_operations::SemanticReleaseOperations,
    app::App,
    types::{AppScreen, AppState, CommitType},
    ui::{CommitField, InputMode},
};

#[allow(async_fn_in_trait)]
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
                // Clear any textarea content if needed
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
                self.ui_state.selected_tab = (self.ui_state.selected_tab + 1) % 6;
            }
            KeyCode::BackTab => {
                self.ui_state.selected_tab = if self.ui_state.selected_tab == 0 {
                    5
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
                    5 => {
                        // Setup GitHub Actions for semantic-release
                        self.setup_github_actions_semantic_release().await?;
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
        let search_input = self.ui_state.search_textarea.lines().join(" ");
        self.message = Some(format!(
            "DEBUG: Key: {:?}, Mode: {:?}, Input: '{}'",
            key, self.ui_state.input_mode, search_input
        ));

        if in_search_input {
            self.handle_search_input_mode(crossterm::event::KeyEvent::new(key, crossterm::event::KeyModifiers::empty())).await?;
        } else {
            self.handle_search_navigation_mode(key).await?;
        }
        Ok(())
    }

    // Helper methods for commit screen handling
    fn handle_monday_search(&mut self) {
        self.current_screen = AppScreen::TaskSearch;
        self.ui_state.search_textarea.select_all();
        self.ui_state.search_textarea.delete_str(self.ui_state.search_textarea.lines().join("\n").len());
        self.monday_tasks.clear();
        self.ui_state.selected_tab = 0;
        self.message = Some("Monday.com Search - Press 'i' or '/' to start typing".to_string());
    }

    fn handle_jira_search(&mut self) {
        self.current_screen = AppScreen::TaskSearch;
        self.ui_state.search_textarea.select_all();
        self.ui_state.search_textarea.delete_str(self.ui_state.search_textarea.lines().join("\n").len());
        self.jira_tasks.clear();
        self.ui_state.selected_tab = 0;
        self.message = Some("JIRA Search - Press 'i' or '/' to start typing".to_string());
    }

    fn handle_commit_preview(&mut self) {
        use crate::app::commit_operations::CommitOperations;
        self.preview_commit_message = self.build_commit_message();
        self.current_screen = AppScreen::CommitPreview;
        self.ui_state.input_mode = InputMode::Editing;
        
        // Load the commit message into the preview textarea
        self.ui_state.commit_preview_textarea.select_all();
        self.ui_state.commit_preview_textarea.delete_str(self.ui_state.commit_preview_textarea.lines().join("\n").len());
        self.ui_state.commit_preview_textarea.insert_str(&self.preview_commit_message);
        
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

    // Navigation functions moved to input_handlers.rs to avoid duplication

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
                // Load the current field content into the appropriate textarea
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
                    textarea.select_all();
                    textarea.delete_str(textarea.lines().join("\n").len());
                    textarea.insert_str(text);
                }
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
