use std::time::{Duration, Instant};
use std::collections::HashMap;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Gauge, Paragraph},
    text::{Line, Span},
};
use tracing::{debug, trace};

use crate::presentation::theme::AppTheme;
use crate::presentation::components::core::ComponentId;

/// Visual effects system for enhanced user experience
#[derive(Debug)]
pub struct EffectsSystem {
    /// Active animations
    active_animations: HashMap<ComponentId, Animation>,
    /// Effect configuration
    config: EffectsConfig,
    /// Performance tracking
    last_update: Instant,
}

/// Configuration for visual effects
#[derive(Debug, Clone)]
pub struct EffectsConfig {
    /// Enable animations (can be disabled for performance)
    pub enable_animations: bool,
    /// Enable loading indicators
    pub enable_loading_indicators: bool,
    /// Enable focus transitions
    pub enable_focus_transitions: bool,
    /// Enable hover effects (for mouse support)
    pub enable_hover_effects: bool,
    /// Animation speed multiplier (1.0 = normal, 0.5 = half speed, 2.0 = double speed)
    pub animation_speed: f32,
    /// Maximum number of concurrent animations
    pub max_concurrent_animations: usize,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            enable_animations: true,
            enable_loading_indicators: true,
            enable_focus_transitions: true,
            enable_hover_effects: false, // Disabled by default for TUI
            animation_speed: 1.0,
            max_concurrent_animations: 20,
        }
    }
}

/// Animation definition
#[derive(Debug, Clone)]
pub struct Animation {
    /// Animation type
    pub animation_type: AnimationType,
    /// Start time
    pub start_time: Instant,
    /// Duration of the animation
    pub duration: Duration,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Whether the animation is complete
    pub is_complete: bool,
    /// Easing function
    pub easing: EasingFunction,
    /// Animation-specific data
    pub data: AnimationData,
}

/// Types of animations
#[derive(Debug, Clone)]
pub enum AnimationType {
    /// Fade in/out effect
    Fade { from_alpha: f32, to_alpha: f32 },
    /// Slide transition
    Slide { from: Rect, to: Rect },
    /// Scale animation
    Scale { from_scale: f32, to_scale: f32 },
    /// Color transition
    ColorTransition { from_color: Color, to_color: Color },
    /// Loading spinner
    LoadingSpinner { style: SpinnerStyle },
    /// Progress bar animation
    Progress { current: f32, target: f32 },
    /// Pulse effect (for focus/attention)
    Pulse { intensity: f32 },
    /// Typewriter effect for text
    Typewriter { text: String, chars_per_second: u32 },
}

/// Easing functions for smooth animations
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

/// Animation-specific data
#[derive(Debug, Clone)]
pub enum AnimationData {
    None,
    Fade { current_alpha: f32 },
    Slide { current_rect: Rect },
    Scale { current_scale: f32 },
    Color { current_color: Color },
    Spinner { current_frame: usize },
    Progress { current_value: f32 },
    Pulse { current_intensity: f32 },
    Typewriter { current_chars: usize },
}

/// Loading spinner styles
#[derive(Debug, Clone)]
pub enum SpinnerStyle {
    Dots,
    Line,
    Circle,
    Braille,
    Custom(Vec<char>),
}

impl EffectsSystem {
    /// Create a new effects system
    pub fn new() -> Self {
        Self::with_config(EffectsConfig::default())
    }

    /// Create a new effects system with custom configuration
    pub fn with_config(config: EffectsConfig) -> Self {
        Self {
            active_animations: HashMap::new(),
            config,
            last_update: Instant::now(),
        }
    }

