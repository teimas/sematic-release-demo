use crate::presentation::components::core::{
    Component, ComponentEvent, ComponentId, ComponentProps, ComponentState, FocusState,
    ValidationState, VisibilityState, CommonComponentState, ComponentResult,
};
use crate::state::StateEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
    pub help_text: Option<String>,
}

impl RadioOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            disabled: false,
            help_text: None,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn with_help_text(mut self, help_text: &str) -> Self {
        self.help_text = Some(help_text.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioProps {
    pub id: ComponentId,
    pub group_name: String,
    pub options: Vec<RadioOption>,
    pub title: String,
    pub required: bool,
    pub readonly: bool,
    pub layout: RadioLayout,
    pub help_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RadioLayout {
    Vertical,
    Horizontal,
    Grid(u16), // Grid with specified number of columns
}

impl Default for RadioProps {
    fn default() -> Self {
        Self {
            id: ComponentId::new("radio"),
            group_name: String::new(),
            options: Vec::new(),
            title: String::new(),
            required: false,
            readonly: false,
            layout: RadioLayout::Vertical,
            help_text: None,
        }
    }
}

impl ComponentProps for RadioProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioState {
    pub common: CommonComponentState,
    pub selected_value: Option<String>,
    pub highlighted_index: usize,
    pub last_validation_message: Option<String>,
}

impl Default for RadioState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            selected_value: None,
            highlighted_index: 0,
            last_validation_message: None,
        }
    }
}

impl ComponentState for RadioState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Radio {
    id: ComponentId,
    props: RadioProps,
    state: RadioState,
}

impl Radio {
    pub fn new(props: RadioProps) -> Self {
        Self {
            id: props.id.clone(),
            props,
            state: RadioState::default(),
        }
    }

    pub fn with_group_name(mut self, group_name: &str) -> Self {
        self.props.group_name = group_name.to_string();
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.props.title = title.to_string();
        self
    }

    pub fn with_options(mut self, options: Vec<RadioOption>) -> Self {
        self.props.options = options;
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

    pub fn with_layout(mut self, layout: RadioLayout) -> Self {
        self.props.layout = layout;
        self
    }

    pub fn with_help_text(mut self, help_text: &str) -> Self {
        self.props.help_text = Some(help_text.to_string());
        self
    }

    pub fn vertical(mut self) -> Self {
        self.props.layout = RadioLayout::Vertical;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.props.layout = RadioLayout::Horizontal;
        self
    }

    pub fn grid(mut self, columns: u16) -> Self {
        self.props.layout = RadioLayout::Grid(columns);
        self
    }

    /// Get currently selected value
    pub fn selected_value(&self) -> Option<&str> {
        self.state.selected_value.as_deref()
    }

    /// Set selected value
    pub fn set_selected_value(&mut self, value: Option<String>) {
        if self.state.selected_value != value {
            self.state.selected_value = value;
            self.state.common.mark_dirty();
            self.validate();
        }
    }

    /// Select a value
    pub fn select_value(&mut self, value: &str) -> bool {
        if self.props.readonly {
            return false;
        }

        // Check if value exists and is not disabled
        if let Some(option) = self.props.options.iter().find(|opt| opt.value == value) {
            if !option.disabled {
                self.set_selected_value(Some(value.to_string()));
                return true;
            }
        }
        false
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.set_selected_value(None);
    }

    /// Move highlight up/previous
    pub fn move_up(&mut self) {
        if self.state.highlighted_index > 0 {
            self.state.highlighted_index -= 1;
        } else {
            self.state.highlighted_index = self.props.options.len().saturating_sub(1);
        }
    }

    /// Move highlight down/next
    pub fn move_down(&mut self) {
        self.state.highlighted_index = (self.state.highlighted_index + 1) % self.props.options.len();
    }

    /// Move highlight left (for horizontal/grid layouts)
    pub fn move_left(&mut self) {
        match self.props.layout {
            RadioLayout::Horizontal => self.move_up(),
            RadioLayout::Grid(columns) => {
                if self.state.highlighted_index % columns as usize > 0 {
                    self.state.highlighted_index -= 1;
                }
            }
            RadioLayout::Vertical => {} // No horizontal movement in vertical layout
        }
    }

    /// Move highlight right (for horizontal/grid layouts)
    pub fn move_right(&mut self) {
        match self.props.layout {
            RadioLayout::Horizontal => self.move_down(),
            RadioLayout::Grid(columns) => {
                if self.state.highlighted_index % (columns as usize) < (columns as usize - 1) &&
                   self.state.highlighted_index + 1 < self.props.options.len() {
                    self.state.highlighted_index += 1;
                }
            }
            RadioLayout::Vertical => {} // No horizontal movement in vertical layout
        }
    }

    /// Select the currently highlighted option
    pub fn select_highlighted(&mut self) -> bool {
        if let Some(option) = self.props.options.get(self.state.highlighted_index) {
            let value = option.value.clone();
            self.select_value(&value)
        } else {
            false
        }
    }

    /// Check if a value is selected
    pub fn is_selected(&self, value: &str) -> bool {
        self.state.selected_value.as_deref() == Some(value)
    }

    /// Get the selected option
    pub fn selected_option(&self) -> Option<&RadioOption> {
        self.state.selected_value.as_ref().and_then(|value| {
            self.props.options.iter().find(|opt| &opt.value == value)
        })
    }

    /// Validate current selection
    fn validate(&mut self) {
        // Check required field
        if self.props.required && self.state.selected_value.is_none() {
            self.state.common.validation_state = ValidationState::Invalid("This field is required".to_string());
            self.state.last_validation_message = Some("This field is required".to_string());
            return;
        }

        // All validations passed
        self.state.common.validation_state = ValidationState::Valid;
        self.state.last_validation_message = None;
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

        if !self.props.group_name.is_empty() {
            title.push_str(&format!(" [{}]", self.props.group_name));
        }

        title
    }

    /// Get radio symbol for an option
    fn get_radio_symbol(&self, option: &RadioOption, is_highlighted: bool) -> (String, Style) {
        let is_selected = self.is_selected(&option.value);
        let symbol = if is_selected { "●" } else { "○" };

        let style = if option.disabled {
            Style::default().fg(Color::DarkGray)
        } else if is_selected {
            Style::default().fg(Color::Green)
        } else if is_highlighted && self.state.common.focus_state == FocusState::Focused {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::Gray)
        };

        (format!("{} {}", symbol, option.label), style)
    }

