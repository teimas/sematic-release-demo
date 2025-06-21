// Text styles, borders, and emphasis for the theme system
// Comprehensive styling configuration for consistent visual design

use ratatui::style::{Modifier, Style, Color};
use ratatui::widgets::BorderType;

/// Text style definitions with modifiers
#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub modifiers: Modifier,
    pub underline_color: Option<ratatui::style::Color>,
}

impl TextStyle {
    /// Create a normal text style
    pub fn normal() -> Self {
        Self {
            modifiers: Modifier::empty(),
            underline_color: None,
        }
    }
    
    /// Create a bold text style
    pub fn bold() -> Self {
        Self {
            modifiers: Modifier::BOLD,
            underline_color: None,
        }
    }
    
    /// Create an italic text style
    pub fn italic() -> Self {
        Self {
            modifiers: Modifier::ITALIC,
            underline_color: None,
        }
    }
    
    /// Create an underlined text style
    pub fn underlined() -> Self {
        Self {
            modifiers: Modifier::UNDERLINED,
            underline_color: None,
        }
    }
    
    /// Create a dim text style
    pub fn dim() -> Self {
        Self {
            modifiers: Modifier::DIM,
            underline_color: None,
        }
    }
    
    /// Create a reversed text style (inverted colors)
    pub fn reversed() -> Self {
        Self {
            modifiers: Modifier::REVERSED,
            underline_color: None,
        }
    }
    
    /// Create a slow blinking text style
    pub fn slow_blink() -> Self {
        Self {
            modifiers: Modifier::SLOW_BLINK,
            underline_color: None,
        }
    }
    
    /// Create a rapid blinking text style
    pub fn rapid_blink() -> Self {
        Self {
            modifiers: Modifier::RAPID_BLINK,
            underline_color: None,
        }
    }
    
    /// Create a hidden text style
    pub fn hidden() -> Self {
        Self {
            modifiers: Modifier::HIDDEN,
            underline_color: None,
        }
    }
    
    /// Create a bold italic style
    pub fn bold_italic() -> Self {
        Self {
            modifiers: Modifier::BOLD | Modifier::ITALIC,
            underline_color: None,
        }
    }
    
    /// Create a bold underlined style
    pub fn bold_underlined() -> Self {
        Self {
            modifiers: Modifier::BOLD | Modifier::UNDERLINED,
            underline_color: None,
        }
    }
    
    /// Create a muted style (dim)
    pub fn muted() -> Self {
        Self {
            modifiers: Modifier::DIM,
            underline_color: None,
        }
    }
    
    /// Create an emphasized style (bold)
    pub fn emphasized() -> Self {
        Self {
            modifiers: Modifier::BOLD,
            underline_color: None,
        }
    }
    
    /// Add underline color to this style
    pub fn with_underline_color(mut self, color: ratatui::style::Color) -> Self {
        self.underline_color = Some(color);
        self
    }
    
    /// Convert to ratatui Style with optional colors
    pub fn to_style(&self, fg: Option<ratatui::style::Color>, bg: Option<ratatui::style::Color>) -> Style {
        let mut style = Style::default().add_modifier(self.modifiers);
        
        if let Some(fg_color) = fg {
            style = style.fg(fg_color);
        }
        
        if let Some(bg_color) = bg {
            style = style.bg(bg_color);
        }
        
        if let Some(underline_color) = self.underline_color {
            style = style.underline_color(underline_color);
        }
        
        style
    }
}

/// Border style definitions
#[derive(Debug, Clone, Copy)]
pub struct BorderStyle {
    pub border_type: ratatui::widgets::BorderType,
    pub thickness: BorderThickness,
    pub corner_style: CornerStyle,
}

/// Border thickness options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderThickness {
    None,
    Thin,
    Thick,
    Double,
    Rounded,
}

/// Corner style options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerStyle {
    Square,
    Rounded,
}

impl BorderStyle {
    /// Create a thin border style
    pub fn thin() -> Self {
        Self {
            border_type: ratatui::widgets::BorderType::Plain,
            thickness: BorderThickness::Thin,
            corner_style: CornerStyle::Square,
        }
    }
    
    /// Create a thick border style
    pub fn thick() -> Self {
        Self {
            border_type: ratatui::widgets::BorderType::Thick,
            thickness: BorderThickness::Thick,
            corner_style: CornerStyle::Square,
        }
    }
    
