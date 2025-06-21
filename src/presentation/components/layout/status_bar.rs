//! Status Bar Layout Component
//!
//! A status bar component for displaying application status, progress,
//! notifications, and contextual information at the bottom of the screen.

use crate::presentation::components::core::{
    Component, ComponentState, ComponentProps, ComponentResult, ComponentId,
    ValidationState, CommonComponentState, ComponentEvent,
};
use crate::presentation::theme::AppTheme;
use crate::state::StateEvent;
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Gauge, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl StatusLevel {
    pub fn color(&self, theme: &AppTheme) -> ratatui::style::Color {
        match self {
            StatusLevel::Info => theme.colors.info,
            StatusLevel::Success => theme.colors.success,
            StatusLevel::Warning => theme.colors.warning,
            StatusLevel::Error => theme.colors.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMessage {
    pub text: String,
    pub level: StatusLevel,
    pub timestamp: Option<std::time::SystemTime>,
    pub persistent: bool,
    pub timeout: Option<Duration>,
}

impl StatusMessage {
    pub fn new(text: impl Into<String>, level: StatusLevel) -> Self {
        Self {
            text: text.into(),
            level,
            timestamp: Some(std::time::SystemTime::now()),
            persistent: false,
            timeout: Some(Duration::from_secs(5)),
        }
    }

    pub fn info(text: impl Into<String>) -> Self {
        Self::new(text, StatusLevel::Info)
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self::new(text, StatusLevel::Success)
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(text, StatusLevel::Warning)
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self::new(text, StatusLevel::Error)
    }

    pub fn persistent(mut self) -> Self {
        self.persistent = true;
        self.timeout = None;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current: u64,
    pub total: u64,
    pub label: String,
    pub show_percentage: bool,
    pub show_eta: bool,
}

impl ProgressInfo {
    pub fn new(current: u64, total: u64, label: impl Into<String>) -> Self {
        Self {
            current,
            total,
            label: label.into(),
            show_percentage: true,
            show_eta: false,
        }
    }

    pub fn percentage(&self) -> u16 {
        if self.total == 0 {
            0
        } else {
            ((self.current * 100) / self.total) as u16
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarProps {
    pub show_time: bool,
    pub show_progress: bool,
    pub show_shortcuts: bool,
    pub left_text: String,
    pub center_text: String,
    pub right_text: String,
    pub shortcuts: HashMap<String, String>,
}

impl Default for StatusBarProps {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("F1".to_string(), "Help".to_string());
        shortcuts.insert("Ctrl+C".to_string(), "Quit".to_string());
        
        Self {
            show_time: true,
            show_progress: true,
            show_shortcuts: true,
            left_text: String::new(),
            center_text: String::new(),
            right_text: String::new(),
            shortcuts,
        }
    }
}

impl ComponentProps for StatusBarProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarComponentState {
    pub common: CommonComponentState,
    pub current_message: Option<StatusMessage>,
    pub progress: Option<ProgressInfo>,
    pub last_update: Option<std::time::SystemTime>,
    pub blink_state: bool,
}

impl ComponentState for StatusBarComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for StatusBarComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            current_message: None,
            progress: None,
            last_update: None,
            blink_state: false,
        }
    }
}

pub struct StatusBar {
    id: ComponentId,
    props: StatusBarProps,
    state: StatusBarComponentState,
}

impl StatusBar {
    pub fn new(id: ComponentId, props: StatusBarProps) -> Self {
        Self {
            id,
            props,
            state: StatusBarComponentState::default(),
        }
    }

    pub fn set_message(&mut self, message: StatusMessage) {
        self.state.current_message = Some(message);
        self.state.last_update = Some(std::time::SystemTime::now());
        self.state.common.mark_dirty();
    }

    pub fn clear_message(&mut self) {
        self.state.current_message = None;
        self.state.common.mark_dirty();
    }

    pub fn set_progress(&mut self, progress: Option<ProgressInfo>) {
        self.state.progress = progress;
        self.state.common.mark_dirty();
    }

    pub fn update_progress(&mut self, current: u64, total: u64) {
        if let Some(ref mut progress) = self.state.progress {
            progress.current = current;
            progress.total = total;
            self.state.common.mark_dirty();
        }
    }

    pub fn set_left_text(&mut self, text: impl Into<String>) {
        self.props.left_text = text.into();
        self.state.common.mark_dirty();
    }

    pub fn set_center_text(&mut self, text: impl Into<String>) {
        self.props.center_text = text.into();
        self.state.common.mark_dirty();
    }

    pub fn set_right_text(&mut self, text: impl Into<String>) {
        self.props.right_text = text.into();
        self.state.common.mark_dirty();
    }

    fn should_clear_message(&self) -> bool {
        if let Some(ref message) = self.state.current_message {
            if message.persistent {
                return false;
            }
            
            if let (Some(timeout), Some(timestamp)) = (&message.timeout, &message.timestamp) {
                if let Ok(elapsed) = timestamp.elapsed() {
                    return elapsed > *timeout;
                }
            }
        }
        false
    }

    fn get_current_time(&self) -> String {
        use std::time::SystemTime;
        let now = SystemTime::now();
        // Simple time formatting - in a real app you'd use chrono
        format!("{:?}", now).split('.').next().unwrap_or("").to_string()
    }

    fn build_shortcuts_text(&self) -> String {
        if !self.props.show_shortcuts || self.props.shortcuts.is_empty() {
            return String::new();
        }

        self.props.shortcuts
            .iter()
            .map(|(key, desc)| format!("{}: {}", key, desc))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

#[async_trait]
impl Component for StatusBar {
    type Props = StatusBarProps;
    type State = StatusBarComponentState;

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

    async fn handle_key(&mut self, _key: KeyEvent) -> ComponentResult<Vec<ComponentEvent>> {
        // Status bar typically doesn't handle keys directly
        Ok(vec![])
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::Activated { .. } => {
                // Handle status activation events
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = AppTheme::default();
        
        // Split the status bar into sections
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(area);

        // Left section: Status message or left text
        let left_content = if let Some(ref message) = self.state.current_message {
            let style = Style::default()
                .fg(message.level.color(&theme))
                .add_modifier(if matches!(message.level, StatusLevel::Error) {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });
            
            Paragraph::new(Line::from(Span::styled(&message.text, style)))
                .alignment(Alignment::Left)
        } else if !self.props.left_text.is_empty() {
            Paragraph::new(Line::from(self.props.left_text.as_str()))
                .style(Style::default().fg(theme.colors.foreground))
                .alignment(Alignment::Left)
        } else {
            Paragraph::new(Line::from("Ready"))
                .style(Style::default().fg(theme.colors.success))
                .alignment(Alignment::Left)
        };
        frame.render_widget(left_content, chunks[0]);

        // Center section: Progress or center text
        let center_content = if let Some(ref progress) = self.state.progress {
            if self.props.show_progress {
                let progress_text = if progress.show_percentage {
                    format!("{} {}%", progress.label, progress.percentage())
                } else {
                    format!("{} {}/{}", progress.label, progress.current, progress.total)
                };
                
                // Create a smaller area for the gauge
                let progress_area = Rect {
                    x: chunks[1].x,
                    y: chunks[1].y,
                    width: chunks[1].width,
                    height: 1,
                };
                
                let gauge = Gauge::default()
                    .block(Block::default())
                    .gauge_style(Style::default().fg(theme.colors.success))
                    .ratio(progress.percentage() as f64 / 100.0)
                    .label(progress_text);
                
                frame.render_widget(gauge, progress_area);
                return; // Early return to avoid rendering center text
            } else {
                Paragraph::new(Line::from(format!("{}: {}/{}", 
                    progress.label, progress.current, progress.total)))
                    .style(Style::default().fg(theme.colors.info))
                    .alignment(Alignment::Center)
            }
        } else if !self.props.center_text.is_empty() {
            Paragraph::new(Line::from(self.props.center_text.as_str()))
                .style(Style::default().fg(theme.colors.foreground))
                .alignment(Alignment::Center)
        } else {
            let shortcuts_text = self.build_shortcuts_text();
            Paragraph::new(Line::from(shortcuts_text))
                .style(Style::default().fg(theme.colors.info))
                .alignment(Alignment::Center)
        };
        frame.render_widget(center_content, chunks[1]);

        // Right section: Time or right text
        let right_content = if self.props.show_time && self.props.right_text.is_empty() {
            let time_text = self.get_current_time();
            Paragraph::new(Line::from(time_text))
                .style(Style::default().fg(theme.colors.foreground))
                .alignment(Alignment::Right)
        } else if !self.props.right_text.is_empty() {
            Paragraph::new(Line::from(self.props.right_text.as_str()))
                .style(Style::default().fg(theme.colors.foreground))
                .alignment(Alignment::Right)
        } else {
            Paragraph::new(Line::from(""))
                .alignment(Alignment::Right)
        };
        frame.render_widget(right_content, chunks[2]);
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        // Check if we should clear expired messages
        if self.should_clear_message() {
            self.clear_message();
            return Ok(true);
        }
        
        // Toggle blink state for blinking elements
        self.state.blink_state = !self.state.blink_state;
        
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        ValidationState::Valid
    }
} 