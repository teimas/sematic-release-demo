use ratatui::style::{Color, Modifier, Style};
use std::collections::HashMap;

pub struct UIState {
    pub selected_tab: usize,
    pub selected_commit_type: usize,
    pub input_mode: InputMode,
    pub current_input: String,
    pub current_field: CommitField,
    pub cursor_position: usize,
    pub focused_search_index: usize,
    pub task_management_mode: bool,
    pub animation_frame: usize,
    // Field-specific scroll offsets for multiline fields
    pub field_scroll_offsets: HashMap<CommitField, u16>,
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
            current_input: String::new(),
            current_field: CommitField::Type,
            cursor_position: 0,
            focused_search_index: 0,
            task_management_mode: false,
            animation_frame: 0,
            field_scroll_offsets: HashMap::new(),
        }
    }
}

// Helper functions for commit screen styling
impl UIState {
    pub fn get_field_style(&self, field: &CommitField) -> Style {
        if self.current_field == *field {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    }

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

    pub fn get_field_scroll_offset(&self, field: &CommitField) -> u16 {
        self.field_scroll_offsets.get(field).copied().unwrap_or(0)
    }

    pub fn set_field_scroll_offset(&mut self, field: CommitField, offset: u16) {
        self.field_scroll_offsets.insert(field, offset);
    }

    pub fn is_multiline_field(field: &CommitField) -> bool {
        matches!(
            field,
            CommitField::Description | CommitField::TestDetails | CommitField::Security | CommitField::MigracionesLentas | CommitField::PartesAEjecutar
        )
    }
} 