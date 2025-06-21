use crate::presentation::components::core::base::{
    Component, ComponentState, ComponentProps, CommonComponentState,
};
use crate::presentation::components::core::{
    ComponentResult, ComponentId, ValidationState,
};
use crate::presentation::theme::AppTheme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tree node structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub expanded: bool,
    pub selectable: bool,
    pub children: Vec<TreeNode>,
    pub metadata: HashMap<String, String>,
}

impl TreeNode {
    pub fn new(id: String, label: String) -> Self {
        Self {
            id,
            label,
            icon: None,
            expanded: false,
            selectable: true,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }
}

/// Tree symbols for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeSymbols {
    pub branch: String,
    pub leaf: String,
    pub expanded: String,
    pub collapsed: String,
    pub vertical: String,
    pub horizontal: String,
    pub corner: String,
    pub tee: String,
}

impl Default for TreeSymbols {
    fn default() -> Self {
        Self {
            branch: "├─".to_string(),
            leaf: "└─".to_string(),
            expanded: "▼".to_string(),
            collapsed: "▶".to_string(),
            vertical: "│".to_string(),
            horizontal: "─".to_string(),
            corner: "└".to_string(),
            tee: "├".to_string(),
        }
    }
}

/// Tree component properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeProps {
    pub root: TreeNode,
    pub title: Option<String>,
    pub show_borders: bool,
    pub show_icons: bool,
    pub multi_select: bool,
    pub search_enabled: bool,
    pub tree_symbols: TreeSymbols,
    pub empty_message: String,
}

impl Default for TreeProps {
    fn default() -> Self {
        Self {
            root: TreeNode::new("root".to_string(), "Root".to_string()),
            title: None,
            show_borders: true,
            show_icons: true,
            multi_select: false,
            search_enabled: true,
            tree_symbols: TreeSymbols::default(),
            empty_message: "No items".to_string(),
        }
    }
}

impl ComponentProps for TreeProps {
    fn default_props() -> Self {
        Self::default()
    }
}

/// Tree component state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeComponentState {
    pub common: CommonComponentState,
    pub selected_nodes: Vec<String>,
    pub current_item: usize,
    pub scroll_offset: usize,
    pub search_mode: bool,
    pub search_query: String,
    pub flat_items: Vec<String>,
}

impl ComponentState for TreeComponentState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

impl Default for TreeComponentState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            selected_nodes: Vec::new(),
            current_item: 0,
            scroll_offset: 0,
            search_mode: false,
            search_query: String::new(),
            flat_items: Vec::new(),
        }
    }
}

/// Tree component
pub struct Tree {
    id: ComponentId,
    props: TreeProps,
    state: TreeComponentState,
    list_state: ListState,
}

impl Tree {
    pub fn new(id: ComponentId, props: TreeProps) -> Self {
        Self {
            id,
            props,
            state: TreeComponentState::default(),
            list_state: ListState::default(),
        }
    }

    pub fn selected_nodes(&self) -> &[String] {
        &self.state.selected_nodes
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ComponentResult<bool> {
        match key.code {
            KeyCode::Up => {
                if self.state.current_item > 0 {
                    self.state.current_item -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                self.state.current_item += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn render_tree(&mut self, frame: &mut Frame, area: Rect, theme: &AppTheme) {
        let items = vec![ListItem::new(Line::from(Span::raw("Tree Item")))];

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.props.title.as_deref().unwrap_or("Tree"))
                    .border_style(Style::default().fg(theme.colors.border))
            )
            .style(Style::default().fg(theme.colors.primary));

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

#[async_trait::async_trait]
impl Component for Tree {
    type Props = TreeProps;
    type State = TreeComponentState;

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    fn props(&self) -> &Self::Props {
        &self.props
    }

    async fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> ComponentResult<Vec<crate::presentation::components::core::ComponentEvent>> {
        let handled = self.handle_key_event(key)?;
        if handled {
            self.state.common.mark_dirty();
        }
        Ok(vec![])
    }

    async fn handle_event(&mut self, _event: crate::presentation::components::core::ComponentEvent) -> ComponentResult<Vec<crate::presentation::components::core::ComponentEvent>> {
        Ok(vec![])
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = AppTheme::default();
        let mut tree_clone = self.clone();
        tree_clone.render_tree(frame, area, &theme);
    }

    async fn update_from_state(&mut self, _state_event: &crate::state::StateEvent) -> ComponentResult<bool> {
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        ValidationState::Valid
    }
}

impl Clone for Tree {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            props: self.props.clone(),
            state: self.state.clone(),
            list_state: ListState::default(),
        }
    }
} 