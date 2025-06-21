//! Split Pane Layout Component
//!
//! A resizable split pane component that can be oriented horizontally or vertically.
//! Supports keyboard resizing, drag handles, and theme integration.

use crate::presentation::components::core::{
    Component, ComponentState, ComponentProps, ComponentResult, ComponentId,
    ValidationState, CommonComponentState, ComponentEvent,
};
use crate::presentation::theme::AppTheme;
use crate::state::StateEvent;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

impl Default for SplitOrientation {
    fn default() -> Self {
        Self::Horizontal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPaneProps {
    pub orientation: SplitOrientation,
    pub first_size: u16,  // Percentage (0-100)
    pub second_size: u16, // Percentage (0-100)
    pub min_size: u16,    // Minimum size percentage
    pub max_size: u16,    // Maximum size percentage
    pub resizable: bool,
    pub show_divider: bool,
    pub title_first: Option<String>,
    pub title_second: Option<String>,
}

impl Default for SplitPaneProps {
    fn default() -> Self {
        Self {
            orientation: SplitOrientation::Horizontal,
            first_size: 50,
            second_size: 50,
            min_size: 10,
            max_size: 90,
            resizable: true,
            show_divider: true,
            title_first: None,
            title_second: None,
        }
    }
}

impl ComponentProps for SplitPaneProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPaneComponentState {
    pub common: CommonComponentState,
    pub active_pane: usize,
    pub is_resizing: bool,
    pub first_size: u16,
    pub second_size: u16,
}

impl ComponentState for SplitPaneComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for SplitPaneComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            active_pane: 0,
            is_resizing: false,
            first_size: 50,
            second_size: 50,
        }
    }
}

pub struct SplitPane {
    id: ComponentId,
    props: SplitPaneProps,
    state: SplitPaneComponentState,
}

impl SplitPane {
    pub fn new(id: ComponentId, props: SplitPaneProps) -> Self {
        let mut state = SplitPaneComponentState::default();
        state.first_size = props.first_size;
        state.second_size = props.second_size;
        
        Self {
            id,
            props,
            state,
        }
    }

    pub fn set_active_pane(&mut self, pane: usize) {
        if pane < 2 {
            self.state.active_pane = pane;
            self.state.common.mark_dirty();
        }
    }

    pub fn resize_panes(&mut self, delta: i16) {
        if !self.props.resizable {
            return;
        }

        let new_first_size = (self.state.first_size as i16 + delta).clamp(
            self.props.min_size as i16,
            self.props.max_size as i16,
        ) as u16;

        let new_second_size = 100 - new_first_size;

        if new_second_size >= self.props.min_size && new_second_size <= self.props.max_size {
            self.state.first_size = new_first_size;
            self.state.second_size = new_second_size;
            self.state.common.mark_dirty();
        }
    }

    fn render_divider(&self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        if !self.props.show_divider {
            return;
        }

        let divider_style = if self.state.is_resizing {
            Style::default().fg(theme.colors.focus)
        } else {
            Style::default().fg(theme.colors.border)
        };

        let divider_symbol = match self.props.orientation {
            SplitOrientation::Horizontal => symbols::line::VERTICAL,
            SplitOrientation::Vertical => symbols::line::HORIZONTAL,
        };

        let divider_text = match self.props.orientation {
            SplitOrientation::Horizontal => {
                let height = area.height;
                (0..height).map(|_| divider_symbol).collect::<String>()
            }
            SplitOrientation::Vertical => divider_symbol.repeat(area.width as usize),
        };

        let divider = Paragraph::new(Line::from(Span::styled(divider_text, divider_style)));
        frame.render_widget(divider, area);
    }
}

