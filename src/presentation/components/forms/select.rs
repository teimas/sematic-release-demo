use crate::presentation::components::core::{
    Component, ComponentEvent, ComponentId, ComponentProps, ComponentState, FocusState,
    ValidationState, VisibilityState, CommonComponentState, ComponentResult, NavigationDirection,
};
use crate::state::StateEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
    pub group: Option<String>,
}

impl SelectOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            disabled: false,
            group: None,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn with_group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectProps {
    pub id: ComponentId,
    pub title: String,
    pub placeholder: String,
    pub options: Vec<SelectOption>,
    pub multi_select: bool,
    pub searchable: bool,
    pub required: bool,
    pub readonly: bool,
    pub max_selections: Option<usize>,
    pub show_clear_button: bool,
    pub help_text: Option<String>,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            id: ComponentId::new("select"),
            title: String::new(),
            placeholder: "Select an option...".to_string(),
            options: Vec::new(),
            multi_select: false,
            searchable: false,
            required: false,
            readonly: false,
            max_selections: None,
            show_clear_button: true,
            help_text: None,
        }
    }
}

impl ComponentProps for SelectProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectState {
    pub common: CommonComponentState,
    pub selected_values: HashSet<String>,
    pub highlighted_index: usize,
    pub is_open: bool,
    pub search_query: String,
    pub filtered_options: Vec<usize>, // Indices into the original options
    pub last_validation_message: Option<String>,
}

impl Default for SelectState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            selected_values: HashSet::new(),
            highlighted_index: 0,
            is_open: false,
            search_query: String::new(),
            filtered_options: Vec::new(),
            last_validation_message: None,
        }
    }
}

impl ComponentState for SelectState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    id: ComponentId,
    props: SelectProps,
    state: SelectState,
    list_state: ListState,
}

impl Select {
    pub fn new(props: SelectProps) -> Self {
        let mut select = Self {
            id: props.id.clone(),
            props,
            state: SelectState::default(),
            list_state: ListState::default(),
        };
        select.update_filtered_options();
        select
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.props.title = title.to_string();
        self
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.props.placeholder = placeholder.to_string();
        self
    }

    pub fn with_options(mut self, options: Vec<SelectOption>) -> Self {
        self.props.options = options;
        self.update_filtered_options();
        self
    }

    pub fn multi_select(mut self, multi: bool) -> Self {
        self.props.multi_select = multi;
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.props.searchable = searchable;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.props.required = required;
        self
    }

    pub fn readonly(mut self, readonly: bool) -> Self {
        self.props.readonly = readonly;
        self
    }

    pub fn max_selections(mut self, max: usize) -> Self {
        self.props.max_selections = Some(max);
        self
    }

    pub fn with_help_text(mut self, help_text: &str) -> Self {
        self.props.help_text = Some(help_text.to_string());
        self
    }

    /// Get currently selected values
    pub fn selected_values(&self) -> Vec<String> {
        self.state.selected_values.iter().cloned().collect()
    }

    /// Get the first selected value (for single-select)
    pub fn selected_value(&self) -> Option<String> {
        self.state.selected_values.iter().next().cloned()
    }

    /// Set selected values
    pub fn set_selected_values(&mut self, values: Vec<String>) {
        self.state.selected_values.clear();
        for value in values {
            if self.props.options.iter().any(|opt| opt.value == value && !opt.disabled) {
                self.state.selected_values.insert(value);
            }
        }
        self.state.common.mark_dirty();
        self.validate();
    }

    /// Add a value to selection (for multi-select)
    pub fn add_selection(&mut self, value: String) -> bool {
        if !self.props.multi_select {
            self.state.selected_values.clear();
        }

        if let Some(max) = self.props.max_selections {
            if self.state.selected_values.len() >= max {
                return false;
            }
        }

        let inserted = self.state.selected_values.insert(value);
        if inserted {
            self.state.common.mark_dirty();
            self.validate();
        }
        inserted
    }

    /// Remove a value from selection
    pub fn remove_selection(&mut self, value: &str) -> bool {
        let removed = self.state.selected_values.remove(value);
        if removed {
            self.state.common.mark_dirty();
            self.validate();
        }
        removed
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        if !self.state.selected_values.is_empty() {
            self.state.selected_values.clear();
            self.state.common.mark_dirty();
            self.validate();
        }
    }

    /// Toggle selection of an option
    pub fn toggle_selection(&mut self, value: String) -> bool {
        if self.state.selected_values.contains(&value) {
            self.remove_selection(&value)
        } else {
            self.add_selection(value)
        }
    }

    /// Open the dropdown
    pub fn open(&mut self) {
        if !self.props.readonly {
            self.state.is_open = true;
            self.state.highlighted_index = 0;
            if !self.state.filtered_options.is_empty() {
                self.list_state.select(Some(0));
            }
        }
    }

    /// Close the dropdown
    pub fn close(&mut self) {
        self.state.is_open = false;
        self.state.search_query.clear();
        self.update_filtered_options();
    }

