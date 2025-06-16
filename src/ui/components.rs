use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::types::AppState;

pub fn draw_title_bar(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("🚀 Semantic Release TUI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

pub fn draw_status_bar(f: &mut Frame, area: Rect, app_state: &AppState, message: Option<&str>) {
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
        AppState::ConfirmingStageAll => (message.unwrap_or("Press 'y' to stage all changes (git add -A), 'n' to cancel"), "❓ Confirmation Required"),
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
        AppState::ConfirmingStageAll => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Green),
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(status, area);
} 