use crate::presentation::components::core::{
    Component, ComponentEvent, ComponentId, ComponentProps, ComponentState, FocusState,
    ValidationState, VisibilityState, CommonComponentState, ComponentResult,
};
use crate::state::StateEvent;
#[cfg(feature = "new-components")]
use crate::presentation::theme::{AppTheme, ThemeColors, ComponentState as ThemeComponentState};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ButtonVariant {
    Primary,    // Main action button
    Secondary,  // Secondary action button
    Success,    // Success/confirmation button
    Warning,    // Warning action button
    Danger,     // Destructive action button
    Ghost,      // Minimal styling button
    Link,       // Link-style button
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// Button state tracking interaction and animation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonState {
    pub common: CommonComponentState,
    pub is_pressed: bool,
    #[serde(skip)] // Skip Instant since it doesn't support serde
    pub loading_start: Option<Instant>,
    #[serde(skip)]
    pub press_start: Option<Instant>,
    pub click_count: u32,
    #[serde(skip)]
    pub last_click_time: Option<Instant>,
}

impl ComponentState for ButtonState {
    fn common(&self) -> &CommonComponentState {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonComponentState {
        &mut self.common
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonProps {
    pub id: ComponentId,
    pub text: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub full_width: bool,
    pub show_icon: bool,
    pub icon: Option<String>,
    pub loading_text: Option<String>,
    pub help_text: Option<String>,
    pub shortcut: Option<String>, // Keyboard shortcut display
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            id: ComponentId::new("button"),
            text: "Button".to_string(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            disabled: false,
            full_width: false,
            show_icon: false,
            icon: None,
            loading_text: None,
            help_text: None,
            shortcut: None,
        }
    }
}

impl ComponentProps for ButtonProps {
    fn default_props() -> Self {
        Self::default()
    }
}

/// Button component for user interactions
#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    id: ComponentId,
    props: ButtonProps,
    state: ButtonState,
}

impl Button {
    pub fn new(props: ButtonProps) -> Self {
        Self {
            id: props.id.clone(),
            props,
            state: ButtonState {
                common: CommonComponentState::default(),
                is_pressed: false,
                loading_start: None,
                press_start: None,
                click_count: 0,
                last_click_time: None,
            },
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.props.text = text.to_string();
        self
    }

    pub fn primary(mut self) -> Self {
        self.props.variant = ButtonVariant::Primary;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.props.variant = ButtonVariant::Secondary;
        self
    }

    pub fn success(mut self) -> Self {
        self.props.variant = ButtonVariant::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.props.variant = ButtonVariant::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.props.variant = ButtonVariant::Danger;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.props.variant = ButtonVariant::Ghost;
        self
    }

    pub fn link(mut self) -> Self {
        self.props.variant = ButtonVariant::Link;
        self
    }

    pub fn small(mut self) -> Self {
        self.props.size = ButtonSize::Small;
        self
    }

    pub fn medium(mut self) -> Self {
        self.props.size = ButtonSize::Medium;
        self
    }

    pub fn large(mut self) -> Self {
        self.props.size = ButtonSize::Large;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.props.full_width = full_width;
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.props.icon = Some(icon.to_string());
        self.props.show_icon = true;
        self
    }

    pub fn with_loading_text(mut self, loading_text: &str) -> Self {
        self.props.loading_text = Some(loading_text.to_string());
        self
    }

    pub fn with_help_text(mut self, help_text: &str) -> Self {
        self.props.help_text = Some(help_text.to_string());
        self
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.props.shortcut = Some(shortcut.to_string());
        self
    }

    /// Start loading state
    pub fn start_loading(&mut self) {
        if !self.is_disabled() {
            self.state.loading_start = Some(Instant::now());
        }
    }

    /// Stop loading state
    pub fn stop_loading(&mut self) {
        self.state.loading_start = None;
    }

    /// Enable the button
    pub fn enable(&mut self) {
        self.props.disabled = false;
    }

    /// Disable the button
    pub fn disable(&mut self) {
        self.props.disabled = true;
    }

    /// Check if button is disabled
    pub fn is_disabled(&self) -> bool {
        self.props.disabled
    }

    /// Check if button is loading
    pub fn is_loading(&self) -> bool {
        self.state.loading_start.is_some()
    }

    /// Check if button is pressed
    pub fn is_pressed(&self) -> bool {
        self.state.is_pressed
    }

    /// Simulate button press
    pub fn press(&mut self) {
        if !self.is_disabled() && !self.is_loading() {
            self.state.is_pressed = true;
            self.state.press_start = Some(Instant::now());
        }
    }

    /// Release button press
    pub fn release(&mut self) {
        if self.state.is_pressed {
            self.state.is_pressed = false;
            self.state.press_start = None;
        }
    }

    /// Trigger button click
    pub fn click(&mut self) -> bool {
        if self.is_disabled() || self.is_loading() {
            return false;
        }

        let now = Instant::now();
        
        // Handle double-click detection
        if let Some(last_click) = self.state.last_click_time {
            if now.duration_since(last_click) < Duration::from_millis(500) {
                self.state.click_count += 1;
            } else {
                self.state.click_count = 1;
            }
        } else {
            self.state.click_count = 1;
        }
        
        self.state.last_click_time = Some(now);
        self.state.common.mark_dirty();
        
        true
    }

    /// Get click count (for double-click detection)
    pub fn click_count(&self) -> u32 {
        self.state.click_count
    }

    /// Reset click count
    pub fn reset_click_count(&mut self) {
        self.state.click_count = 0;
        self.state.last_click_time = None;
    }

    /// Get button dimensions based on size
    fn get_button_dimensions(&self) -> (u16, u16) {
        let text_width = self.get_display_text().len() as u16;
        let min_width = match self.props.size {
            ButtonSize::Small => text_width + 2,
            ButtonSize::Medium => text_width + 4,
            ButtonSize::Large => text_width + 6,
        };
        
        let height = match self.props.size {
            ButtonSize::Small => 1,
            ButtonSize::Medium => 3,
            ButtonSize::Large => 3,
        };

        (min_width, height)
    }

    /// Get display text including loading and icons
    fn get_display_text(&self) -> String {
        let mut text = match self.state.loading_start {
            Some(_) => {
                if let Some(loading_text) = &self.props.loading_text {
                    loading_text.clone()
                } else {
                    "Loading...".to_string()
                }
            }
            _ => self.props.text.clone(),
        };

        // Add icon if specified
        if self.props.show_icon {
            if let Some(icon) = &self.props.icon {
                text = if self.is_loading() {
                    format!("⏳ {}", text)
                } else {
                    format!("{} {}", icon, text)
                };
            }
        } else if self.is_loading() {
            text = format!("⏳ {}", text);
        }

        // Add shortcut hint
        if let Some(shortcut) = &self.props.shortcut {
            text = format!("{} ({})", text, shortcut);
        }

        text
    }

    /// Get button style with theme support
    fn get_button_style(&self) -> (Style, Style) {
        self.get_button_style_with_theme(None)
    }
    
    /// Get button style with optional theme override
    #[cfg(feature = "new-components")]
    fn get_button_style_with_theme(&self, theme: Option<&AppTheme>) -> (Style, Style) {
        let default_theme = AppTheme::dark();
        let theme = theme.unwrap_or(&default_theme);
        // Determine component state for theming
        let theme_state = if self.props.disabled {
            ThemeComponentState::Disabled
        } else if self.state.common.focus_state == FocusState::Focused {
            ThemeComponentState::Focused
        } else if self.state.loading_start.is_some() {
            ThemeComponentState::Loading
        } else if self.state.is_pressed {
            ThemeComponentState::Selected
        } else {
            ThemeComponentState::Normal
        };
        
        let (fg_color, bg_color) = match self.props.variant {
            ButtonVariant::Primary => {
                (theme.colors.palette.fg_inverse, theme.colors.primary)
            }
            ButtonVariant::Secondary => {
                (theme.colors.foreground, theme.colors.secondary)
            }
            ButtonVariant::Success => {
                (theme.colors.palette.fg_inverse, theme.colors.success)
            }
            ButtonVariant::Warning => {
                (theme.colors.palette.fg_inverse, theme.colors.warning)
            }
            ButtonVariant::Danger => {
                (theme.colors.palette.fg_inverse, theme.colors.error)
            }
            ButtonVariant::Ghost => {
                (theme.colors.foreground, theme.colors.background)
            }
            ButtonVariant::Link => {
                (theme.colors.primary, theme.colors.background)
            }
        };
        
        // Apply state-specific color modifications
        let final_fg = theme.colors.get_state_color(theme_state);
        let final_bg = if theme_state != ThemeComponentState::Normal {
            theme.colors.get_state_bg_color(theme_state)
        } else {
            bg_color
        };
        
        let text_style = Style::default()
            .fg(final_fg)
            .bg(final_bg)
            .add_modifier(theme.styles.text.normal.modifiers);
            
        let border_style = if theme_state == ThemeComponentState::Focused {
            Style::default().fg(theme.colors.focus)
        } else if theme_state == ThemeComponentState::Error {
            Style::default().fg(theme.colors.error)
        } else {
            Style::default().fg(theme.colors.border)
        };
        
        (text_style, border_style)
    }

    #[cfg(not(feature = "new-components"))]
    fn get_button_style_with_theme(&self, _theme: Option<&AppTheme>) -> (Style, Style) {
        // Fallback to original hardcoded colors when theme system is disabled
        let (fg_color, bg_color) = match self.props.variant {
            ButtonVariant::Primary => (Color::White, Color::Blue),
            ButtonVariant::Secondary => (Color::Black, Color::Gray),
            ButtonVariant::Success => (Color::White, Color::Green),
            ButtonVariant::Warning => (Color::Black, Color::Yellow),
            ButtonVariant::Danger => (Color::White, Color::Red),
            ButtonVariant::Ghost => (Color::White, Color::Reset),
            ButtonVariant::Link => (Color::Blue, Color::Reset),
        };

        let text_style = if self.props.disabled {
            Style::default().fg(Color::DarkGray).bg(Color::Reset)
        } else if self.state.common.focus_state == FocusState::Focused {
            Style::default().fg(fg_color).bg(bg_color).add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };

        let border_style = if self.state.common.focus_state == FocusState::Focused {
            Style::default().fg(Color::Cyan)
        } else if self.state.common.validation_state != ValidationState::Valid {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };

        (text_style, border_style)
    }

    /// Check if button needs border
    fn needs_border(&self) -> bool {
        !matches!(self.props.variant, ButtonVariant::Ghost | ButtonVariant::Link)
    }

    /// Update button state based on time
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // Auto-release press after short duration
        if let Some(press_start) = self.state.press_start {
            if now.duration_since(press_start) > Duration::from_millis(150) {
                self.release();
            }
        }
        
        // Reset click count after timeout
        if let Some(last_click) = self.state.last_click_time {
            if now.duration_since(last_click) > Duration::from_millis(1000) {
                self.reset_click_count();
            }
        }
    }
}

#[async_trait]
impl Component for Button {
    type Props = ButtonProps;
    type State = ButtonState;

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
        
