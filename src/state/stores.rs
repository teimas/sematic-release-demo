//! State Stores
//! 
//! This module defines all application state structures using typed-builder
//! for safe construction and derive_more for reduced boilerplate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use derive_more::Display;
use typed_builder::TypedBuilder;
use chrono::{DateTime, Utc};

/// Application-level state that tracks global application behavior
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AppState {
    /// Current application mode/state
    #[builder(default = AppMode::Normal)]
    pub mode: AppMode,
    
    /// Currently active screen
    #[builder(default = Screen::Main)]
    pub current_screen: Screen,
    
    /// Global loading state
    #[builder(default = false)]
    pub is_loading: bool,
    
    /// Global loading message
    #[builder(default = "Ready".to_string())]
    pub loading_message: String,
    
    /// Global status message
    #[builder(default = "Ready".to_string())]
    pub status_message: String,
    
    /// Application should quit flag
    #[builder(default = false)]
    pub should_quit: bool,
    
    /// Application configuration state
    #[builder(default)]
    pub config_dirty: bool,
    
    /// Application version and build info
    #[builder(default)]
    pub app_info: AppInfo,
    
    /// Performance metrics
    #[builder(default)]
    pub performance: PerformanceMetrics,
}

impl Default for AppState {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Application modes/states
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Hash)]
pub enum AppMode {
    #[display(fmt = "normal")]
    Normal,
    #[display(fmt = "loading")]
    Loading,
    #[display(fmt = "error")]
    Error,
    #[display(fmt = "confirming")]
    Confirming,
    #[display(fmt = "background_processing")]
    BackgroundProcessing,
}

/// Application screens
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Hash)]
pub enum Screen {
    #[display(fmt = "main")]
    Main,
    #[display(fmt = "config")]
    Config,
    #[display(fmt = "commit")]
    Commit,
    #[display(fmt = "commit_preview")]
    CommitPreview,
    #[display(fmt = "release_notes")]
    ReleaseNotes,
    #[display(fmt = "semantic_release")]
    SemanticRelease,
    #[display(fmt = "task_search")]
    TaskSearch,
    #[display(fmt = "help")]
    Help,
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AppInfo {
    #[builder(default = env!("CARGO_PKG_VERSION").to_string())]
    pub version: String,
    
    #[builder(default = env!("CARGO_PKG_NAME").to_string())]
    pub name: String,
    
    #[builder(default = Utc::now())]
    pub started_at: DateTime<Utc>,
    
    #[builder(default = 0)]
    pub session_operations: u64,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Performance tracking metrics
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct PerformanceMetrics {
    #[builder(default = 0)]
    pub render_count: u64,
    
    #[builder(default = 0.0)]
    pub average_render_time_ms: f64,
    
    #[builder(default = 0)]
    pub state_updates: u64,
    
    #[builder(default = 0)]
    pub background_operations: u64,
    
    #[builder(default = Utc::now())]
    pub last_updated: DateTime<Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// UI-specific state for managing interface elements
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct UiState {
    /// Current input mode
    #[builder(default = InputMode::Normal)]
    pub input_mode: InputMode,
    
    /// Currently focused component
    #[builder(default = "main_menu".to_string())]
    pub focused_component: String,
    
    /// Form field values
    #[builder(default)]
    pub form_fields: HashMap<String, String>,
    
    /// Current error being displayed
    #[builder(default)]
    pub current_error: Option<UiError>,
    
    /// UI theme and styling
    #[builder(default)]
    pub theme: UiTheme,
    
    /// Layout and positioning
    #[builder(default)]
    pub layout: UiLayout,
    
    /// Animation and visual effects state
    #[builder(default)]
    pub animations: AnimationState,
}

impl Default for UiState {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Input modes for the UI
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq)]
pub enum InputMode {
    #[display(fmt = "normal")]
    Normal,
    #[display(fmt = "editing")]
    Editing,
    #[display(fmt = "navigating")]
    Navigating,
    #[display(fmt = "selecting")]
    Selecting,
}

/// UI error information
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct UiError {
    pub message: String,
    #[builder(default)]
    pub context: Option<String>,
    #[builder(default = Utc::now())]
    pub timestamp: DateTime<Utc>,
    #[builder(default = UiErrorSeverity::Error)]
    pub severity: UiErrorSeverity,
}

/// UI error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum UiErrorSeverity {
    #[display(fmt = "info")]
    Info,
    #[display(fmt = "warning")]
    Warning,
    #[display(fmt = "error")]
    Error,
    #[display(fmt = "critical")]
    Critical,
}

/// UI theme and styling configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct UiTheme {
    #[builder(default = "Dark".to_string())]
    pub name: String,
    