    /// Update all active animations
    pub fn update(&mut self) {
        if !self.config.enable_animations {
            return;
        }

        let delta_time = self.last_update.elapsed();
        self.last_update = Instant::now();

        // Collect component IDs to avoid borrowing issues
        let component_ids: Vec<ComponentId> = self.active_animations.keys().cloned().collect();
        let mut completed_animations = Vec::new();
        
        for component_id in component_ids {
            if let Some(animation) = self.active_animations.get_mut(&component_id) {
                // Update animation inline to avoid borrow checker issues
                let elapsed = animation.start_time.elapsed();
                let raw_progress = elapsed.as_secs_f32() / animation.duration.as_secs_f32();
                
                if raw_progress >= 1.0 {
                    animation.progress = 1.0;
                    animation.is_complete = true;
                } else {
                    animation.progress = Self::apply_easing_static(raw_progress, animation.easing);
                }

                // Update animation-specific data based on progress
                match &animation.animation_type {
                    AnimationType::Fade { from_alpha, to_alpha } => {
                        let current_alpha = from_alpha + (to_alpha - from_alpha) * animation.progress;
                        animation.data = AnimationData::Fade { current_alpha };
                    },
                    AnimationType::Slide { from, to } => {
                        let current_rect = Rect {
                            x: (from.x as f32 + (to.x as f32 - from.x as f32) * animation.progress) as u16,
                            y: (from.y as f32 + (to.y as f32 - from.y as f32) * animation.progress) as u16,
                            width: (from.width as f32 + (to.width as f32 - from.width as f32) * animation.progress) as u16,
                            height: (from.height as f32 + (to.height as f32 - from.height as f32) * animation.progress) as u16,
                        };
                        animation.data = AnimationData::Slide { current_rect };
                    },
                    AnimationType::Scale { from_scale, to_scale } => {
                        let current_scale = from_scale + (to_scale - from_scale) * animation.progress;
                        animation.data = AnimationData::Scale { current_scale };
                    },
                    AnimationType::ColorTransition { from_color, to_color } => {
                        let current_color = Self::interpolate_color_static(*from_color, *to_color, animation.progress);
                        animation.data = AnimationData::Color { current_color };
                    },
                    AnimationType::LoadingSpinner { .. } => {
                        let current_frame = (elapsed.as_millis() / 100) % 10; // 10 frames for spinner
                        animation.data = AnimationData::Spinner { current_frame: current_frame as usize };
                    },
                    AnimationType::Progress { current, target } => {
                        let current_value = current + (target - current) * animation.progress;
                        animation.data = AnimationData::Progress { current_value };
                    },
                    AnimationType::Pulse { intensity } => {
                        let pulse_value = (elapsed.as_millis() as f32 / 1000.0 * 2.0).sin().abs();
                        let current_intensity = intensity * pulse_value;
                        animation.data = AnimationData::Pulse { current_intensity };
                    },
                    AnimationType::Typewriter { text, chars_per_second } => {
                        let chars_shown = (elapsed.as_secs_f32() * *chars_per_second as f32) as usize;
                        let current_chars = chars_shown.min(text.len());
                        animation.data = AnimationData::Typewriter { current_chars };
                        
                        if current_chars >= text.len() {
                            animation.is_complete = true;
                        }
                    },
                }
                
                if animation.is_complete {
                    completed_animations.push(component_id);
                }
            }
        }

        // Remove completed animations
        for component_id in completed_animations {
            self.active_animations.remove(&component_id);
            trace!("Animation completed for component: {:?}", component_id);
        }
    }

    /// Start a new animation for a component
    pub fn start_animation(&mut self, component_id: ComponentId, animation_type: AnimationType, duration: Duration) {
        if !self.config.enable_animations {
            return;
        }

        // Check concurrent animation limit
        if self.active_animations.len() >= self.config.max_concurrent_animations {
            debug!("Max concurrent animations reached, skipping new animation");
            return;
        }

        let animation = Animation {
            animation_type: animation_type.clone(),
            start_time: Instant::now(),
            duration: Duration::from_secs_f32(duration.as_secs_f32() / self.config.animation_speed),
            progress: 0.0,
            is_complete: false,
            easing: EasingFunction::EaseInOut,
            data: AnimationData::from_type(&animation_type),
        };

        self.active_animations.insert(component_id.clone(), animation);
        trace!("Started animation for component: {:?}", component_id);
    }

    /// Stop an animation for a component
    pub fn stop_animation(&mut self, component_id: &ComponentId) {
        if self.active_animations.remove(component_id).is_some() {
            trace!("Stopped animation for component: {:?}", component_id);
        }
    }

