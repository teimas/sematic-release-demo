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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckboxValue {
    Unchecked,
    Checked,
    Indeterminate, // For tristate support
}

impl Default for CheckboxValue {
    fn default() -> Self {
        Self::Unchecked
    }
}

impl CheckboxValue {
    pub fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    pub fn is_unchecked(self) -> bool {
        matches!(self, Self::Unchecked)
    }

    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Unchecked => Self::Checked,
            Self::Checked => Self::Unchecked,
            Self::Indeterminate => Self::Checked,
        }
    }

    pub fn to_symbol(self) -> &'static str {
        match self {
            Self::Unchecked => "☐",
            Self::Checked => "☑",
            Self::Indeterminate => "☒",
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Self::Unchecked => '☐',
            Self::Checked => '☑',
            Self::Indeterminate => '☒',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxProps {
    pub id: ComponentId,
    pub label: String,
    pub help_text: Option<String>,
    pub required: bool,
    pub readonly: bool,
    pub tristate: bool, // Allow indeterminate state
    pub group: Option<String>, // For checkbox groups
}

impl Default for CheckboxProps {
    fn default() -> Self {
        Self {
            id: ComponentId::new("checkbox"),
            label: String::new(),
            help_text: None,
            required: false,
            readonly: false,
            tristate: false,
            group: None,
        }
    }
}

impl ComponentProps for CheckboxProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxState {
    pub common: CommonComponentState,
    pub value: CheckboxValue,
    pub last_validation_message: Option<String>,
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            value: CheckboxValue::default(),
            last_validation_message: None,
        }
    }
}

impl ComponentState for CheckboxState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Checkbox {
    id: ComponentId,
    props: CheckboxProps,
    state: CheckboxState,
}

