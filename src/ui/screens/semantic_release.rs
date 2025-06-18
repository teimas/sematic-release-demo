use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};

use crate::{
    types::{AppConfig, AppState, SemanticReleaseState},
    ui::UIState,
};

pub fn draw_semantic_release_screen(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ui_state: &mut UIState,
    config: &AppConfig,
    current_state: &AppState,
    message: Option<&str>,
    semantic_release_state: Option<&SemanticReleaseState>,
) {
    // Check if we have results to show
    let has_results = semantic_release_state
        .and_then(|state| {
            state.finished.lock().ok().and_then(|finished| {
                if *finished {
                    state.result.lock().ok().map(|result| !result.is_empty())
                } else {
                    None
                }
            })
        })
        .unwrap_or(false);

    if has_results {
        draw_with_results(
            f,
            area,
            ui_state,
            config,
            current_state,
            message,
            semantic_release_state.unwrap(),
        );
    } else {
        draw_without_results(f, area, ui_state, config, current_state, message);
    }
}

fn draw_with_results(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ui_state: &mut UIState,
    config: &AppConfig,
    _current_state: &AppState,
    _message: Option<&str>,
    semantic_release_state: &SemanticReleaseState,
) {
    // Split into left (options) and right (results) panels
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left panel - options
    draw_options_panel(f, chunks[0], ui_state, config);

    // Right panel - results
    draw_results_panel(f, chunks[1], ui_state, semantic_release_state);
}

fn draw_without_results(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ui_state: &mut UIState,
    config: &AppConfig,
    current_state: &AppState,
    message: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(4), // Description
            Constraint::Min(10),   // Options
            Constraint::Length(4), // Instructions
            Constraint::Length(3), // Status
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🚀 Semantic Release")
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Description
    let description_text = vec![
        Line::from("Automated version management and package publishing using semantic-release"),
        Line::from("Based on conventional commits for determining version bumps"),
    ];
    let description = Paragraph::new(description_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("About"));
    f.render_widget(description, chunks[1]);

    // Options
    let options = create_semantic_release_options(ui_state.selected_tab, config);
    let options_list = List::new(options)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
        .highlight_symbol("► ");
    f.render_stateful_widget(
        options_list,
        chunks[2],
        &mut ratatui::widgets::ListState::default().with_selected(Some(ui_state.selected_tab)),
    );

    // Instructions
    let instructions_text = if matches!(current_state, AppState::Loading) {
        vec![Line::from(vec![
            Span::styled("Processing... ", Style::default().fg(Color::Yellow)),
            Span::styled("Please wait", Style::default().fg(Color::White)),
        ])]
    } else {
        vec![Line::from(vec![
            Span::styled("Tab/Shift+Tab: ", Style::default().fg(Color::Green)),
            Span::styled("Navigate options  ", Style::default().fg(Color::White)),
            Span::styled("Enter: ", Style::default().fg(Color::Green)),
            Span::styled("Execute  ", Style::default().fg(Color::White)),
            Span::styled("q/Esc: ", Style::default().fg(Color::Red)),
            Span::styled("Back", Style::default().fg(Color::White)),
        ])]
    };
    let instructions = Paragraph::new(instructions_text)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(instructions, chunks[3]);

    // Status
    let status_text = match current_state {
        AppState::Loading => {
            // Simple loading animation without importing create_loading_animation
            let animation_chars = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
            let animation = animation_chars[ui_state.animation_frame % animation_chars.len()];
            vec![Line::from(vec![
                Span::styled(animation, Style::default().fg(Color::Yellow)),
                Span::styled(" ", Style::default()),
                Span::styled(
                    message.unwrap_or("Processing..."),
                    Style::default().fg(Color::Yellow),
                ),
            ])]
        }
        AppState::Error(error) => {
            vec![Line::from(vec![
                Span::styled("❌ Error: ", Style::default().fg(Color::Red)),
                Span::styled(error, Style::default().fg(Color::White)),
            ])]
        }
        _ => {
            if let Some(msg) = message {
                vec![Line::from(vec![
                    Span::styled("💡 ", Style::default().fg(Color::Blue)),
                    Span::styled(msg, Style::default().fg(Color::White)),
                ])]
            } else {
                vec![Line::from(vec![Span::styled(
                    "Ready to execute semantic-release operations",
                    Style::default().fg(Color::Green),
                )])]
            }
        }
    };

    let status = Paragraph::new(status_text)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[4]);
}

