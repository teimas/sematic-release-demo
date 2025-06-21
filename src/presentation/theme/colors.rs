// Color palette definitions for the theme system
// Comprehensive color schemes with semantic meaning

use ratatui::style::Color;

/// Main color scheme definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
    HighContrast,
}

/// Complete color palette for the application
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette {
    // Background colors
    pub bg_primary: Color,      // Main background
    pub bg_secondary: Color,    // Secondary panels
    pub bg_tertiary: Color,     // Cards, sections
    pub bg_surface: Color,      // Elevated surfaces
    pub bg_overlay: Color,      // Modal overlays
    
    // Foreground/text colors  
    pub fg_primary: Color,      // Primary text
    pub fg_secondary: Color,    // Secondary text
    pub fg_muted: Color,        // Disabled/muted text
    pub fg_inverse: Color,      // Text on colored backgrounds
    
    // Semantic colors
    pub success: Color,         // Success states
    pub warning: Color,         // Warning states  
    pub error: Color,           // Error states
    pub info: Color,           // Informational
    
    // Interactive colors
    pub primary: Color,         // Primary interactive elements
    pub secondary: Color,       // Secondary interactive elements
    pub focus: Color,          // Focus indicators
    pub focus_bg: Color,       // Focus background
    pub selected: Color,       // Selected text
    pub selected_bg: Color,    // Selected background
    pub hover: Color,          // Hover states
    pub hover_bg: Color,       // Hover background
    
    // Border colors
    pub border_primary: Color,  // Primary borders
    pub border_secondary: Color, // Secondary borders
    pub border_focus: Color,    // Focused borders
    pub border_error: Color,    // Error borders
    
    // Special colors
    pub accent: Color,          // Accent color
    pub highlight: Color,       // Highlight color
    pub shadow: Color,          // Drop shadows
    pub loading: Color,         // Loading indicators
}

impl ColorPalette {
    /// Light theme color palette
    pub fn light() -> Self {
        Self {
            // Light backgrounds - subtle grays and whites
            bg_primary: Color::Rgb(255, 255, 255),      // Pure white
            bg_secondary: Color::Rgb(248, 249, 250),    // Very light gray
            bg_tertiary: Color::Rgb(241, 243, 244),     // Light gray
            bg_surface: Color::Rgb(255, 255, 255),      // Card white
            bg_overlay: Color::Rgb(0, 0, 0),            // Semi-transparent black (fallback)
            
            // Light foreground - dark text on light
            fg_primary: Color::Rgb(33, 37, 41),         // Dark gray
            fg_secondary: Color::Rgb(108, 117, 125),    // Medium gray
            fg_muted: Color::Rgb(173, 181, 189),        // Light gray
            fg_inverse: Color::Rgb(255, 255, 255),      // White on dark
            
            // Semantic colors - vibrant but accessible
            success: Color::Rgb(40, 167, 69),           // Green
            warning: Color::Rgb(255, 193, 7),           // Amber
            error: Color::Rgb(220, 53, 69),             // Red
            info: Color::Rgb(23, 162, 184),             // Cyan
            
            // Interactive colors - blue theme
            primary: Color::Rgb(0, 123, 255),           // Blue
            secondary: Color::Rgb(108, 117, 125),       // Gray
            focus: Color::Rgb(0, 123, 255),             // Blue
            focus_bg: Color::Rgb(230, 244, 255),        // Light blue
            selected: Color::Rgb(255, 255, 255),        // White text
            selected_bg: Color::Rgb(0, 123, 255),       // Blue background
            hover: Color::Rgb(0, 86, 179),              // Darker blue
            hover_bg: Color::Rgb(241, 248, 255),        // Very light blue
            
            // Borders - subtle definition
            border_primary: Color::Rgb(222, 226, 230),  // Light border
            border_secondary: Color::Rgb(241, 243, 244), // Very light border
            border_focus: Color::Rgb(0, 123, 255),      // Blue border
            border_error: Color::Rgb(220, 53, 69),      // Red border
            
            // Special colors
            accent: Color::Rgb(111, 66, 193),           // Purple accent
            highlight: Color::Rgb(255, 235, 59),        // Yellow highlight
            shadow: Color::Rgb(0, 0, 0),               // Subtle shadow (fallback)
            loading: Color::Rgb(0, 123, 255),           // Blue loading
        }
    }
    
