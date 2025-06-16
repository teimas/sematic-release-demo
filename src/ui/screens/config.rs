use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_config_screen(f: &mut Frame, area: Rect) {
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