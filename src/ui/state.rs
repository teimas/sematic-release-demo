use ratatui::{
    style::{Color, Modifier, Style},
    widgets::ListState,
};

pub struct UIState {
    pub selected_tab: usize,
    #[allow(dead_code)]
    pub list_state: ListState,
    pub selected_commit_type: usize,
    pub input_mode: InputMode,
    pub current_input: String,
    pub current_field: CommitField,
    #[allow(dead_code)]
    pub scroll_offset: u16,
    pub cursor_position: usize,
    pub focused_search_index: usize,
    pub task_management_mode: bool,
    pub animation_frame: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    #[allow(dead_code)]
    Selecting,
}

#[derive(Debug, Clone, PartialEq)]
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
            list_state: ListState::default(),
            selected_commit_type: 0,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            current_field: CommitField::Type,
            scroll_offset: 0,
            cursor_position: 0,
            focused_search_index: 0,
            task_management_mode: false,
            animation_frame: 0,
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
} 