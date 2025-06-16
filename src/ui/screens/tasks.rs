use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::ui::state::{UIState, InputMode};
use crate::types::{MondayTask, CommitForm};

pub fn draw_task_search_screen(f: &mut Frame, area: Rect, ui_state: &UIState, tasks: &[MondayTask], commit_form: &CommitForm) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(5),    // Search results or selected tasks
            Constraint::Length(6), // Currently selected tasks
        ])
        .split(area);

    // Search input with visual feedback for input mode
    let (search_title, search_style) = if ui_state.input_mode == InputMode::Editing {
        ("🔍 Search Monday.com Tasks (Type to search, Enter to submit, Esc to exit)", Style::default().fg(Color::Green))
    } else {
        ("🔍 Search Monday.com Tasks (Press 'i' or '/' to start typing, Enter to search)", Style::default())
    };
    
    let search_input = Paragraph::new(format!("Search: {}", ui_state.current_input))
        .block(Block::default().borders(Borders::ALL).title(search_title).border_style(search_style));
    f.render_widget(search_input, chunks[0]);

    // Determine what to show: search results or a message
    let (list_title, task_items) = if tasks.is_empty() && ui_state.current_input.is_empty() {
        // No search query, show placeholder
        ("Type to search for tasks".to_string(), vec![
            ListItem::new(vec![
                Line::from("💡 Type a search query above and press Enter"),
                Line::from("   Or press 'q' to return to commit screen"),
            ])
        ])
    } else if tasks.is_empty() && !ui_state.current_input.is_empty() {
        // Search performed but no results
        ("No tasks found".to_string(), vec![
            ListItem::new(vec![
                Line::from("❌ No tasks found for your search"),
                Line::from("   Try a different search term"),
            ])
        ])
    } else {
                 // Show search results with selection status and numbers
         let items: Vec<ListItem> = tasks
             .iter()
             .enumerate()
             .map(|(i, task)| {
                 // Check if this task is already selected
                 let is_selected = commit_form.selected_tasks.iter().any(|selected| selected.id == task.id);
                 
                 // Check if this task is currently focused (for yellow highlighting)
                 let is_focused = i == ui_state.focused_search_index;
                 
                 // Checkbox symbol and styling
                 let checkbox = if is_selected { "☑ " } else { "☐ " };
                 let checkbox_style = if is_focused {
                     Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                 } else if is_selected {
                     Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                 } else {
                     Style::default().fg(Color::Blue)
                 };
                 
                 // Task title style - yellow if focused, green if selected, normal otherwise
                 let title_style = if is_focused {
                     Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                 } else if is_selected {
                     Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                 } else {
                     Style::default()
                 };
                 
                 // Number for selection (1-9, then 0)
                 let number = if i < 9 { (i + 1).to_string() } else if i == 9 { "0".to_string() } else { " ".to_string() };
                 let number_style = if is_focused {
                     Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                 } else {
                     Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                 };
                 
                 // Add focus indicator
                 let focus_indicator = if is_focused { "→ " } else { "  " };
                 let focus_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
                 
                 ListItem::new(vec![
                     Line::from(vec![
                         Span::styled(focus_indicator, focus_style),
                         Span::styled(format!("[{}] ", number), number_style),
                         Span::styled(checkbox, checkbox_style),
                         Span::styled(&task.title, title_style),
                     ]),
                     Line::from(format!("     ID: {} | Board: {}", 
                         task.id, 
                         task.board_name.as_deref().unwrap_or("Unknown"))),
                 ])
             })
             .collect();
         
         ("Search Results (Press 1-9,0 or Space to select tasks)".to_string(), items)
    };

    let tasks_list = List::new(task_items)
        .block(Block::default().borders(Borders::ALL).title(list_title));
    f.render_widget(tasks_list, chunks[1]);

    // Show currently selected tasks - this is where the cursor navigates
    let selected_items: Vec<ListItem> = if commit_form.selected_tasks.is_empty() {
        vec![
            ListItem::new(vec![
                Line::from("No tasks selected yet"),
                Line::from("Search and select tasks above"),
            ])
        ]
    } else {
        commit_form.selected_tasks
            .iter()
            .map(|task| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("✅ ", Style::default().fg(Color::Green)),
                        Span::styled(&task.title, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(format!("   ID: {} | Use ↑↓ to navigate, Del/r to remove", task.id)),
                ])
            })
            .collect()
    };

    // Create list state for selected tasks navigation
    let mut selected_list_state = ListState::default();
    if !commit_form.selected_tasks.is_empty() {
        // Use a separate index for selected tasks navigation
        let selected_index = ui_state.selected_tab.min(commit_form.selected_tasks.len().saturating_sub(1));
        selected_list_state.select(Some(selected_index));
    }

    let selected_list = List::new(selected_items)
        .block(Block::default().borders(Borders::ALL).title(format!("Selected Tasks ({}) - Use ↑↓ to navigate, Del/r to remove", commit_form.selected_tasks.len())))
        .highlight_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(selected_list, chunks[2], &mut selected_list_state);
}

pub fn draw_task_selection_screen(f: &mut Frame, area: Rect, ui_state: &UIState, tasks: &[MondayTask], commit_form: &CommitForm) {
    let task_items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            // Check if this task is selected
            let is_selected = commit_form.selected_tasks.iter().any(|selected| selected.id == task.id);
            
            // Style for the currently highlighted item (yellow highlight)
            let _is_highlighted = i == ui_state.selected_tab;
            
            // Checkbox symbol based on selection
            let checkbox = if is_selected { "☑ " } else { "☐ " };
            let checkbox_style = if is_selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Blue)
            };
            
            // Task title style - green if selected, normal otherwise
            let title_style = if is_selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(checkbox, checkbox_style),
                    Span::styled(&task.title, title_style),
                ]),
                Line::from(format!("   ID: {} | URL: {}", task.id, task.url)),
            ])
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(ui_state.selected_tab));

    let tasks_list = List::new(task_items)
        .block(Block::default().borders(Borders::ALL).title("Select Tasks (Space to toggle, Enter to confirm)"))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(tasks_list, area, &mut list_state);
} 