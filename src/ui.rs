use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame,
};

use crate::types::{AppScreen, AppState, CommitType, MondayTask, CommitForm};
use crate::git::GitStatus;

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

pub fn draw(
    f: &mut Frame,
    app_screen: &AppScreen,
    app_state: &AppState,
    ui_state: &mut UIState,
    commit_form: &CommitForm,
    tasks: &[MondayTask],
    message: Option<&str>,
    git_status: Option<&GitStatus>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Title bar
    draw_title_bar(f, chunks[0]);

    // Main content based on current screen
    match app_screen {
        AppScreen::Main => draw_main_screen(f, chunks[1], ui_state, git_status),
        AppScreen::Config => draw_config_screen(f, chunks[1]),
        AppScreen::Commit => draw_commit_screen(f, chunks[1], ui_state, commit_form),
        AppScreen::CommitPreview => draw_commit_preview_screen(f, chunks[1], ui_state),
        AppScreen::ReleaseNotes => draw_release_notes_screen(f, chunks[1]),
        AppScreen::TaskSearch => draw_task_search_screen(f, chunks[1], ui_state, tasks, commit_form),
        AppScreen::TaskSelection => draw_task_selection_screen(f, chunks[1], ui_state, tasks, commit_form),
    }

    // Status bar
    draw_status_bar(f, chunks[2], app_state, message);

    // Loading overlay
    if matches!(app_state, AppState::Loading) {
        // Increment animation frame for spinner
        ui_state.animation_frame = ui_state.animation_frame.wrapping_add(1);
        draw_loading_overlay(f, f.area(), ui_state.animation_frame, message);
    }

    // Set cursor position when editing
    if ui_state.input_mode == InputMode::Editing {
        match app_screen {
            AppScreen::Commit => set_cursor_position(f, chunks[1], ui_state),
            AppScreen::CommitPreview => set_commit_preview_cursor_position(f, chunks[1], ui_state),
            AppScreen::TaskSearch => set_search_cursor_position(f, chunks[1], ui_state),
            _ => {}
        }
    }
}

fn draw_title_bar(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("🚀 Semantic Release TUI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_main_screen(f: &mut Frame, area: Rect, ui_state: &mut UIState, git_status: Option<&GitStatus>) {
    let tabs = Tabs::new(vec!["💾 Commit", "📝 Release Notes", "⚙️ Config", "📋 Help"])
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(ui_state.selected_tab);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    
    f.render_widget(tabs, chunks[0]);

    // Build git status lines
    let mut content_lines = vec![
        Line::from(""),
        Line::from("📋 Semantic Release TUI - Help"),
        Line::from(""),
    ];

    // Add git status information
    if let Some(status) = git_status {
        content_lines.push(Line::from("📂 Repository Status:"));
        
        if !status.staged.is_empty() {
            content_lines.push(Line::from(format!("✅ Staged files ({}): Ready to commit", status.staged.len())));
            for file in status.staged.iter().take(5) {
                content_lines.push(Line::from(format!("   • {}", file)));
            }
            if status.staged.len() > 5 {
                content_lines.push(Line::from(format!("   ... and {} more", status.staged.len() - 5)));
            }
        }
        
        if !status.modified.is_empty() {
            content_lines.push(Line::from(format!("🔄 Modified files ({}): Use 'git add' to stage", status.modified.len())));
            for file in status.modified.iter().take(3) {
                content_lines.push(Line::from(format!("   • {}", file)));
            }
            if status.modified.len() > 3 {
                content_lines.push(Line::from(format!("   ... and {} more", status.modified.len() - 3)));
            }
        }
        
        if !status.untracked.is_empty() {
            content_lines.push(Line::from(format!("❓ Untracked files ({}): New files", status.untracked.len())));
            for file in status.untracked.iter().take(3) {
                content_lines.push(Line::from(format!("   • {}", file)));
            }
            if status.untracked.len() > 3 {
                content_lines.push(Line::from(format!("   ... and {} more", status.untracked.len() - 3)));
            }
        }
        
        if status.staged.is_empty() && status.modified.is_empty() && status.untracked.is_empty() {
            content_lines.push(Line::from("✨ Repository clean: No changes to commit"));
        }
        content_lines.push(Line::from(""));
    } else {
        content_lines.push(Line::from("📂 Repository Status: Not available (not in git repo?)"));
        content_lines.push(Line::from(""));
    }

    // Add help content
    content_lines.extend(vec![
        Line::from("🔧 General Navigation:"),
        Line::from("• Tab/Shift+Tab: Navigate between tabs and fields"),
        Line::from("• Enter: Select option or edit field"),
        Line::from("• Esc: Go back or cancel editing"),
        Line::from("• q: Quit application"),
        Line::from(""),
        Line::from("📦 Commit Screen:"),
        Line::from("• Tab: Navigate between fields (auto-edit mode)"),
        Line::from("• Enter: Add new line in multiline fields"),
        Line::from("• s: Search Monday.com tasks"),
        Line::from("• c: Create commit"),
        Line::from(""),
        Line::from("📝 Release Notes: Generate AI-powered release notes"),
        Line::from("⚙️ Config: Setup API keys (.env file)"),
        Line::from(""),
        Line::from("Press Tab to navigate to Commit screen to get started!"),
    ]);

    let main_content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::ALL).title("Help & Navigation"))
        .wrap(Wrap { trim: true });

    f.render_widget(main_content, chunks[1]);
}