    /// Check if component is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.state.common.validation_state, ValidationState::Valid)
    }

    /// Get validation message
    pub fn validation_message(&self) -> Option<&str> {
        self.state.last_validation_message.as_deref()
    }

    /// Get group name
    pub fn group_name(&self) -> &str {
        &self.props.group_name
    }
}

#[async_trait]
impl Component for Radio {
    type Props = RadioProps;
    type State = RadioState;

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

        let old_value = self.state.selected_value.clone();

        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Up, _) => {
                self.move_up();
            }
            (KeyCode::Down, _) => {
                self.move_down();
            }
            (KeyCode::Left, _) => {
                self.move_left();
            }
            (KeyCode::Right, _) => {
                self.move_right();
            }
            
            // Selection
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                self.select_highlighted();
            }
            
            // Direct selection by number keys
            (KeyCode::Char(c), _) if c.is_ascii_digit() => {
                let index = c.to_digit(10).unwrap() as usize;
                if index > 0 && index <= self.props.options.len() {
                    if let Some(option) = self.props.options.get(index - 1) {
                        let value = option.value.clone();
                        self.select_value(&value);
                        self.state.highlighted_index = index - 1;
                    }
                }
            }
            
            // Clear selection
            (KeyCode::Delete, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                if !self.props.required {
                    self.clear_selection();
                }
            }            
            _ => {}
        }

        // Check if selection changed
        if old_value != self.state.selected_value {
            events.push(ComponentEvent::ValueChanged {
                component_id: self.id.clone(),
                old_value: old_value.unwrap_or_default(),
                new_value: self.state.selected_value.clone().unwrap_or_default(),
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

        // Create block with title if provided
        let block = if title.is_empty() {
            Block::default().borders(Borders::ALL).border_style(border_style)
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render options based on layout
        match self.props.layout {
            RadioLayout::Vertical => self.render_vertical(frame, inner),
            RadioLayout::Horizontal => self.render_horizontal(frame, inner),
            RadioLayout::Grid(columns) => self.render_grid(frame, inner, columns),
        }

        // Render help text if available and component has focus
        if let Some(help_text) = &self.props.help_text {
            if self.state.common.focus_state == FocusState::Focused {
                self.render_help_text(frame, area, help_text);
            }
        }

        // Render validation message if there's an error
        if let Some(validation_msg) = &self.state.last_validation_message {
            self.render_validation_message(frame, area, validation_msg);
        }
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        // Could react to theme changes, group state changes, etc.
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        self.state.common.validation_state.clone()
    }
}

impl Radio {
    /// Render options vertically
    fn render_vertical(&self, frame: &mut Frame, area: Rect) {
        for (index, option) in self.props.options.iter().enumerate() {
            if index >= area.height as usize {
                break;
            }

            let option_area = Rect {
                x: area.x,
                y: area.y + index as u16,
                width: area.width,
                height: 1,
            };

            let is_highlighted = index == self.state.highlighted_index;
            let (text, style) = self.get_radio_symbol(option, is_highlighted);

            let paragraph = Paragraph::new(text).style(style);
            frame.render_widget(paragraph, option_area);
        }
    }

    /// Render options horizontally
    fn render_horizontal(&self, frame: &mut Frame, area: Rect) {
        let option_width = area.width / self.props.options.len() as u16;
        
        for (index, option) in self.props.options.iter().enumerate() {
            let option_area = Rect {
                x: area.x + (index as u16 * option_width),
                y: area.y,
                width: option_width,
                height: 1,
            };

            let is_highlighted = index == self.state.highlighted_index;
            let (text, style) = self.get_radio_symbol(option, is_highlighted);

            let paragraph = Paragraph::new(text).style(style);
            frame.render_widget(paragraph, option_area);
        }
    }

    /// Render options in a grid
    fn render_grid(&self, frame: &mut Frame, area: Rect, columns: u16) {
        let option_width = area.width / columns;
        let rows = (self.props.options.len() as u16 + columns - 1) / columns;

        for (index, option) in self.props.options.iter().enumerate() {
            let row = index as u16 / columns;
            let col = index as u16 % columns;

            if row >= area.height {
                break;
            }

            let option_area = Rect {
                x: area.x + (col * option_width),
                y: area.y + row,
                width: option_width,
                height: 1,
            };

            let is_highlighted = index == self.state.highlighted_index;
            let (text, style) = self.get_radio_symbol(option, is_highlighted);

            let paragraph = Paragraph::new(text).style(style);
            frame.render_widget(paragraph, option_area);
        }
    }

    /// Render help text below the radio group
    fn render_help_text(&self, frame: &mut Frame, area: Rect, help_text: &str) {
        let help_area = Rect {
            x: area.x,
            y: area.y + area.height,
            width: area.width,
            height: 1,
        };

        if help_area.y >= frame.area().height {
            return;
        }

        let help_paragraph = Paragraph::new(format!("💡 {}", help_text))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(help_paragraph, help_area);
    }

    /// Render validation message below the radio group
    fn render_validation_message(&self, frame: &mut Frame, area: Rect, message: &str) {
        let msg_area = Rect {
            x: area.x,
            y: area.y + area.height + if self.props.help_text.is_some() { 1 } else { 0 },
            width: area.width,
            height: 1,
        };

        if msg_area.y >= frame.area().height {
            return;
        }

        let msg_paragraph = Paragraph::new(format!("❌ {}", message))
            .style(Style::default().fg(Color::Red));
        frame.render_widget(msg_paragraph, msg_area);
    }
}

// Utility for managing multiple radio groups
pub struct RadioGroupManager {
    groups: HashMap<String, String>, // group_name -> selected_value
}

impl RadioGroupManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn set_selection(&mut self, group_name: &str, value: &str) {
        self.groups.insert(group_name.to_string(), value.to_string());
    }

    pub fn get_selection(&self, group_name: &str) -> Option<&str> {
        self.groups.get(group_name).map(|s| s.as_str())
    }

    pub fn clear_selection(&mut self, group_name: &str) {
        self.groups.remove(group_name);
    }

    pub fn get_all_selections(&self) -> &HashMap<String, String> {
        &self.groups
    }

    pub fn clear_all(&mut self) {
        self.groups.clear();
    }
}

impl Default for RadioGroupManager {
    fn default() -> Self {
        Self::new()
    }
}
