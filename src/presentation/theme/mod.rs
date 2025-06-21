// Theme and styling system for the semantic release TUI
// Phase 2.3.3: Complete theme engine with light/dark modes and visual polish

pub mod colors;
pub mod styles;
pub mod layout;
pub mod animations;
pub mod theme_manager;

// Re-export main theme types for convenience
pub use colors::{ColorScheme, ColorPalette, ThemeColors, ComponentState};
pub use styles::{TextStyle, BorderStyle, HighlightStyle, StyleSet};
pub use layout::{Spacing, LayoutConstants, LayoutTheme};
pub use animations::{AnimationConfig, LoadingIndicator, TransitionEffect};
pub use theme_manager::{ThemeManager, ThemePreference, ThemeEvent};

#[cfg(feature = "new-components")]
use serde::{Deserialize, Serialize};

/// Core theme trait that all theme components must implement
pub trait Theme: Clone + std::fmt::Debug {
    /// Apply this theme to a component or UI element
    fn apply(&self) -> ratatui::style::Style;
    
    /// Get the theme variant (light/dark/custom)
    fn variant(&self) -> ThemeVariant;
    
    /// Check if this theme supports animations
    fn supports_animations(&self) -> bool {
        true
    }
}

/// Theme variant enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum ThemeVariant {
    Light,
    Dark,
    HighContrast,
    Custom(u32), // Custom theme ID
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Dark
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeVariant::Light => write!(f, "Light"),
            ThemeVariant::Dark => write!(f, "Dark"),
            ThemeVariant::HighContrast => write!(f, "High Contrast"),
            ThemeVariant::Custom(id) => write!(f, "Custom {}", id),
        }
    }
}

/// Main application theme containing all styling components
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "new-components"), derive(Serialize, Deserialize))]
pub struct AppTheme {
    pub variant: ThemeVariant,
    pub colors: ThemeColors,
    pub styles: StyleSet,
    pub layout: LayoutTheme,
    pub animations: AnimationConfig,
    pub name: String,
    pub description: String,
}

impl AppTheme {
    /// Create a new light theme
    pub fn light() -> Self {
        Self {
            variant: ThemeVariant::Light,
            colors: ThemeColors::light(),
            styles: StyleSet::light(),
            layout: LayoutTheme::default(),
            animations: AnimationConfig::default(),
            name: "Light".to_string(),
            description: "Clean light theme for daytime use".to_string(),
        }
    }
    
    /// Create a new dark theme
    pub fn dark() -> Self {
        Self {
            variant: ThemeVariant::Dark,
            colors: ThemeColors::dark(),
            styles: StyleSet::dark(),
            layout: LayoutTheme::default(),
            animations: AnimationConfig::default(),
            name: "Dark".to_string(),
            description: "Professional dark theme for extended use".to_string(),
        }
    }
    
    /// Create a high contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            variant: ThemeVariant::HighContrast,
            colors: ThemeColors::high_contrast(),
            styles: StyleSet::high_contrast(),
            layout: LayoutTheme::accessibility(),
            animations: AnimationConfig::minimal(),
            name: "High Contrast".to_string(),
            description: "High contrast theme for accessibility".to_string(),
        }
    }
    
    /// Get the primary style for components
    pub fn primary_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(self.colors.primary)
            .add_modifier(self.styles.text.normal.modifiers)
    }
    
    /// Get the secondary style for components
    pub fn secondary_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(self.colors.secondary)
            .add_modifier(self.styles.text.muted.modifiers)
    }
    
    /// Get the success style for positive feedback
    pub fn success_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(self.colors.success)
            .add_modifier(self.styles.text.normal.modifiers)
    }
    
    /// Get the warning style for warnings
    pub fn warning_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(self.colors.warning)
            .add_modifier(self.styles.text.bold.modifiers)
    }
    
    /// Get the error style for errors
    pub fn error_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(self.colors.error)
            .add_modifier(self.styles.text.bold.modifiers)
    }
    
    /// Get the focused style for interactive elements
    pub fn focused_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(self.colors.focus)
            .bg(self.colors.focus_bg)
            .add_modifier(self.styles.highlight.focused.modifiers)
    }
    
    /// Get the selected style for selected items
    pub fn selected_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(self.colors.selected)
            .bg(self.colors.selected_bg)
            .add_modifier(self.styles.highlight.selected.modifiers)
    }
    
    /// Create theme from variant
    pub fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Light => Self::light(),
            ThemeVariant::Dark => Self::dark(),
            ThemeVariant::HighContrast => Self::high_contrast(),
            ThemeVariant::Custom(_) => Self::light(), // Default to light for custom
        }
    }
    
    /// Get theme variant
    pub fn variant(&self) -> &ThemeVariant {
        &self.variant
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Utility function to get all available themes
pub fn available_themes() -> Vec<AppTheme> {
    vec![
        AppTheme::light(),
        AppTheme::dark(),
        AppTheme::high_contrast(),
    ]
}

/// Utility function to get theme by name
pub fn get_theme_by_name(name: &str) -> Option<AppTheme> {
    match name.to_lowercase().as_str() {
        "light" => Some(AppTheme::light()),
        "dark" => Some(AppTheme::dark()),
        "high_contrast" | "high-contrast" => Some(AppTheme::high_contrast()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_creation() {
        let light = AppTheme::light();
        assert_eq!(light.variant, ThemeVariant::Light);
        assert_eq!(light.name, "Light");
        
        let dark = AppTheme::dark();
        assert_eq!(dark.variant, ThemeVariant::Dark);
        assert_eq!(dark.name, "Dark");
        
        let high_contrast = AppTheme::high_contrast();
        assert_eq!(high_contrast.variant, ThemeVariant::HighContrast);
        assert_eq!(high_contrast.name, "High Contrast");
    }
    
    #[test]
    fn test_theme_styles() {
        let theme = AppTheme::dark();
        
        // Test that styles are properly configured
        let primary = theme.primary_style();
        assert_eq!(primary.fg, Some(theme.colors.primary));
        
        let error = theme.error_style();
        assert_eq!(error.fg, Some(theme.colors.error));
    }
    
    #[test]
    fn test_theme_variants() {
        assert_eq!(ThemeVariant::Light.to_string(), "Light");
        assert_eq!(ThemeVariant::Dark.to_string(), "Dark");
        assert_eq!(ThemeVariant::HighContrast.to_string(), "High Contrast");
        assert_eq!(ThemeVariant::Custom(1).to_string(), "Custom 1");
    }
    
    #[test]
    fn test_available_themes() {
        let themes = available_themes();
        assert_eq!(themes.len(), 3);
        
        let names: Vec<String> = themes.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"Light".to_string()));
        assert!(names.contains(&"Dark".to_string()));
        assert!(names.contains(&"High Contrast".to_string()));
    }
    
    #[test]
    fn test_get_theme_by_name() {
        assert!(get_theme_by_name("light").is_some());
        assert!(get_theme_by_name("dark").is_some());
        assert!(get_theme_by_name("high_contrast").is_some());
        assert!(get_theme_by_name("high-contrast").is_some());
        assert!(get_theme_by_name("nonexistent").is_none());
    }
} 