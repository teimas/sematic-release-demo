use crate::presentation::components::core::{
    Component, ComponentEvent, ComponentId, ComponentProps, ComponentState, FocusState,
    ValidationState, VisibilityState, CommonComponentState, ComponentResult, NavigationDirection,
};
use crate::presentation::components::forms::{
    TextInput, TextInputProps, Select, SelectProps, Checkbox, CheckboxProps, 
    Radio, RadioProps, Button, ButtonProps,
};
use crate::state::StateEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

/// Individual form field input types
#[derive(Clone)]
pub enum FormFieldInput {
    TextInput(TextInput),
    Select(Select),
    Checkbox(Checkbox),
    Radio(Radio),
    Button(Button),
}

#[derive(Clone)]
pub enum FormField {
    TextInput(TextInput),
    Select(Select),
    Checkbox(Checkbox),
    Radio(Radio),
    Button(Button),
    Spacer { height: u16 },
    Separator { title: Option<String> },
    Group { 
        title: String, 
        fields: Vec<FormField>,
        layout: FormLayout,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormLayout {
    Vertical,
    Horizontal,
    Grid { columns: u16 },
    TwoColumn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormProps {
    pub id: ComponentId,
    pub title: String,
    pub description: Option<String>,
    pub layout: FormLayout,
    pub submit_on_enter: bool,
    pub reset_on_submit: bool,
    pub show_validation_summary: bool,
    pub auto_focus_first: bool,
    pub show_help_text: bool,
}

impl Default for FormProps {
    fn default() -> Self {
        Self {
            id: ComponentId::new("form"),
            title: String::new(),
            description: None,
            layout: FormLayout::Vertical,
            submit_on_enter: true,
            reset_on_submit: false,
            show_validation_summary: true,
            auto_focus_first: true,
            show_help_text: false,
        }
    }
}

impl ComponentProps for FormProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormState {
    pub common: CommonComponentState,
    pub current_field_index: usize,
    pub field_order: Vec<ComponentId>,
    pub form_values: HashMap<ComponentId, String>,
    pub validation_errors: HashMap<ComponentId, String>,
    pub is_submitting: bool,
    pub is_submitted: bool,
    pub submission_count: u32,
    #[serde(skip)]
    pub last_validation_time: Option<Instant>,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            current_field_index: 0,
            field_order: Vec::new(),
            form_values: HashMap::new(),
            validation_errors: HashMap::new(),
            is_submitting: false,
            is_submitted: false,
            submission_count: 0,
            last_validation_time: None,
        }
    }
}

impl ComponentState for FormState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

pub struct Form {
    id: ComponentId,
    props: FormProps,
    state: FormState,
    fields: Vec<FormField>,
}

impl Form {
    pub fn new(props: FormProps) -> Self {
        Self {
            id: props.id.clone(),
            props,
            state: FormState::default(),
            fields: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.props.title = title.to_string();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.props.description = Some(description.to_string());
        self
    }

    pub fn vertical(mut self) -> Self {
        self.props.layout = FormLayout::Vertical;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.props.layout = FormLayout::Horizontal;
        self
    }

    pub fn two_column(mut self) -> Self {
        self.props.layout = FormLayout::TwoColumn;
        self
    }

    pub fn grid(mut self, columns: u16) -> Self {
        self.props.layout = FormLayout::Grid { columns };
        self
    }

    pub fn submit_on_enter(mut self, submit: bool) -> Self {
        self.props.submit_on_enter = submit;
        self
    }

    pub fn reset_on_submit(mut self, reset: bool) -> Self {
        self.props.reset_on_submit = reset;
        self
    }

    pub fn auto_focus_first(mut self, auto_focus: bool) -> Self {
        self.props.auto_focus_first = auto_focus;
        self
    }

    /// Add a text input field
    pub fn add_text_input(mut self, props: TextInputProps) -> Self {
        let text_input = TextInput::new(props);
        self.fields.push(FormField::TextInput(text_input));
        self.update_field_order();
        self
    }

    /// Add a select field
    pub fn add_select(mut self, props: SelectProps) -> Self {
        let select = Select::new(props);
        self.fields.push(FormField::Select(select));
        self.update_field_order();
        self
    }

    /// Add a checkbox field
    pub fn add_checkbox(mut self, props: CheckboxProps) -> Self {
        let checkbox = Checkbox::new(props);
        self.fields.push(FormField::Checkbox(checkbox));
        self.update_field_order();
        self
    }

    /// Add a radio group field
    pub fn add_radio(mut self, props: RadioProps) -> Self {
        let radio = Radio::new(props);
        self.fields.push(FormField::Radio(radio));
        self.update_field_order();
        self
    }

    /// Add a button
    pub fn add_button(mut self, props: ButtonProps) -> Self {
        let button = Button::new(props);
        self.fields.push(FormField::Button(button));
        self.update_field_order();
        self
    }

    /// Add a spacer
    pub fn add_spacer(mut self, height: u16) -> Self {
        self.fields.push(FormField::Spacer { height });
        self
    }

    /// Add a separator with optional title
    pub fn add_separator(mut self, title: Option<String>) -> Self {
        self.fields.push(FormField::Separator { title });
        self
    }

    /// Add a field group
    pub fn add_group(mut self, title: &str, fields: Vec<FormField>, layout: FormLayout) -> Self {
        self.fields.push(FormField::Group {
            title: title.to_string(),
            fields,
            layout,
        });
        self.update_field_order();
        self
    }

    /// Update the field order for navigation
    fn update_field_order(&mut self) {
        self.state.field_order.clear();
        self.collect_field_ids(&self.fields.clone());
    }

    /// Recursively collect field IDs for navigation
    fn collect_field_ids(&mut self, fields: &[FormField]) {
        for field in fields {
            match field {
                FormField::TextInput(input) => {
                    self.state.field_order.push(input.id().clone());
                }
                FormField::Select(select) => {
                    self.state.field_order.push(select.id().clone());
                }
                FormField::Checkbox(checkbox) => {
                    self.state.field_order.push(checkbox.id().clone());
                }
                FormField::Radio(radio) => {
                    self.state.field_order.push(radio.id().clone());
                }
                FormField::Button(button) => {
                    self.state.field_order.push(button.id().clone());
                }
                FormField::Group { fields, .. } => {
                    self.collect_field_ids(fields);
                }
                _ => {} // Spacers and separators are not focusable
            }
        }
    }

    /// Navigate to the next field
    pub fn next_field(&mut self) -> bool {
        if self.state.current_field_index < self.state.field_order.len().saturating_sub(1) {
            self.state.current_field_index += 1;
            self.update_focus();
            true
        } else {
            false
        }
    }

    /// Navigate to the previous field
    pub fn previous_field(&mut self) -> bool {
        if self.state.current_field_index > 0 {
            self.state.current_field_index -= 1;
            self.update_focus();
            true
        } else {
            false
        }
    }

    /// Update focus state for current field
    fn update_focus(&mut self) {
        // Unfocus all fields first
        self.set_all_fields_unfocused(&mut self.fields.clone());
        
        // Focus current field
        if let Some(current_id) = self.state.field_order.get(self.state.current_field_index).cloned() {
            self.set_field_focused(&mut self.fields.clone(), &current_id);
        }
    }

    /// Set all fields to unfocused state
    fn set_all_fields_unfocused(&mut self, fields: &mut [FormField]) {
        for field in fields {
            match field {
                FormField::TextInput(input) => {
                    input.set_focus_state(FocusState::Unfocused);
                }
                FormField::Select(select) => {
                    select.set_focus_state(FocusState::Unfocused);
                }
                FormField::Checkbox(checkbox) => {
                    checkbox.set_focus_state(FocusState::Unfocused);
                }
                FormField::Radio(radio) => {
                    radio.set_focus_state(FocusState::Unfocused);
                }
                FormField::Button(button) => {
                    button.set_focus_state(FocusState::Unfocused);
                }
                FormField::Group { fields, .. } => {
                    self.set_all_fields_unfocused(fields);
                }
                _ => {}
            }
        }
    }

    /// Set specific field to focused state
    fn set_field_focused(&mut self, fields: &mut [FormField], target_id: &ComponentId) {
        for field in fields {
            match field {
                FormField::TextInput(input) => {
                    if input.id() == target_id {
                        input.set_focus_state(FocusState::Focused);
                    }
                }
                FormField::Select(select) => {
                    if select.id() == target_id {
                        select.set_focus_state(FocusState::Focused);
                    }
                }
                FormField::Checkbox(checkbox) => {
                    if checkbox.id() == target_id {
                        checkbox.set_focus_state(FocusState::Focused);
                    }
                }
                FormField::Radio(radio) => {
                    if radio.id() == target_id {
                        radio.set_focus_state(FocusState::Focused);
                    }
                }
                FormField::Button(button) => {
                    if button.id() == target_id {
                        button.set_focus_state(FocusState::Focused);
                    }
                }
                FormField::Group { fields, .. } => {
                    self.set_field_focused(fields, target_id);
                }
                _ => {}
            }
        }
    }

    /// Get all form values
    pub fn get_values(&self) -> HashMap<ComponentId, String> {
        let mut values = HashMap::new();
        self.collect_values(&self.fields, &mut values);
        values
    }

    /// Recursively collect form values
    fn collect_values(&self, fields: &[FormField], values: &mut HashMap<ComponentId, String>) {
        for field in fields {
            match field {
                FormField::TextInput(input) => {
                    values.insert(input.id().clone(), input.text());
                }
                FormField::Select(select) => {
                    let selected = select.selected_values().join(",");
                    values.insert(select.id().clone(), selected);
                }
                FormField::Checkbox(checkbox) => {
                    values.insert(checkbox.id().clone(), checkbox.to_bool().to_string());
                }
                FormField::Radio(radio) => {
                    let selected = radio.selected_value().unwrap_or("").to_string();
                    values.insert(radio.id().clone(), selected);
                }
                FormField::Group { fields, .. } => {
                    self.collect_values(fields, values);
                }
                _ => {}
            }
        }
    }

    /// Validate the entire form
    pub fn validate(&mut self) -> bool {
        self.state.validation_errors.clear();
        let is_valid = self.validate_fields(&self.fields.clone());
        
        if is_valid {
            self.state.common.validation_state = ValidationState::Valid;
        } else {
            self.state.common.validation_state = ValidationState::Invalid("Form has validation errors".to_string());
        }
        
        is_valid
    }

    /// Recursively validate fields
    fn validate_fields(&mut self, fields: &[FormField]) -> bool {
        let mut all_valid = true;
        
        for field in fields {
            match field {
                FormField::TextInput(input) => {
                    let validation = input.validate();
                    if let ValidationState::Invalid(msg) = validation {
                        self.state.validation_errors.insert(input.id().clone(), msg);
                        all_valid = false;
                    }
                }
                FormField::Select(select) => {
                    let validation = select.validate();
                    if let ValidationState::Invalid(msg) = validation {
                        self.state.validation_errors.insert(select.id().clone(), msg);
                        all_valid = false;
                    }
                }
                FormField::Checkbox(checkbox) => {
                    let validation = checkbox.validate();
                    if let ValidationState::Invalid(msg) = validation {
                        self.state.validation_errors.insert(checkbox.id().clone(), msg);
                        all_valid = false;
                    }
                }
                FormField::Radio(radio) => {
                    let validation = radio.validate();
                    if let ValidationState::Invalid(msg) = validation {
                        self.state.validation_errors.insert(radio.id().clone(), msg);
                        all_valid = false;
                    }
                }
                FormField::Group { fields, .. } => {
                    if !self.validate_fields(fields) {
                        all_valid = false;
                    }
                }
                _ => {}
            }
        }
        
        all_valid
    }

    /// Submit the form
    pub fn submit(&mut self) -> bool {
        if self.state.is_submitting {
            return false;
        }
        
        self.state.is_submitting = true;
        
        if self.validate() {
            self.state.form_values = self.get_values();
            self.state.is_submitted = true;
            self.state.submission_count += 1;
            
            if self.props.reset_on_submit {
                self.reset();
            }
            
            self.state.is_submitting = false;
            true
        } else {
            self.state.is_submitting = false;
            false
        }
    }

    /// Reset the form
    pub fn reset(&mut self) {
        self.reset_fields(&mut self.fields.clone());
        self.state.validation_errors.clear();
        self.state.is_submitted = false;
        self.state.common.validation_state = ValidationState::Valid;
        
        if self.props.auto_focus_first {
            self.state.current_field_index = 0;
            self.update_focus();
        }
    }

    /// Recursively reset fields
    fn reset_fields(&mut self, fields: &mut [FormField]) {
        for field in fields {
            match field {
                FormField::TextInput(input) => {
                    input.clear();
                }
                FormField::Select(select) => {
                    select.clear_selections();
                }
                FormField::Checkbox(checkbox) => {
                    checkbox.uncheck();
                }
                FormField::Radio(radio) => {
                    radio.clear_selection();
                }
                FormField::Group { fields, .. } => {
                    self.reset_fields(fields);
                }
                _ => {}
            }
        }
    }

    /// Check if form is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.state.common.validation_state, ValidationState::Valid) && 
        self.state.validation_errors.is_empty()
    }