        if self.focus_state() != FocusState::Focused {
            return Ok(events);
        }

        match (key.code, key.modifiers) {
            // Button activation
            (KeyCode::Enter, _) | (KeyCode::Char(' '), _) => {
                if self.click() {
                    self.press();
                    
                    events.push(ComponentEvent::ButtonClicked {
                        component_id: self.id.clone(),
                        button_text: self.props.text.clone(),
                        click_count: self.state.click_count,
                    });
                }
            }
            
            // Quick shortcuts for specific variants
            (KeyCode::Char('y'), _) if matches!(self.props.variant, ButtonVariant::Success) => {
                if self.click() {
                    self.press();
                    events.push(ComponentEvent::ButtonClicked {
                        component_id: self.id.clone(),
                        button_text: self.props.text.clone(),
                        click_count: self.state.click_count,
                    });
                }
            }
            
            (KeyCode::Char('n'), _) if matches!(self.props.variant, ButtonVariant::Danger) => {
                if self.click() {
                    self.press();
                    events.push(ComponentEvent::ButtonClicked {
                        component_id: self.id.clone(),
                        button_text: self.props.text.clone(),
                        click_count: self.state.click_count,
                    });
                }
            }
            
            _ => {}
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
                self.release(); // Release any press state when losing focus
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.visibility_state() != VisibilityState::Visible {
            return;
        }

        let display_text = self.get_display_text();
        let (text_style, border_style) = self.get_button_style();
        let (min_width, min_height) = self.get_button_dimensions();

        // Calculate button area
        let button_area = if self.props.full_width {
            area
        } else {
            let width = min_width.min(area.width);
            let height = min_height.min(area.height);
            Rect {
                x: area.x + (area.width.saturating_sub(width)) / 2,
                y: area.y + (area.height.saturating_sub(height)) / 2,
                width,
                height,
            }
        };

        if self.needs_border() {
            // Render with border
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            let inner = block.inner(button_area);
            frame.render_widget(block, button_area);

            let paragraph = Paragraph::new(display_text)
                .style(text_style)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, inner);
        } else {
            // Render without border (ghost/link style)
            let paragraph = Paragraph::new(display_text)
                .style(text_style)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, button_area);
        }

