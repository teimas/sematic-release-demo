//! Sidebar Layout Component
//!
//! A collapsible navigation sidebar with support for hierarchical items,
//! icons, search functionality, and theme integration.

use crate::presentation::components::core::{
    Component, ComponentState, ComponentProps, ComponentResult, ComponentId,
    ValidationState, CommonComponentState, ComponentEvent,
};
use crate::presentation::theme::AppTheme;
use crate::state::StateEvent;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub children: Vec<SidebarItem>,
    pub expanded: bool,
    pub selectable: bool,
    pub metadata: HashMap<String, String>,
}

impl SidebarItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
            selectable: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_children(mut self, children: Vec<SidebarItem>) -> Self {
        self.children = children;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarProps {
    pub items: Vec<SidebarItem>,
    pub title: String,
    pub collapsible: bool,
    pub width: u16,
    pub min_width: u16,
    pub max_width: u16,
    pub show_search: bool,
    pub show_icons: bool,
}

impl Default for SidebarProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            title: "Navigation".to_string(),
            collapsible: true,
            width: 25,
            min_width: 15,
            max_width: 50,
            show_search: true,
            show_icons: true,
        }
    }
}

impl ComponentProps for SidebarProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarComponentState {
    pub common: CommonComponentState,
    pub collapsed: bool,
    pub selected_index: usize,
    pub search_query: String,
    pub search_mode: bool,
    pub scroll_offset: usize,
    pub expanded_items: std::collections::HashSet<String>,
}

impl ComponentState for SidebarComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for SidebarComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            collapsed: false,
            selected_index: 0,
            search_query: String::new(),
            search_mode: false,
            scroll_offset: 0,
            expanded_items: std::collections::HashSet::new(),
        }
    }
}

pub struct Sidebar {
    id: ComponentId,
    props: SidebarProps,
    state: SidebarComponentState,
    list_state: ListState,
}

impl Sidebar {
    pub fn new(id: ComponentId, props: SidebarProps) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            id,
            props,
            state: SidebarComponentState::default(),
            list_state,
        }
    }

    pub fn toggle_collapse(&mut self) {
        if self.props.collapsible {
            self.state.collapsed = !self.state.collapsed;
            self.state.common.mark_dirty();
        }
    }

    pub fn select_next(&mut self) {
        if !self.props.items.is_empty() {
            self.state.selected_index = (self.state.selected_index + 1) % self.props.items.len();
            self.list_state.select(Some(self.state.selected_index));
            self.state.common.mark_dirty();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.props.items.is_empty() {
            if self.state.selected_index == 0 {
                self.state.selected_index = self.props.items.len() - 1;
            } else {
                self.state.selected_index -= 1;
            }
            self.list_state.select(Some(self.state.selected_index));
            self.state.common.mark_dirty();
        }
    }

    fn build_list_items(&self) -> Vec<ListItem> {
        self.props.items.iter().map(|item| {
            let mut spans = Vec::new();
            
            // Add icon if present and enabled
            if self.props.show_icons {
                if let Some(icon) = &item.icon {
                    spans.push(Span::raw(format!("{} ", icon)));
                }
            }
            
            // Add label
            spans.push(Span::raw(&item.label));
            
            ListItem::new(Line::from(spans))
        }).collect()
    }
}

#[async_trait]
impl Component for Sidebar {
    type Props = SidebarProps;
    type State = SidebarComponentState;

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
        if self.state.search_mode {
            match key.code {
                KeyCode::Char(c) => {
                    self.state.search_query.push(c);
                    self.state.common.mark_dirty();
                    Ok(vec![])
                }
                KeyCode::Backspace => {
                    self.state.search_query.pop();
                    self.state.common.mark_dirty();
                    Ok(vec![])
                }
                KeyCode::Enter | KeyCode::Esc => {
                    self.state.search_mode = false;
                    self.state.search_query.clear();
                    self.state.common.mark_dirty();
                    Ok(vec![])
                }
                _ => Ok(vec![])
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_previous();
                    Ok(vec![])
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next();
                    Ok(vec![])
                }
                KeyCode::Enter => {
                    Ok(vec![ComponentEvent::Activated {
                        component_id: self.id.clone(),
                    }])
                }
                KeyCode::Char('/') => {
                    if self.props.show_search {
                        self.state.search_mode = true;
                        self.state.search_query.clear();
                        self.state.common.mark_dirty();
                    }
                    Ok(vec![])
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.toggle_collapse();
                    Ok(vec![ComponentEvent::Activated {
                        component_id: self.id.clone(),
                    }])
                }
                _ => Ok(vec![])
            }
        }
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::Activated { .. } => {
                // Handle item activation
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = AppTheme::default();
        
        if self.state.collapsed {
            // Render collapsed sidebar
            let collapsed_block = Block::default()
                .borders(Borders::ALL)
                .title("≡")
                .style(Style::default().fg(theme.colors.border));
            frame.render_widget(collapsed_block, area);
            return;
        }

        // Render sidebar content
        let list_items = self.build_list_items();
        
        let list_widget = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.props.title.as_str())
                    .style(Style::default().fg(theme.colors.border))
            )
            .style(Style::default().fg(theme.colors.foreground))
            .highlight_style(
                Style::default()
                    .fg(theme.colors.focus)
                    .add_modifier(Modifier::BOLD)
            );

        frame.render_stateful_widget(list_widget, area, &mut self.list_state.clone());

        // Render keyboard shortcuts hint
        let hint_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };
        
        let hint_text = if self.state.search_mode {
            "Enter/Esc: exit search, Type to search"
        } else {
            "j/k: navigate, Enter: select, /: search, Ctrl+C: collapse"
        };
        
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
        if self.state.selected_index >= self.props.items.len() && !self.props.items.is_empty() {
            return ValidationState::Invalid("Selected item index out of bounds".to_string());
        }
        
        ValidationState::Valid
    }
} 