    /// Dark theme color palette
    pub fn dark() -> Self {
        Self {
            // Dark backgrounds - deep grays and blacks
            bg_primary: Color::Rgb(13, 17, 23),         // Very dark gray
            bg_secondary: Color::Rgb(21, 32, 43),       // Dark gray
            bg_tertiary: Color::Rgb(33, 38, 45),        // Medium dark gray
            bg_surface: Color::Rgb(22, 27, 34),         // Card dark
            bg_overlay: Color::Rgb(0, 0, 0),            // Semi-transparent black (fallback)
            
            // Dark foreground - light text on dark
            fg_primary: Color::Rgb(248, 249, 250),      // Very light gray
            fg_secondary: Color::Rgb(173, 186, 199),    // Light gray  
            fg_muted: Color::Rgb(110, 118, 129),        // Medium gray
            fg_inverse: Color::Rgb(33, 37, 41),         // Dark on light
            
            // Semantic colors - muted but visible
            success: Color::Rgb(46, 160, 67),           // Green
            warning: Color::Rgb(187, 128, 9),           // Orange
            error: Color::Rgb(248, 81, 73),             // Red
            info: Color::Rgb(58, 176, 255),             // Light blue
            
            // Interactive colors - vibrant blue theme
            primary: Color::Rgb(58, 176, 255),          // Light blue
            secondary: Color::Rgb(110, 118, 129),       // Gray
            focus: Color::Rgb(58, 176, 255),            // Light blue
            focus_bg: Color::Rgb(0, 33, 85),            // Dark blue
            selected: Color::Rgb(248, 249, 250),        // Light text
            selected_bg: Color::Rgb(58, 176, 255),      // Blue background
            hover: Color::Rgb(116, 202, 255),           // Lighter blue
            hover_bg: Color::Rgb(0, 22, 56),            // Very dark blue
            
            // Borders - visible but subtle
            border_primary: Color::Rgb(48, 54, 61),     // Medium border
            border_secondary: Color::Rgb(33, 38, 45),   // Dark border
            border_focus: Color::Rgb(58, 176, 255),     // Blue border
            border_error: Color::Rgb(248, 81, 73),      // Red border
            
            // Special colors
            accent: Color::Rgb(180, 83, 255),           // Purple accent
            highlight: Color::Rgb(255, 235, 59),        // Yellow highlight
            shadow: Color::Rgb(0, 0, 0),               // Dark shadow (fallback)
            loading: Color::Rgb(58, 176, 255),          // Blue loading
        }
    }
    
    /// High contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            // High contrast backgrounds - pure black/white
            bg_primary: Color::Rgb(0, 0, 0),            // Pure black
            bg_secondary: Color::Rgb(16, 16, 16),       // Very dark gray
            bg_tertiary: Color::Rgb(32, 32, 32),        // Dark gray
            bg_surface: Color::Rgb(0, 0, 0),            // Pure black
            bg_overlay: Color::Rgb(0, 0, 0),            // Nearly opaque black (fallback)
            
            // High contrast foreground - pure white
            fg_primary: Color::Rgb(255, 255, 255),      // Pure white
            fg_secondary: Color::Rgb(224, 224, 224),    // Light gray
            fg_muted: Color::Rgb(160, 160, 160),        // Medium gray
            fg_inverse: Color::Rgb(0, 0, 0),            // Pure black
            
            // High contrast semantic colors
            success: Color::Rgb(0, 255, 0),             // Pure green
            warning: Color::Rgb(255, 255, 0),           // Pure yellow
            error: Color::Rgb(255, 0, 0),               // Pure red
            info: Color::Rgb(0, 255, 255),              // Pure cyan
            
            // High contrast interactive
            primary: Color::Rgb(255, 255, 255),         // Pure white
            secondary: Color::Rgb(192, 192, 192),       // Light gray
            focus: Color::Rgb(255, 255, 0),             // Yellow focus
            focus_bg: Color::Rgb(64, 64, 0),            // Dark yellow
            selected: Color::Rgb(0, 0, 0),              // Black text
            selected_bg: Color::Rgb(255, 255, 255),     // White background
            hover: Color::Rgb(255, 255, 0),             // Yellow hover
            hover_bg: Color::Rgb(32, 32, 0),            // Very dark yellow
            
