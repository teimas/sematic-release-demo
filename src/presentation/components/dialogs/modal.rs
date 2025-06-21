//! Modal Dialog Component
//!
//! A flexible modal dialog component with support for custom content,
//! theme integration, and keyboard navigation.

use crate::presentation::components::core::{
    Component, ComponentState, ComponentProps, ComponentResult, ComponentId,
    ValidationState, CommonComponentState, ComponentEvent,
};
use crate::presentation::theme::AppTheme;
use crate::state::StateEvent;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    Custom { width: u16, height: u16 },
}

impl ModalSize {
    pub fn dimensions(&self) -> (u16, u16) {
        match self {
            ModalSize::Small => (40, 15),
            ModalSize::Medium => (60, 25),
            ModalSize::Large => (80, 35),
            ModalSize::Custom { width, height } => (*width, *height),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalProps {
    pub title: String,
    pub content: String,
    pub size: ModalSize,
    pub closable: bool,
    pub show_close_button: bool,
    pub center_content: bool,
    pub wrap_content: bool,
}

impl Default for ModalProps {
    fn default() -> Self {
        Self {
            title: "Modal".to_string(),
            content: "Modal content".to_string(),
            size: ModalSize::Medium,
            closable: true,
            show_close_button: true,
            center_content: true,
            wrap_content: true,
        }
    }
}

impl ComponentProps for ModalProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalComponentState {
    pub common: CommonComponentState,
    pub is_open: bool,
    pub can_close: bool,
}

impl ComponentState for ModalComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for ModalComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            is_open: false,
            can_close: true,
        }
    }
}

pub struct Modal {
    id: ComponentId,
    props: ModalProps,
    state: ModalComponentState,
}

impl Modal {
    pub fn new(id: ComponentId, props: ModalProps) -> Self {
        Self {
            id,
            props,
            state: ModalComponentState::default(),
        }
    }

    pub fn open(&mut self) {
        self.state.is_open = true;
        self.state.common.mark_dirty();
    }

    pub fn close(&mut self) {
        if self.state.can_close && self.props.closable {
            self.state.is_open = false;
            self.state.common.mark_dirty();
        }
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.props.content = content.into();
        self.state.common.mark_dirty();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.props.title = title.into();
        self.state.common.mark_dirty();
    }

    pub fn set_closable(&mut self, closable: bool) {
        self.state.can_close = closable;
        self.state.common.mark_dirty();
    }

    pub fn is_open(&self) -> bool {
        self.state.is_open
    }

    fn build_title(&self, theme: &AppTheme) -> Line {
        let mut spans = vec![Span::styled(
            &self.props.title,
            Style::default().fg(theme.colors.foreground).add_modifier(Modifier::BOLD)
        )];

        if self.props.show_close_button && self.props.closable {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                "[×]",
                Style::default().fg(theme.colors.error)
            ));
        }

        Line::from(spans)
    }
}

#[async_trait]
impl Component for Modal {
    type Props = ModalProps;
    type State = ModalComponentState;

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn props(&self) -> &Self::Props {
        &self.props
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    async fn handle_key(&mut self, key: KeyEvent) -> ComponentResult<Vec<ComponentEvent>> {
        if !self.state.is_open {
            return Ok(vec![]);
        }

        match key.code {
            KeyCode::Esc => {
                if self.props.closable && self.state.can_close {
                    self.close();
                    Ok(vec![ComponentEvent::Activated {
                        component_id: self.id.clone(),
                    }])
                } else {
                    Ok(vec![])
                }
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.props.closable && self.state.can_close {
                    self.close();
                    Ok(vec![ComponentEvent::Activated {
                        component_id: self.id.clone(),
                    }])
                } else {
                    Ok(vec![])
                }
            }
            KeyCode::Enter => {
                Ok(vec![ComponentEvent::Activated {
                    component_id: self.id.clone(),
                }])
            }
            _ => Ok(vec![])
        }
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::Activated { .. } => {
                // Handle modal activation (e.g., close or confirm)
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.state.is_open {
            return;
        }

        let theme = AppTheme::default();
        let (width, height) = self.props.size.dimensions();

        // Calculate modal position (centered)
        let modal_area = centered_rect(width, height, area);

        // Clear the area behind the modal (creates overlay effect)
        frame.render_widget(Clear, modal_area);

        // Render modal background with theme colors
        let modal_block = Block::default()
            .title(self.build_title(&theme))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.colors.focus))
            .style(Style::default().bg(theme.colors.background).fg(theme.colors.foreground));

        // Calculate inner area before moving the block
        let inner_area = modal_block.inner(modal_area);
        frame.render_widget(modal_block, modal_area);

        // Render modal content
        let content_area = if self.props.closable && self.state.can_close {
            // Leave space for close instructions
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(inner_area)[0]
        } else {
            inner_area
        };

        let content_style = Style::default().fg(theme.colors.foreground);
        let content_paragraph = if self.props.wrap_content {
            Paragraph::new(self.props.content.as_str())
                .style(content_style)
                .wrap(Wrap { trim: true })
                .alignment(if self.props.center_content {
                    Alignment::Center
                } else {
                    Alignment::Left
                })
        } else {
            Paragraph::new(Line::from(self.props.content.as_str()))
                .style(content_style)
                .alignment(if self.props.center_content {
                    Alignment::Center
                } else {
                    Alignment::Left
                })
        };

        frame.render_widget(content_paragraph, content_area);

        // Render close instructions if closable
        if self.props.closable && self.state.can_close {
            let close_area = Rect {
                x: inner_area.x,
                y: inner_area.y + inner_area.height - 1,
                width: inner_area.width,
                height: 1,
            };

            let close_text = "Press ESC or Ctrl+Q to close, Enter to confirm";
            let close_paragraph = Paragraph::new(Line::from(Span::styled(
                close_text,
                Style::default().fg(theme.colors.info)
            )))
            .alignment(Alignment::Center);

            frame.render_widget(close_paragraph, close_area);
        }
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        if self.props.title.is_empty() {
            return ValidationState::Invalid("Modal title cannot be empty".to_string());
        }

        let (width, height) = self.props.size.dimensions();
        if width < 10 || height < 5 {
            return ValidationState::Invalid("Modal size too small".to_string());
        }

        ValidationState::Valid
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