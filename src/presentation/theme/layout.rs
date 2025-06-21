// Layout and spacing constants for consistent positioning
// Comprehensive layout configuration for responsive design

#[cfg(feature = "new-components")]
use serde::{Deserialize, Serialize};

/// Spacing values for consistent padding and margins
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub struct Spacing {
    pub none: u16,
    pub xs: u16,    // Extra small
    pub sm: u16,    // Small
    pub md: u16,    // Medium (default)
    pub lg: u16,    // Large
    pub xl: u16,    // Extra large
    pub xxl: u16,   // Extra extra large
}

impl Spacing {
    /// Create default spacing values
    pub fn default() -> Self {
        Self {
            none: 0,
            xs: 1,
            sm: 2,
            md: 3,
            lg: 4,
            xl: 6,
            xxl: 8,
        }
    }
    
    /// Create compact spacing values for dense layouts
    pub fn compact() -> Self {
        Self {
            none: 0,
            xs: 1,
            sm: 1,
            md: 2,
            lg: 3,
            xl: 4,
            xxl: 6,
        }
    }
    
    /// Create comfortable spacing values for accessibility
    pub fn comfortable() -> Self {
        Self {
            none: 0,
            xs: 2,
            sm: 3,
            md: 4,
            lg: 6,
            xl: 8,
            xxl: 12,
        }
    }
    
    /// Get spacing value by size
    pub fn get(&self, size: SpacingSize) -> u16 {
        match size {
            SpacingSize::None => self.none,
            SpacingSize::XS => self.xs,
            SpacingSize::SM => self.sm,
            SpacingSize::MD => self.md,
            SpacingSize::LG => self.lg,
            SpacingSize::XL => self.xl,
            SpacingSize::XXL => self.xxl,
        }
    }
}

/// Spacing size enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum SpacingSize {
    None,
    XS,
    SM,
    MD,
    LG,
    XL,
    XXL,
}

impl Default for SpacingSize {
    fn default() -> Self {
        SpacingSize::MD
    }
}

/// Layout constants for consistent component sizing
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub struct LayoutConstants {
    // Component heights
    pub button_height: u16,
    pub input_height: u16,
    pub select_height: u16,
    pub header_height: u16,
    pub footer_height: u16,
    pub sidebar_width: u16,
    
    // Container dimensions
    pub dialog_min_width: u16,
    pub dialog_max_width: u16,
    pub dialog_min_height: u16,
    pub dialog_max_height: u16,
    
    // List and table dimensions
    pub list_item_height: u16,
    pub table_row_height: u16,
    pub table_header_height: u16,
    
    // Responsive breakpoints
    pub small_terminal_width: u16,
    pub medium_terminal_width: u16,
    pub large_terminal_width: u16,
    
    // Minimum dimensions for usability
    pub min_component_width: u16,
    pub min_component_height: u16,
}

impl LayoutConstants {
    /// Create default layout constants
    pub fn default() -> Self {
        Self {
            // Component heights
            button_height: 3,
            input_height: 3,
            select_height: 3,
            header_height: 3,
            footer_height: 2,
            sidebar_width: 25,
            
            // Container dimensions
            dialog_min_width: 40,
            dialog_max_width: 120,
            dialog_min_height: 10,
            dialog_max_height: 40,
            
            // List and table dimensions
            list_item_height: 1,
            table_row_height: 1,
            table_header_height: 2,
            
            // Responsive breakpoints
            small_terminal_width: 80,
            medium_terminal_width: 120,
            large_terminal_width: 160,
            
            // Minimum dimensions
            min_component_width: 20,
            min_component_height: 3,
        }
    }
    
    /// Create compact layout constants for smaller terminals
    pub fn compact() -> Self {
        Self {
            // Component heights - smaller for compact layout
            button_height: 1,
            input_height: 1,
            select_height: 1,
            header_height: 2,
            footer_height: 1,
            sidebar_width: 20,
            
            // Container dimensions - smaller for compact
            dialog_min_width: 30,
            dialog_max_width: 80,
            dialog_min_height: 8,
            dialog_max_height: 30,
            
            // List and table dimensions - same
            list_item_height: 1,
            table_row_height: 1,
            table_header_height: 1,
            
            // Responsive breakpoints - adjusted
            small_terminal_width: 60,
            medium_terminal_width: 100,
            large_terminal_width: 140,
            
            // Minimum dimensions - smaller
            min_component_width: 15,
            min_component_height: 1,
        }
    }
    
    /// Create accessibility-focused layout constants
    pub fn accessibility() -> Self {
        Self {
            // Component heights - larger for accessibility
            button_height: 4,
            input_height: 4,
            select_height: 4,
            header_height: 4,
            footer_height: 3,
            sidebar_width: 30,
            
            // Container dimensions - larger
            dialog_min_width: 50,
            dialog_max_width: 140,
            dialog_min_height: 12,
            dialog_max_height: 50,
            
            // List and table dimensions - larger
            list_item_height: 2,
            table_row_height: 2,
            table_header_height: 3,
            
            // Responsive breakpoints - more generous
            small_terminal_width: 100,
            medium_terminal_width: 140,
            large_terminal_width: 180,
            
            // Minimum dimensions - larger
            min_component_width: 25,
            min_component_height: 4,
        }
    }
    
