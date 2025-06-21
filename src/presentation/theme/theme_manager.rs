// Theme manager for runtime switching and persistence
// Central theme management with event system

#[cfg(feature = "new-components")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::{AppTheme, ThemeVariant};

/// Theme preference configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub struct ThemePreference {
    pub current_theme: String,
    pub auto_switch: bool,
    pub light_theme: String,
    pub dark_theme: String,
    pub high_contrast_mode: bool,
    pub custom_themes: Vec<String>,
    pub follow_system: bool,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self {
            current_theme: "dark".to_string(),
            auto_switch: false,
            light_theme: "light".to_string(),
            dark_theme: "dark".to_string(),
            high_contrast_mode: false,
            custom_themes: Vec::new(),
            follow_system: false,
        }
    }
}

/// Theme event enumeration for reactive updates
#[derive(Debug, Clone)]
pub enum ThemeEvent {
    ThemeChanged { 
        old_theme: String, 
        new_theme: String 
    },
    ThemeLoaded { 
        theme_name: String 
    },
    ThemeRegistered { 
        theme_name: String 
    },
    ThemeUnregistered { 
        theme_name: String 
    },
    PreferencesChanged,
    SystemThemeChanged { 
        is_dark: bool 
    },
    HighContrastToggled { 
        enabled: bool 
    },
}

/// Theme manager for centralized theme control
pub struct ThemeManager {
    current_theme: AppTheme,
    registered_themes: HashMap<String, AppTheme>,
    preferences: ThemePreference,
    config_path: Option<PathBuf>,
    event_listeners: Vec<Box<dyn Fn(&ThemeEvent) + Send + Sync>>,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let mut manager = Self {
            current_theme: AppTheme::dark(),
            registered_themes: HashMap::new(),
            preferences: ThemePreference::default(),
            config_path: None,
            event_listeners: Vec::new(),
        };
        