    /// Check if form has been submitted
    pub fn is_submitted(&self) -> bool {
        self.state.is_submitted
    }

    /// Check if form is currently submitting
    pub fn is_submitting(&self) -> bool {
        self.state.is_submitting
    }

    /// Get validation errors
    pub fn validation_errors(&self) -> &HashMap<ComponentId, String> {
        &self.state.validation_errors
    }

    /// Get form submission count
    pub fn submission_count(&self) -> u32 {
        self.state.submission_count
    }
}

#[async_trait]
impl Component for Form {
    type Props = FormProps;
    type State = FormState;

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
        
        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Tab, KeyModifiers::NONE) => {
                if self.next_field() {
                    events.push(ComponentEvent::NavigationRequested {
                        component_id: self.id.clone(),
                        direction: NavigationDirection::Next,
                    });
                }
            }
            (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                if self.previous_field() {
                    events.push(ComponentEvent::NavigationRequested {
                        component_id: self.id.clone(),
                        direction: NavigationDirection::Previous,
                    });
                }
            }
            
            // Form submission
            (KeyCode::Enter, KeyModifiers::CONTROL) if self.props.submit_on_enter => {
                if self.submit() {
                    events.push(ComponentEvent::Activated {
                        component_id: self.id.clone(),
                    });
                }
            }
            