    /// Get appropriate height for component type
    pub fn get_component_height(&self, component_type: ComponentType) -> u16 {
        match component_type {
            ComponentType::Button => self.button_height,
            ComponentType::Input => self.input_height,
            ComponentType::Select => self.select_height,
            ComponentType::Header => self.header_height,
            ComponentType::Footer => self.footer_height,
            ComponentType::ListItem => self.list_item_height,
            ComponentType::TableRow => self.table_row_height,
            ComponentType::TableHeader => self.table_header_height,
        }
    }
    
    /// Check if terminal size qualifies as small
    pub fn is_small_terminal(&self, width: u16) -> bool {
        width < self.small_terminal_width
    }
    
    /// Check if terminal size qualifies as medium
    pub fn is_medium_terminal(&self, width: u16) -> bool {
        width >= self.small_terminal_width && width < self.large_terminal_width
    }
    
    /// Check if terminal size qualifies as large
    pub fn is_large_terminal(&self, width: u16) -> bool {
        width >= self.large_terminal_width
    }
}

/// Component type for layout calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum ComponentType {
    Button,
    Input,
    Select,
    Header,
    Footer,
    ListItem,
    TableRow,
    TableHeader,
}

/// Layout theme that combines spacing and constants
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub struct LayoutTheme {
    pub spacing: Spacing,
    pub constants: LayoutConstants,
    pub layout_style: LayoutStyle,
}

/// Layout style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum LayoutStyle {
    Default,
    Compact,
    Comfortable,
    Accessibility,
}

impl LayoutTheme {
    /// Create default layout theme
    pub fn default() -> Self {
        Self {
            spacing: Spacing::default(),
            constants: LayoutConstants::default(),
            layout_style: LayoutStyle::Default,
        }
    }
    
    /// Create compact layout theme
    pub fn compact() -> Self {
        Self {
            spacing: Spacing::compact(),
            constants: LayoutConstants::compact(),
            layout_style: LayoutStyle::Compact,
        }
    }
    
    /// Create comfortable layout theme
    pub fn comfortable() -> Self {
        Self {
            spacing: Spacing::comfortable(),
            constants: LayoutConstants::default(),
            layout_style: LayoutStyle::Comfortable,
        }
    }
    
    /// Create accessibility layout theme
    pub fn accessibility() -> Self {
        Self {
            spacing: Spacing::comfortable(),
            constants: LayoutConstants::accessibility(),
            layout_style: LayoutStyle::Accessibility,
        }
    }
    
    /// Get margin for component sides
    pub fn get_margin(&self, side: MarginSide, size: SpacingSize) -> u16 {
        let base_spacing = self.spacing.get(size);
        match side {
            MarginSide::Top | MarginSide::Bottom => base_spacing,
            MarginSide::Left | MarginSide::Right => base_spacing,
            MarginSide::All => base_spacing,
        }
    }
    
    /// Get padding for component
    pub fn get_padding(&self, size: SpacingSize) -> u16 {
        self.spacing.get(size)
    }
    
    /// Calculate responsive dialog size
    pub fn get_dialog_size(&self, terminal_width: u16, terminal_height: u16) -> (u16, u16) {
        let width = if self.constants.is_small_terminal(terminal_width) {
            (terminal_width * 90 / 100).max(self.constants.dialog_min_width)
        } else if self.constants.is_medium_terminal(terminal_width) {
            (terminal_width * 80 / 100).max(self.constants.dialog_min_width)
        } else {
            (terminal_width * 70 / 100).max(self.constants.dialog_min_width)
        }.min(self.constants.dialog_max_width);
        
        let height = (terminal_height * 80 / 100)
            .max(self.constants.dialog_min_height)
            .min(self.constants.dialog_max_height);
        
        (width, height)
    }
    
    /// Calculate responsive sidebar width
    pub fn get_sidebar_width(&self, terminal_width: u16) -> u16 {
        if self.constants.is_small_terminal(terminal_width) {
            0 // Hide sidebar on small terminals
        } else if self.constants.is_medium_terminal(terminal_width) {
            (self.constants.sidebar_width * 80 / 100).max(15)
        } else {
            self.constants.sidebar_width
        }
    }
}

/// Margin side enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum MarginSide {
    Top,
    Right,
    Bottom,
    Left,
    All,
}

/// Utility functions for layout calculations
pub mod utils {
    use super::*;
    