        // Render help text if available and button has focus
        if let Some(help_text) = &self.props.help_text {
            if self.state.common.focus_state == FocusState::Focused {
                self.render_help_text(frame, area, help_text);
            }
        }
    }

    async fn update_from_state(&mut self, _state_event: &StateEvent) -> ComponentResult<bool> {
        // Update time-based states
        self.update();
        // Could react to theme changes, global loading states, etc.
        Ok(false)
    }

    fn validate(&self) -> ValidationState {
        // Buttons are always valid
        ValidationState::Valid
    }
}

impl Button {
    /// Render help text below the button
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
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, help_area);
    }
}

// Utility for button groups and toolbars
pub struct ButtonGroup {
    buttons: Vec<Button>,
    orientation: ButtonGroupOrientation,
    spacing: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonGroupOrientation {
    Horizontal,
    Vertical,
}

impl ButtonGroup {
    pub fn new(orientation: ButtonGroupOrientation) -> Self {
        Self {
            buttons: Vec::new(),
            orientation,
            spacing: 1,
        }
    }

    pub fn horizontal() -> Self {
        Self::new(ButtonGroupOrientation::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(ButtonGroupOrientation::Vertical)
    }

    pub fn with_spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn add_button(&mut self, button: Button) {
        self.buttons.push(button);
    }

    pub fn get_button_mut(&mut self, id: &ComponentId) -> Option<&mut Button> {
        self.buttons.iter_mut().find(|b| b.id() == id)
    }

    pub fn get_button(&self, id: &ComponentId) -> Option<&Button> {
        self.buttons.iter().find(|b| b.id() == id)
    }

    pub fn button_count(&self) -> usize {
        self.buttons.len()
    }

    pub fn set_all_loading(&mut self, loading: bool) {
        for button in &mut self.buttons {
            if loading {
                button.start_loading();
            } else {
                button.stop_loading();
            }
        }
    }

    pub fn enable_all(&mut self) {
        for button in &mut self.buttons {
            button.enable();
        }
    }

    pub fn disable_all(&mut self) {
        for button in &mut self.buttons {
            button.disable();
        }
    }
}