            // High contrast borders
            border_primary: Color::Rgb(255, 255, 255),  // White border
            border_secondary: Color::Rgb(128, 128, 128), // Gray border
            border_focus: Color::Rgb(255, 255, 0),      // Yellow border
            border_error: Color::Rgb(255, 0, 0),        // Red border
            
            // High contrast special colors
            accent: Color::Rgb(255, 0, 255),            // Magenta accent
            highlight: Color::Rgb(255, 255, 0),         // Yellow highlight
            shadow: Color::Rgb(0, 0, 0),               // White shadow (fallback)
            loading: Color::Rgb(255, 255, 255),         // White loading
        }
    }
}

/// Theme-specific color implementation
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub palette: ColorPalette,
    
    // Convenience accessors for common colors
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub focus: Color,
    pub focus_bg: Color,
    pub selected: Color,
    pub selected_bg: Color,
    pub border: Color,
    pub accent: Color,
}

impl ThemeColors {
    /// Create light theme colors
    pub fn light() -> Self {
        let palette = ColorPalette::light();
        Self {
            palette,
            background: palette.bg_primary,
            foreground: palette.fg_primary,
            primary: palette.primary,
            secondary: palette.secondary,
            success: palette.success,
            warning: palette.warning,
            error: palette.error,
            info: palette.info,
            focus: palette.focus,
            focus_bg: palette.focus_bg,
            selected: palette.selected,
            selected_bg: palette.selected_bg,
            border: palette.border_primary,
            accent: palette.accent,
        }
    }
    
    /// Create dark theme colors
    pub fn dark() -> Self {
        let palette = ColorPalette::dark();
        Self {
            palette,
            background: palette.bg_primary,
            foreground: palette.fg_primary,
            primary: palette.primary,
            secondary: palette.secondary,
            success: palette.success,
            warning: palette.warning,
            error: palette.error,
            info: palette.info,
            focus: palette.focus,
            focus_bg: palette.focus_bg,
            selected: palette.selected,
            selected_bg: palette.selected_bg,
            border: palette.border_primary,
            accent: palette.accent,
        }
    }
    
    /// Create high contrast theme colors
    pub fn high_contrast() -> Self {
        let palette = ColorPalette::high_contrast();
        Self {
            palette,
            background: palette.bg_primary,
            foreground: palette.fg_primary,
            primary: palette.primary,
            secondary: palette.secondary,
            success: palette.success,
            warning: palette.warning,
            error: palette.error,
            info: palette.info,
            focus: palette.focus,
            focus_bg: palette.focus_bg,
            selected: palette.selected,
            selected_bg: palette.selected_bg,
            border: palette.border_primary,
            accent: palette.accent,
        }
    }
    
    /// Get color for component state
    pub fn get_state_color(&self, state: ComponentState) -> Color {
        match state {
            ComponentState::Normal => self.foreground,
            ComponentState::Focused => self.focus,
            ComponentState::Selected => self.selected,
            ComponentState::Disabled => self.palette.fg_muted,
            ComponentState::Error => self.error,
            ComponentState::Warning => self.warning,
            ComponentState::Success => self.success,
            ComponentState::Loading => self.palette.loading,
            ComponentState::Hover => self.palette.hover,
        }
    }
    
    /// Get background color for component state
    pub fn get_state_bg_color(&self, state: ComponentState) -> Color {
        match state {
            ComponentState::Normal => self.background,
            ComponentState::Focused => self.focus_bg,
            ComponentState::Selected => self.selected_bg,
            ComponentState::Disabled => self.palette.bg_secondary,
            ComponentState::Error => self.palette.bg_primary,
            ComponentState::Warning => self.palette.bg_primary,
            ComponentState::Success => self.palette.bg_primary,
            ComponentState::Loading => self.palette.bg_secondary,
            ComponentState::Hover => self.palette.hover_bg,
        }
    }
}

/// Component state enumeration for color selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    Normal,
    Focused,
    Selected,
    Disabled,
    Error,
    Warning,
    Success,
    Loading,
    Hover,
}

impl Default for ComponentState {
    fn default() -> Self {
        ComponentState::Normal
    }
}