    /// Move highlight up
    pub fn move_up(&mut self) {
        if self.state.highlighted_index > 0 {
            self.state.highlighted_index -= 1;
            self.list_state.select(Some(self.state.highlighted_index));
        }
    }

    /// Move highlight down
    pub fn move_down(&mut self) {
        if self.state.highlighted_index < self.state.filtered_options.len().saturating_sub(1) {
            self.state.highlighted_index += 1;
            self.list_state.select(Some(self.state.highlighted_index));
        }
    }

    /// Select the currently highlighted option
    pub fn select_highlighted(&mut self) -> bool {
        if let Some(&option_index) = self.state.filtered_options.get(self.state.highlighted_index) {
            if let Some(option) = self.props.options.get(option_index) {
                if !option.disabled {
                    return self.toggle_selection(option.value.clone());
                }
            }
        }
        false
    }

    /// Update search query and filter options
    pub fn set_search_query(&mut self, query: String) {
        self.state.search_query = query;
        self.update_filtered_options();
        self.state.highlighted_index = 0;
        if !self.state.filtered_options.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Update the filtered options based on search query
    fn update_filtered_options(&mut self) {
        self.state.filtered_options.clear();
        
        if self.state.search_query.is_empty() {
            self.state.filtered_options = (0..self.props.options.len()).collect();
        } else {
            let query = self.state.search_query.to_lowercase();
            for (index, option) in self.props.options.iter().enumerate() {
                if option.label.to_lowercase().contains(&query) ||
                   option.value.to_lowercase().contains(&query) {
                    self.state.filtered_options.push(index);
                }
            }
        }
    }

    /// Validate current selection
    fn validate(&mut self) {
        // Check required field
        if self.props.required && self.state.selected_values.is_empty() {
            self.state.common.validation_state = ValidationState::Invalid("This field is required".to_string());
            self.state.last_validation_message = Some("This field is required".to_string());
            return;
        }

        // Check max selections
        if let Some(max) = self.props.max_selections {
            if self.state.selected_values.len() > max {
                let msg = format!("Maximum {} selections allowed", max);
                self.state.common.validation_state = ValidationState::Invalid(msg.clone());
                self.state.last_validation_message = Some(msg);
                return;
            }
        }

        // All validations passed
        self.state.common.validation_state = ValidationState::Valid;
        self.state.last_validation_message = None;
    }

    /// Get display text for current selection
    pub fn display_text(&self) -> String {
        if self.state.selected_values.is_empty() {
            return self.props.placeholder.clone();
        }

        let selected_labels: Vec<String> = self.state.selected_values
            .iter()
            .filter_map(|value| {
                self.props.options
                    .iter()
                    .find(|opt| &opt.value == value)
                    .map(|opt| opt.label.clone())
            })
            .collect();

        if selected_labels.is_empty() {
            return self.props.placeholder.clone();
        }

        if self.props.multi_select {
            if selected_labels.len() == 1 {
                selected_labels[0].clone()
            } else {
                format!("{} items selected", selected_labels.len())
            }
        } else {
            selected_labels[0].clone()
        }
    }

    /// Get border style based on component state
    fn get_border_style(&self) -> Style {
        match (&self.state.common.focus_state, &self.state.common.validation_state) {
            (FocusState::Focused, ValidationState::Valid) => Style::default().fg(Color::Green),
            (FocusState::Focused, ValidationState::Invalid(_)) => Style::default().fg(Color::Red),
            (FocusState::Focused, ValidationState::Pending) => Style::default().fg(Color::Yellow),
            (FocusState::Unfocused, ValidationState::Invalid(_)) => Style::default().fg(Color::LightRed),
            (FocusState::Unfocused, ValidationState::Pending) => Style::default().fg(Color::LightYellow),
            (FocusState::Unfocused, ValidationState::Valid) => Style::default().fg(Color::Gray),
            (FocusState::Disabled, _) => Style::default().fg(Color::DarkGray),
        }
    }

    /// Get title with indicators
    fn get_title_with_indicators(&self) -> String {
        let mut title = self.props.title.clone();
        
        if self.props.required {
            title.push_str(" *");
        }
        
        if self.state.common.dirty {
            title.push_str(" (modified)");
        }

        match &self.state.common.validation_state {
            ValidationState::Invalid(_) => title.push_str(" ❌"),
            ValidationState::Pending => title.push_str(" ⚠️"),
            ValidationState::Valid => {
                if self.state.common.dirty {
                    title.push_str(" ✅");
                }
            }
        }

        if self.props.searchable {
            title.push_str(" 🔍");
        }

        if self.props.multi_select {
            title.push_str(" ☰");
        }

        title
    }

    /// Check if component is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.state.common.validation_state, ValidationState::Valid)
    }

    /// Get validation message
    pub fn validation_message(&self) -> Option<&str> {
        self.state.last_validation_message.as_deref()
    }
}