    /// Create a double border style
    pub fn double() -> Self {
        Self {
            border_type: ratatui::widgets::BorderType::Double,
            thickness: BorderThickness::Double,
            corner_style: CornerStyle::Square,
        }
    }
    
    /// Create a rounded border style
    pub fn rounded() -> Self {
        Self {
            border_type: ratatui::widgets::BorderType::Rounded,
            thickness: BorderThickness::Rounded,
            corner_style: CornerStyle::Rounded,
        }
    }
    
    /// Create no border
    pub fn none() -> Self {
        Self {
            border_type: ratatui::widgets::BorderType::Plain,
            thickness: BorderThickness::None,
            corner_style: CornerStyle::Square,
        }
    }
    
    /// Get the ratatui BorderType
    pub fn to_border_type(&self) -> Option<ratatui::widgets::BorderType> {
        match self.thickness {
            BorderThickness::None => None,
            _ => Some(self.border_type),
        }
    }
}

/// Highlight style for interactive elements
#[derive(Debug, Clone, Copy)]
pub struct HighlightStyle {
    pub focused: TextStyle,
    pub selected: TextStyle,
    pub hover: TextStyle,
    pub active: TextStyle,
    pub disabled: TextStyle,
}

impl HighlightStyle {
    /// Create light theme highlight styles
    pub fn light() -> Self {
        Self {
            focused: TextStyle::bold(),
            selected: TextStyle::bold().with_underline_color(ratatui::style::Color::Blue),
            hover: TextStyle::underlined(),
            active: TextStyle::bold_italic(),
            disabled: TextStyle::dim(),
        }
    }
    
    /// Create dark theme highlight styles
    pub fn dark() -> Self {
        Self {
            focused: TextStyle::bold(),
            selected: TextStyle::bold().with_underline_color(ratatui::style::Color::Cyan),
            hover: TextStyle::underlined(),
            active: TextStyle::bold_italic(),
            disabled: TextStyle::dim(),
        }
    }
    
    /// Create high contrast highlight styles
    pub fn high_contrast() -> Self {
        Self {
            focused: TextStyle::bold_underlined(),
            selected: TextStyle::reversed(),
            hover: TextStyle::bold(),
            active: TextStyle::bold_italic(),
            disabled: TextStyle::dim(),
        }
    }
}

/// Complete text style configuration
#[derive(Debug, Clone, Copy)]
pub struct TextStyleSet {
    pub normal: TextStyle,
    pub bold: TextStyle,
    pub italic: TextStyle,
    pub muted: TextStyle,
    pub emphasized: TextStyle,
    pub title: TextStyle,
    pub subtitle: TextStyle,
    pub caption: TextStyle,
    pub code: TextStyle,
    pub link: TextStyle,
    pub error: TextStyle,
    pub warning: TextStyle,
    pub success: TextStyle,
    pub info: TextStyle,
}

impl TextStyleSet {
    /// Create light theme text styles
    pub fn light() -> Self {
        Self {
            normal: TextStyle::normal(),
            bold: TextStyle::bold(),
            italic: TextStyle::italic(),
            muted: TextStyle::dim(),
            emphasized: TextStyle::bold(),
            title: TextStyle::bold(),
            subtitle: TextStyle::normal(),
            caption: TextStyle::dim(),
            code: TextStyle::normal(), // Could add specific styling
            link: TextStyle::underlined(),
            error: TextStyle::bold(),
            warning: TextStyle::bold(),
            success: TextStyle::bold(),
            info: TextStyle::normal(),
        }
    }
    
    /// Create dark theme text styles
    pub fn dark() -> Self {
        Self {
            normal: TextStyle::normal(),
            bold: TextStyle::bold(),
            italic: TextStyle::italic(),
            muted: TextStyle::dim(),
            emphasized: TextStyle::bold(),
            title: TextStyle::bold(),
            subtitle: TextStyle::normal(),
            caption: TextStyle::dim(),
            code: TextStyle::normal(), // Could add specific styling
            link: TextStyle::underlined(),
            error: TextStyle::bold(),
            warning: TextStyle::bold(),
            success: TextStyle::bold(),
            info: TextStyle::normal(),
        }
    }
    
