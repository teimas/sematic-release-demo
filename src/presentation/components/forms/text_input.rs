use crate::presentation::components::core::{
    Component, ComponentEvent, ComponentId, ComponentProps, ComponentState, FocusState,
    ValidationState, VisibilityState, CommonComponentState, ComponentResult, NavigationDirection,
};
use crate::state::StateEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use serde::{Deserialize, Serialize};
use tui_textarea::{Input, Key, TextArea};
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextInputProps {
    pub id: ComponentId,
    pub placeholder: String,
    pub title: String,
    pub multiline: bool,
    pub max_length: Option<usize>,
    pub required: bool,
    pub readonly: bool,
    pub tab_length: u8,
    pub initial_value: String,
    pub validation_pattern: Option<String>,
    pub help_text: Option<String>,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            id: ComponentId::new("text_input"),
            placeholder: String::new(),
            title: String::new(),
            multiline: false,
            max_length: None,
            required: false,
            readonly: false,
            tab_length: 4,
            initial_value: String::new(),
            validation_pattern: None,
            help_text: None,
        }
    }
}

impl ComponentProps for TextInputProps {
    fn default_props() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextInputState {
    pub common: CommonComponentState,
    pub last_validation_message: Option<String>,
    pub cursor_position: (usize, usize), // (row, col)
    pub scroll_offset: (usize, usize),   // (row, col)
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            common: CommonComponentState::default(),
            last_validation_message: None,
            cursor_position: (0, 0),
            scroll_offset: (0, 0),
        }
    }
}

impl ComponentState for TextInputState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

/// Text input component with validation and styling
#[derive(Debug, Clone)]
pub struct TextInput {
    id: ComponentId,
    props: TextInputProps,
    state: TextInputState,
    textarea: TextArea<'static>,
}

impl TextInput {
    pub fn new(props: TextInputProps) -> Self {
        let mut textarea = TextArea::default();
        
        // Configure textarea based on props
        textarea.set_placeholder_text(&props.placeholder);
        textarea.set_tab_length(props.tab_length);
        
        // Set initial value if provided
        if !props.initial_value.is_empty() {
            textarea.insert_str(&props.initial_value);
        }

        // Configure multiline behavior
        if !props.multiline {
            // For single-line inputs, we'll handle newlines specially
            textarea.set_hard_tab_indent(false);
        }

        let id = props.id.clone();
        let mut instance = Self {
            id,
            props,
            state: TextInputState::default(),
            textarea,
        };
        
        // Validate initial content
        instance.validate_content();
        instance
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.props.placeholder = placeholder.to_string();
        self.textarea.set_placeholder_text(placeholder);
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.props.title = title.to_string();
        self
    }

    pub fn multiline(mut self, multiline: bool) -> Self {
        self.props.multiline = multiline;
        if !multiline {
            self.textarea.set_hard_tab_indent(false);
        }
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.props.required = required;
        self.validate_content();
        self
    }

    pub fn readonly(mut self, readonly: bool) -> Self {
        self.props.readonly = readonly;
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.props.max_length = Some(max_length);
        self
    }

    pub fn with_validation_pattern(mut self, pattern: &str) -> Self {
        self.props.validation_pattern = Some(pattern.to_string());
        self
    }

    pub fn with_help_text(mut self, help_text: &str) -> Self {
        self.props.help_text = Some(help_text.to_string());
        self
    }

    pub fn with_initial_value(mut self, value: &str) -> Self {
        self.props.initial_value = value.to_string();
        self.textarea.select_all();
        self.textarea.delete_str(self.textarea.lines().join("\n").len());
        self.textarea.insert_str(value);
        self
    }

    /// Get the current text content
    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Get the current text content as lines
    pub fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    /// Clear the text content
    pub fn clear(&mut self) {
        self.textarea.select_all();
        self.textarea.delete_str(self.textarea.lines().join("\n").len());
        self.state.common.mark_dirty();
        self.validate_content();
    }

    /// Set the text content
    pub fn set_text(&mut self, text: &str) {
        self.textarea.select_all();
        self.textarea.delete_str(self.textarea.lines().join("\n").len());
        self.textarea.insert_str(text);
        self.state.common.mark_dirty();
        self.validate_content();
    }

    /// Insert text at the current cursor position
    pub fn insert_text(&mut self, text: &str) {
        self.textarea.insert_str(text);
        self.state.common.mark_dirty();
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, input: Input) -> bool {
        if self.props.readonly {
            return false;
        }

        // For single-line inputs, prevent newlines
        if !self.props.multiline && input.key == Key::Enter {
            return false;
        }

        // Check max length constraint
        if let Some(max_length) = self.props.max_length {
            let current_length = self.text().len();
            if current_length >= max_length && matches!(input.key, Key::Char(_)) {
                return false;
            }
        }

        let result = self.textarea.input(input);
        if result {
            self.state.common.mark_dirty();
            self.update_cursor_position();
            self.validate_content();
        }
        result
    }