    /// Check if a component has an active animation
    pub fn has_animation(&self, component_id: &ComponentId) -> bool {
        self.active_animations.contains_key(component_id)
    }

    /// Get the current animation state for a component
    pub fn get_animation(&self, component_id: &ComponentId) -> Option<&Animation> {
        self.active_animations.get(component_id)
    }

    /// Render loading indicator
    pub fn render_loading_indicator(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &AppTheme,
        message: Option<&str>,
    ) {
        if !self.config.enable_loading_indicators {
            return;
        }

        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let spinner_index = (Instant::now().elapsed().as_millis() / 100) % spinner_chars.len() as u128;
        let spinner_char = spinner_chars[spinner_index as usize];

        let text = if let Some(msg) = message {
            format!("{} {}", spinner_char, msg)
        } else {
            format!("{} Loading...", spinner_char)
        };

        let paragraph = Paragraph::new(text)
            .style(theme.styles.text.normal.to_style(Some(theme.colors.primary), None))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.colors.primary))
                    .title("Loading")
            );

        frame.render_widget(paragraph, area);
    }

    /// Render progress bar with animation
    pub fn render_progress_bar(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &AppTheme,
        progress: f32,
        label: Option<&str>,
    ) {
        let progress_clamped = progress.clamp(0.0, 1.0);
        
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.colors.primary))
                    .title(label.unwrap_or("Progress"))
            )
            .gauge_style(Style::default().fg(theme.colors.palette.accent))
            .percent((progress_clamped * 100.0) as u16);

        frame.render_widget(gauge, area);
    }

    /// Render pulsing effect for focused components
    pub fn render_focus_pulse(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &AppTheme,
        intensity: f32,
    ) {
        if !self.config.enable_focus_transitions {
            return;
        }

        let pulse_value = (Instant::now().elapsed().as_millis() as f32 / 1000.0 * 2.0).sin().abs();
        let alpha = (intensity * pulse_value * 255.0) as u8;
        
        // Use a simple color adjustment instead of accessing RGB fields
        let pulse_color = match theme.colors.palette.accent {
            Color::Rgb(r, g, b) => Color::Rgb(
                r.saturating_add(alpha / 4),
                g.saturating_add(alpha / 4),
                b.saturating_add(alpha / 4),
            ),
            _ => theme.colors.palette.accent, // Fallback for non-RGB colors
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(pulse_color));

        frame.render_widget(block, area);
    }

    /// Apply fade effect to a style
    pub fn apply_fade_effect(&self, style: Style, component_id: &ComponentId) -> Style {
        if let Some(animation) = self.get_animation(component_id) {
            if let AnimationData::Fade { current_alpha } = &animation.data {
                let alpha = (current_alpha * 255.0) as u8;
                return style.fg(self.adjust_color_alpha(style.fg.unwrap_or(Color::White), alpha));
            }
        }
        style
    }

    /// Apply color transition effect
    pub fn apply_color_transition(&self, base_color: Color, component_id: &ComponentId) -> Color {
        if let Some(animation) = self.get_animation(component_id) {
            if let AnimationData::Color { current_color } = &animation.data {
                return *current_color;
            }
        }
        base_color
    }

    /// Start a fade in animation
    pub fn fade_in(&mut self, component_id: ComponentId, duration: Duration) {
        self.start_animation(
            component_id,
            AnimationType::Fade { from_alpha: 0.0, to_alpha: 1.0 },
            duration,
        );
    }

    /// Start a fade out animation
    pub fn fade_out(&mut self, component_id: ComponentId, duration: Duration) {
        self.start_animation(
            component_id,
            AnimationType::Fade { from_alpha: 1.0, to_alpha: 0.0 },
            duration,
        );
    }

    /// Start a pulse animation for focus indication
    pub fn pulse_focus(&mut self, component_id: ComponentId) {
        self.start_animation(
            component_id,
            AnimationType::Pulse { intensity: 0.8 },
            Duration::from_millis(1000),
        );
    }

    /// Start a loading spinner
    pub fn start_loading_spinner(&mut self, component_id: ComponentId) {
        self.start_animation(
            component_id,
            AnimationType::LoadingSpinner { style: SpinnerStyle::Dots },
            Duration::from_secs(60), // Long duration for continuous spinner
        );
    }

    /// Update a single animation
    fn update_single_animation(&mut self, animation: &mut Animation, _delta_time: Duration) {
        let elapsed = animation.start_time.elapsed();
        let raw_progress = elapsed.as_secs_f32() / animation.duration.as_secs_f32();
        
        if raw_progress >= 1.0 {
            animation.progress = 1.0;
            animation.is_complete = true;
        } else {
            animation.progress = self.apply_easing(raw_progress, animation.easing);
        }

        // Update animation-specific data based on progress
        match &animation.animation_type {
            AnimationType::Fade { from_alpha, to_alpha } => {
                let current_alpha = from_alpha + (to_alpha - from_alpha) * animation.progress;
                animation.data = AnimationData::Fade { current_alpha };
            },
            AnimationType::Slide { from, to } => {
                let current_rect = Rect {
                    x: (from.x as f32 + (to.x as f32 - from.x as f32) * animation.progress) as u16,
                    y: (from.y as f32 + (to.y as f32 - from.y as f32) * animation.progress) as u16,
                    width: (from.width as f32 + (to.width as f32 - from.width as f32) * animation.progress) as u16,
                    height: (from.height as f32 + (to.height as f32 - from.height as f32) * animation.progress) as u16,
                };
                animation.data = AnimationData::Slide { current_rect };
            },
            AnimationType::Scale { from_scale, to_scale } => {
                let current_scale = from_scale + (to_scale - from_scale) * animation.progress;
                animation.data = AnimationData::Scale { current_scale };
            },
            AnimationType::ColorTransition { from_color, to_color } => {
                let current_color = self.interpolate_color(*from_color, *to_color, animation.progress);
                animation.data = AnimationData::Color { current_color };
            },
            AnimationType::LoadingSpinner { .. } => {
                let current_frame = (elapsed.as_millis() / 100) % 10; // 10 frames for spinner
                animation.data = AnimationData::Spinner { current_frame: current_frame as usize };
            },
            AnimationType::Progress { current, target } => {
                let current_value = current + (target - current) * animation.progress;
                animation.data = AnimationData::Progress { current_value };
            },
            AnimationType::Pulse { intensity } => {
                let pulse_value = (elapsed.as_millis() as f32 / 1000.0 * 2.0).sin().abs();
                let current_intensity = intensity * pulse_value;
                animation.data = AnimationData::Pulse { current_intensity };
            },
            AnimationType::Typewriter { text, chars_per_second } => {
                let chars_shown = (elapsed.as_secs_f32() * *chars_per_second as f32) as usize;
                let current_chars = chars_shown.min(text.len());
                animation.data = AnimationData::Typewriter { current_chars };
                
                if current_chars >= text.len() {
                    animation.is_complete = true;
                }
            },
        }
    }

    /// Apply easing function to progress value (static version)
    fn apply_easing_static(progress: f32, easing: EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => progress,
            EasingFunction::EaseIn => progress * progress,
            EasingFunction::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            EasingFunction::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - 2.0 * (1.0 - progress) * (1.0 - progress)
                }
            }
            EasingFunction::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                
                if progress < 1.0 / d1 {
                    n1 * progress * progress
                } else if progress < 2.0 / d1 {
                    let progress = progress - 1.5 / d1;
                    n1 * progress * progress + 0.75
                } else if progress < 2.5 / d1 {
                    let progress = progress - 2.25 / d1;
                    n1 * progress * progress + 0.9375
                } else {
                    let progress = progress - 2.625 / d1;
                    n1 * progress * progress + 0.984375
                }
            }
            EasingFunction::Elastic => {
                if progress == 0.0 || progress == 1.0 {
                    progress
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    -(2.0_f32.powf(10.0 * progress - 10.0)) * ((progress * 10.0 - 10.75) * c4).sin()
                }
            }
        }
    }

    /// Apply easing function to progress value (instance method)
    fn apply_easing(&self, progress: f32, easing: EasingFunction) -> f32 {
        Self::apply_easing_static(progress, easing)
    }

    /// Interpolate between two colors (static version)
    fn interpolate_color_static(from: Color, to: Color, progress: f32) -> Color {
        match (from, to) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r = (r1 as f32 + (r2 as f32 - r1 as f32) * progress) as u8;
                let g = (g1 as f32 + (g2 as f32 - g1 as f32) * progress) as u8;
                let b = (b1 as f32 + (b2 as f32 - b1 as f32) * progress) as u8;
                Color::Rgb(r, g, b)
            }
            _ => to, // Fallback for non-RGB colors
        }
    }

    /// Interpolate between two colors (instance method)
    fn interpolate_color(&self, from: Color, to: Color, progress: f32) -> Color {
        Self::interpolate_color_static(from, to, progress)
    }

    /// Adjust color alpha (simplified for RGB colors)
    fn adjust_color_alpha(&self, color: Color, alpha: u8) -> Color {
        match color {
            Color::Rgb(r, g, b) => {
                let factor = alpha as f32 / 255.0;
                Color::Rgb(
                    (r as f32 * factor) as u8,
                    (g as f32 * factor) as u8,
                    (b as f32 * factor) as u8,
                )
            }
            _ => color,
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &EffectsConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: EffectsConfig) {
        self.config = config;
        
        // Clear animations if they're disabled
        if !self.config.enable_animations {
            self.active_animations.clear();
        }
    }

    /// Get the number of active animations
    pub fn active_animation_count(&self) -> usize {
        self.active_animations.len()
    }
}

