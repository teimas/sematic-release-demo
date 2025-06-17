use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_release_notes_screen(f: &mut Frame, area: Rect) {
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Release Notes"),
    )
    .wrap(Wrap { trim: true });

    f.render_widget(content, area);
}
