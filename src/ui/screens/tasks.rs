use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::types::{AppConfig, CommitForm, JiraTask, MondayTask, TaskLike, TaskSystem};
use crate::ui::state::{InputMode, UIState};

// =============================================================================
// MAIN DRAW FUNCTION
// =============================================================================

pub fn draw_task_search_screen(
    f: &mut Frame,
    area: Rect,
    ui_state: &UIState,
    monday_tasks: &[MondayTask],
    jira_tasks: &[JiraTask],
    config: &AppConfig,
    commit_form: &CommitForm,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(5),    // Search results
            Constraint::Length(6), // Currently selected tasks
        ])
        .split(area);

    // Render each section
    render_search_input(f, chunks[0], ui_state, config);
    render_search_results(
        f,
        chunks[1],
        ui_state,
        monday_tasks,
        jira_tasks,
        config,
        commit_form,
    );
    render_selected_tasks(f, chunks[2], ui_state, commit_form);
}

// =============================================================================
// SEARCH INPUT SECTION
// =============================================================================

fn render_search_input(f: &mut Frame, area: Rect, ui_state: &UIState, config: &AppConfig) {
    let task_system_name = get_task_system_name(config);

    let (search_title, search_style) = if ui_state.input_mode == InputMode::Editing {
        (
            format!(
                "🔍 Search {} Tasks (Typing... Press Enter to search, Esc to stop)",
                task_system_name
            ),
            Style::default().fg(Color::Green),
        )
    } else {
        (
            format!(
                "🔍 Search {} Tasks (Press 'i' or '/' to start typing, Enter to search)",
                task_system_name
            ),
            Style::default(),
        )
    };

    let search_block = Block::default()
        .borders(Borders::ALL)
        .title(search_title.as_str())
        .border_style(search_style);
    
    let mut search_textarea = ui_state.search_textarea.clone();
    search_textarea.set_block(search_block);

    f.render_widget(&search_textarea, area);
}

// =============================================================================
// SEARCH RESULTS SECTION
// =============================================================================

fn render_search_results(
    f: &mut Frame,
    area: Rect,
    ui_state: &UIState,
    monday_tasks: &[MondayTask],
    jira_tasks: &[JiraTask],
    config: &AppConfig,
    commit_form: &CommitForm,
) {
    let (list_title, task_items) = match config.get_task_system() {
        TaskSystem::Monday => build_monday_task_list(ui_state, monday_tasks, commit_form),
        TaskSystem::Jira => build_jira_task_list(ui_state, jira_tasks, commit_form),
        TaskSystem::None => build_no_system_list(),
    };

    let tasks_list =
        List::new(task_items).block(Block::default().borders(Borders::ALL).title(list_title));

    f.render_widget(tasks_list, area);
}

// =============================================================================
// TASK LIST BUILDERS
// =============================================================================

fn build_monday_task_list<'a>(
    ui_state: &UIState,
    monday_tasks: &'a [MondayTask],
    commit_form: &CommitForm,
) -> (String, Vec<ListItem<'a>>) {
    if monday_tasks.is_empty() {
        return build_empty_results_list("Monday.com", ui_state.search_textarea.lines().join(" ").as_str());
    }

    let items = monday_tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let is_selected = commit_form
                .selected_tasks
                .iter()
                .any(|selected| selected.id == task.id);
            let is_focused = i == ui_state.focused_search_index;

            build_task_item(
                i,
                task.get_title(),
                &format!(
                    "ID: {} | Board: {} | State: {}",
                    task.id,
                    task.board_name.as_deref().unwrap_or("Unknown"),
                    task.state
                ),
                is_selected,
                is_focused,
            )
        })
        .collect();

    (
        "Monday.com Search Results (Press 1-9,0 or Space to select tasks)".to_string(),
        items,
    )
}

fn build_jira_task_list<'a>(
    ui_state: &UIState,
    jira_tasks: &'a [JiraTask],
    commit_form: &CommitForm,
) -> (String, Vec<ListItem<'a>>) {
    if jira_tasks.is_empty() {
        return build_empty_results_list("JIRA", ui_state.search_textarea.lines().join(" ").as_str());
    }

    let items = jira_tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let is_selected = commit_form
                .selected_jira_tasks
                .iter()
                .any(|selected| selected.id == task.id);
            let is_focused = i == ui_state.focused_search_index;

            build_task_item(
                i,
                task.get_title(),
                &format!(
                    "Key: {} | Status: {} | Type: {}",
                    task.key, task.status, task.issue_type
                ),
                is_selected,
                is_focused,
            )
        })
        .collect();

    (
        "JIRA Search Results (Press 1-9,0 or Space to select tasks)".to_string(),
        items,
    )
}

