use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::ui::state::{UIState, CommitField, InputMode};
use crate::ui::scrollable_text::render_scrollable_text;
use crate::types::{CommitType, CommitForm};

pub fn draw_commit_screen(f: &mut Frame, area: Rect, ui_state: &UIState, commit_form: &CommitForm) {
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

    // Use helper methods from UIState for consistent styling

    // Commit Type Selection
    let commit_types: Vec<ListItem> = CommitType::all()
        .iter()
        .enumerate()
        .map(|(i, ct)| {
            let style = if Some(ct) == commit_form.commit_type.as_ref() {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if i == ui_state.selected_commit_type && ui_state.current_field == CommitField::Type {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}: {}", ct.as_str(), ct.description())).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    if ui_state.current_field == CommitField::Type {
        list_state.select(Some(ui_state.selected_commit_type));
    }

    let commit_type_list = List::new(commit_types)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Commit Type (↑↓ to select, Tab to move)")
            .border_style(ui_state.get_field_border_style(&CommitField::Type)));

    f.render_stateful_widget(commit_type_list, chunks[0], &mut list_state);

    // Scope
    let scope_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::Scope {
        &ui_state.current_input
    } else if commit_form.scope.is_empty() {
        "Enter scope (e.g., auth, ui, api)..."
    } else {
        &commit_form.scope
    };
    let scope = Paragraph::new(scope_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Scope (auto-edit on Tab)")
            .border_style(ui_state.get_field_border_style(&CommitField::Scope)))
        .style(ui_state.get_field_style(&CommitField::Scope));
    f.render_widget(scope, chunks[1]);

    // Title
    let title_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::Title {
        &ui_state.current_input
    } else if commit_form.title.is_empty() {
        "Enter commit title..."
    } else {
        &commit_form.title
    };
    let title = Paragraph::new(title_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Title")
            .border_style(ui_state.get_field_border_style(&CommitField::Title)))
        .style(ui_state.get_field_style(&CommitField::Title));
    f.render_widget(title, chunks[2]);

    // Description (multiline)
    let desc_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::Description {
        &ui_state.current_input
    } else if commit_form.description.is_empty() {
        "Press 'r' to generate AI description, or type manually... (Enter for new line)"
    } else {
        &commit_form.description
    };
    
    let desc_block = Block::default()
        .borders(Borders::ALL)
        .title("Description (multiline, press 'r' to generate)")
        .border_style(ui_state.get_field_border_style(&CommitField::Description));
    let desc_style = ui_state.get_field_style(&CommitField::Description);
    
    render_scrollable_text(f, chunks[3], desc_text, &CommitField::Description, ui_state, desc_block, desc_style);

    // Breaking Change
    let breaking_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::BreakingChange {
        &ui_state.current_input
    } else if commit_form.breaking_change.is_empty() {
        "Enter breaking change details (if any)..."
    } else {
        &commit_form.breaking_change
    };
    let breaking = Paragraph::new(breaking_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Breaking Change")
            .border_style(ui_state.get_field_border_style(&CommitField::BreakingChange)))
        .style(ui_state.get_field_style(&CommitField::BreakingChange));
    f.render_widget(breaking, chunks[4]);

    // Test Details (multiline)
    let test_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::TestDetails {
        &ui_state.current_input
    } else if commit_form.test_details.is_empty() {
        "Enter test details... (Enter for new line)"
    } else {
        &commit_form.test_details
    };
    
    let test_block = Block::default()
        .borders(Borders::ALL)
        .title("Test Details (multiline)")
        .border_style(ui_state.get_field_border_style(&CommitField::TestDetails));
    let test_style = ui_state.get_field_style(&CommitField::TestDetails);
    
    render_scrollable_text(f, chunks[5], test_text, &CommitField::TestDetails, ui_state, test_block, test_style);

    // Security (multiline)
    let security_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::Security {
        &ui_state.current_input
    } else if commit_form.security.is_empty() {
        "Enter security info (or NA)... (Enter for new line)"
    } else {
        &commit_form.security
    };
    
    let security_block = Block::default()
        .borders(Borders::ALL)
        .title("Security (multiline)")
        .border_style(ui_state.get_field_border_style(&CommitField::Security));
    let security_style = ui_state.get_field_style(&CommitField::Security);
    
    render_scrollable_text(f, chunks[6], security_text, &CommitField::Security, ui_state, security_block, security_style);

    // Migraciones Lentas (multiline)
    let migraciones_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::MigracionesLentas {
        &ui_state.current_input
    } else if commit_form.migraciones_lentas.is_empty() {
        "Enter slow migrations details... (Enter for new line)"
    } else {
        &commit_form.migraciones_lentas
    };
    
    let migraciones_block = Block::default()
        .borders(Borders::ALL)
        .title("Migraciones Lentas (multiline)")
        .border_style(ui_state.get_field_border_style(&CommitField::MigracionesLentas));
    let migraciones_style = ui_state.get_field_style(&CommitField::MigracionesLentas);
    
    render_scrollable_text(f, chunks[7], migraciones_text, &CommitField::MigracionesLentas, ui_state, migraciones_block, migraciones_style);

    // Partes a Ejecutar (multiline)
    let partes_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::PartesAEjecutar {
        &ui_state.current_input
    } else if commit_form.partes_a_ejecutar.is_empty() {
        "Enter parts to execute... (Enter for new line)"
    } else {
        &commit_form.partes_a_ejecutar
    };
    
    let partes_block = Block::default()
        .borders(Borders::ALL)
        .title("Partes a Ejecutar (multiline)")
        .border_style(ui_state.get_field_border_style(&CommitField::PartesAEjecutar));
    let partes_style = ui_state.get_field_style(&CommitField::PartesAEjecutar);
    
    render_scrollable_text(f, chunks[8], partes_text, &CommitField::PartesAEjecutar, ui_state, partes_block, partes_style);

    // Instructions
    let instructions = if ui_state.input_mode == InputMode::Editing {
        if matches!(ui_state.current_field, CommitField::Description | CommitField::TestDetails | CommitField::Security | CommitField::MigracionesLentas | CommitField::PartesAEjecutar) {
            "🔤 EDITING MULTILINE - Type text, Enter for new line, Tab/arrows to save & move, Esc to cancel"
        } else {
            "🔤 EDITING SINGLE LINE - Type text, Tab/arrows to save & move, Esc to cancel"
        }
    } else {
        "📋 Navigation: Tab/Shift+Tab to move & edit, ↑↓ for commit type/tasks, 's' to search tasks, 'r' to generate AI description, 'c' to commit, 'q' to quit"
    };
    let instructions_widget = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Instructions"))
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });
    f.render_widget(instructions_widget, chunks[9]);

    // Selected Tasks
    let task_items: Vec<ListItem> = commit_form
        .selected_tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let is_focused = ui_state.task_management_mode && i == ui_state.selected_tab;
            let style = if is_focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_focused { "→ " } else { "  " };
            ListItem::new(format!("{}{} (ID: {})", prefix, task.title, task.id)).style(style)
        })
        .collect();

    let tasks_title = if ui_state.task_management_mode {
        "Selected Monday.com Tasks (Task Management Mode - Use Up/Down, Delete/r/Space to remove, 't' to exit)"
    } else {
        "Selected Monday.com Tasks (Press Enter to manage tasks)"
    };

    let tasks_list = List::new(task_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(tasks_title)
            .border_style(ui_state.get_field_border_style(&CommitField::SelectedTasks)))
        .style(ui_state.get_field_style(&CommitField::SelectedTasks));
    f.render_widget(tasks_list, chunks[10]);
}

pub fn draw_commit_preview_screen(f: &mut Frame, area: Rect, ui_state: &UIState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Instructions
            Constraint::Min(0),     // Commit message editor
        ])
        .split(area);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from("📝 Commit Message Preview & Editor"),
        Line::from("Edit with arrow keys, Home/End, Enter for new line. Ctrl+C to commit, Esc to cancel"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Instructions"))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(instructions, chunks[0]);

    // Commit message editor
    let commit_editor = Paragraph::new(ui_state.current_input.as_str())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Commit Message (editable)")
            .border_style(Style::default().fg(Color::Green)))
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .wrap(Wrap { trim: true });
    f.render_widget(commit_editor, chunks[1]);
} 