        // Register default themes
        manager.register_default_themes();
        manager
    }
    
    /// Create theme manager with config path
    pub fn with_config_path(config_path: PathBuf) -> Self {
        let mut manager = Self::new();
        manager.config_path = Some(config_path);
        manager.load_preferences().ok(); // Ignore errors on first load
        manager
    }
    
    /// Register default themes
    fn register_default_themes(&mut self) {
        self.registered_themes.insert("light".to_string(), AppTheme::light());
        self.registered_themes.insert("dark".to_string(), AppTheme::dark());
        self.registered_themes.insert("high_contrast".to_string(), AppTheme::high_contrast());
    }
    
    /// Get current theme
    pub fn current_theme(&self) -> &AppTheme {
        &self.current_theme
    }
    
    /// Get theme preferences
    pub fn preferences(&self) -> &ThemePreference {
        &self.preferences
    }
    
    /// Set theme by name
    pub fn set_theme(&mut self, theme_name: &str) -> Result<(), ThemeError> {
        let theme = self.registered_themes
            .get(theme_name)
            .ok_or_else(|| ThemeError::ThemeNotFound(theme_name.to_string()))?
            .clone();
        
        let old_theme = self.preferences.current_theme.clone();
        self.current_theme = theme;
        self.preferences.current_theme = theme_name.to_string();
        
        self.emit_event(ThemeEvent::ThemeChanged {
            old_theme,
            new_theme: theme_name.to_string(),
        });
        
        self.save_preferences().ok(); // Don't fail if save fails
        Ok(())
    }
    
    /// Switch to next available theme
    pub fn cycle_theme(&mut self) -> Result<(), ThemeError> {
        let theme_names: Vec<String> = self.registered_themes.keys().cloned().collect();
        if theme_names.is_empty() {
            return Err(ThemeError::NoThemesAvailable);
        }
        
        let current_index = theme_names
            .iter()
            .position(|name| name == &self.preferences.current_theme)
            .unwrap_or(0);
        
        let next_index = (current_index + 1) % theme_names.len();
        let next_theme = &theme_names[next_index];
        
        self.set_theme(next_theme)
    }
    
    /// Toggle between light and dark themes
    pub fn toggle_light_dark(&mut self) -> Result<(), ThemeError> {
        let current_variant = self.current_theme.variant;
        let target_theme = match current_variant {
            ThemeVariant::Light => &self.preferences.dark_theme,
            ThemeVariant::Dark => &self.preferences.light_theme,
            ThemeVariant::HighContrast => &self.preferences.dark_theme,
            ThemeVariant::Custom(_) => &self.preferences.light_theme,
        };
        
        let target_theme_name = target_theme.to_string();
        self.set_theme(&target_theme_name)
    }
    
    /// Toggle high contrast mode
    pub fn toggle_high_contrast(&mut self) -> Result<(), ThemeError> {
        let new_state = !self.preferences.high_contrast_mode;
        self.preferences.high_contrast_mode = new_state;
        
        let target_theme = if new_state {
            "high_contrast"
        } else {
            match self.current_theme.variant {
                ThemeVariant::HighContrast => &self.preferences.dark_theme,
                _ => &self.preferences.current_theme,
            }
        };
        
        self.emit_event(ThemeEvent::HighContrastToggled { enabled: new_state });
        let target_theme_name = target_theme.to_string();
        self.set_theme(&target_theme_name)
    }
    
    /// Register a custom theme
    pub fn register_theme(&mut self, name: String, theme: AppTheme) -> Result<(), ThemeError> {
        if self.registered_themes.contains_key(&name) {
            return Err(ThemeError::ThemeAlreadyExists(name));
        }
        
        self.registered_themes.insert(name.clone(), theme);
        self.preferences.custom_themes.push(name.clone());
        
        self.emit_event(ThemeEvent::ThemeRegistered { 
            theme_name: name 
        });
        
        self.save_preferences().ok();
        Ok(())
    }
    
    /// Unregister a theme
    pub fn unregister_theme(&mut self, name: &str) -> Result<(), ThemeError> {
        // Don't allow removing default themes
        if ["light", "dark", "high_contrast"].contains(&name) {
            return Err(ThemeError::CannotRemoveDefaultTheme(name.to_string()));
        }
        
        self.registered_themes.remove(name)
            .ok_or_else(|| ThemeError::ThemeNotFound(name.to_string()))?;
        
        self.preferences.custom_themes.retain(|t| t != name);
        
        // Switch to default theme if current theme is being removed
        if self.preferences.current_theme == name {
            self.set_theme("dark")?;
        }
        
        self.emit_event(ThemeEvent::ThemeUnregistered { 
            theme_name: name.to_string() 
        });
        
        self.save_preferences().ok();
        Ok(())
    }
    
    /// List all available themes
    pub fn list_themes(&self) -> Vec<(String, &AppTheme)> {
        self.registered_themes
            .iter()
            .map(|(name, theme)| (name.clone(), theme))
            .collect()
    }
    
    /// Check if theme exists
    pub fn has_theme(&self, name: &str) -> bool {
        self.registered_themes.contains_key(name)
    }
    
    /// Get theme by name
    pub fn get_theme(&self, name: &str) -> Option<&AppTheme> {
        self.registered_themes.get(name)
    }
    
    /// Update preferences
    pub fn update_preferences(&mut self, preferences: ThemePreference) -> Result<(), ThemeError> {
        self.preferences = preferences;
        
        // Apply current theme from preferences
        if self.has_theme(&self.preferences.current_theme) {
            self.set_theme(&self.preferences.current_theme.clone())?;
        }
        
        self.emit_event(ThemeEvent::PreferencesChanged);
        self.save_preferences().ok();
        Ok(())
    }

    /// Handle system theme change (for auto-switching)
    pub fn handle_system_theme_change(&mut self, is_dark: bool) -> Result<(), ThemeError> {
        if self.preferences.follow_system {
            let target_theme = if is_dark {
                &self.preferences.dark_theme
            } else {
                &self.preferences.light_theme
            };
            
            self.emit_event(ThemeEvent::SystemThemeChanged { is_dark });
            let target_theme_name = target_theme.to_string();
            self.set_theme(&target_theme_name)?;
        }
        
        Ok(())
    }

    /// Add event listener
    pub fn add_event_listener<F>(&mut self, listener: F)
    where
        F: Fn(&ThemeEvent) + Send + Sync + 'static,
    {
        self.event_listeners.push(Box::new(listener));
    }
    /// Emit theme event
    fn emit_event(&self, event: ThemeEvent) {
        for listener in &self.event_listeners {
            listener(&event);
        }
    }
    
    /// Save preferences to file
    fn save_preferences(&self) -> Result<(), ThemeError> {
        if let Some(config_path) = &self.config_path {
            let preferences_str = serde_json::to_string_pretty(&self.preferences)
                .map_err(|e| ThemeError::SerializationError(e.to_string()))?;
            
            std::fs::write(config_path, preferences_str)
                .map_err(|e| ThemeError::FileError(e.to_string()))?;
        }
        Ok(())
    }
    
    /// Load preferences from file
    fn load_preferences(&mut self) -> Result<(), ThemeError> {
        if let Some(config_path) = &self.config_path {
            if config_path.exists() {
                let preferences_str = std::fs::read_to_string(config_path)
                    .map_err(|e| ThemeError::FileError(e.to_string()))?;
                
                let preferences: ThemePreference = serde_json::from_str(&preferences_str)
                    .map_err(|e| ThemeError::SerializationError(e.to_string()))?;
                
                self.preferences = preferences;
                
                // Apply loaded theme
                if self.has_theme(&self.preferences.current_theme) {
                    let theme_name = self.preferences.current_theme.clone();
                    let theme = self.registered_themes[&theme_name].clone();
                    self.current_theme = theme;
                    
                    self.emit_event(ThemeEvent::ThemeLoaded { 
                        theme_name 
                    });
                }
            }
        }
        Ok(())
    }
    
    /// Export theme to JSON string (simplified without serialization)
    pub fn export_theme(&self, theme: &AppTheme) -> Result<String, ThemeError> {
        // For now, return a simple string representation instead of JSON
        Ok(format!(
            "{{\"name\":\"{}\",\"variant\":\"{}\",\"description\":\"{}\"}}",
            theme.name, theme.variant, theme.description
        ))
    }
    
    /// Import theme from JSON string (simplified without deserialization)
    pub fn import_theme(&mut self, theme_str: &str) -> Result<AppTheme, ThemeError> {
        // For now, just return a default theme since we can't deserialize
        // In a real implementation, this would parse the JSON and create an AppTheme
        if theme_str.contains("Dark") {
            Ok(AppTheme::dark())
        } else if theme_str.contains("High") {
            Ok(AppTheme::high_contrast())
        } else {
            Ok(AppTheme::light())
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Theme error enumeration
#[derive(Debug, Clone)]
pub enum ThemeError {
    ThemeNotFound(String),
    ThemeAlreadyExists(String),
    CannotRemoveDefaultTheme(String),
    NoThemesAvailable,
    FileError(String),
    SerializationError(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::ThemeNotFound(name) => write!(f, "Theme '{}' not found", name),
            ThemeError::ThemeAlreadyExists(name) => write!(f, "Theme '{}' already exists", name),
            ThemeError::CannotRemoveDefaultTheme(name) => {
                write!(f, "Cannot remove default theme '{}'", name)
            }
            ThemeError::NoThemesAvailable => write!(f, "No themes available"),
            ThemeError::FileError(msg) => write!(f, "File error: {}", msg),
            ThemeError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ThemeError {}

/// Utility functions for theme management
pub mod utils {
    use super::*;
    
    /// Detect system theme preference (placeholder - platform specific)
    pub fn detect_system_theme() -> Option<bool> {
        // This would need platform-specific implementation
        // For now, return None to indicate unknown
        None
    }
    
    /// Get theme config directory
    pub fn get_theme_config_dir() -> Option<PathBuf> {
        dirs::config_dir()
            .map(|dir| dir.join("semantic-release-tui").join("themes"))
    }
    
    /// Ensure theme config directory exists
    pub fn ensure_theme_config_dir() -> Result<PathBuf, std::io::Error> {
        let config_dir = get_theme_config_dir()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ))?;
        
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }
    
    /// Get default theme config file path
    pub fn get_theme_config_file() -> Option<PathBuf> {
        get_theme_config_dir()
            .map(|dir| dir.join("preferences.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current_theme().name, "Dark");
        assert!(manager.has_theme("light"));
        assert!(manager.has_theme("dark"));
        assert!(manager.has_theme("high_contrast"));
    }
    
    #[test]
    fn test_theme_switching() {
        let mut manager = ThemeManager::new();
        
        // Test setting theme
        assert!(manager.set_theme("light").is_ok());
        assert_eq!(manager.current_theme().name, "Light");
        assert_eq!(manager.preferences().current_theme, "light");
        
        // Test invalid theme
        assert!(manager.set_theme("nonexistent").is_err());
    }
    
    #[test]
    fn test_theme_cycling() {
        let mut manager = ThemeManager::new();
        let initial_theme = manager.preferences().current_theme.clone();
        
        assert!(manager.cycle_theme().is_ok());
        assert_ne!(manager.preferences().current_theme, initial_theme);
    }
    
    #[test]
    fn test_light_dark_toggle() {
        let mut manager = ThemeManager::new();
        
        // Start with dark theme
        manager.set_theme("dark").unwrap();
        assert_eq!(manager.current_theme().variant, ThemeVariant::Dark);
        
        // Toggle to light
        assert!(manager.toggle_light_dark().is_ok());
        assert_eq!(manager.current_theme().variant, ThemeVariant::Light);
        
        // Toggle back to dark
        assert!(manager.toggle_light_dark().is_ok());
        assert_eq!(manager.current_theme().variant, ThemeVariant::Dark);
    }
    
    #[test]
    fn test_high_contrast_toggle() {
        let mut manager = ThemeManager::new();
        
        assert!(!manager.preferences().high_contrast_mode);
        
        assert!(manager.toggle_high_contrast().is_ok());
        assert!(manager.preferences().high_contrast_mode);
        assert_eq!(manager.current_theme().variant, ThemeVariant::HighContrast);
        
        assert!(manager.toggle_high_contrast().is_ok());
        assert!(!manager.preferences().high_contrast_mode);
    }
    
    #[test]
    fn test_custom_theme_registration() {
        let mut manager = ThemeManager::new();
        let custom_theme = AppTheme::dark(); // Use dark as template
        
        // Register custom theme
        assert!(manager.register_theme("custom".to_string(), custom_theme).is_ok());
        assert!(manager.has_theme("custom"));
        
        // Try to register same name again
        let duplicate_theme = AppTheme::light();
        assert!(manager.register_theme("custom".to_string(), duplicate_theme).is_err());
        
        // Unregister custom theme
        assert!(manager.unregister_theme("custom").is_ok());
        assert!(!manager.has_theme("custom"));
        
        // Try to unregister default theme
        assert!(manager.unregister_theme("dark").is_err());
    }
    
    #[test]
    fn test_theme_listing() {
        let manager = ThemeManager::new();
        let themes = manager.list_themes();
        
        assert!(themes.len() >= 3); // At least light, dark, high_contrast
        
        let theme_names: Vec<String> = themes.iter().map(|(name, _)| name.clone()).collect();
        assert!(theme_names.contains(&"light".to_string()));
        assert!(theme_names.contains(&"dark".to_string()));
        assert!(theme_names.contains(&"high_contrast".to_string()));
    }
    
    #[test]
    fn test_preferences_persistence() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("theme_preferences.json");
        
        // Create manager with config path
        let mut manager = ThemeManager::with_config_path(config_path.clone());
        
        // Change theme and preferences
        manager.set_theme("light").unwrap();
        manager.preferences.auto_switch = true;
        
        // Force save and create new manager
        manager.save_preferences().unwrap();
        let new_manager = ThemeManager::with_config_path(config_path);
        
        // Check that preferences were loaded
        assert_eq!(new_manager.preferences().current_theme, "light");
        assert!(new_manager.preferences().auto_switch);
    }
    
    #[test]
    fn test_event_system() {
        let mut manager = ThemeManager::new();
        let mut events_received: Vec<ThemeEvent> = Vec::new();
        
        // Add event listener
        let events_ref = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events_ref.clone();
        
        manager.add_event_listener(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        });
        
        // Trigger theme change
        manager.set_theme("light").unwrap();
        
        // Check that event was received
        let events = events_ref.lock().unwrap();
        assert!(!events.is_empty());
        
        match &events[0] {
            ThemeEvent::ThemeChanged { old_theme, new_theme } => {
                assert_eq!(old_theme, "dark");
                assert_eq!(new_theme, "light");
            }
            _ => panic!("Expected ThemeChanged event"),
        }
    }
    
    #[test]
    fn test_system_theme_handling() {
        let mut manager = ThemeManager::new();
        manager.preferences.follow_system = true;
        
        // Simulate system dark mode
        assert!(manager.handle_system_theme_change(true).is_ok());
        assert_eq!(manager.current_theme().variant, ThemeVariant::Dark);
        
        // Simulate system light mode
        assert!(manager.handle_system_theme_change(false).is_ok());
        assert_eq!(manager.current_theme().variant, ThemeVariant::Light);
        
        // Disable system following
        manager.preferences.follow_system = false;
        let current_theme = manager.preferences.current_theme.clone();
        
        // Should not change when system changes
        assert!(manager.handle_system_theme_change(true).is_ok());
        assert_eq!(manager.preferences.current_theme, current_theme);
    }
} 