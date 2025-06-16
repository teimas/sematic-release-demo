use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::ui::state::UIState;
use crate::git::GitStatus;

pub fn draw_main_screen(f: &mut Frame, area: Rect, ui_state: &mut UIState, git_status: Option<&GitStatus>) {
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
        Line::from("• r: AI analysis (description + security + breaking changes)"),
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