use ratatui::style::{Color, Style};
use tui_textarea::TextArea;

#[derive(Debug)]
pub struct UIState {
    pub selected_tab: usize,
    pub selected_commit_type: usize,
    pub input_mode: InputMode,
    pub current_field: CommitField,
    pub focused_search_index: usize,
    pub task_management_mode: bool,
    pub animation_frame: usize,
    pub scroll_offset: usize,
    // TextArea instances for each editable field
    pub scope_textarea: TextArea<'static>,
    pub title_textarea: TextArea<'static>,
    pub description_textarea: TextArea<'static>,
    pub breaking_change_textarea: TextArea<'static>,
    pub test_details_textarea: TextArea<'static>,
    pub security_textarea: TextArea<'static>,
    pub migraciones_lentas_textarea: TextArea<'static>,
    pub partes_a_ejecutar_textarea: TextArea<'static>,
    pub search_textarea: TextArea<'static>,
    pub commit_preview_textarea: TextArea<'static>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommitField {
    Type,
    Scope,
    Title,
    Description,
    BreakingChange,
    TestDetails,
    Security,
    MigracionesLentas,
    PartesAEjecutar,
    SelectedTasks,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            selected_tab: 0,
            selected_commit_type: 0,
            input_mode: InputMode::Normal,
            current_field: CommitField::Type,
            focused_search_index: 0,
            task_management_mode: false,
            animation_frame: 0,
            scroll_offset: 0,
            scope_textarea: create_single_line_textarea("Enter scope (e.g., auth, ui, api)..."),
            title_textarea: create_single_line_textarea("Enter commit title..."),
            description_textarea: create_multiline_textarea(
                "Press 't' for comprehensive AI analysis, or type manually...",
            ),
            breaking_change_textarea: create_single_line_textarea(
                "Enter breaking change details (if any)...",
            ),
            test_details_textarea: create_multiline_textarea(
                "Enter test details manually, or press 't' for AI analysis...",
            ),
            security_textarea: create_multiline_textarea("Enter security info (or NA)..."),
            migraciones_lentas_textarea: create_multiline_textarea("Enter migraciones lentas..."),
            partes_a_ejecutar_textarea: create_multiline_textarea("Enter partes a ejecutar..."),
            search_textarea: create_single_line_textarea("Search tasks..."),
            commit_preview_textarea: create_multiline_textarea(""),
        }
    }
}

// Helper functions for creating TextArea instances
fn create_single_line_textarea(placeholder: &str) -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_placeholder_text(placeholder);
    textarea.set_tab_length(4);
    textarea
}

fn create_multiline_textarea(placeholder: &str) -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_placeholder_text(placeholder);
    textarea.set_tab_length(4);
    textarea
}

// Helper functions for commit screen styling
impl UIState {
    pub fn get_field_border_style(&self, field: &CommitField) -> Style {
        if self.current_field == *field {
            if self.input_mode == InputMode::Editing {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            }
        } else {
            Style::default()
        }
    }

    pub fn get_textarea_mut(&mut self, field: &CommitField) -> Option<&mut TextArea<'static>> {
        match field {
            CommitField::Scope => Some(&mut self.scope_textarea),
            CommitField::Title => Some(&mut self.title_textarea),
            CommitField::Description => Some(&mut self.description_textarea),
            CommitField::BreakingChange => Some(&mut self.breaking_change_textarea),
            CommitField::TestDetails => Some(&mut self.test_details_textarea),
            CommitField::Security => Some(&mut self.security_textarea),
            CommitField::MigracionesLentas => Some(&mut self.migraciones_lentas_textarea),
            CommitField::PartesAEjecutar => Some(&mut self.partes_a_ejecutar_textarea),
            _ => None,
        }
    }

    pub fn get_textarea(&self, field: &CommitField) -> Option<&TextArea<'static>> {
        match field {
            CommitField::Scope => Some(&self.scope_textarea),
            CommitField::Title => Some(&self.title_textarea),
            CommitField::Description => Some(&self.description_textarea),
            CommitField::BreakingChange => Some(&self.breaking_change_textarea),
            CommitField::TestDetails => Some(&self.test_details_textarea),
            CommitField::Security => Some(&self.security_textarea),
            CommitField::MigracionesLentas => Some(&self.migraciones_lentas_textarea),
            CommitField::PartesAEjecutar => Some(&self.partes_a_ejecutar_textarea),
            _ => None,
        }
    }

    pub fn is_multiline_field(field: &CommitField) -> bool {
        matches!(
            field,
            CommitField::Description
                | CommitField::TestDetails
                | CommitField::Security
                | CommitField::MigracionesLentas
                | CommitField::PartesAEjecutar
        )
    }

    pub fn update_textarea_styles(&mut self) {
        // Store current field and input mode to avoid borrowing issues
        let current_field = self.current_field.clone();
        let input_mode = self.input_mode.clone();

        // Update all textarea styles based on current field and input mode
        for field in [
            CommitField::Scope,
            CommitField::Title,
            CommitField::Description,
            CommitField::BreakingChange,
            CommitField::TestDetails,
            CommitField::Security,
            CommitField::MigracionesLentas,
            CommitField::PartesAEjecutar,
        ] {
            if let Some(textarea) = self.get_textarea_mut(&field) {
                if current_field == field && input_mode == InputMode::Editing {
                    textarea.set_cursor_line_style(Style::default().bg(Color::DarkGray));
                    textarea.set_cursor_style(Style::default().bg(Color::Yellow));
                } else {
                    textarea.set_cursor_line_style(Style::default());
                    textarea.set_cursor_style(Style::default());
                }

                // Update block style - we'll do this separately to avoid borrowing issues
            }
        }

        // Update block styles in a separate loop
        for field in [
            CommitField::Scope,
            CommitField::Title,
            CommitField::Description,
            CommitField::BreakingChange,
            CommitField::TestDetails,
            CommitField::Security,
            CommitField::MigracionesLentas,
            CommitField::PartesAEjecutar,
        ] {
            let border_style = self.get_field_border_style(&field);
            if let Some(textarea) = self.get_textarea_mut(&field) {
                let block = ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(border_style);
                textarea.set_block(block);
            }
        }
    }
}
