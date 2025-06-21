//! Confirmation Dialog Component
//!
//! A confirmation dialog component with customizable buttons, theme integration,
//! and keyboard navigation for user confirmation workflows.

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
pub enum ConfirmationResult {
    Confirmed,
    Cancelled,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationType {
    YesNo,
    OkCancel,
    Custom { confirm_text: String, cancel_text: String },
}

impl ConfirmationType {
    pub fn button_texts(&self) -> (String, String) {
        match self {
            ConfirmationType::YesNo => ("Yes".to_string(), "No".to_string()),
            ConfirmationType::OkCancel => ("OK".to_string(), "Cancel".to_string()),
            ConfirmationType::Custom { confirm_text, cancel_text } => {
                (confirm_text.clone(), cancel_text.clone())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationProps {
    pub title: String,
    pub message: String,
    pub confirmation_type: ConfirmationType,
    pub destructive: bool,
    pub show_icons: bool,
    pub auto_focus_confirm: bool,
}

impl Default for ConfirmationProps {
    fn default() -> Self {
        Self {
            title: "Confirm".to_string(),
            message: "Are you sure?".to_string(),
            confirmation_type: ConfirmationType::YesNo,
            destructive: false,
            show_icons: true,
            auto_focus_confirm: true,
        }
    }
}

impl ComponentProps for ConfirmationProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationComponentState {
    pub common: CommonComponentState,
    pub is_open: bool,
    pub selected_button: usize, // 0 = confirm, 1 = cancel
    pub result: Option<ConfirmationResult>,
}

impl ComponentState for ConfirmationComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for ConfirmationComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            is_open: false,
            selected_button: 0,
            result: None,
        }
    }
}

pub struct Confirmation {
    id: ComponentId,
    props: ConfirmationProps,
    state: ConfirmationComponentState,
}

impl Confirmation {
    pub fn new(id: ComponentId, props: ConfirmationProps) -> Self {
        let selected_button = if props.auto_focus_confirm { 0 } else { 1 };
        
        Self {
            id,
            props,
            state: ConfirmationComponentState {
                selected_button,
                ..ConfirmationComponentState::default()
            },
        }
    }

    pub fn open(&mut self) {
        self.state.is_open = true;
        self.state.result = None;
        self.state.common.mark_dirty();
    }

    pub fn close(&mut self) {
        self.state.is_open = false;
        self.state.common.mark_dirty();
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.props.message = message.into();
        self.state.common.mark_dirty();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.props.title = title.into();
        self.state.common.mark_dirty();
    }

    pub fn get_result(&self) -> Option<&ConfirmationResult> {
        self.state.result.as_ref()
    }

    pub fn is_confirmed(&self) -> bool {
        matches!(self.state.result, Some(ConfirmationResult::Confirmed))
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self.state.result, Some(ConfirmationResult::Cancelled))
    }

    pub fn is_open(&self) -> bool {
        self.state.is_open
    }

    fn confirm(&mut self) -> ComponentResult<Vec<ComponentEvent>> {
        self.state.result = Some(ConfirmationResult::Confirmed);
        self.close();
        Ok(vec![ComponentEvent::Activated {
            component_id: self.id.clone(),
        }])
    }

    fn cancel(&mut self) -> ComponentResult<Vec<ComponentEvent>> {
        self.state.result = Some(ConfirmationResult::Cancelled);
        self.close();
        Ok(vec![ComponentEvent::Activated {
            component_id: self.id.clone(),
        }])
    }

    fn dismiss(&mut self) -> ComponentResult<Vec<ComponentEvent>> {
        self.state.result = Some(ConfirmationResult::Dismissed);
        self.close();
        Ok(vec![ComponentEvent::Activated {
            component_id: self.id.clone(),
        }])
    }