#[async_trait]
impl Component for SplitPane {
    type Props = SplitPaneProps;
    type State = SplitPaneComponentState;

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
        match key.code {
            KeyCode::Tab => {
                self.state.active_pane = 1 - self.state.active_pane;
                self.state.common.mark_dirty();
                Ok(vec![ComponentEvent::FocusGained {
                    component_id: self.id.clone(),
                }])
            }
            KeyCode::Left | KeyCode::Right if self.props.resizable => {
                let delta = match (key.code, self.props.orientation) {
                    (KeyCode::Left, SplitOrientation::Horizontal) => -5,
                    (KeyCode::Right, SplitOrientation::Horizontal) => 5,
                    _ => 0,
                };
                if delta != 0 {
                    self.resize_panes(delta);
                }
                Ok(vec![])
            }
            KeyCode::Up | KeyCode::Down if self.props.resizable => {
                let delta = match (key.code, self.props.orientation) {
                    (KeyCode::Up, SplitOrientation::Vertical) => -5,
                    (KeyCode::Down, SplitOrientation::Vertical) => 5,
                    _ => 0,
                };
                if delta != 0 {
                    self.resize_panes(delta);
                }
                Ok(vec![])
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Reset to default sizes
                self.state.first_size = self.props.first_size;
                self.state.second_size = self.props.second_size;
                self.state.common.mark_dirty();
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::FocusGained { .. } => {
                self.state.is_resizing = true;
                self.state.common.mark_dirty();
                Ok(vec![])
            }
            ComponentEvent::FocusLost { .. } => {
                self.state.is_resizing = false;
                self.state.common.mark_dirty();
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = AppTheme::default();
        
        // Calculate constraints based on current sizes
        let constraints = vec![
            Constraint::Percentage(self.state.first_size),
            Constraint::Percentage(self.state.second_size),
        ];

        let direction = match self.props.orientation {
            SplitOrientation::Horizontal => Direction::Horizontal,
            SplitOrientation::Vertical => Direction::Vertical,
        };

        let layout = Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(area);

        // Render first pane
        let first_style = if self.state.active_pane == 0 {
            Style::default().fg(theme.colors.focus)
        } else {
            Style::default().fg(theme.colors.border)
        };
        
        let first_title = self.props.title_first
            .as_deref()
            .unwrap_or("Pane 1");
        
        let first_block = Block::default()
            .borders(Borders::ALL)
            .border_style(first_style)
            .title(first_title);
        frame.render_widget(first_block, layout[0]);

        // Render second pane
        let second_style = if self.state.active_pane == 1 {
            Style::default().fg(theme.colors.focus)
        } else {
            Style::default().fg(theme.colors.border)
        };
        
        let second_title = self.props.title_second
            .as_deref()
            .unwrap_or("Pane 2");
        
        let second_block = Block::default()
            .borders(Borders::ALL)
            .border_style(second_style)
            .title(second_title);
        frame.render_widget(second_block, layout[1]);

        // Render resize hint if resizing
        if self.state.is_resizing && self.props.resizable {
            let hint_area = Rect {
                x: area.x,
                y: area.y + area.height - 1,
                width: area.width,
                height: 1,
            };
            
            let hint_text = match self.props.orientation {
                SplitOrientation::Horizontal => "← → to resize, Ctrl+R to reset",
                SplitOrientation::Vertical => "↑ ↓ to resize, Ctrl+R to reset",
            };
            
            let hint = Paragraph::new(Line::from(Span::styled(
                hint_text,
                Style::default().fg(theme.colors.info)
            )));
            frame.render_widget(hint, hint_area);
        }
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        if self.state.first_size + self.state.second_size != 100 {
            return ValidationState::Invalid("Pane sizes must sum to 100%".to_string());
        }
        
        if self.state.first_size < self.props.min_size || self.state.first_size > self.props.max_size {
            return ValidationState::Invalid("First pane size out of bounds".to_string());
        }
        
        if self.state.second_size < self.props.min_size || self.state.second_size > self.props.max_size {
            return ValidationState::Invalid("Second pane size out of bounds".to_string());
        }
        
        ValidationState::Valid
    }
} 