    /// Create high contrast text styles
    pub fn high_contrast() -> Self {
        Self {
            normal: TextStyle::normal(),
            bold: TextStyle::bold(),
            italic: TextStyle::italic(),
            muted: TextStyle::dim(),
            emphasized: TextStyle::bold_underlined(),
            title: TextStyle::bold_underlined(),
            subtitle: TextStyle::bold(),
            caption: TextStyle::normal(),
            code: TextStyle::reversed(), // High contrast for code
            link: TextStyle::bold_underlined(),
            error: TextStyle::bold_underlined(),
            warning: TextStyle::bold_underlined(),
            success: TextStyle::bold_underlined(),
            info: TextStyle::bold(),
        }
    }
}

/// Border style configuration
#[derive(Debug, Clone, Copy)]
pub struct BorderStyleSet {
    pub default: BorderStyle,
    pub focus: BorderStyle,
    pub error: BorderStyle,
    pub success: BorderStyle,
    pub warning: BorderStyle,
    pub info: BorderStyle,
    pub subtle: BorderStyle,
    pub prominent: BorderStyle,
}

impl BorderStyleSet {
    /// Create light theme border styles
    pub fn light() -> Self {
        Self {
            default: BorderStyle::thin(),
            focus: BorderStyle::thick(),
            error: BorderStyle::thick(),
            success: BorderStyle::thick(),
            warning: BorderStyle::thick(),
            info: BorderStyle::thin(),
            subtle: BorderStyle::thin(),
            prominent: BorderStyle::double(),
        }
    }
    
    /// Create dark theme border styles
    pub fn dark() -> Self {
        Self {
            default: BorderStyle::thin(),
            focus: BorderStyle::thick(),
            error: BorderStyle::thick(),
            success: BorderStyle::thick(),
            warning: BorderStyle::thick(),
            info: BorderStyle::thin(),
            subtle: BorderStyle::thin(),
            prominent: BorderStyle::double(),
        }
    }
    
    /// Create high contrast border styles
    pub fn high_contrast() -> Self {
        Self {
            default: BorderStyle::thick(),
            focus: BorderStyle::double(),
            error: BorderStyle::double(),
            success: BorderStyle::double(),
            warning: BorderStyle::double(),
            info: BorderStyle::thick(),
            subtle: BorderStyle::thin(),
            prominent: BorderStyle::double(),
        }
    }
}

/// Complete style set combining all style types
#[derive(Debug, Clone, Copy)]
pub struct StyleSet {
    pub text: TextStyleSet,
    pub border: BorderStyleSet,
    pub highlight: HighlightStyle,
}

impl StyleSet {
    /// Create light theme style set
    pub fn light() -> Self {
        Self {
            text: TextStyleSet::light(),
            border: BorderStyleSet::light(),
            highlight: HighlightStyle::light(),
        }
    }
    
    /// Create dark theme style set
    pub fn dark() -> Self {
        Self {
            text: TextStyleSet::dark(),
            border: BorderStyleSet::dark(),
            highlight: HighlightStyle::dark(),
        }
    }
    
    /// Create high contrast style set
    pub fn high_contrast() -> Self {
        Self {
            text: TextStyleSet::high_contrast(),
            border: BorderStyleSet::high_contrast(),
            highlight: HighlightStyle::high_contrast(),
        }
    }
    
    /// Get text style by semantic meaning
    pub fn get_text_style(&self, style_type: TextStyleType) -> TextStyle {
        match style_type {
            TextStyleType::Normal => self.text.normal,
            TextStyleType::Bold => self.text.bold,
            TextStyleType::Italic => self.text.italic,
            TextStyleType::Muted => self.text.muted,
            TextStyleType::Emphasized => self.text.emphasized,
            TextStyleType::Title => self.text.title,
            TextStyleType::Subtitle => self.text.subtitle,
            TextStyleType::Caption => self.text.caption,
            TextStyleType::Code => self.text.code,
            TextStyleType::Link => self.text.link,
            TextStyleType::Error => self.text.error,
            TextStyleType::Warning => self.text.warning,
            TextStyleType::Success => self.text.success,
            TextStyleType::Info => self.text.info,
        }
    }
    