    /// Calculate centered position for a component
    pub fn center_rect(area: ratatui::layout::Rect, width: u16, height: u16) -> ratatui::layout::Rect {
        let popup_layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length((area.height.saturating_sub(height)) / 2),
                ratatui::layout::Constraint::Length(height),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area);
        
        ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Length((area.width.saturating_sub(width)) / 2),
                ratatui::layout::Constraint::Length(width),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(popup_layout[1])[1]
    }
    
    /// Create responsive layout constraints based on terminal size
    pub fn responsive_constraints(
        terminal_width: u16,
        layout_theme: &LayoutTheme,
    ) -> Vec<ratatui::layout::Constraint> {
        if layout_theme.constants.is_small_terminal(terminal_width) {
            // Single column layout for small terminals
            vec![ratatui::layout::Constraint::Percentage(100)]
        } else if layout_theme.constants.is_medium_terminal(terminal_width) {
            // Two column layout for medium terminals
            vec![
                ratatui::layout::Constraint::Percentage(60),
                ratatui::layout::Constraint::Percentage(40),
            ]
        } else {
            // Three column layout for large terminals
            vec![
                ratatui::layout::Constraint::Percentage(25),
                ratatui::layout::Constraint::Percentage(50),
                ratatui::layout::Constraint::Percentage(25),
            ]
        }
    }
    
    /// Calculate optimal component distribution
    pub fn distribute_space(
        total_space: u16,
        component_count: usize,
        min_size: u16,
        spacing: u16,
    ) -> Vec<u16> {
        let total_spacing = spacing * (component_count.saturating_sub(1)) as u16;
        let available_space = total_space.saturating_sub(total_spacing);
        let base_size = available_space / component_count as u16;
        
        if base_size < min_size {
            // Not enough space, use minimum sizes
            vec![min_size; component_count]
        } else {
            // Distribute evenly
            let mut sizes = vec![base_size; component_count];
            let remainder = available_space % component_count as u16;
            
            // Distribute remainder to first components
            for i in 0..remainder as usize {
                if i < sizes.len() {
                    sizes[i] += 1;
                }
            }
            
            sizes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;
    
    #[test]
    fn test_spacing() {
        let spacing = Spacing::default();
        assert_eq!(spacing.get(SpacingSize::None), 0);
        assert_eq!(spacing.get(SpacingSize::MD), 3);
        assert_eq!(spacing.get(SpacingSize::XL), 6);
        
        let compact = Spacing::compact();
        assert!(compact.get(SpacingSize::MD) < spacing.get(SpacingSize::MD));
        
        let comfortable = Spacing::comfortable();
        assert!(comfortable.get(SpacingSize::MD) > spacing.get(SpacingSize::MD));
    }
    
    #[test]
    fn test_layout_constants() {
        let constants = LayoutConstants::default();
        
        assert_eq!(constants.get_component_height(ComponentType::Button), 3);
        assert_eq!(constants.get_component_height(ComponentType::Input), 3);
        
        assert!(constants.is_small_terminal(70));
        assert!(constants.is_medium_terminal(100));
        assert!(constants.is_large_terminal(180));
    }
    
    #[test]
    fn test_layout_theme() {
        let theme = LayoutTheme::default();
        
        assert_eq!(theme.get_padding(SpacingSize::MD), 3);
        assert_eq!(theme.get_margin(MarginSide::Top, SpacingSize::SM), 2);
        
        // Test responsive dialog sizing
        let (width, height) = theme.get_dialog_size(100, 30);
        assert!(width >= theme.constants.dialog_min_width);
        assert!(width <= theme.constants.dialog_max_width);
        assert!(height >= theme.constants.dialog_min_height);
        assert!(height <= theme.constants.dialog_max_height);
        
        // Test responsive sidebar
        let sidebar_width = theme.get_sidebar_width(70); // Small terminal
        assert_eq!(sidebar_width, 0); // Should hide sidebar
        
        let sidebar_width = theme.get_sidebar_width(120); // Medium terminal
        assert!(sidebar_width > 0 && sidebar_width < theme.constants.sidebar_width);
    }
    
    #[test]
    fn test_responsive_constraints() {
        let theme = LayoutTheme::default();
        
        let small_constraints = responsive_constraints(70, &theme);
        assert_eq!(small_constraints.len(), 1); // Single column
        
        let medium_constraints = responsive_constraints(100, &theme);
        assert_eq!(medium_constraints.len(), 2); // Two columns
        
        let large_constraints = responsive_constraints(180, &theme);
        assert_eq!(large_constraints.len(), 3); // Three columns
    }
    
    #[test]
    fn test_space_distribution() {
        let sizes = distribute_space(100, 3, 10, 2);
        assert_eq!(sizes.len(), 3);
        
        // Total should account for spacing
        let total_used: u16 = sizes.iter().sum::<u16>() + 2 * 2; // 2 spaces between 3 components
        assert!(total_used <= 100);
        
        // All sizes should be at least minimum
        assert!(sizes.iter().all(|&size| size >= 10));
    }
    
    #[test]
    fn test_center_rect() {
        let area = ratatui::layout::Rect::new(0, 0, 100, 50);
        let centered = center_rect(area, 60, 30);
        
        // Should be centered
        assert_eq!(centered.x, 20); // (100 - 60) / 2
        assert_eq!(centered.y, 10); // (50 - 30) / 2
        assert_eq!(centered.width, 60);
        assert_eq!(centered.height, 30);
    }
} 