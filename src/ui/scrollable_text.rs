use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use crate::ui::state::{CommitField, UIState};

pub fn render_scrollable_text(
    f: &mut Frame,
    area: Rect,
    text: &str,
    field: &CommitField,
    ui_state: &UIState,
    block: Block,
    style: Style,
) {
    let scroll_offset = ui_state.get_field_scroll_offset(field);
    let visible_height = area.height.saturating_sub(2); // Account for borders

    // Split text into lines
    let lines: Vec<&str> = text.lines().collect();

    // Calculate which lines should be visible based on scroll offset
    let start_line = scroll_offset as usize;
    let end_line = (start_line + visible_height as usize).min(lines.len());

    // Create the visible text
    let visible_lines = if start_line < lines.len() {
        &lines[start_line..end_line]
    } else {
        &[]
    };

    let visible_text = visible_lines.join("\n");

    let paragraph = Paragraph::new(visible_text)
        .block(block)
        .style(style)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

pub fn calculate_required_scroll(
    text: &str,
    cursor_pos: usize,
    field_width: u16,
    visible_height: u16,
    current_scroll: u16,
) -> u16 {
    // Calculate what line the cursor is on
    let cursor_line = get_cursor_line(text, cursor_pos, field_width);

    // Calculate the scroll needed to keep cursor visible
    if cursor_line < current_scroll as usize {
        // Cursor is above visible area, scroll up
        cursor_line as u16
    } else if cursor_line >= (current_scroll + visible_height) as usize {
        // Cursor is below visible area, scroll down
        (cursor_line + 1).saturating_sub(visible_height as usize) as u16
    } else {
        // Cursor is already visible
        current_scroll
    }
}

fn get_cursor_line(text: &str, cursor_pos: usize, field_width: u16) -> usize {
    let text_chars: Vec<char> = text.chars().collect();
    let cursor_pos = cursor_pos.min(text_chars.len());

    let mut line = 0;
    let mut x = 0;

    for (i, &ch) in text_chars.iter().enumerate() {
        if i >= cursor_pos {
            break;
        }

        if ch == '\n' {
            line += 1;
            x = 0;
        } else {
            x += 1;
            if x >= field_width {
                line += 1;
                x = 0;
            }
        }
    }

    line
}