impl Default for EffectsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationData {
    /// Create initial animation data from animation type
    fn from_type(animation_type: &AnimationType) -> Self {
        match animation_type {
            AnimationType::Fade { from_alpha, .. } => AnimationData::Fade { current_alpha: *from_alpha },
            AnimationType::ColorTransition { from_color, .. } => AnimationData::Color { current_color: *from_color },
            AnimationType::Progress { current, .. } => AnimationData::Progress { current_value: *current },
            AnimationType::Pulse { .. } => AnimationData::Pulse { current_intensity: 0.0 },
            AnimationType::LoadingSpinner { .. } => AnimationData::Spinner { current_frame: 0 },
            AnimationType::Typewriter { .. } => AnimationData::Typewriter { current_chars: 0 },
            _ => AnimationData::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effects_system_creation() {
        let effects = EffectsSystem::new();
        assert_eq!(effects.active_animations.len(), 0);
        assert!(effects.config.enable_animations);
    }

    #[test]
    fn test_animation_start_stop() {
        let mut effects = EffectsSystem::new();
        let component_id = ComponentId::new("test_component");
        
        // Start animation
        effects.fade_in(component_id.clone(), Duration::from_millis(100));
        assert!(effects.has_animation(&component_id));
        
        // Stop animation
        effects.stop_animation(&component_id);
        assert!(!effects.has_animation(&component_id));
    }

    #[test]
    fn test_easing_functions() {
        let effects = EffectsSystem::new();
        
        // Test linear easing
        assert_eq!(effects.apply_easing(0.5, EasingFunction::Linear), 0.5);
        
        // Test ease in
        let ease_in_result = effects.apply_easing(0.5, EasingFunction::EaseIn);
        assert!(ease_in_result < 0.5);
        
        // Test ease out
        let ease_out_result = effects.apply_easing(0.5, EasingFunction::EaseOut);
        assert!(ease_out_result > 0.5);
    }

    #[test]
    fn test_color_interpolation() {
        let effects = EffectsSystem::new();
        let from = Color::Rgb(0, 0, 0);
        let to = Color::Rgb(255, 255, 255);
        
        let mid = effects.interpolate_color(from, to, 0.5);
        if let Color::Rgb(r, g, b) = mid {
            assert!(r > 100 && r < 200);
            assert!(g > 100 && g < 200);
            assert!(b > 100 && b < 200);
        } else {
            panic!("Expected RGB color");
        }
    }
} 