fn build_no_system_list() -> (String, Vec<ListItem<'static>>) {
    let items = vec![ListItem::new(vec![
        Line::from("⚠️  No task system configured"),
        Line::from("   Configure Monday.com or JIRA in config screen"),
    ])];
    ("No task system configured".to_string(), items)
}

fn build_empty_results_list(
    system_name: &str,
    search_text: &str,
) -> (String, Vec<ListItem<'static>>) {
    if search_text.is_empty() {
        // No search query, show placeholder
        let items = vec![ListItem::new(vec![
            Line::from("💡 Type a search query above and press Enter"),
            Line::from("   Or press 'q' to return to commit screen"),
        ])];
        (format!("Type to search for {} tasks", system_name), items)
    } else {
        // Search performed but no results
        let items = vec![ListItem::new(vec![
            Line::from(format!("❌ No {} tasks found for your search", system_name)),
            Line::from("   Try a different search term"),
        ])];
        (format!("No {} tasks found", system_name), items)
    }
}

// =============================================================================
// TASK ITEM BUILDER (UNIFIED FOR BOTH SYSTEMS)
// =============================================================================

fn build_task_item(
    index: usize,
    title: &str,
    details: &str,
    is_selected: bool,
    is_focused: bool,
) -> ListItem<'static> {
    // Build styles based on state
    let styles = TaskItemStyles::new(is_focused, is_selected);

    // Build number and checkbox
    let number = get_selection_number(index);
    let checkbox = if is_selected { "☑ " } else { "☐ " };
    let focus_indicator = if is_focused { "→ " } else { "  " };

    // Convert borrowed strings to owned strings for static lifetime
    let title_owned = title.to_string();
    let details_owned = details.to_string();

    ListItem::new(vec![
        Line::from(vec![
            Span::styled(focus_indicator, styles.focus_style),
            Span::styled(format!("[{}] ", number), styles.number_style),
            Span::styled(checkbox, styles.checkbox_style),
            Span::styled(title_owned, styles.title_style),
        ]),
        Line::from(format!("     {}", details_owned)),
    ])
}

// =============================================================================
// SELECTED TASKS SECTION
// =============================================================================

fn render_selected_tasks(f: &mut Frame, area: Rect, ui_state: &UIState, commit_form: &CommitForm) {
    let selected_items = build_selected_tasks_list(&commit_form.selected_tasks);

    // Create list state for navigation
    let mut selected_list_state = ListState::default();
    if !commit_form.selected_tasks.is_empty() {
        let selected_index = ui_state
            .selected_tab
            .min(commit_form.selected_tasks.len().saturating_sub(1));
        selected_list_state.select(Some(selected_index));
    }

    let selected_list = List::new(selected_items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Selected Tasks ({}) - Use ↑↓ to navigate, Del/r to remove",
            commit_form.selected_tasks.len()
        )))
        .highlight_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(selected_list, area, &mut selected_list_state);
}

fn build_selected_tasks_list(selected_tasks: &[MondayTask]) -> Vec<ListItem<'static>> {
    if selected_tasks.is_empty() {
        vec![ListItem::new(vec![
            Line::from("No tasks selected yet"),
            Line::from("Search and select tasks above"),
        ])]
    } else {
        selected_tasks
            .iter()
            .map(|task| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("✅ ", Style::default().fg(Color::Green)),
                        Span::styled(
                            task.get_title().to_string(),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(format!(
                        "   ID: {} | Use ↑↓ to navigate, Del/r to remove",
                        task.id
                    )),
                ])
            })
            .collect()
    }
}

// =============================================================================
// STYLING HELPERS
// =============================================================================

struct TaskItemStyles {
    focus_style: Style,
    number_style: Style,
    checkbox_style: Style,
    title_style: Style,
}

impl TaskItemStyles {
    fn new(is_focused: bool, is_selected: bool) -> Self {
        let (base_color, modifier) = match (is_focused, is_selected) {
            (true, _) => (Color::Yellow, Modifier::BOLD),
            (false, true) => (Color::Green, Modifier::BOLD),
            (false, false) => (Color::White, Modifier::empty()),
        };

        Self {
            focus_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            number_style: if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            },
            checkbox_style: if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Blue)
            },
            title_style: Style::default().fg(base_color).add_modifier(modifier),
        }
    }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

fn get_task_system_name(config: &AppConfig) -> &'static str {
    match config.get_task_system() {
        TaskSystem::Monday => "Monday.com",
        TaskSystem::Jira => "JIRA",
        TaskSystem::None => "Task",
    }
}

fn get_selection_number(index: usize) -> String {
    match index {
        0..=8 => (index + 1).to_string(),
        9 => "0".to_string(),
        _ => " ".to_string(),
    }
}