    #[builder(default = true)]
    pub dark_mode: bool,
    
    #[builder(default = "#00FF00".to_string())]
    pub accent_color: String,
    
    #[builder(default = 1.0)]
    pub animation_speed: f32,
    
    /// Enable high contrast mode for accessibility
    #[builder(default = false)]
    pub high_contrast: bool,
    
    /// User preferences for theme persistence
    #[builder(default)]
    pub user_preferences: ThemeUserPreferences,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// User preferences for theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct ThemeUserPreferences {
    /// Auto-switch theme based on system preference
    #[builder(default = false)]
    pub auto_detect_system_theme: bool,
    
    /// Preferred theme for light mode
    #[builder(default = "Light".to_string())]
    pub light_theme_name: String,
    
    /// Preferred theme for dark mode
    #[builder(default = "Dark".to_string())]
    pub dark_theme_name: String,
    
    /// Custom theme configurations
    #[builder(default)]
    pub custom_themes: HashMap<String, String>, // theme_name -> serialized_theme
    
    /// Theme switching hotkey preference
    #[builder(default = "Ctrl+T".to_string())]
    pub theme_hotkey: String,
}

impl Default for ThemeUserPreferences {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// UI layout configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct UiLayout {
    #[builder(default = 80)]
    pub terminal_width: u16,
    
    #[builder(default = 24)]
    pub terminal_height: u16,
    
    #[builder(default = false)]
    pub sidebar_visible: bool,
    
    #[builder(default = 0)]
    pub scroll_offset: usize,
}

