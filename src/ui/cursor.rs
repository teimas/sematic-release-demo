use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::ui::state::{UIState, CommitField};

pub fn set_search_cursor_position(f: &mut Frame, area: Rect, ui_state: &UIState) {
    // Calculate the layout chunks to match the task search screen layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(5),    // Search results
            Constraint::Length(6), // Selected tasks
        ])
        .split(area);

    // Position cursor in the search input field
    let search_area = chunks[0];
    let search_text = format!("Search: {}", ui_state.current_input);
    
    // Calculate cursor position within the search field
    let cursor_x = search_area.x + 1 + search_text.len() as u16;
    let cursor_y = search_area.y + 1;
    
    // Set cursor position and make it visible
    f.set_cursor_position((cursor_x, cursor_y));
}

pub fn set_cursor_position(f: &mut Frame, area: Rect, ui_state: &UIState) {
    // Calculate the layout chunks to match the commit screen layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Commit type
            Constraint::Length(3),  // Scope
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Description (multiline)
            Constraint::Length(3),  // Breaking change
            Constraint::Length(5),  // Test details (multiline)
            Constraint::Length(5),  // Security (multiline)
            Constraint::Length(5),  // Migraciones lentas (multiline)
            Constraint::Length(5),  // Partes a ejecutar (multiline)
            Constraint::Length(3),  // Instructions
            Constraint::Min(0),     // Selected tasks
        ])
        .split(area);

    // Find the current field's area and calculate cursor position
    let field_area = match ui_state.current_field {
        CommitField::Type => return, // Type field doesn't need cursor (it's a list)
        CommitField::Scope => chunks[1],
        CommitField::Title => chunks[2],
        CommitField::Description => chunks[3],
        CommitField::BreakingChange => chunks[4],
        CommitField::TestDetails => chunks[5],
        CommitField::Security => chunks[6],
        CommitField::MigracionesLentas => chunks[7],
        CommitField::PartesAEjecutar => chunks[8],
        CommitField::SelectedTasks => return, // Selected tasks field doesn't need cursor (it's a list)
    };

    // Calculate cursor position within the field
    let (cursor_x, cursor_y) = calculate_cursor_position(&ui_state.current_input, ui_state.cursor_position, field_area.width.saturating_sub(2));
    
    // Account for scroll offset in multiline fields
    let scroll_offset = if crate::ui::state::UIState::is_multiline_field(&ui_state.current_field) {
        ui_state.get_field_scroll_offset(&ui_state.current_field)
    } else {
        0
    };
    
    // Position cursor within the field (accounting for border and scroll)
    let cursor_x = field_area.x + 1 + cursor_x;
    let cursor_y = field_area.y + 1 + cursor_y.saturating_sub(scroll_offset);
    
    // Set cursor position and make it visible
    f.set_cursor_position((cursor_x, cursor_y));
}

pub fn set_commit_preview_cursor_position(f: &mut Frame, area: Rect, ui_state: &UIState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Instructions
            Constraint::Min(0),     // Commit message editor
        ])
        .split(area);

    // Position cursor in the commit message editor field
    let editor_area = chunks[1];
    
    // Calculate cursor position within the multiline editor
    let (cursor_x, cursor_y) = calculate_cursor_position(&ui_state.current_input, ui_state.cursor_position, editor_area.width.saturating_sub(2));
    
    // Account for scroll offset (commit preview scrolls like a description field)
    let scroll_offset = ui_state.get_field_scroll_offset(&CommitField::Description);
    
    // Position cursor within the editor field (accounting for border and scroll)
    let cursor_x = editor_area.x + 1 + cursor_x;
    let cursor_y = editor_area.y + 1 + cursor_y.saturating_sub(scroll_offset);
    
    // Set cursor position and make it visible
    f.set_cursor_position((cursor_x, cursor_y));
}

pub fn calculate_cursor_position(input: &str, cursor_pos: usize, field_width: u16) -> (u16, u16) {
    let text_chars: Vec<char> = input.chars().collect();
    let cursor_pos = cursor_pos.min(text_chars.len());
    
    let mut x = 0u16;
    let mut y = 0u16;
    
    for (i, &ch) in text_chars.iter().enumerate() {
        if i >= cursor_pos {
            break;
        }
        
        if ch == '\n' {
            x = 0;
            y += 1;
        } else {
            x += 1;
            if x >= field_width {
                x = 0;
                y += 1;
            }
        }
    }
    
    (x, y)
} 