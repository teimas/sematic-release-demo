use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use log::{debug, info, error};

use crate::{
    app::App,
    types::{AppScreen, AppState},
    ui::{InputMode, CommitField},
};

impl App {
    pub async fn handle_input_mode(&mut self, key: KeyEvent) -> Result<()> {
        // Handle CommitPreview screen differently - it's just a text editor
        if self.current_screen == AppScreen::CommitPreview {
            return self.handle_commit_preview_text_editing(key).await;
        }

        match key.code {
            KeyCode::Esc => {
                // Cancel editing without saving
                self.ui_state.input_mode = InputMode::Normal;
                self.ui_state.current_input.clear();
            }
            KeyCode::Enter => {
                self.handle_enter_in_input_mode().await?;
            }
            KeyCode::Tab => {
                self.handle_tab_in_input_mode();
            }
            KeyCode::BackTab => {
                self.handle_back_tab_in_input_mode();
            }
            KeyCode::Up => {
                if self.is_multiline_field() {
                    self.move_cursor_up();
                }
            }
            KeyCode::Down => {
                if self.is_multiline_field() {
                    self.move_cursor_down();
                }
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            KeyCode::Home => {
                self.move_cursor_to_line_start();
            }
            KeyCode::End => {
                self.move_cursor_to_line_end();
            }
            KeyCode::Char(c) => {
                self.insert_character(c);
            }
            KeyCode::Backspace => {
                self.delete_character_before_cursor();
            }
            KeyCode::Delete => {
                self.delete_character_at_cursor();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_enter_in_input_mode(&mut self) -> Result<()> {
        use crate::app::task_operations::TaskOperations;

        // Special handling for TaskSearch screen - trigger search
        if self.current_screen == AppScreen::TaskSearch {
            self.message = Some(format!("DEBUG: Starting search with query: '{}'", self.ui_state.current_input));
            if !self.ui_state.current_input.is_empty() {
                self.current_state = AppState::Loading;
                match self.search_monday_tasks(&self.ui_state.current_input).await {
                    Ok(tasks) => {
                        self.tasks = tasks;
                        self.ui_state.selected_tab = 0;
                        self.current_state = AppState::Normal;
                        self.ui_state.input_mode = InputMode::Normal;
                        self.message = Some(format!("DEBUG: Search completed! Found {} tasks", self.tasks.len()));
                    }
                    Err(e) => {
                        self.current_state = AppState::Error(format!("DEBUG: Search failed: {}", e));
                    }
                }
            } else {
                self.message = Some("DEBUG: Search query is empty".to_string());
            }
        } else {
            debug!("Enter pressed, is_multiline: {}", self.is_multiline_field());
            
            if self.is_multiline_field() {
                debug!("Adding newline in multiline field");
                self.ui_state.current_input.push('\n');
                self.ui_state.cursor_position = self.ui_state.current_input.len();
                self.update_scroll_for_cursor();
            } else {
                debug!("Enter ignored in single-line field - use Tab to navigate");
            }
        }
        Ok(())
    }

    fn handle_tab_in_input_mode(&mut self) {
        self.save_current_field();
        self.ui_state.current_input.clear();
        
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

    fn handle_back_tab_in_input_mode(&mut self) {
        self.save_current_field();
        self.ui_state.current_input.clear();
        
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

    fn enter_edit_mode_if_text_field_input(&mut self) {
        if !matches!(self.ui_state.current_field, CommitField::Type | CommitField::SelectedTasks) {
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
            self.ui_state.cursor_position = self.ui_state.current_input.len();
        } else {
            self.ui_state.input_mode = InputMode::Normal;
        }
    }

    pub async fn handle_commit_preview_text_editing(&mut self, key: KeyEvent) -> Result<()> {
        use crate::app::commit_operations::CommitOperations;

        // Check for Ctrl+C first (before general character handling)
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            self.preview_commit_message = self.ui_state.current_input.clone();
            if let Err(e) = self.create_commit_with_message(&self.preview_commit_message).await {
                self.current_state = AppState::Error(e.to_string());
            } else {
                self.message = Some("Commit created successfully!".to_string());
                self.current_screen = AppScreen::Main;
                self.ui_state.input_mode = InputMode::Normal;
                self.ui_state.current_input.clear();
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                self.current_screen = AppScreen::Commit;
                self.ui_state.input_mode = InputMode::Normal;
                self.ui_state.current_input.clear();
                self.message = Some("Commit cancelled".to_string());
            }
            KeyCode::Enter => {
                self.ui_state.current_input.push('\n');
                self.ui_state.cursor_position = self.ui_state.current_input.len();
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            KeyCode::Up => {
                self.move_cursor_up();
            }
            KeyCode::Down => {
                self.move_cursor_down();
            }
            KeyCode::Home => {
                self.move_cursor_to_line_start();
            }
            KeyCode::End => {
                self.move_cursor_to_line_end();
            }
            KeyCode::Char(c) => {
                self.insert_character(c);
            }
            KeyCode::Backspace => {
                self.delete_character_before_cursor();
            }
            KeyCode::Delete => {
                self.delete_character_at_cursor();
            }
            _ => {}
        }
        Ok(())
    }

    // Text manipulation methods
    fn move_cursor_left(&mut self) {
        let text = &self.ui_state.current_input;
        if self.ui_state.cursor_position > 0 {
            let mut new_pos = self.ui_state.cursor_position.saturating_sub(1);
            while new_pos > 0 && !text.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.ui_state.cursor_position = new_pos;
            self.update_scroll_for_cursor();
        }
    }

    fn move_cursor_right(&mut self) {
        let text = &self.ui_state.current_input;
        if self.ui_state.cursor_position < text.len() {
            let mut new_pos = self.ui_state.cursor_position + 1;
            while new_pos < text.len() && !text.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.ui_state.cursor_position = new_pos.min(text.len());
            self.update_scroll_for_cursor();
        }
    }

    fn move_cursor_up(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        let text_before_cursor = &text[..cursor_pos];
        let current_line_start = text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
        let current_column = cursor_pos - current_line_start;
        
        if current_line_start > 0 {
            let prev_line_end = current_line_start - 1;
            let text_before_prev_line = &text[..prev_line_end];
            let prev_line_start = text_before_prev_line.rfind('\n').map_or(0, |pos| pos + 1);
            let prev_line_length = prev_line_end - prev_line_start;
            
            let new_column = current_column.min(prev_line_length);
            self.ui_state.cursor_position = prev_line_start + new_column;
            self.update_scroll_for_cursor();
        }
    }

    fn move_cursor_down(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        let text_before_cursor = &text[..cursor_pos];
        let current_line_start = text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
        let current_column = cursor_pos - current_line_start;
        
        let text_after_cursor = &text[cursor_pos..];
        if let Some(current_line_end_offset) = text_after_cursor.find('\n') {
            let current_line_end = cursor_pos + current_line_end_offset;
            let next_line_start = current_line_end + 1;
            
            if next_line_start < text.len() {
                let text_after_next_line = &text[next_line_start..];
                let next_line_end = text_after_next_line.find('\n')
                    .map_or(text.len(), |pos| next_line_start + pos);
                let next_line_length = next_line_end - next_line_start;
                
                let new_column = current_column.min(next_line_length);
                self.ui_state.cursor_position = next_line_start + new_column;
                self.update_scroll_for_cursor();
            }
        }
    }

    fn move_cursor_to_line_start(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        let text_before_cursor = &text[..cursor_pos];
        let line_start = text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
        self.ui_state.cursor_position = line_start;
        self.update_scroll_for_cursor();
    }

    fn move_cursor_to_line_end(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        let text_after_cursor = &text[cursor_pos..];
        let line_end = text_after_cursor.find('\n')
            .map_or(text.len(), |pos| cursor_pos + pos);
        self.ui_state.cursor_position = line_end;
        self.update_scroll_for_cursor();
    }

    fn insert_character(&mut self, c: char) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        if text.is_char_boundary(cursor_pos) {
            self.ui_state.current_input.insert(cursor_pos, c);
            self.ui_state.cursor_position = cursor_pos + c.len_utf8();
        } else {
            self.ui_state.current_input.push(c);
            self.ui_state.cursor_position = self.ui_state.current_input.len();
        }
        self.update_scroll_for_cursor();
    }

    fn delete_character_before_cursor(&mut self) {
        let text = &self.ui_state.current_input;
        if self.ui_state.cursor_position > 0 {
            let cursor_pos = self.ui_state.cursor_position.min(text.len());
            
            let mut prev_pos = cursor_pos.saturating_sub(1);
            while prev_pos > 0 && !text.is_char_boundary(prev_pos) {
                prev_pos -= 1;
            }
            
            if text.is_char_boundary(prev_pos) {
                self.ui_state.current_input.remove(prev_pos);
                self.ui_state.cursor_position = prev_pos;
                self.update_scroll_for_cursor();
            }
        }
    }

    fn delete_character_at_cursor(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        if cursor_pos < text.len() && text.is_char_boundary(cursor_pos) {
            self.ui_state.current_input.remove(cursor_pos);
            self.update_scroll_for_cursor();
        }
    }

    fn is_multiline_field(&self) -> bool {
        crate::ui::state::UIState::is_multiline_field(&self.ui_state.current_field)
    }

    fn update_scroll_for_cursor(&mut self) {
        if self.is_multiline_field() {
            let field_width = 80; // Approximate field width, this could be dynamic
            let visible_height = 3; // Height of multiline fields minus borders
            let current_scroll = self.ui_state.get_field_scroll_offset(&self.ui_state.current_field);
            
            let new_scroll = crate::ui::scrollable_text::calculate_required_scroll(
                &self.ui_state.current_input,
                self.ui_state.cursor_position,
                field_width,
                visible_height,
                current_scroll,
            );
            
            self.ui_state.set_field_scroll_offset(self.ui_state.current_field.clone(), new_scroll);
        }
    }

    // Search input handling
    pub async fn handle_search_input_mode(&mut self, key: KeyCode) -> Result<()> {
        use crate::app::task_operations::TaskOperations;

        match key {
            KeyCode::Esc => {
                self.ui_state.input_mode = InputMode::Normal;
                self.message = Some("Exited search mode".to_string());
            }
            KeyCode::Enter => {
                self.message = Some(format!("DEBUG: Starting search with query: '{}'", self.ui_state.current_input));
                if !self.ui_state.current_input.is_empty() {
                    self.current_state = AppState::Loading;
                    match self.search_monday_tasks(&self.ui_state.current_input).await {
                        Ok(tasks) => {
                            self.tasks = tasks;
                            self.ui_state.selected_tab = 0;
                            self.current_state = AppState::Normal;
                            self.ui_state.input_mode = InputMode::Normal;
                            self.message = Some(format!("DEBUG: Search completed! Found {} tasks", self.tasks.len()));
                        }
                        Err(e) => {
                            self.current_state = AppState::Error(format!("DEBUG: Search failed: {}", e));
                        }
                    }
                } else {
                    self.message = Some("DEBUG: Search query is empty".to_string());
                }
            }
            KeyCode::Char(c) => {
                self.ui_state.current_input.push(c);
            }
            KeyCode::Backspace => {
                self.ui_state.current_input.pop();
            }
            _ => {}
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
                self.tasks.clear();
                self.ui_state.current_input.clear();
                self.ui_state.focused_search_index = 0;
                self.message = Some("Search cleared".to_string());
            }
            KeyCode::Char('i') | KeyCode::Char('/') => {
                self.ui_state.input_mode = InputMode::Editing;
                self.message = Some("DEBUG: Entered edit mode - you can now type".to_string());
            }
            KeyCode::Enter => {
                if !self.ui_state.current_input.is_empty() {
                    self.current_state = AppState::Loading;
                    match self.search_monday_tasks(&self.ui_state.current_input).await {
                        Ok(tasks) => {
                            self.tasks = tasks;
                            self.ui_state.selected_tab = 0;
                            self.current_state = AppState::Normal;
                            self.message = Some(format!("Found {} tasks", self.tasks.len()));
                        }
                        Err(e) => {
                            self.current_state = AppState::Error(e.to_string());
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
                self.ui_state.current_input.clear();
                self.tasks.clear();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_up_navigation(&mut self) {
        if !self.tasks.is_empty() {
            if self.ui_state.focused_search_index > 0 {
                self.ui_state.focused_search_index -= 1;
            }
        } else if !self.selected_tasks.is_empty() {
            if self.ui_state.selected_tab > 0 {
                self.ui_state.selected_tab -= 1;
            }
        }
    }

    fn handle_search_down_navigation(&mut self) {
        if !self.tasks.is_empty() {
            if self.ui_state.focused_search_index < self.tasks.len().saturating_sub(1) {
                self.ui_state.focused_search_index += 1;
            }
        } else if !self.selected_tasks.is_empty() {
            if self.ui_state.selected_tab < self.selected_tasks.len().saturating_sub(1) {
                self.ui_state.selected_tab += 1;
            }
        }
    }

    fn handle_search_task_removal(&mut self) {
        use crate::app::task_operations::TaskOperations;

        if !self.tasks.is_empty() {
            if let Some(task) = self.tasks.get(self.ui_state.focused_search_index) {
                if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                    self.selected_tasks.remove(pos);
                    self.update_task_selection();
                    self.message = Some("Task removed from selection".to_string());
                } else {
                    self.message = Some("Task is not selected".to_string());
                }
            }
        } else if !self.selected_tasks.is_empty() && self.ui_state.selected_tab < self.selected_tasks.len() {
            self.selected_tasks.remove(self.ui_state.selected_tab);
            
            if self.ui_state.selected_tab >= self.selected_tasks.len() && !self.selected_tasks.is_empty() {
                self.ui_state.selected_tab = self.selected_tasks.len() - 1;
            }
            
            self.update_task_selection();
            self.message = Some("Task removed from selection".to_string());
        }
    }

    fn handle_search_task_toggle(&mut self) {
        use crate::app::task_operations::TaskOperations;

        if !self.tasks.is_empty() {
            if let Some(task) = self.tasks.get(self.ui_state.focused_search_index) {
                if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                    self.selected_tasks.remove(pos);
                    self.message = Some("Task deselected".to_string());
                } else {
                    self.selected_tasks.push(task.clone());
                    self.message = Some("Task selected".to_string());
                }
                self.update_task_selection();
            }
        } else if !self.selected_tasks.is_empty() {
            if self.ui_state.selected_tab < self.selected_tasks.len() {
                self.selected_tasks.remove(self.ui_state.selected_tab);
                
                if self.ui_state.selected_tab >= self.selected_tasks.len() && !self.selected_tasks.is_empty() {
                    self.ui_state.selected_tab = self.selected_tasks.len() - 1;
                }
                
                self.update_task_selection();
                self.message = Some("Task removed from selection".to_string());
            }
        }
    }

    fn handle_numeric_task_selection(&mut self, c: char) {
        use crate::app::task_operations::TaskOperations;

        let index = if c == '0' { 9 } else { (c as usize) - ('1' as usize) };
        if let Some(task) = self.tasks.get(index) {
            if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                self.selected_tasks.remove(pos);
            } else {
                self.selected_tasks.push(task.clone());
            }
            self.update_task_selection();
        }
    }
} 