impl Default for UiLayout {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Animation state tracking
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AnimationState {
    #[builder(default = 0)]
    pub frame_count: u64,
    
    #[builder(default = 0)]
    pub current_frame: usize,
    
    #[builder(default = false)]
    pub animating: bool,
    
    #[builder(default)]
    pub active_animations: Vec<String>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Git repository and operation state
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct GitState {
    /// Current repository path
    #[builder(default = ".".to_string())]
    pub repository_path: String,
    
    /// Current branch name
    #[builder(default)]
    pub current_branch: Option<String>,
    
    /// Git status summary
    #[builder(default)]
    pub status: GitStatus,
    
    /// Recent commits
    #[builder(default)]
    pub recent_commits: Vec<GitCommit>,
    
    /// Modified files
    #[builder(default)]
    pub modified_files: Vec<String>,
    
    /// Staged files
    #[builder(default)]
    pub staged_files: Vec<String>,
    
    /// Current operation being performed
    #[builder(default)]
    pub current_operation: Option<GitOperation>,
}

impl Default for GitState {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Git repository status
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct GitStatus {
    #[builder(default = false)]
    pub is_dirty: bool,
    
    #[builder(default = 0)]
    pub modified_count: usize,
    
    #[builder(default = 0)]
    pub staged_count: usize,
    
    #[builder(default = 0)]
    pub untracked_count: usize,
    
    #[builder(default = false)]
    pub has_conflicts: bool,
}

impl Default for GitStatus {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    #[builder(default)]
    pub files_changed: Vec<String>,
}

/// Git operation in progress
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct GitOperation {
    pub operation_type: String,
    pub progress: f32,
    pub message: String,
    #[builder(default = Utc::now())]
    pub started_at: DateTime<Utc>,
}

/// Task management state
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct TaskState {
    /// Tasks from different systems
    #[builder(default)]
    pub monday_tasks: Vec<Task>,
    
    #[builder(default)]
    pub jira_tasks: Vec<Task>,
    
    /// Currently selected tasks
    #[builder(default)]
    pub selected_task_ids: Vec<String>,
    
    /// Task synchronization status
    #[builder(default)]
    pub sync_status: SyncStatus,
    
    /// Active task filters
    #[builder(default)]
    pub filters: TaskFilters,
    
    /// Task search query
    #[builder(default)]
    pub search_query: String,
    
    /// Currently focused task index
    #[builder(default = 0)]
    pub focused_index: usize,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Task information
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub source: String, // "monday", "jira", etc.
    #[builder(default)]
    pub assignee: Option<String>,
    #[builder(default)]
    pub labels: Vec<String>,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Task synchronization status
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct SyncStatus {
    #[builder(default = false)]
    pub is_syncing: bool,
    
    #[builder(default)]
    pub last_sync: Option<DateTime<Utc>>,
    
    #[builder(default)]
    pub sync_errors: Vec<String>,
    
    #[builder(default = 0)]
    pub synced_count: usize,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Task filtering options
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct TaskFilters {
    #[builder(default)]
    pub status_filter: Option<String>,
    
    #[builder(default)]
    pub assignee_filter: Option<String>,
    
    #[builder(default)]
    pub source_filter: Option<String>,
    
    #[builder(default = false)]
    pub show_completed: bool,
}

impl Default for TaskFilters {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// AI analysis and operation state
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AiState {
    /// Current AI provider
    #[builder(default = "gemini".to_string())]
    pub current_provider: String,
    
    /// Available AI providers
    #[builder(default)]
    pub available_providers: Vec<String>,
    
    /// Current AI analysis operation
    #[builder(default)]
    pub current_analysis: Option<AiAnalysis>,
    
    /// Recent AI results
    #[builder(default)]
    pub recent_results: Vec<AiResult>,
    
    /// AI provider configuration
    #[builder(default)]
    pub provider_config: AiProviderConfig,
    
    /// Usage statistics
    #[builder(default)]
    pub usage_stats: AiUsageStats,
}

impl Default for AiState {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// AI analysis operation
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AiAnalysis {
    pub id: String,
    pub analysis_type: String,
    pub progress: f32,
    pub message: String,
    #[builder(default = Utc::now())]
    pub started_at: DateTime<Utc>,
    #[builder(default)]
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// AI analysis result
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AiResult {
    pub id: String,
    pub analysis_type: String,
    pub result_data: serde_json::Value,
    pub confidence: f32,
    #[builder(default = Utc::now())]
    pub completed_at: DateTime<Utc>,
    #[builder(default)]
    pub metadata: HashMap<String, String>,
}

/// AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AiProviderConfig {
    #[builder(default = 0.7)]
    pub temperature: f32,
    
    #[builder(default = 1000)]
    pub max_tokens: u32,
    
    #[builder(default = "gpt-4".to_string())]
    pub model: String,
    
    #[builder(default = 30)]
    pub timeout_seconds: u64,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// AI usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AiUsageStats {
    #[builder(default = 0)]
    pub total_requests: u64,
    
    #[builder(default = 0)]
    pub successful_requests: u64,
    
    #[builder(default = 0)]
    pub failed_requests: u64,
    
    #[builder(default = 0)]
    pub total_tokens_used: u64,
    
    #[builder(default = 0.0)]
    pub average_response_time_ms: f64,
}

impl Default for AiUsageStats {
    fn default() -> Self {
        Self::builder().build()
    }
}

// Implementation for converting between UiTheme and AppTheme
#[cfg(all(feature = "new-components", feature = "new-domains"))]
impl UiTheme {
    /// Convert to AppTheme for use with the comprehensive theme system
    pub fn to_app_theme(&self) -> crate::presentation::theme::AppTheme {
        use crate::presentation::theme::{AppTheme, ThemeVariant};
        
        let variant = if self.high_contrast {
            ThemeVariant::HighContrast
        } else if self.dark_mode {
            ThemeVariant::Dark
        } else {
            ThemeVariant::Light
        };
        
        AppTheme::from_variant(variant)
    }
    
    /// Update from AppTheme changes
    pub fn from_app_theme(app_theme: &crate::presentation::theme::AppTheme) -> Self {
        use crate::presentation::theme::ThemeVariant;
        
        let (dark_mode, high_contrast) = match app_theme.variant() {
            ThemeVariant::Light => (false, false),
            ThemeVariant::Dark => (true, false),
            ThemeVariant::HighContrast => (true, true),
            ThemeVariant::Custom(_) => (true, false), // Default for custom
        };
        
        Self::builder()
            .name(app_theme.name.clone())
            .dark_mode(dark_mode)
            .high_contrast(high_contrast)
            .build()
    }
    
    /// Check if theme preferences match system settings
    pub fn should_auto_switch(&self) -> bool {
        self.user_preferences.auto_detect_system_theme
    }
    
    /// Get preferred theme name for current mode
    pub fn get_preferred_theme_name(&self) -> &str {
        if self.dark_mode {
            &self.user_preferences.dark_theme_name
        } else {
            &self.user_preferences.light_theme_name
        }
    }
}
