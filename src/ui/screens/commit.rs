use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::types::{CommitForm, CommitType};
use crate::ui::state::{CommitField, InputMode, UIState};

pub fn draw_commit_screen(f: &mut Frame, area: Rect, ui_state: &UIState, commit_form: &CommitForm) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Commit type
            Constraint::Length(3), // Scope
            Constraint::Length(3), // Title
            Constraint::Length(5), // Description (multiline)
            Constraint::Length(3), // Breaking change
            Constraint::Length(5), // Test details (multiline)
            Constraint::Length(5), // Security (multiline)
            Constraint::Length(5), // Migraciones lentas (multiline)
            Constraint::Length(5), // Partes a ejecutar (multiline)
            Constraint::Length(3), // Instructions
            Constraint::Min(0),    // Selected tasks
        ])
        .split(area);

    // Commit Type Selection
    let commit_types: Vec<ListItem> = CommitType::all()
        .iter()
        .enumerate()
        .map(|(i, ct)| {
            let style = if Some(ct) == commit_form.commit_type.as_ref() {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if i == ui_state.selected_commit_type
                && ui_state.current_field == CommitField::Type
            {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
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

    let commit_type_list = List::new(commit_types).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Commit Type (↑↓ to select, Tab to move)")
            .border_style(ui_state.get_field_border_style(&CommitField::Type)),
    );

    f.render_stateful_widget(commit_type_list, chunks[0], &mut list_state);

    // Render TextArea widgets with titles
    let scope_block = Block::default()
        .borders(Borders::ALL)
        .title("Scope (auto-edit on Tab)")
        .border_style(ui_state.get_field_border_style(&CommitField::Scope));
    let mut scope_textarea = ui_state.scope_textarea.clone();
    scope_textarea.set_block(scope_block);
    f.render_widget(&scope_textarea, chunks[1]);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title("Title")
        .border_style(ui_state.get_field_border_style(&CommitField::Title));
    let mut title_textarea = ui_state.title_textarea.clone();
    title_textarea.set_block(title_block);
    f.render_widget(&title_textarea, chunks[2]);

    let description_block = Block::default()
        .borders(Borders::ALL)
        .title("Description (multiline, 't' for comprehensive AI analysis)")
        .border_style(ui_state.get_field_border_style(&CommitField::Description));
    let mut description_textarea = ui_state.description_textarea.clone();
    description_textarea.set_block(description_block);
    f.render_widget(&description_textarea, chunks[3]);

    let breaking_change_block = Block::default()
        .borders(Borders::ALL)
        .title("Breaking Change")
        .border_style(ui_state.get_field_border_style(&CommitField::BreakingChange));
    let mut breaking_change_textarea = ui_state.breaking_change_textarea.clone();
    breaking_change_textarea.set_block(breaking_change_block);
    f.render_widget(&breaking_change_textarea, chunks[4]);

    let test_details_block = Block::default()
        .borders(Borders::ALL)
        .title("Test Details (multiline, auto-filled by 't' AI analysis)")
        .border_style(ui_state.get_field_border_style(&CommitField::TestDetails));
    let mut test_details_textarea = ui_state.test_details_textarea.clone();
    test_details_textarea.set_block(test_details_block);
    f.render_widget(&test_details_textarea, chunks[5]);

    let security_block = Block::default()
        .borders(Borders::ALL)
        .title("Security (multiline)")
        .border_style(ui_state.get_field_border_style(&CommitField::Security));
    let mut security_textarea = ui_state.security_textarea.clone();
    security_textarea.set_block(security_block);
    f.render_widget(&security_textarea, chunks[6]);

    let migraciones_lentas_block = Block::default()
        .borders(Borders::ALL)
        .title("Migraciones Lentas (multiline)")
        .border_style(ui_state.get_field_border_style(&CommitField::MigracionesLentas));
    let mut migraciones_lentas_textarea = ui_state.migraciones_lentas_textarea.clone();
    migraciones_lentas_textarea.set_block(migraciones_lentas_block);
    f.render_widget(&migraciones_lentas_textarea, chunks[7]);

    let partes_a_ejecutar_block = Block::default()
        .borders(Borders::ALL)
        .title("Partes a Ejecutar (multiline)")
        .border_style(ui_state.get_field_border_style(&CommitField::PartesAEjecutar));
    let mut partes_a_ejecutar_textarea = ui_state.partes_a_ejecutar_textarea.clone();
    partes_a_ejecutar_textarea.set_block(partes_a_ejecutar_block);
    f.render_widget(&partes_a_ejecutar_textarea, chunks[8]);

    // Instructions
    let instructions = if ui_state.input_mode == InputMode::Editing {
        if UIState::is_multiline_field(&ui_state.current_field) {
            "🔤 EDITING MULTILINE - Advanced text editing with TextArea, Tab/arrows to save & move, Esc to cancel"
        } else {
            "🔤 EDITING SINGLE LINE - Advanced text editing with TextArea, Tab/arrows to save & move, Esc to cancel"
        }
    } else {
        "📋 Navigation: Tab/Shift+Tab to move & edit, ↑↓ for commit type/tasks, 's' Monday.com/'j' JIRA search, 't' AI analysis, 'm' manage tasks, 'c' commit, 'q' quit"
    };
    let instructions_widget = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Instructions (TextArea Mode)"))
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });
    f.render_widget(instructions_widget, chunks[9]);

    // Render selected tasks if any
    if !commit_form.selected_tasks.is_empty() {
        let task_items: Vec<ListItem> = commit_form
            .selected_tasks
            .iter()
            .map(|task| ListItem::new(format!("• {}", task.title)))
            .collect();

        let selected_task_list = List::new(task_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Selected Tasks ({})", commit_form.selected_tasks.len()))
                .border_style(ui_state.get_field_border_style(&CommitField::SelectedTasks)),
        );

        f.render_widget(selected_task_list, chunks[10]);
    } else {
        let no_tasks = Paragraph::new("No tasks selected. Use 's' for Monday.com or 'j' for JIRA search to add tasks.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Selected Tasks (0)")
                    .border_style(ui_state.get_field_border_style(&CommitField::SelectedTasks)),
            )
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(no_tasks, chunks[10]);
    }
}

pub fn draw_commit_preview_screen(f: &mut Frame, area: Rect, ui_state: &UIState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Instructions
            Constraint::Min(0),    // Commit message editor
        ])
        .split(area);

    // Instructions
    let instructions = Paragraph::new("📋 Commit Preview: Edit message above, 'c' to commit, Esc to go back")
        .block(Block::default().borders(Borders::ALL).title("Instructions"))
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });
    f.render_widget(instructions, chunks[0]);

    // Commit message editor using TextArea
    let editor_block = Block::default()
        .borders(Borders::ALL)
        .title("Commit Message Editor")
        .border_style(Style::default().fg(Color::Green));
    let mut commit_editor_textarea = ui_state.commit_preview_textarea.clone();
    commit_editor_textarea.set_block(editor_block);
    f.render_widget(&commit_editor_textarea, chunks[1]);
}
