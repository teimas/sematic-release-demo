use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use log::{debug, info, error};

use crate::{
    app::App,
    types::{AppScreen, AppState, CommitType},
    ui::{self, InputMode, CommitField},
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

        match (&self.current_screen, &self.ui_state.input_mode) {
            (_, InputMode::Editing) => {
                use crate::app::input_handlers::*;
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
            (AppScreen::TaskSearch, _) => {
                self.handle_task_search_screen(key.code).await?;
            }
            (AppScreen::TaskSelection, _) => {
                self.handle_task_selection_screen(key.code).await?;
            }
        }

        Ok(())
    }
}

impl App {
    async fn handle_main_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.ui_state.selected_tab = (self.ui_state.selected_tab + 1) % 4;
            }
            KeyCode::BackTab => {
                self.ui_state.selected_tab = if self.ui_state.selected_tab == 0 {
                    3
                } else {
                    self.ui_state.selected_tab - 1
                };
            }
            KeyCode::Enter => {
                match self.ui_state.selected_tab {
                    0 => self.current_screen = AppScreen::Commit,
                    1 => self.current_screen = AppScreen::ReleaseNotes,
                    2 => self.current_screen = AppScreen::Config,
                    3 => {}, // Help - stay here
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
        use crate::app::task_operations::TaskOperations;
        use crate::app::background_operations::BackgroundOperations;
        use crate::app::commit_operations::CommitOperations;

        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            KeyCode::Char('s') => {
                self.handle_task_search();
            }
            KeyCode::Char('c') => {
                self.handle_commit_preview();
            }
            KeyCode::Char('t') => {
                self.handle_task_management_toggle();
            }
            KeyCode::Char('r') => {
                if !matches!(self.current_state, AppState::Loading) && self.gemini_analysis_state.is_none() {
                    use crate::app::background_operations::BackgroundOperations;
                    self.start_gemini_analysis_wrapper().await;
                }
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
            KeyCode::Char('i') => {
                self.generate_release_notes_internal_wrapper().await?;
            }
            KeyCode::Char('o') => {
                self.generate_release_notes_with_npm_wrapper().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_task_search_screen(&mut self, key: KeyCode) -> Result<()> {
        use crate::app::task_operations::TaskOperations;

        // Check if we're in search input mode (typing in the search box)
        let in_search_input = self.ui_state.input_mode == InputMode::Editing;
        
        // Debug: log key events and current mode
        self.message = Some(format!("DEBUG: Key: {:?}, Mode: {:?}, Input: '{}'", key, self.ui_state.input_mode, self.ui_state.current_input));
        
        if in_search_input {
            self.handle_search_input_mode(key).await?;
        } else {
            self.handle_search_navigation_mode(key).await?;
        }
        Ok(())
    }

    async fn handle_task_selection_screen(&mut self, key: KeyCode) -> Result<()> {
        use crate::app::task_operations::TaskOperations;

        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::TaskSearch;
            }
            KeyCode::Up => {
                if self.ui_state.selected_tab > 0 {
                    self.ui_state.selected_tab -= 1;
                }
            }
            KeyCode::Down => {
                if self.ui_state.selected_tab < self.tasks.len().saturating_sub(1) {
                    self.ui_state.selected_tab += 1;
                }
            }
            KeyCode::Char(' ') => {
                self.toggle_task_selection();
            }
            KeyCode::Enter => {
                self.confirm_task_selection();
            }
            _ => {}
        }
        Ok(())
    }

    // Helper methods for commit screen handling
    fn handle_task_search(&mut self) {
        self.current_screen = AppScreen::TaskSearch;
        self.ui_state.current_input.clear();
        self.tasks.clear();
        self.ui_state.selected_tab = 0;
        self.message = Some("Search screen - Press 'i' or '/' to start typing".to_string());
    }

    fn handle_commit_preview(&mut self) {
        use crate::app::commit_operations::CommitOperations;
        self.preview_commit_message = self.build_commit_message();
        self.current_screen = AppScreen::CommitPreview;
        self.ui_state.input_mode = InputMode::Editing;
        self.ui_state.current_input = self.preview_commit_message.clone();
        self.ui_state.cursor_position = self.preview_commit_message.len();
        self.message = Some("Review and edit your commit message. Press Ctrl+C to commit, Esc to cancel".to_string());
    }

    fn handle_task_management_toggle(&mut self) {
        self.ui_state.task_management_mode = !self.ui_state.task_management_mode;
        if self.ui_state.task_management_mode {
            self.ui_state.selected_tab = 0;
            self.message = Some("Task management mode ON - Use ↑↓ to navigate, Delete/Space to remove tasks".to_string());
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
            if !self.selected_tasks.is_empty() && self.ui_state.selected_tab > 0 {
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
                    if !self.selected_tasks.is_empty() && self.ui_state.selected_tab > 0 {
                        self.ui_state.selected_tab -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_down_navigation(&mut self) {
        if self.ui_state.task_management_mode {
            if !self.selected_tasks.is_empty() && self.ui_state.selected_tab < self.selected_tasks.len().saturating_sub(1) {
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
                    if !self.selected_tasks.is_empty() && self.ui_state.selected_tab < self.selected_tasks.len().saturating_sub(1) {
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
        use crate::app::task_operations::TaskOperations;
        if (self.ui_state.task_management_mode || self.ui_state.current_field == CommitField::SelectedTasks) 
            && !self.selected_tasks.is_empty() 
            && self.ui_state.selected_tab < self.selected_tasks.len() {
            
            self.selected_tasks.remove(self.ui_state.selected_tab);
            
            if self.ui_state.selected_tab >= self.selected_tasks.len() && !self.selected_tasks.is_empty() {
                self.ui_state.selected_tab = self.selected_tasks.len() - 1;
            }
            
            self.update_task_selection();
            self.message = Some("Task removed from selection".to_string());
        }
    }

    fn enter_edit_mode_if_text_field(&mut self) {
        if !matches!(self.ui_state.current_field, CommitField::Type | CommitField::SelectedTasks) {
            self.ui_state.input_mode = InputMode::Editing;
            self.load_current_field_content();
            self.ui_state.cursor_position = self.ui_state.current_input.len();
        } else {
            self.ui_state.input_mode = InputMode::Normal;
            self.ui_state.current_input.clear();
        }
    }

    fn load_current_field_content(&mut self) {
        self.ui_state.current_input = match self.ui_state.current_field {
            CommitField::Scope => self.commit_form.scope.clone(),
            CommitField::Title => self.commit_form.title.clone(),
            CommitField::Description => self.commit_form.description.clone(),
            CommitField::BreakingChange => self.commit_form.breaking_change.clone(),
            CommitField::TestDetails => self.commit_form.test_details.clone(),
            CommitField::Security => self.commit_form.security.clone(),
            CommitField::MigracionesLentas => self.commit_form.migraciones_lentas.clone(),
            CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar.clone(),
            _ => String::new(),
        };
    }

    pub fn save_current_field(&mut self) {
        match self.ui_state.current_field {
            CommitField::Scope => {
                self.commit_form.scope = self.ui_state.current_input.clone();
            }
            CommitField::Title => {
                self.commit_form.title = self.ui_state.current_input.clone();
            }
            CommitField::Description => {
                self.commit_form.description = self.ui_state.current_input.clone();
            }
            CommitField::BreakingChange => {
                self.commit_form.breaking_change = self.ui_state.current_input.clone();
            }
            CommitField::TestDetails => {
                self.commit_form.test_details = self.ui_state.current_input.clone();
            }
            CommitField::Security => {
                self.commit_form.security = self.ui_state.current_input.clone();
            }
            CommitField::MigracionesLentas => {
                self.commit_form.migraciones_lentas = self.ui_state.current_input.clone();
            }
            CommitField::PartesAEjecutar => {
                self.commit_form.partes_a_ejecutar = self.ui_state.current_input.clone();
            }
            _ => {}
        }
    }
} 