    /// Get border style by semantic meaning
    pub fn get_border_style(&self, border_type: BorderStyleType) -> BorderStyle {
        match border_type {
            BorderStyleType::Default => self.border.default,
            BorderStyleType::Focus => self.border.focus,
            BorderStyleType::Error => self.border.error,
            BorderStyleType::Success => self.border.success,
            BorderStyleType::Warning => self.border.warning,
            BorderStyleType::Info => self.border.info,
            BorderStyleType::Subtle => self.border.subtle,
            BorderStyleType::Prominent => self.border.prominent,
        }
    }
    
    /// Get highlight style by interaction state
    pub fn get_highlight_style(&self, state: HighlightState) -> TextStyle {
        match state {
            HighlightState::Focused => self.highlight.focused,
            HighlightState::Selected => self.highlight.selected,
            HighlightState::Hover => self.highlight.hover,
            HighlightState::Active => self.highlight.active,
            HighlightState::Disabled => self.highlight.disabled,
        }
    }
}

/// Text style type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStyleType {
    Normal,
    Bold,
    Italic,
    Muted,
    Emphasized,
    Title,
    Subtitle,
    Caption,
    Code,
    Link,
    Error,
    Warning,
    Success,
    Info,
}

/// Border style type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyleType {
    Default,
    Focus,
    Error,
    Success,
    Warning,
    Info,
    Subtle,
    Prominent,
}

/// Highlight state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightState {
    Focused,
    Selected,
    Hover,
    Active,
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_styles() {
        let normal = TextStyle::normal();
        assert_eq!(normal.modifiers, Modifier::empty());
        
        let bold = TextStyle::bold();
        assert!(bold.modifiers.contains(Modifier::BOLD));
        
        let bold_italic = TextStyle::bold_italic();
        assert!(bold_italic.modifiers.contains(Modifier::BOLD));
        assert!(bold_italic.modifiers.contains(Modifier::ITALIC));
    }
    
    #[test]
    fn test_border_styles() {
        let thin = BorderStyle::thin();
        assert_eq!(thin.thickness, BorderThickness::Thin);
        assert_eq!(thin.corner_style, CornerStyle::Square);
        
        let rounded = BorderStyle::rounded();
        assert_eq!(rounded.thickness, BorderThickness::Rounded);
        assert_eq!(rounded.corner_style, CornerStyle::Rounded);
        
        let none = BorderStyle::none();
        assert_eq!(none.thickness, BorderThickness::None);
        assert!(none.to_border_type().is_none());
    }
    
    #[test]
    fn test_style_sets() {
        let light = StyleSet::light();
        let dark = StyleSet::dark();
        let high_contrast = StyleSet::high_contrast();
        
        // Test that different themes have different configurations
        // High contrast should use more prominent styles
        assert_eq!(high_contrast.text.emphasized.modifiers, Modifier::BOLD | Modifier::UNDERLINED);
        assert_eq!(light.text.emphasized.modifiers, Modifier::BOLD);
        
        // Test semantic style retrieval
        let title_style = light.get_text_style(TextStyleType::Title);
        assert!(title_style.modifiers.contains(Modifier::BOLD));
        
        let focus_border = dark.get_border_style(BorderStyleType::Focus);
        assert_eq!(focus_border.thickness, BorderThickness::Thick);
    }
    
    #[test]
    fn test_highlight_styles() {
        let light_highlight = HighlightStyle::light();
        let dark_highlight = HighlightStyle::dark();
        let hc_highlight = HighlightStyle::high_contrast();
        
        // Test that high contrast uses more prominent styles
        assert_eq!(hc_highlight.selected.modifiers, Modifier::REVERSED);
        assert!(light_highlight.selected.modifiers.contains(Modifier::BOLD));
        
        // Test that disabled state uses appropriate styling
        assert_eq!(light_highlight.disabled.modifiers, Modifier::DIM);
        assert_eq!(hc_highlight.disabled.modifiers, Modifier::DIM);
    }
    
    #[test]
    fn test_style_conversion() {
        let text_style = TextStyle::bold();
        let ratatui_style = text_style.to_style(
            Some(ratatui::style::Color::Red),
            Some(ratatui::style::Color::Blue)
        );
        
        assert_eq!(ratatui_style.fg, Some(ratatui::style::Color::Red));
        assert_eq!(ratatui_style.bg, Some(ratatui::style::Color::Blue));
        assert!(ratatui_style.add_modifier.contains(Modifier::BOLD));
    }
} 