    fn build_button_text(&self, is_confirm: bool, theme: &AppTheme) -> Line {
        let (confirm_text, cancel_text) = self.props.confirmation_type.button_texts();
        let text = if is_confirm { confirm_text } else { cancel_text };
        
        let mut spans = Vec::new();
        
        if self.props.show_icons {
            let icon = if is_confirm {
                if self.props.destructive { "⚠ " } else { "✓ " }
            } else {
                "✗ "
            };
            spans.push(Span::raw(icon));
        }
        
        spans.push(Span::raw(text));
        
        Line::from(spans)
    }
}

#[async_trait]
impl Component for Confirmation {
    type Props = ConfirmationProps;
    type State = ConfirmationComponentState;

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
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.state.selected_button = 1 - self.state.selected_button;
                self.state.common.mark_dirty();
                Ok(vec![])
            }
            KeyCode::Enter => {
                if self.state.selected_button == 0 {
                    self.confirm()
                } else {
                    self.cancel()
                }
            }
            KeyCode::Esc => {
                self.dismiss()
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if matches!(self.props.confirmation_type, ConfirmationType::YesNo) {
                    self.confirm()
                } else {
                    Ok(vec![])
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                if matches!(self.props.confirmation_type, ConfirmationType::YesNo) {
                    self.cancel()
                } else {
                    Ok(vec![])
                }
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                if matches!(self.props.confirmation_type, ConfirmationType::OkCancel) {
                    self.confirm()
                } else {
                    Ok(vec![])
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if matches!(self.props.confirmation_type, ConfirmationType::OkCancel) {
                    self.cancel()
                } else {
                    Ok(vec![])
                }
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.dismiss()
            }
            _ => Ok(vec![])
        }
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::Activated { .. } => {
                // Handle confirmation result
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

        // Calculate dialog position (centered)
        let dialog_area = centered_rect(60, 30, area);

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Render dialog background with theme colors
        let dialog_block = Block::default()
            .title(self.props.title.as_str())
            .borders(Borders::ALL)
            .border_style(if self.props.destructive {
                Style::default().fg(theme.colors.error)
            } else {
                Style::default().fg(theme.colors.focus)
            })
            .style(Style::default().bg(theme.colors.background).fg(theme.colors.foreground));

        // Calculate inner area before moving the block
        let inner_area = dialog_block.inner(dialog_area);
        frame.render_widget(dialog_block, dialog_area);

        // Split dialog into message and buttons
        let layout_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(inner_area);

        // Render message
        let message_paragraph = Paragraph::new(self.props.message.as_str())
            .style(Style::default().fg(theme.colors.foreground))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(message_paragraph, layout_chunks[0]);

        // Render buttons
        let button_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout_chunks[1]);

        let (confirm_text, cancel_text) = self.props.confirmation_type.button_texts();

        // Confirm button
        let confirm_style = if self.state.selected_button == 0 {
            if self.props.destructive {
                Style::default()
                    .bg(theme.colors.error)
                    .fg(theme.colors.background)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(theme.colors.success)
                    .fg(theme.colors.background)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(theme.colors.foreground)
        };

        let confirm_button = Paragraph::new(self.build_button_text(true, &theme))
            .style(confirm_style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(confirm_button, button_area[0]);

        // Cancel button
        let cancel_style = if self.state.selected_button == 1 {
            Style::default()
                .bg(theme.colors.info)
                .fg(theme.colors.background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.colors.foreground)
        };

        let cancel_button = Paragraph::new(self.build_button_text(false, &theme))
            .style(cancel_style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(cancel_button, button_area[1]);

        // Render keyboard shortcuts hint
        let hint_text = match self.props.confirmation_type {
            ConfirmationType::YesNo => "Y/N: quick select, Tab/←→: navigate, Enter: confirm, Esc: dismiss",
            ConfirmationType::OkCancel => "O/C: quick select, Tab/←→: navigate, Enter: confirm, Esc: dismiss",
            ConfirmationType::Custom { .. } => "Tab/←→: navigate, Enter: confirm, Esc: dismiss",
        };

        let hint_paragraph = Paragraph::new(Line::from(Span::styled(
            hint_text,
            Style::default().fg(theme.colors.info)
        )))
        .alignment(Alignment::Center);
        frame.render_widget(hint_paragraph, layout_chunks[2]);
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        if self.props.title.is_empty() {
            return ValidationState::Invalid("Confirmation title cannot be empty".to_string());
        }

        if self.props.message.is_empty() {
            return ValidationState::Invalid("Confirmation message cannot be empty".to_string());
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