            // Form reset
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.reset();
                events.push(ComponentEvent::ValueChanged {
                    component_id: self.id.clone(),
                    old_value: "submitted".to_string(),
                    new_value: "reset".to_string(),
                });
            }
            
            _ => {
                // Forward key events to focused field
                if let Some(current_id) = self.state.field_order.get(self.state.current_field_index).cloned() {
                    let field_events = self.handle_field_key(&mut self.fields.clone(), &current_id, key).await?;
                    events.extend(field_events);
                }
            }
        }

        Ok(events)
    }

    async fn handle_event(&mut self, event: ComponentEvent) -> ComponentResult<Vec<ComponentEvent>> {
        match event {
            ComponentEvent::FocusGained { component_id } if component_id == self.id => {
                self.set_focus_state(FocusState::Focused);
                if self.props.auto_focus_first && !self.state.field_order.is_empty() {
                    self.state.current_field_index = 0;
                    self.update_focus();
                }
                Ok(vec![])
            }
            ComponentEvent::FocusLost { component_id } if component_id == self.id => {
                self.set_focus_state(FocusState::Unfocused);
                self.set_all_fields_unfocused(&mut self.fields.clone());
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.visibility_state() != VisibilityState::Visible {
            return;
        }

        let title = if self.props.title.is_empty() {
            String::new()
        } else {
            format!("{} {}", self.props.title, if self.state.is_submitting { "(Submitting...)" } else { "" })
        };

        let block = if title.is_empty() {
            Block::default().borders(Borders::ALL)
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(title)
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Reserve space for description and validation summary
        let mut content_area = inner;
        let mut y_offset = 0;

        // Render description
        if let Some(description) = &self.props.description {
            let desc_area = Rect {
                x: content_area.x,
                y: content_area.y + y_offset,
                width: content_area.width,
                height: 2,
            };
            
            let desc_para = Paragraph::new(description.as_str())
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(desc_para, desc_area);
            y_offset += 2;
        }

        // Render validation summary
        if self.props.show_validation_summary && !self.state.validation_errors.is_empty() {
            let error_count = self.state.validation_errors.len();
            let summary_text = format!("⚠️ {} validation error{}", error_count, if error_count > 1 { "s" } else { "" });
            
            let summary_area = Rect {
                x: content_area.x,
                y: content_area.y + y_offset,
                width: content_area.width,
                height: 1,
            };
            
            let summary_para = Paragraph::new(summary_text)
                .style(Style::default().fg(Color::Red));
            frame.render_widget(summary_para, summary_area);
            y_offset += 2;
        }

        // Update content area
        content_area = Rect {
            x: content_area.x,
            y: content_area.y + y_offset,
            width: content_area.width,
            height: content_area.height.saturating_sub(y_offset),
        };

        // Render form fields based on layout
        self.render_fields(frame, content_area, &self.fields, &self.props.layout);
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        // Could react to global validation changes, theme updates, etc.
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        self.state.common.validation_state.clone()
    }
}

impl Form {
    /// Handle key events for specific fields
    async fn handle_field_key(&mut self, fields: &mut [FormField], target_id: &ComponentId, key: crossterm::event::KeyEvent) -> ComponentResult<Vec<ComponentEvent>> {
        for field in fields {
            match field {
                FormField::TextInput(input) => {
                    if input.id() == target_id {
                        return input.handle_key(key).await;
                    }
                }
                FormField::Select(select) => {
                    if select.id() == target_id {
                        return select.handle_key(key).await;
                    }
                }
                FormField::Checkbox(checkbox) => {
                    if checkbox.id() == target_id {
                        return checkbox.handle_key(key).await;
                    }
                }
                FormField::Radio(radio) => {
                    if radio.id() == target_id {
                        return radio.handle_key(key).await;
                    }
                }
                FormField::Button(button) => {
                    if button.id() == target_id {
                        return button.handle_key(key).await;
                    }
                }
                FormField::Group { title: _, fields, layout: _ } => {
                    // Handle Group fields by iterating through them
                    for field in fields {
                        match field {
                            FormField::TextInput(input) if input.id() == target_id => {
                                return input.handle_key(key).await;
                            }
                            FormField::Select(select) if select.id() == target_id => {
                                return select.handle_key(key).await;
                            }
                            FormField::Checkbox(checkbox) if checkbox.id() == target_id => {
                                return checkbox.handle_key(key).await;
                            }
                            FormField::Radio(radio) if radio.id() == target_id => {
                                return radio.handle_key(key).await;
                            }
                            FormField::Button(button) if button.id() == target_id => {
                                return button.handle_key(key).await;
                            }
                            _ => continue,
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(vec![])
    }

    /// Render form fields based on layout
    fn render_fields(&self, frame: &mut Frame, area: Rect, fields: &[FormField], layout: &FormLayout) {
        match layout {
            FormLayout::Vertical => self.render_vertical_layout(frame, area, fields),
            FormLayout::Horizontal => self.render_horizontal_layout(frame, area, fields),
            FormLayout::TwoColumn => self.render_two_column_layout(frame, area, fields),
            FormLayout::Grid { columns } => self.render_grid_layout(frame, area, fields, *columns),
        }
    }

    /// Render fields in vertical layout
    fn render_vertical_layout(&self, frame: &mut Frame, area: Rect, fields: &[FormField]) {
        let mut y_offset = 0;
        
        for field in fields {
            if y_offset >= area.height {
                break;
            }
            
            let field_height = self.get_field_height(field);
            let field_area = Rect {
                x: area.x,
                y: area.y + y_offset,
                width: area.width,
                height: field_height.min(area.height - y_offset),
            };
            
            self.render_single_field(frame, field_area, field);
            y_offset += field_height;
        }
    }

    /// Render fields in horizontal layout
    fn render_horizontal_layout(&self, frame: &mut Frame, area: Rect, fields: &[FormField]) {
        if fields.is_empty() {
            return;
        }
        
        let field_width = area.width / fields.len() as u16;
        
        for (index, field) in fields.iter().enumerate() {
            let field_area = Rect {
                x: area.x + (index as u16 * field_width),
                y: area.y,
                width: field_width,
                height: area.height,
            };
            
            self.render_single_field(frame, field_area, field);
        }
    }

    /// Render fields in two-column layout
    fn render_two_column_layout(&self, frame: &mut Frame, area: Rect, fields: &[FormField]) {
        let column_width = area.width / 2;
        let mut left_y = 0;
        let mut right_y = 0;
        
        for (index, field) in fields.iter().enumerate() {
            let is_left_column = index % 2 == 0;
            let field_height = self.get_field_height(field);
            
            if is_left_column {
                let field_area = Rect {
                    x: area.x,
                    y: area.y + left_y,
                    width: column_width,
                    height: field_height.min(area.height - left_y),
                };
                self.render_single_field(frame, field_area, field);
                left_y += field_height;
            } else {
                let field_area = Rect {
                    x: area.x + column_width,
                    y: area.y + right_y,
                    width: column_width,
                    height: field_height.min(area.height - right_y),
                };
                self.render_single_field(frame, field_area, field);
                right_y += field_height;
            }
        }
    }

    /// Render fields in grid layout
    fn render_grid_layout(&self, frame: &mut Frame, area: Rect, fields: &[FormField], columns: u16) {
        let column_width = area.width / columns;
        let rows = (fields.len() as u16 + columns - 1) / columns;
        let row_height = area.height / rows.max(1);
        
        for (index, field) in fields.iter().enumerate() {
            let row = index as u16 / columns;
            let col = index as u16 % columns;
            
            let field_area = Rect {
                x: area.x + (col * column_width),
                y: area.y + (row * row_height),
                width: column_width,
                height: row_height,
            };
            
            self.render_single_field(frame, field_area, field);
        }
    }

    /// Render a single form field
    fn render_single_field(&self, frame: &mut Frame, area: Rect, field: &FormField) {
        match field {
            FormField::TextInput(input) => input.render(frame, area),
            FormField::Select(select) => select.render(frame, area),
            FormField::Checkbox(checkbox) => checkbox.render(frame, area),
            FormField::Radio(radio) => radio.render(frame, area),
            FormField::Button(button) => button.render(frame, area),
            FormField::Spacer { .. } => {
                // Spacers don't render anything
            }
            FormField::Separator { title } => {
                let text = if let Some(title) = title {
                    format!("─── {} ───", title)
                } else {
                    "─".repeat(area.width as usize)
                };
                
                let separator = Paragraph::new(text)
                    .style(Style::default().fg(Color::Gray));
                frame.render_widget(separator, area);
            }
            FormField::Group { title, fields, layout } => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(title.clone())
                    .border_style(Style::default().fg(Color::Blue));
                
                let inner = block.inner(area);
                frame.render_widget(block, area);
                
                self.render_fields(frame, inner, fields, layout);
            }
        }
    }

    /// Get the height required for a field
    fn get_field_height(&self, field: &FormField) -> u16 {
        match field {
            FormField::TextInput(_) => 3,
            FormField::Select(_) => 3,
            FormField::Checkbox(_) => 1,
            FormField::Radio(radio) => {
                match radio.props().layout {
                    crate::presentation::components::forms::RadioLayout::Vertical => radio.props().options.len() as u16 + 2,
                    crate::presentation::components::forms::RadioLayout::Horizontal => 3,
                    crate::presentation::components::forms::RadioLayout::Grid(columns) => {
                        let rows = (radio.props().options.len() as u16 + columns - 1) / columns;
                        rows + 2
                    }
                }
            }
            FormField::Button(_) => 3,
            FormField::Spacer { height } => *height,
            FormField::Separator { .. } => 1,
            FormField::Group { title: _, fields, layout } => {
                match layout {
                    FormLayout::Vertical => {
                        fields.iter().map(|f| self.get_field_height(f)).sum::<u16>() + 2
                    }
                    _ => 5, // Minimum height for other layouts
                }
            }
        }
    }
}

/// Builder for creating forms with a fluent interface
pub struct FormBuilder {
    props: FormProps,
    fields: Vec<FormField>,
}

impl FormBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            props: FormProps {
                id: ComponentId::new(id),
                ..Default::default()
            },
            fields: Vec::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.props.title = title.to_string();
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.props.description = Some(description.to_string());
        self
    }

    pub fn vertical(mut self) -> Self {
        self.props.layout = FormLayout::Vertical;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.props.layout = FormLayout::Horizontal;
        self
    }

    pub fn two_column(mut self) -> Self {
        self.props.layout = FormLayout::TwoColumn;
        self
    }

    pub fn grid(mut self, columns: u16) -> Self {
        self.props.layout = FormLayout::Grid { columns };
        self
    }

    pub fn text_input(mut self, id: &str, label: &str) -> Self {
        let props = TextInputProps {
            id: ComponentId::new(id),
            title: label.to_string(),
            ..Default::default()
        };
        self.fields.push(FormField::TextInput(TextInput::new(props)));
        self
    }

    pub fn select(mut self, id: &str, label: &str) -> Self {
        let props = SelectProps {
            id: ComponentId::new(id),
            title: label.to_string(),
            ..Default::default()
        };
        self.fields.push(FormField::Select(Select::new(props)));
        self
    }

    pub fn checkbox(mut self, id: &str, label: &str) -> Self {
        let props = CheckboxProps {
            id: ComponentId::new(id),
            label: label.to_string(),
            ..Default::default()
        };
        self.fields.push(FormField::Checkbox(Checkbox::new(props)));
        self
    }

    pub fn submit_button(mut self, text: &str) -> Self {
        let props = ButtonProps {
            id: ComponentId::new("submit"),
            text: text.to_string(),
            variant: crate::presentation::components::forms::ButtonVariant::Primary,
            ..Default::default()
        };
        self.fields.push(FormField::Button(Button::new(props)));
        self
    }

    pub fn spacer(mut self, height: u16) -> Self {
        self.fields.push(FormField::Spacer { height });
        self
    }

    pub fn separator(mut self, title: Option<&str>) -> Self {
        self.fields.push(FormField::Separator { 
            title: title.map(|t| t.to_string()) 
        });
        self
    }

    pub fn build(self) -> Form {
        let mut form = Form::new(self.props);
        form.fields = self.fields;
        form.update_field_order();
        form
    }
}