    /// Convert crossterm KeyEvent to tui-textarea Input
    pub fn crossterm_to_textarea_input(key: crossterm::event::KeyEvent) -> Input {
        use crossterm::event::{KeyCode, KeyModifiers};

        let tui_key = match key.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::Tab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Insert => Key::Null,
            KeyCode::Esc => Key::Esc,
            KeyCode::F(n) => Key::F(n),
            _ => Key::Null,
        };

        Input {
            key: tui_key,
            ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
            alt: key.modifiers.contains(KeyModifiers::ALT),
            shift: key.modifiers.contains(KeyModifiers::SHIFT),
        }
    }

    /// Handle crossterm KeyEvent directly
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> bool {
        let input = Self::crossterm_to_textarea_input(key);
        self.handle_input(input)
    }

    /// Update cursor position tracking
    fn update_cursor_position(&mut self) {
        let cursor = self.textarea.cursor();
        self.state.cursor_position = (cursor.0, cursor.1);
    }

    /// Validate the current content
    fn validate_content(&mut self) {
        let text = self.text();
        
        // Check required field
        if self.props.required && text.trim().is_empty() {
            self.state.common.validation_state = ValidationState::Invalid("This field is required".to_string());
            self.state.last_validation_message = Some("This field is required".to_string());
            return;
        }

        // Check validation pattern
        if let Some(pattern) = &self.props.validation_pattern {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(&text) {
                    self.state.common.validation_state = ValidationState::Invalid("Invalid format".to_string());
                    self.state.last_validation_message = Some("Invalid format".to_string());
                    return;
                }
            }
        }

        // Check max length
        if let Some(max_length) = self.props.max_length {
            if text.len() > max_length {
                let msg = format!("Text exceeds maximum length of {}", max_length);
                self.state.common.validation_state = ValidationState::Invalid(msg.clone());
                self.state.last_validation_message = Some(msg);
                return;
            }
        }

        // All validations passed
        self.state.common.validation_state = ValidationState::Valid;
        self.state.last_validation_message = None;
    }

    /// Get the border style based on component state
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

    /// Get the title with validation indicators
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

        // Add help indicator if help text is available
        if self.props.help_text.is_some() {
            title.push_str(" (?)");
        }

        title
    }

    /// Get validation message if any
    pub fn validation_message(&self) -> Option<&str> {
        self.state.last_validation_message.as_deref()
    }

    /// Check if the component is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.state.common.validation_state, ValidationState::Valid)
    }

    /// Check if the component has any content
    pub fn is_empty(&self) -> bool {
        self.text().trim().is_empty()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.text().len()
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.textarea.lines().len()
    }
}

#[async_trait]
impl Component for TextInput {
    type Props = TextInputProps;
    type State = TextInputState;

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
        let mut events = Vec::new();
        
        if self.focus_state() == FocusState::Focused {
            let old_text = self.text();
            let handled = self.handle_key_event(key);
            
            if handled {
                let new_text = self.text();
                if old_text != new_text {
                    events.push(ComponentEvent::ValueChanged {
                        component_id: self.id.clone(),
                        old_value: old_text,
                        new_value: new_text,
                    });
                }
                
                // Check if validation state changed
                let validation_state = self.validate();
                if self.state.common.validation_state != validation_state {
                    events.push(ComponentEvent::ValidationChanged {
                        component_id: self.id.clone(),
                        state: validation_state.clone(),
                    });
                    self.state.common.validation_state = validation_state;
                }
            }
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
                self.validate_content();
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.visibility_state() != VisibilityState::Visible {
            return;
        }

        // Create block with dynamic styling
        let title = self.get_title_with_indicators();
        let border_style = self.get_border_style();
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        // Clone textarea for rendering since render takes &self
        let mut textarea = self.textarea.clone();
        
        // Configure textarea styling based on focus
        if self.focus_state() == FocusState::Focused {
            textarea.set_cursor_line_style(Style::default().bg(Color::DarkGray));
            textarea.set_cursor_style(Style::default().bg(Color::Yellow));
        } else {
            textarea.set_cursor_line_style(Style::default());
            textarea.set_cursor_style(Style::default());
        }

        // Set the block and render
        textarea.set_block(block);
        frame.render_widget(&textarea, area);
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        // For now, text input doesn't react to global state changes
        // This could be extended to react to theme changes, etc.
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        self.state.common.validation_state.clone()
    }
} 