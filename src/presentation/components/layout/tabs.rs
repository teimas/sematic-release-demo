//! Tabs Layout Component
//!
//! A tabbed interface component with support for multiple tabs, keyboard navigation,
//! closable tabs, icons, and theme integration.

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
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs as RatatuiTabs},
    Frame,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabItem {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub closable: bool,
    pub modified: bool,
    pub tooltip: Option<String>,
}

impl TabItem {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon: None,
            closable: true,
            modified: false,
            tooltip: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabsProps {
    pub tabs: Vec<TabItem>,
    pub show_close_buttons: bool,
    pub scrollable: bool,
    pub max_tab_width: Option<u16>,
    pub show_add_button: bool,
}

impl Default for TabsProps {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            show_close_buttons: true,
            scrollable: true,
            max_tab_width: Some(20),
            show_add_button: false,
        }
    }
}

impl ComponentProps for TabsProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabsComponentState {
    pub common: CommonComponentState,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub hover_index: Option<usize>,
}

impl ComponentState for TabsComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for TabsComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            selected_index: 0,
            scroll_offset: 0,
            hover_index: None,
        }
    }
}

pub struct Tabs {
    id: ComponentId,
    props: TabsProps,
    state: TabsComponentState,
}

impl Tabs {
    pub fn new(id: ComponentId, props: TabsProps) -> Self {
        Self {
            id,
            props,
            state: TabsComponentState::default(),
        }
    }

    pub fn add_tab(&mut self, tab: TabItem) {
        self.props.tabs.push(tab);
        self.state.common.mark_dirty();
    }

    pub fn remove_tab(&mut self, index: usize) -> Option<TabItem> {
        if index < self.props.tabs.len() {
            let tab = self.props.tabs.remove(index);
            
            // Adjust selected index if necessary
            if self.state.selected_index >= self.props.tabs.len() && !self.props.tabs.is_empty() {
                self.state.selected_index = self.props.tabs.len() - 1;
            }
            
            self.state.common.mark_dirty();
            Some(tab)
        } else {
            None
        }
    }

    pub fn select_tab(&mut self, index: usize) {
        if index < self.props.tabs.len() {
            self.state.selected_index = index;
            self.adjust_scroll_for_selection();
            self.state.common.mark_dirty();
        }
    }

    pub fn select_next_tab(&mut self) {
        if !self.props.tabs.is_empty() {
            self.state.selected_index = (self.state.selected_index + 1) % self.props.tabs.len();
            self.adjust_scroll_for_selection();
            self.state.common.mark_dirty();
        }
    }

    pub fn select_previous_tab(&mut self) {
        if !self.props.tabs.is_empty() {
            if self.state.selected_index == 0 {
                self.state.selected_index = self.props.tabs.len() - 1;
            } else {
                self.state.selected_index -= 1;
            }
            self.adjust_scroll_for_selection();
            self.state.common.mark_dirty();
        }
    }

    fn adjust_scroll_for_selection(&mut self) {
        if !self.props.scrollable {
            return;
        }

        // Ensure selected tab is visible
        if self.state.selected_index < self.state.scroll_offset {
            self.state.scroll_offset = self.state.selected_index;
        }
        // Note: We'd need the actual rendered width to calculate the right boundary
        // For now, we'll keep it simple
    }

    fn build_tab_titles(&self) -> Vec<Line> {
        self.props.tabs.iter().enumerate().map(|(i, tab)| {
            let mut spans = Vec::new();
            
            // Add icon if present
            if let Some(icon) = &tab.icon {
                spans.push(Span::raw(format!("{} ", icon)));
            }
            
            // Add title
            spans.push(Span::raw(&tab.title));
            
            // Add modified indicator
            if tab.modified {
                spans.push(Span::raw(" ●"));
            }
            
            // Add close button if enabled and tab is closable
            if self.props.show_close_buttons && tab.closable {
                spans.push(Span::raw(" ×"));
            }
            
            Line::from(spans)
        }).collect()
    }
}

#[async_trait]
impl Component for Tabs {
    type Props = TabsProps;
    type State = TabsComponentState;

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
            KeyCode::Left | KeyCode::Char('h') => {
                self.select_previous_tab();
                Ok(vec![ComponentEvent::Activated {
                    component_id: self.id.clone(),
                }])
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.select_next_tab();
                Ok(vec![ComponentEvent::Activated {
                    component_id: self.id.clone(),
                }])
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Close current tab
                if self.state.selected_index < self.props.tabs.len() {
                    let tab = &self.props.tabs[self.state.selected_index];
                    if tab.closable {
                        let _removed_tab = self.remove_tab(self.state.selected_index);
                        return Ok(vec![ComponentEvent::Activated {
                            component_id: self.id.clone(),
                        }]);
                    }
                }
                Ok(vec![])
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // New tab
                if self.props.show_add_button {
                    Ok(vec![ComponentEvent::Activated {
                        component_id: self.id.clone(),
                    }])
                } else {
                    Ok(vec![])
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                // Switch to tab by number (1-9)
                let tab_num = c.to_digit(10).unwrap_or(0) as usize;
                if tab_num > 0 && tab_num <= self.props.tabs.len() {
                    self.select_tab(tab_num - 1);
                    Ok(vec![ComponentEvent::Activated {
                        component_id: self.id.clone(),
                    }])
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![])
        }
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::Activated { .. } => {
                // Handle tab activation
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = AppTheme::default();
        
        if self.props.tabs.is_empty() {
            let empty_block = Block::default()
                .borders(Borders::ALL)
                .title("No Tabs")
                .style(Style::default().fg(theme.colors.border));
            frame.render_widget(empty_block, area);
            return;
        }

        // Split area for tabs and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Render tabs
        let tab_titles = self.build_tab_titles();
        
        let tabs_widget = RatatuiTabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .style(Style::default().fg(theme.colors.foreground))
            .highlight_style(
                Style::default()
                    .fg(theme.colors.focus)
                    .add_modifier(Modifier::BOLD)
            )
            .select(self.state.selected_index);

        frame.render_widget(tabs_widget, chunks[0]);

        // Render content area
        let content_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.colors.border));
        frame.render_widget(content_block, chunks[1]);

        // Render tab content placeholder
        if let Some(selected_tab) = self.props.tabs.get(self.state.selected_index) {
            let content_area = Block::default()
                .borders(Borders::NONE)
                .title("")
                .inner(chunks[1]);

            let content_text = format!("Content for tab: {}", selected_tab.title);
            let content = Paragraph::new(Line::from(content_text))
                .style(Style::default().fg(theme.colors.foreground));
            frame.render_widget(content, content_area);
        }

        // Render keyboard shortcuts hint
        let hint_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };
        
        let hint_text = "← → or h/l: switch tabs, Ctrl+W: close, Ctrl+T: new, 1-9: select";
        let hint = Paragraph::new(Line::from(Span::styled(
            hint_text,
            Style::default().fg(theme.colors.info)
        )));
        frame.render_widget(hint, hint_area);
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        if self.state.selected_index >= self.props.tabs.len() && !self.props.tabs.is_empty() {
            return ValidationState::Invalid("Selected tab index out of bounds".to_string());
        }
        
        ValidationState::Valid
    }
} 