fn draw_options_panel(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ui_state: &mut UIState,
    config: &AppConfig,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(8),    // Options
            Constraint::Length(4), // Instructions
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🚀 Options")
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Options
    let options = create_semantic_release_options(ui_state.selected_tab, config);
    let options_list = List::new(options)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Operations"),
        )
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
        .highlight_symbol("► ");
    f.render_stateful_widget(
        options_list,
        chunks[1],
        &mut ratatui::widgets::ListState::default().with_selected(Some(ui_state.selected_tab)),
    );

    // Instructions
    let instructions_text = vec![
        Line::from(vec![
            Span::styled("Tab/Shift+Tab: ", Style::default().fg(Color::Green)),
            Span::styled("Navigate  ", Style::default().fg(Color::White)),
            Span::styled("Enter: ", Style::default().fg(Color::Green)),
            Span::styled("Execute", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("r: ", Style::default().fg(Color::Blue)),
            Span::styled("Clear results  ", Style::default().fg(Color::White)),
            Span::styled("q/Esc: ", Style::default().fg(Color::Red)),
            Span::styled("Back", Style::default().fg(Color::White)),
        ]),
    ];
    let instructions = Paragraph::new(instructions_text)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(instructions, chunks[2]);
}

fn draw_results_panel(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ui_state: &mut UIState,
    semantic_release_state: &SemanticReleaseState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Results content
        ])
        .split(area);

    // Get status and result
    let status = semantic_release_state
        .status
        .lock()
        .map(|s| s.clone())
        .unwrap_or("Unknown".to_string());
    let success = semantic_release_state
        .success
        .lock()
        .map(|s| *s)
        .unwrap_or(false);
    let result = semantic_release_state
        .result
        .lock()
        .map(|r| r.clone())
        .unwrap_or("No output available".to_string());

    // Title with status indicator
    let (title_text, title_color) = if success {
        ("✅ Results", Color::Green)
    } else {
        ("❌ Error Results", Color::Red)
    };

    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Results content with scrolling
    let result_lines: Vec<Line> = result
        .lines()
        .map(|line| {
            // Color code different types of output
            let style = if line.contains("error")
                || line.contains("Error")
                || line.contains("ERROR")
            {
                Style::default().fg(Color::Red)
            } else if line.contains("warning") || line.contains("Warning") || line.contains("WARN")
            {
                Style::default().fg(Color::Yellow)
            } else if line.contains("success") || line.contains("Success") || line.contains("✓") {
                Style::default().fg(Color::Green)
            } else if line.starts_with("[") && line.contains("]") {
                Style::default().fg(Color::Cyan) // Timestamps and info
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(vec![Span::styled(line, style)])
        })
        .collect();

    let results_paragraph = Paragraph::new(result_lines)
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Output: {}", status)),
        )
        .wrap(Wrap { trim: false })
        .scroll((ui_state.scroll_offset as u16, 0));

    f.render_widget(results_paragraph, chunks[1]);

    // Optional: Add scrollbar if content is long
    if result.lines().count() > (chunks[1].height as usize - 2) {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(result.lines().count())
            .viewport_content_length(chunks[1].height as usize - 2)
            .position(ui_state.scroll_offset);

        f.render_stateful_widget(
            scrollbar,
            chunks[1].inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn create_semantic_release_options(selected: usize, _config: &AppConfig) -> Vec<ListItem> {
    let options = vec![
        (
            "🔍 Dry Run",
            "Check what would be released without making changes",
            check_prerequisites(),
        ),
        (
            "🚀 Release",
            "Execute semantic-release and publish new version",
            check_prerequisites(),
        ),
        (
            "📊 Version Info",
            "Get detailed version analysis and prediction",
            check_prerequisites(),
        ),
        (
            "📦 Last Release",
            "View information about the last release",
            true,
        ),
        (
            "⚙️ Configuration",
            "Check semantic-release configuration status",
            true,
        ),
        (
            "🔧 Setup GitHub Actions",
            "Configure GitHub Actions for automated semantic-release",
            true,
        ),
    ];

    options
        .into_iter()
        .enumerate()
        .map(|(i, (title, description, enabled))| {
            let style = if i == selected {
                if enabled {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray).bg(Color::Yellow)
                }
            } else if enabled {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let status_indicator = if enabled { "✅" } else { "❌" };
            let content = vec![
                Line::from(vec![Span::styled(
                    format!("{} {}", status_indicator, title),
                    style.add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::styled(format!("   {}", description), style)]),
            ];

            ListItem::new(content).style(style)
        })
        .collect()
}

fn check_prerequisites() -> bool {
    // Check if we have the basic requirements for semantic-release
    // This is a simple check - the actual verification happens when running

    // Check if package.json exists (basic Node.js project requirement)
    let has_package_json = std::path::Path::new("package.json").exists();

    // Check if we're in a git repository
    let is_git_repo = std::path::Path::new(".git").exists();

    // For now, just check these basic requirements
    // More detailed checks happen in the actual operations
    has_package_json && is_git_repo
}