fn draw_config_screen(f: &mut Frame, area: Rect) {
    let content = Paragraph::new(vec![
        Line::from("⚙️ Configuration"),
        Line::from(""),
        Line::from("Use 'semantic-release-tui config' command to configure:"),
        Line::from(""),
        Line::from("• Monday.com API Key"),
        Line::from("• Monday.com Account Slug"),
        Line::from("• Monday.com Board ID (optional)"),
        Line::from("• Google Gemini API Token (optional)"),
        Line::from(""),
        Line::from("Configuration is stored in .env file (same as original project)"),
        Line::from(""),
        Line::from("Press 'q' to go back to main menu."),
    ])
    .block(Block::default().borders(Borders::ALL).title("Configuration"))
    .wrap(Wrap { trim: true });

    f.render_widget(content, area);
}

fn draw_commit_screen(f: &mut Frame, area: Rect, ui_state: &UIState, commit_form: &CommitForm) {
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
            Constraint::Length(3),  // Instructions
            Constraint::Min(0),     // Selected tasks
        ])
        .split(area);

    // Helper function to get field style
    let get_field_style = |field: &CommitField| -> Style {
        if ui_state.current_field == *field {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    // Helper function to get field border style
    let get_field_border_style = |field: &CommitField| -> Style {
        if ui_state.current_field == *field {
            if ui_state.input_mode == InputMode::Editing {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            }
        } else {
            Style::default()
        }
    };

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
            .border_style(get_field_border_style(&CommitField::Type)));

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
            .border_style(get_field_border_style(&CommitField::Scope)))
        .style(get_field_style(&CommitField::Scope));
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
            .border_style(get_field_border_style(&CommitField::Title)))
        .style(get_field_style(&CommitField::Title));
    f.render_widget(title, chunks[2]);

    // Description (multiline)
    let desc_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::Description {
        &ui_state.current_input
    } else if commit_form.description.is_empty() {
        "Press 'r' to generate AI description, or type manually... (Enter for new line)"
    } else {
        &commit_form.description
    };
    let description = Paragraph::new(desc_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Description (multiline, press 'r' to generate)")
            .border_style(get_field_border_style(&CommitField::Description)))
        .style(get_field_style(&CommitField::Description))
        .wrap(Wrap { trim: true });
    f.render_widget(description, chunks[3]);

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
            .border_style(get_field_border_style(&CommitField::BreakingChange)))
        .style(get_field_style(&CommitField::BreakingChange));
    f.render_widget(breaking, chunks[4]);

    // Test Details (multiline)
    let test_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::TestDetails {
        &ui_state.current_input
    } else if commit_form.test_details.is_empty() {
        "Enter test details... (Enter for new line)"
    } else {
        &commit_form.test_details
    };
    let test_details = Paragraph::new(test_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Test Details (multiline)")
            .border_style(get_field_border_style(&CommitField::TestDetails)))
        .style(get_field_style(&CommitField::TestDetails))
        .wrap(Wrap { trim: true });
    f.render_widget(test_details, chunks[5]);

    // Security (multiline)
    let security_text = if ui_state.input_mode == InputMode::Editing && ui_state.current_field == CommitField::Security {
        &ui_state.current_input
    } else if commit_form.security.is_empty() {
        "Enter security info (or NA)... (Enter for new line)"
    } else {
        &commit_form.security
    };
    let security = Paragraph::new(security_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Security (multiline)")
            .border_style(get_field_border_style(&CommitField::Security)))
        .style(get_field_style(&CommitField::Security))
        .wrap(Wrap { trim: true });
    f.render_widget(security, chunks[6]);

    // Instructions
    let instructions = if ui_state.input_mode == InputMode::Editing {
        if matches!(ui_state.current_field, CommitField::Description | CommitField::TestDetails | CommitField::Security) {
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
    f.render_widget(instructions_widget, chunks[7]);

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
            .border_style(get_field_border_style(&CommitField::SelectedTasks)))
        .style(get_field_style(&CommitField::SelectedTasks));
    f.render_widget(tasks_list, chunks[8]);
}

