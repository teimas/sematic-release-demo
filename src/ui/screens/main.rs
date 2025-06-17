use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::ui::state::UIState;
use crate::git::GitStatus;

pub fn draw_main_screen(f: &mut Frame, area: Rect, ui_state: &mut UIState, git_status: Option<&GitStatus>) {
    let tabs = Tabs::new(vec!["💾 Commit", "📝 Release Notes", "🚀 Semantic Release", "⚙️ Config", "📋 Help"])
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(ui_state.selected_tab);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    
    f.render_widget(tabs, chunks[0]);

    // Build content with TEIMAS ASCII logo using non-breaking spaces
    let mut content_lines = vec![
        Line::from(""),
        // TEIMAS ASCII Logo - using dots for spacing to prevent trimming
        Line::from(vec![
            Span::styled("#################  ################ ###### ######        ######  ##############      #############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("################## ################ ###### ########     #######  ###############   ################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("#################  ################ ###### #########   ########   ############### ################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("#####", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("............", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("##########", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("#########", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("....", Style::default().fg(Color::Black)),
            Span::styled("#", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("......", Style::default().fg(Color::Black)),
            Span::styled("##", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("##############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("...", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("####################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("############################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("##############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("...", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("####################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("..", Style::default().fg(Color::Black)),
            Span::styled("#################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("###############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("##############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("...", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("##############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("#####", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("##################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("...", Style::default().fg(Color::Black)),
            Span::styled("##############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("#####", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("............", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("..", Style::default().fg(Color::Black)),
            Span::styled("#####", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("..", Style::default().fg(Color::Black)),
            Span::styled("##", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("......", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("..", Style::default().fg(Color::Black)),
            Span::styled("####", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("...", Style::default().fg(Color::Black)),
            Span::styled("#####", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("##################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(".....", Style::default().fg(Color::Black)),
            Span::styled("#######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".......", Style::default().fg(Color::Black)),
            Span::styled("################", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::Black)),
            Span::styled("######", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".........", Style::default().fg(Color::Black)),
            Span::styled("#####", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("...", Style::default().fg(Color::Black)),
            Span::styled("###############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("..", Style::default().fg(Color::Black)),
            Span::styled("##############", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("                           Semantic Release TUI - Terminal Interface", Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(""),
    ];

    // Add git status information
    if let Some(status) = git_status {
        content_lines.push(Line::from(vec![
            Span::styled("📂 Repository Status:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
        
        if !status.staged.is_empty() {
            content_lines.push(Line::from(vec![
                Span::styled("✅ ", Style::default().fg(Color::Green)),
                Span::styled(format!("Staged files ({}): Ready to commit", status.staged.len()), Style::default().fg(Color::Green)),
            ]));
            for file in status.staged.iter().take(5) {
                content_lines.push(Line::from(vec![
                    Span::styled("   • ", Style::default().fg(Color::Green)),
                    Span::styled(file, Style::default().fg(Color::White)),
                ]));
            }
            if status.staged.len() > 5 {
                content_lines.push(Line::from(vec![
                    Span::styled(format!("   ... and {} more", status.staged.len() - 5), Style::default().fg(Color::Gray)),
                ]));
            }
        }
        
        if !status.modified.is_empty() {
            content_lines.push(Line::from(vec![
                Span::styled("🔄 ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("Modified files ({}): Use 'git add' to stage", status.modified.len()), Style::default().fg(Color::Yellow)),
            ]));
            for file in status.modified.iter().take(3) {
                content_lines.push(Line::from(vec![
                    Span::styled("   • ", Style::default().fg(Color::Yellow)),
                    Span::styled(file, Style::default().fg(Color::White)),
                ]));
            }
            if status.modified.len() > 3 {
                content_lines.push(Line::from(vec![
                    Span::styled(format!("   ... and {} more", status.modified.len() - 3), Style::default().fg(Color::Gray)),
                ]));
            }
        }
        
        if !status.untracked.is_empty() {
            content_lines.push(Line::from(vec![
                Span::styled("❓ ", Style::default().fg(Color::Magenta)),
                Span::styled(format!("Untracked files ({}): New files", status.untracked.len()), Style::default().fg(Color::Magenta)),
            ]));
            for file in status.untracked.iter().take(3) {
                content_lines.push(Line::from(vec![
                    Span::styled("   • ", Style::default().fg(Color::Magenta)),
                    Span::styled(file, Style::default().fg(Color::White)),
                ]));
            }
            if status.untracked.len() > 3 {
                content_lines.push(Line::from(vec![
                    Span::styled(format!("   ... and {} more", status.untracked.len() - 3), Style::default().fg(Color::Gray)),
                ]));
            }
        }
        
        if status.staged.is_empty() && status.modified.is_empty() && status.untracked.is_empty() {
            content_lines.push(Line::from(vec![
                Span::styled("✨ ", Style::default().fg(Color::Green)),
                Span::styled("Repository clean: No changes to commit", Style::default().fg(Color::Green)),
            ]));
        }
        content_lines.push(Line::from(""));
    } else {
        content_lines.push(Line::from(vec![
            Span::styled("📂 ", Style::default().fg(Color::Red)),
            Span::styled("Repository Status: Not available (not in git repo?)", Style::default().fg(Color::Red)),
        ]));
        content_lines.push(Line::from(""));
    }

    // Add help content with better styling
    content_lines.extend(vec![
        Line::from(vec![
            Span::styled("🔧 General Navigation:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("Tab/Shift+Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Navigate between tabs and fields", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Select option or edit field", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Go back or cancel editing", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Quit application", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📦 Commit Screen:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Navigate between fields (auto-edit mode)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Add new line in multiline fields", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("t", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Comprehensive AI analysis (JSON response)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Search Monday.com tasks", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Blue)),
            Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Create commit", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📝 Release Notes: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled("Generate AI-powered release notes", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("⚙️ Config: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled("Setup API keys (.env file)", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🚀 ", Style::default().fg(Color::Green)),
            Span::styled("Press Tab to navigate to Commit screen to get started!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
    ]);

    let main_content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::ALL).title("TEIMAS Semantic Release"))
        .wrap(Wrap { trim: true });

    f.render_widget(main_content, chunks[1]);
} 