/// Color utility functions
pub mod utils {
    use super::*;
    
    /// Convert hex color string to ratatui Color
    pub fn hex_to_color(hex: &str) -> Result<Color, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Hex color must be 6 characters".to_string());
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| "Invalid blue component")?;
            
        Ok(Color::Rgb(r, g, b))
    }
    
    /// Convert ratatui Color to hex string
    pub fn color_to_hex(color: Color) -> String {
        match color {
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            _ => "#000000".to_string(), // Default fallback
        }
    }
    
    /// Blend two colors with given ratio (0.0 = first color, 1.0 = second color)
    pub fn blend_colors(color1: Color, color2: Color, ratio: f32) -> Color {
        let ratio = ratio.clamp(0.0, 1.0);
        match (color1, color2) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r = (r1 as f32 * (1.0 - ratio) + r2 as f32 * ratio) as u8;
                let g = (g1 as f32 * (1.0 - ratio) + g2 as f32 * ratio) as u8;
                let b = (b1 as f32 * (1.0 - ratio) + b2 as f32 * ratio) as u8;
                Color::Rgb(r, g, b)
            }
            _ => color1, // Fallback for non-RGB colors
        }
    }
    
    /// Lighten a color by given amount (0.0 = no change, 1.0 = white)
    pub fn lighten_color(color: Color, amount: f32) -> Color {
        blend_colors(color, Color::Rgb(255, 255, 255), amount)
    }
    
    /// Darken a color by given amount (0.0 = no change, 1.0 = black)
    pub fn darken_color(color: Color, amount: f32) -> Color {
        blend_colors(color, Color::Rgb(0, 0, 0), amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;
    
    #[test]
    fn test_color_palettes() {
        let light = ColorPalette::light();
        let dark = ColorPalette::dark();
        let high_contrast = ColorPalette::high_contrast();
        
        // Test that palettes have different backgrounds
        assert_ne!(light.bg_primary, dark.bg_primary);
        assert_ne!(dark.bg_primary, high_contrast.bg_primary);
        
        // Test that high contrast has extreme values
        assert_eq!(high_contrast.bg_primary, Color::Rgb(0, 0, 0));
        assert_eq!(high_contrast.fg_primary, Color::Rgb(255, 255, 255));
    }
    
    #[test]
    fn test_theme_colors() {
        let light = ThemeColors::light();
        let dark = ThemeColors::dark();
        
        // Test state colors
        assert_eq!(light.get_state_color(ComponentState::Normal), light.foreground);
        assert_eq!(light.get_state_color(ComponentState::Error), light.error);
        
        // Test background colors
        assert_eq!(dark.get_state_bg_color(ComponentState::Focused), dark.focus_bg);
        assert_eq!(dark.get_state_bg_color(ComponentState::Selected), dark.selected_bg);
    }
    
    #[test]
    fn test_hex_conversion() {
        let color = hex_to_color("#FF0000").unwrap();
        assert_eq!(color, Color::Rgb(255, 0, 0));
        
        let hex = color_to_hex(Color::Rgb(255, 0, 0));
        assert_eq!(hex, "#ff0000");
        
        // Test invalid hex
        assert!(hex_to_color("#invalid").is_err());
        assert!(hex_to_color("#FF").is_err());
    }
    
    #[test]
    fn test_color_blending() {
        let red = Color::Rgb(255, 0, 0);
        let blue = Color::Rgb(0, 0, 255);
        
        // Test blending
        let purple = blend_colors(red, blue, 0.5);
        if let Color::Rgb(r, g, b) = purple {
            assert_eq!(r, 127); // Mid-point between 255 and 0
            assert_eq!(g, 0);
            assert_eq!(b, 127); // Mid-point between 0 and 255
        }
        
        // Test lighten/darken
        let lighter = lighten_color(Color::Rgb(100, 100, 100), 0.5);
        let darker = darken_color(Color::Rgb(100, 100, 100), 0.5);
        
        if let (Color::Rgb(lr, lg, lb), Color::Rgb(dr, dg, db)) = (lighter, darker) {
            assert!(lr > 100 && lg > 100 && lb > 100); // Lighter
            assert!(dr < 100 && dg < 100 && db < 100); // Darker
        }
    }
} 