impl Checkbox {
    pub fn new(props: CheckboxProps) -> Self {
        Self {
            id: props.id.clone(),
            props,
            state: CheckboxState::default(),
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.props.label = label.to_string();
        self
    }

    pub fn with_help_text(mut self, help_text: &str) -> Self {
        self.props.help_text = Some(help_text.to_string());
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

    pub fn tristate(mut self, tristate: bool) -> Self {
        self.props.tristate = tristate;
        self
    }

    pub fn with_group(mut self, group: &str) -> Self {
        self.props.group = Some(group.to_string());
        self
    }

    pub fn with_value(mut self, value: CheckboxValue) -> Self {
        self.state.value = value;
        self.validate();
        self
    }

    pub fn checked(mut self) -> Self {
        self.state.value = CheckboxValue::Checked;
        self.validate();
        self
    }

    pub fn unchecked(mut self) -> Self {
        self.state.value = CheckboxValue::Unchecked;
        self.validate();
        self
    }

    /// Get current value
    pub fn value(&self) -> CheckboxValue {
        self.state.value
    }

    /// Set value
    pub fn set_value(&mut self, value: CheckboxValue) {
        if self.state.value != value {
            self.state.value = value;
            self.state.common.mark_dirty();
            self.validate();
        }
    }

    /// Check if checkbox is checked
    pub fn is_checked(&self) -> bool {
        self.state.value.is_checked()
    }

    /// Check if checkbox is unchecked
    pub fn is_unchecked(&self) -> bool {
        self.state.value.is_unchecked()
    }

    /// Check if checkbox is indeterminate
    pub fn is_indeterminate(&self) -> bool {
        self.state.value.is_indeterminate()
    }

    /// Toggle the checkbox value
    pub fn toggle(&mut self) {
        if self.props.readonly {
            return;
        }

        let old_value = self.state.value;
        
        if self.props.tristate {
            self.state.value = match old_value {
                CheckboxValue::Unchecked => CheckboxValue::Checked,
                CheckboxValue::Checked => CheckboxValue::Indeterminate,
                CheckboxValue::Indeterminate => CheckboxValue::Unchecked,
            };
        } else {
            self.state.value = old_value.toggle();
        }

        self.state.common.mark_dirty();
        self.validate();
    }

    /// Check the checkbox
    pub fn check(&mut self) {
        self.set_value(CheckboxValue::Checked);
    }

    /// Uncheck the checkbox
    pub fn uncheck(&mut self) {
        self.set_value(CheckboxValue::Unchecked);
    }

    /// Set to indeterminate (if tristate is enabled)
    pub fn set_indeterminate(&mut self) {
        if self.props.tristate {
            self.set_value(CheckboxValue::Indeterminate);
        }
    }

    /// Validate current value
    fn validate(&mut self) {
        // Check required field
        if self.props.required && !self.state.value.is_checked() {
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

    /// Get checkbox symbol with style
    fn get_checkbox_symbol_with_style(&self) -> (String, Style) {
        let symbol = self.state.value.to_symbol();
        let style = match self.state.value {
            CheckboxValue::Checked => Style::default().fg(Color::Green),
            CheckboxValue::Indeterminate => Style::default().fg(Color::Yellow),
            CheckboxValue::Unchecked => {
                if self.state.common.focus_state == FocusState::Focused {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default().fg(Color::Gray)
                }
            }
        };

        (symbol.to_string(), style)
    }

    /// Get label with indicators
    fn get_label_with_indicators(&self) -> String {
        let mut label = self.props.label.clone();
        
        if self.props.required {
            label.push_str(" *");
        }
        
        if self.state.common.dirty {
            label.push_str(" (modified)");
        }

        match &self.state.common.validation_state {
            ValidationState::Invalid(_) => label.push_str(" ❌"),
            ValidationState::Pending => label.push_str(" ⚠️"),
            ValidationState::Valid => {
                if self.state.common.dirty {
                    label.push_str(" ✅");
                }
            }
        }

        if self.props.tristate {
            label.push_str(" (tristate)");
        }

        if let Some(group) = &self.props.group {
            label.push_str(&format!(" [{}]", group));
        }

        label
    }

    /// Check if component is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.state.common.validation_state, ValidationState::Valid)
    }

    /// Get validation message
    pub fn validation_message(&self) -> Option<&str> {
        self.state.last_validation_message.as_deref()
    }

    /// Convert to boolean (checked = true, others = false)
    pub fn to_bool(&self) -> bool {
        self.is_checked()
    }

    /// Get group name
    pub fn group(&self) -> Option<&str> {
        self.props.group.as_deref()
    }
}

#[async_trait]
impl Component for Checkbox {
    type Props = CheckboxProps;
    type State = CheckboxState;

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

        let old_value = self.state.value;

        match (key.code, key.modifiers) {
            // Toggle checkbox
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                self.toggle();
            }
            
            // Direct value setting
            (KeyCode::Char('y'), _) | (KeyCode::Char('1'), _) => {
                self.check();
            }
            (KeyCode::Char('n'), _) | (KeyCode::Char('0'), _) => {
                self.uncheck();
            }
            (KeyCode::Char('i'), _) if self.props.tristate => {
                self.set_indeterminate();
            }
            
            // Quick toggle with keyboard shortcuts
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                self.toggle();
            }
            
            _ => {}
        }

        // Check if value changed
        if old_value != self.state.value {
            events.push(ComponentEvent::ValueChanged {
                component_id: self.id.clone(),
                old_value: format!("{:?}", old_value),
                new_value: format!("{:?}", self.state.value),
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

        let border_style = self.get_border_style();
        let (checkbox_symbol, checkbox_style) = self.get_checkbox_symbol_with_style();
        let label = self.get_label_with_indicators();

        // Create the content with checkbox and label
        let content = if self.props.label.is_empty() {
            checkbox_symbol
        } else {
            format!("{} {}", checkbox_symbol, label)
        };

        // Determine if we need a border (for focus indication when part of forms)
        let needs_border = self.state.common.focus_state == FocusState::Focused ||
                          !matches!(self.state.common.validation_state, ValidationState::Valid);

        if needs_border {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let paragraph = Paragraph::new(content)
                .style(checkbox_style);
            frame.render_widget(paragraph, inner);
        } else {
            // Render without border for cleaner look
            let paragraph = Paragraph::new(content)
                .style(checkbox_style);
            frame.render_widget(paragraph, area);
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

impl Checkbox {
    /// Render help text below the checkbox
    fn render_help_text(&self, frame: &mut Frame, area: Rect, help_text: &str) {
        let help_area = Rect {
            x: area.x,
            y: area.y + area.height,
            width: area.width,
            height: 1,
        };

        // Don't render if it would go off screen
        if help_area.y >= frame.area().height {
            return;
        }

        let help_paragraph = Paragraph::new(format!("💡 {}", help_text))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(help_paragraph, help_area);
    }

    /// Render validation message below the checkbox
    fn render_validation_message(&self, frame: &mut Frame, area: Rect, message: &str) {
        let msg_area = Rect {
            x: area.x,
            y: area.y + area.height + if self.props.help_text.is_some() { 1 } else { 0 },
            width: area.width,
            height: 1,
        };

        // Don't render if it would go off screen
        if msg_area.y >= frame.area().height {
            return;
        }

        let msg_paragraph = Paragraph::new(format!("❌ {}", message))
            .style(Style::default().fg(Color::Red));
        frame.render_widget(msg_paragraph, msg_area);
    }
}


// Utility functions for working with checkbox groups
pub struct CheckboxGroup {
    checkboxes: Vec<Checkbox>,
    allow_multiple: bool,
}

impl CheckboxGroup {
    pub fn new(allow_multiple: bool) -> Self {
        Self {
            checkboxes: Vec::new(),
            allow_multiple,
        }
    }

    pub fn add_checkbox(&mut self, checkbox: Checkbox) {
        self.checkboxes.push(checkbox);
    }

    pub fn toggle_checkbox(&mut self, checkbox_id: &ComponentId) -> bool {
        let target_index = self.checkboxes.iter().position(|cb| cb.id() == checkbox_id);
        
        if let Some(index) = target_index {
            let was_checked = self.checkboxes[index].is_checked();
            
            if !self.allow_multiple && !was_checked {
                for (i, checkbox) in self.checkboxes.iter_mut().enumerate() {
                    if i != index {
                        checkbox.uncheck();
                    }
                }
            }
            
            self.checkboxes[index].toggle();
            true
        } else {
            false
        }
    }

    pub fn get_checked_values(&self) -> Vec<ComponentId> {
        self.checkboxes
            .iter()
            .filter(|cb| cb.is_checked())
            .map(|cb| cb.id().clone())
            .collect()
    }

    pub fn get_checked_count(&self) -> usize {
        self.checkboxes.iter().filter(|cb| cb.is_checked()).count()
    }

    pub fn clear_all(&mut self) {
        for checkbox in &mut self.checkboxes {
            checkbox.uncheck();
        }
    }

    pub fn check_all(&mut self) {
        if self.allow_multiple {
            for checkbox in &mut self.checkboxes {
                checkbox.check();
            }
        }
    }
}