fn draw_commit_preview_screen(f: &mut Frame, area: Rect, ui_state: &UIState) {
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

fn draw_release_notes_screen(f: &mut Frame, area: Rect) {
    let content = Paragraph::new(vec![
        Line::from("📝 Release Notes Generation"),
        Line::from(""),
        Line::from("This feature will:"),
        Line::from("1. Analyze commits since the last version tag"),
        Line::from("2. Extract Monday.com task information"),
        Line::from("3. Generate structured release notes using AI"),
        Line::from(""),
        Line::from("Generated files will be saved in the release-notes/ directory:"),
        Line::from("• release-notes-YYYY-MM-DD.md - Raw data"),
        Line::from("• release-notes-YYYY-MM-DD_GEMINI.md - AI generated notes"),
        Line::from(""),
        Line::from("Press Enter to generate release notes"),
        Line::from("Press 'q' to go back to main menu"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Release Notes"))
    .wrap(Wrap { trim: true });

    f.render_widget(content, area);
}

fn draw_task_search_screen(f: &mut Frame, area: Rect, ui_state: &UIState, tasks: &[MondayTask], commit_form: &CommitForm) {
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

fn draw_task_selection_screen(f: &mut Frame, area: Rect, ui_state: &UIState, tasks: &[MondayTask], commit_form: &CommitForm) {
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

fn draw_status_bar(f: &mut Frame, area: Rect, app_state: &AppState, message: Option<&str>) {
    let (status_text, title) = match app_state {
        AppState::Normal => (message.unwrap_or("Listo"), "Estado"),
        AppState::Loading => {
            let loading_message = message.unwrap_or("Procesando...");
            if loading_message.contains("Gemini") {
                (loading_message, "🧠 Gemini AI")
            } else {
                (loading_message, "⏳ Cargando")
            }
        },
        AppState::Error(err) => (err.as_str(), "⚠️ ERROR - Presiona cualquier tecla para continuar"),
        AppState::Input => ("Modo entrada - Presiona Esc para cancelar", "Entrada"),
    };

    let status_style = match app_state {
        AppState::Error(_) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        AppState::Loading => {
            if message.is_some_and(|m| m.contains("Gemini")) {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            }
        },
        _ => Style::default().fg(Color::Green),
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(status, area);
}

fn draw_loading_overlay(f: &mut Frame, area: Rect, animation_frame: usize, message: Option<&str>) {
    // Spinner characters for animation
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner_char = spinner_chars[animation_frame % spinner_chars.len()];
    
    // Determine the loading message
    let loading_message = match message {
        Some(msg) if msg.contains("Gemini") => "🧠 Generando descripción con Gemini IA...",
        Some(msg) if msg.contains("search") => "🔍 Buscando tareas en Monday.com...",
        Some(msg) if msg.contains("release") => "📝 Generando notas de versión...",
        _ => "⏳ Cargando...",
    };
    
    let loading_area = centered_rect(50, 15, area);
    f.render_widget(Clear, loading_area);
    
    // Main loading block
    let block = Block::default()
        .title(format!(" {} Procesando ", spinner_char))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, loading_area);

    // Create content area inside the block
    let content_area = Rect {
        x: loading_area.x + 2,
        y: loading_area.y + 2,
        width: loading_area.width.saturating_sub(4),
        height: loading_area.height.saturating_sub(4),
    };

    // Loading message
    let message_para = Paragraph::new(loading_message)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    let message_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: 2,
    };
    f.render_widget(message_para, message_area);

    // Animated progress bar with pulsing effect
    let progress_percent = ((animation_frame * 3) % 100) as u16;
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .percent(progress_percent)
        .label(format!("{}%", progress_percent));
    
    let gauge_area = Rect {
        x: content_area.x,
        y: content_area.y + 3,
        width: content_area.width,
        height: 1,
    };
    f.render_widget(gauge, gauge_area);

    // Additional spinner info for Gemini specifically
    if loading_message.contains("Gemini") {
        let spinner_info = Paragraph::new(vec![
            Line::from(""),
            Line::from("🔍 Analizando cambios en el código..."),
            Line::from("📝 Generando descripción técnica..."),
            Line::from("🌐 Comunicándose con Gemini AI..."),
        ])
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
        
        let info_area = Rect {
            x: content_area.x,
            y: content_area.y + 5,
            width: content_area.width,
            height: 4,
        };
        f.render_widget(spinner_info, info_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn set_search_cursor_position(f: &mut Frame, area: Rect, ui_state: &UIState) {
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

fn set_cursor_position(f: &mut Frame, area: Rect, ui_state: &UIState) {
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
        CommitField::SelectedTasks => return, // Selected tasks field doesn't need cursor (it's a list)
    };

    // Calculate cursor position within the field
    let (cursor_x, cursor_y) = calculate_cursor_position(&ui_state.current_input, ui_state.cursor_position, field_area.width.saturating_sub(2));
    
    // Position cursor within the field (accounting for border)
    let cursor_x = field_area.x + 1 + cursor_x;
    let cursor_y = field_area.y + 1 + cursor_y;
    
    // Set cursor position and make it visible
    f.set_cursor_position((cursor_x, cursor_y));
}

fn set_commit_preview_cursor_position(f: &mut Frame, area: Rect, ui_state: &UIState) {
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
    
    // Position cursor within the editor field (accounting for border)
    let cursor_x = editor_area.x + 1 + cursor_x;
    let cursor_y = editor_area.y + 1 + cursor_y;
    
    // Set cursor position and make it visible
    f.set_cursor_position((cursor_x, cursor_y));
}

fn calculate_cursor_position(input: &str, cursor_pos: usize, field_width: u16) -> (u16, u16) {
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