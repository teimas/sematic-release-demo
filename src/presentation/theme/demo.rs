// Theme system demonstration and testing
// Shows all theme features in action

#[cfg(feature = "new-components")]
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Gauge},
    Frame,
};

#[cfg(feature = "new-components")]
use crate::presentation::theme::{
    AppTheme, available_themes, get_theme_by_name,
    colors::ThemeColors,
    styles::StyleSet,
    layout::LayoutTheme,
    animations::{LoadingIndicator, ProgressBar, ProgressBarStyle, LoadingIndicatorConfig},
};

/// Demo showcasing theme system capabilities
#[cfg(feature = "new-components")]
pub struct ThemeDemo {
    current_theme: AppTheme,
    theme_index: usize,
    available_themes: Vec<AppTheme>,
    loading_indicator: LoadingIndicator,
    progress: f64,
}

#[cfg(feature = "new-components")]
impl ThemeDemo {
    pub fn new() -> Self {
        let themes = available_themes();
        let current_theme = themes[0].clone();
        
        Self {
            current_theme: current_theme.clone(),
            theme_index: 0,
            available_themes: themes,
            loading_indicator: LoadingIndicator::new(
                LoadingIndicatorConfig::default(),
                &current_theme.animations,
            ),
            progress: 0.0,
        }
    }
    
    /// Switch to next theme
    pub fn next_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % self.available_themes.len();
        self.current_theme = self.available_themes[self.theme_index].clone();
        self.update_animations();
    }
    
    /// Switch to previous theme
    pub fn previous_theme(&mut self) {
        if self.theme_index == 0 {
            self.theme_index = self.available_themes.len() - 1;
        } else {
            self.theme_index -= 1;
        }
        self.current_theme = self.available_themes[self.theme_index].clone();
        self.update_animations();
    }
    
    /// Update animations based on current theme
    fn update_animations(&mut self) {
        self.loading_indicator = LoadingIndicator::new(
            LoadingIndicatorConfig::default(),
            &self.current_theme.animations,
        );
    }
    
    /// Update demo state
    pub fn update(&mut self) {
        self.loading_indicator.update();
        self.progress = (self.progress + 0.01) % 1.0;
    }
    
    /// Render the theme demo
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Main container
        let block = Block::default()
            .title(format!("Theme Demo - {} (Press Tab to switch)", self.current_theme.name))
            .borders(Borders::ALL)
            .border_style(self.current_theme.primary_style());
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Theme info
                Constraint::Length(3), // Color palette
                Constraint::Length(3), // Text styles
                Constraint::Length(3), // Loading indicators
                Constraint::Min(0),    // Progress bars
            ])
            .split(inner);
        
        self.render_theme_info(frame, chunks[0]);
        self.render_color_palette(frame, chunks[1]);
        self.render_text_styles(frame, chunks[2]);
        self.render_loading_indicators(frame, chunks[3]);
        self.render_progress_bars(frame, chunks[4]);
    }
    
    fn render_theme_info(&self, frame: &mut Frame, area: Rect) {
        let info = format!(
            "Current Theme: {} | Variant: {} | Description: {}",
            self.current_theme.name,
            self.current_theme.variant,
            self.current_theme.description
        );
        
        let paragraph = Paragraph::new(info)
            .style(self.current_theme.primary_style())
            .alignment(Alignment::Center);
        
        frame.render_widget(paragraph, area);
    }
    
    fn render_color_palette(&self, frame: &mut Frame, area: Rect) {
        let colors_text = format!(
            "Colors: Primary: {:?} | Success: {:?} | Warning: {:?} | Error: {:?}",
            self.current_theme.colors.primary,
            self.current_theme.colors.success,
            self.current_theme.colors.warning,
            self.current_theme.colors.error
        );
        
        let paragraph = Paragraph::new(colors_text)
            .style(self.current_theme.secondary_style());
        
        frame.render_widget(paragraph, area);
    }
    
    fn render_text_styles(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);
        
        // Primary style example
        let primary = Paragraph::new("Primary")
            .style(self.current_theme.primary_style());
        frame.render_widget(primary, layout[0]);
        
        // Success style example
        let success = Paragraph::new("Success")
            .style(self.current_theme.success_style());
        frame.render_widget(success, layout[1]);
        
        // Warning style example
        let warning = Paragraph::new("Warning")
            .style(self.current_theme.warning_style());
        frame.render_widget(warning, layout[2]);
        
        // Error style example
        let error = Paragraph::new("Error")
            .style(self.current_theme.error_style());
        frame.render_widget(error, layout[3]);
    }
    
    fn render_loading_indicators(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);
        
        // Loading indicator
        let loading_text = format!("Loading: {}", self.loading_indicator.current_frame());
        let loading = Paragraph::new(loading_text)
            .style(self.current_theme.primary_style());
        frame.render_widget(loading, layout[0]);
        
        // Status text
        let status = Paragraph::new("Theme system active and responsive")
            .style(self.current_theme.success_style());
        frame.render_widget(status, layout[1]);
    }
    
    fn render_progress_bars(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);
        
        // Progress bar
        let progress_bar = Gauge::default()
            .block(Block::default().title("Progress"))
            .gauge_style(self.current_theme.primary_style())
            .ratio(self.progress);
        frame.render_widget(progress_bar, layout[0]);
        
        // Secondary progress
        let secondary_progress = Gauge::default()
            .block(Block::default().title("Secondary"))
            .gauge_style(self.current_theme.secondary_style())
            .ratio((self.progress * 0.7) % 1.0);
        frame.render_widget(secondary_progress, layout[1]);
    }
}

#[cfg(feature = "new-components")]
impl Default for ThemeDemo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "new-components")]
    #[test]
    fn test_theme_demo_creation() {
        let demo = ThemeDemo::new();
        assert_eq!(demo.theme_index, 0);
        assert_eq!(demo.available_themes.len(), 3);
    }
    
    #[cfg(feature = "new-components")]
    #[test]
    fn test_theme_switching() {
        let mut demo = ThemeDemo::new();
        let initial_theme = demo.current_theme.name.clone();
        
        demo.next_theme();
        assert_ne!(demo.current_theme.name, initial_theme);
        
        demo.previous_theme();
        assert_eq!(demo.current_theme.name, initial_theme);
    }
} 