#[async_trait]
impl Component for Select {
    type Props = SelectProps;
    type State = SelectState;

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

    async fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> ComponentResult<Vec<ComponentEvent>> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        let mut events = Vec::new();
        
        if self.focus_state() != FocusState::Focused || self.props.readonly {
            return Ok(events);
        }

        let old_selected = self.selected_values();

        match (key.code, key.modifiers) {
            // Open/close dropdown
            (KeyCode::Enter, _) | (KeyCode::Char(' '), _) if !self.state.is_open => {
                self.open();
            }
            (KeyCode::Esc, _) if self.state.is_open => {
                self.close();
            }
            
            // Navigation when dropdown is open
            (KeyCode::Up, _) if self.state.is_open => {
                self.move_up();
            }
            (KeyCode::Down, _) if self.state.is_open => {
                self.move_down();
            }
            
            // Selection when dropdown is open
            (KeyCode::Enter, _) | (KeyCode::Char(' '), _) if self.state.is_open => {
                if self.select_highlighted() {
                    if !self.props.multi_select {
                        self.close();
                    }
                }
            }
            
            // Search when dropdown is open and searchable
            (KeyCode::Char(c), _) if self.state.is_open && self.props.searchable => {
                let mut query = self.state.search_query.clone();
                query.push(c);
                self.set_search_query(query);
            }
            (KeyCode::Backspace, _) if self.state.is_open && self.props.searchable => {
                let mut query = self.state.search_query.clone();
                query.pop();
                self.set_search_query(query);
            }
            
            // Clear selection
            (KeyCode::Delete, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                if self.props.show_clear_button {
                    self.clear_selections();
                }
            }
            
            _ => {}
        }

        // Check if selection changed
        let new_selected = self.selected_values();
        if old_selected != new_selected {
            events.push(ComponentEvent::ValueChanged {
                component_id: self.id.clone(),
                old_value: old_selected.join(","),
                new_value: new_selected.join(","),
            });
        }

        Ok(events)
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::FocusGained { component_id } if component_id == self.id => {
                self.set_focus_state(FocusState::Focused);
                Ok(vec![])
            }
            ComponentEvent::FocusLost { component_id } if component_id == self.id => {
                self.set_focus_state(FocusState::Unfocused);
                self.close();
                self.validate();
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.visibility_state() != VisibilityState::Visible {
            return;
        }

        let title = self.get_title_with_indicators();
        let border_style = self.get_border_style();
        
        // Main select area
        let display_text = self.display_text();
        let dropdown_indicator = if self.state.is_open { "▲" } else { "▼" };
        let content = format!("{} {}", display_text, dropdown_indicator);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Render content
        let content_para = Paragraph::new(content)
            .style(if self.state.selected_values.is_empty() { 
                Style::default().fg(Color::DarkGray) 
            } else { 
                Style::default() 
            });
        frame.render_widget(content_para, inner);

        // Render dropdown if open
        if self.state.is_open && !self.state.filtered_options.is_empty() {
            self.render_dropdown(frame, area);
        }
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        // Could react to theme changes, option updates, etc.
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        self.state.common.validation_state.clone()
    }
}

impl Select {
    /// Render the dropdown list
    fn render_dropdown(&self, frame: &mut Frame, area: Rect) {
        // Calculate dropdown position (below the select box)
        let dropdown_height = (self.state.filtered_options.len() + 2).min(10) as u16; // Max 10 items
        let dropdown_area = Rect {
            x: area.x,
            y: area.y + area.height,
            width: area.width,
            height: dropdown_height,
        };

        // Don't render if dropdown would go off screen
        if dropdown_area.y + dropdown_area.height > frame.area().height {
            return;
        }

        // Create list items
        let items: Vec<ListItem> = self.state.filtered_options
            .iter()
            .filter_map(|&index| self.props.options.get(index))
            .map(|option| {
                let mut style = Style::default();
                
                if option.disabled {
                    style = style.fg(Color::DarkGray);
                }
                
                if self.state.selected_values.contains(&option.value) {
                    style = style.fg(Color::Green);
                }

                let prefix = if self.state.selected_values.contains(&option.value) {
                    if self.props.multi_select { "☑ " } else { "● " }
                } else {
                    if self.props.multi_select { "☐ " } else { "  " }
                };

                ListItem::new(format!("{}{}", prefix, option.label)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        let mut list_state = self.list_state.clone();
        frame.render_stateful_widget(list, dropdown_area, &mut list_state);

        // Show search query if applicable
        if self.props.searchable && !self.state.search_query.is_empty() {
            let search_area = Rect {
                x: dropdown_area.x + 1,
                y: dropdown_area.y + dropdown_area.height,
                width: dropdown_area.width.saturating_sub(2),
                height: 1,
            };
            
            let search_text = format!("Search: {}", self.state.search_query);
            let search_para = Paragraph::new(search_text)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(search_para, search_area);
        }
    }
} 