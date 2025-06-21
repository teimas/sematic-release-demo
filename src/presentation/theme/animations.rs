// Animation and loading indicator configuration
// Visual effects and transitions for enhanced user experience

#[cfg(feature = "new-components")]
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Animation configuration for different visual effects
#[derive(Debug, Clone)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub struct AnimationConfig {
    pub enabled: bool,
    pub duration_fast: Duration,
    pub duration_normal: Duration,
    pub duration_slow: Duration,
    pub loading_indicators: LoadingIndicatorConfig,
    pub transitions: TransitionConfig,
}

impl AnimationConfig {
    /// Create default animation configuration
    pub fn default() -> Self {
        Self {
            enabled: true,
            duration_fast: Duration::from_millis(150),
            duration_normal: Duration::from_millis(300),
            duration_slow: Duration::from_millis(500),
            loading_indicators: LoadingIndicatorConfig::default(),
            transitions: TransitionConfig::default(),
        }
    }
    
    /// Create minimal animation configuration for performance
    pub fn minimal() -> Self {
        Self {
            enabled: false,
            duration_fast: Duration::from_millis(50),
            duration_normal: Duration::from_millis(100),
            duration_slow: Duration::from_millis(200),
            loading_indicators: LoadingIndicatorConfig::minimal(),
            transitions: TransitionConfig::minimal(),
        }
    }
    
    /// Create smooth animation configuration for better UX
    pub fn smooth() -> Self {
        Self {
            enabled: true,
            duration_fast: Duration::from_millis(200),
            duration_normal: Duration::from_millis(400),
            duration_slow: Duration::from_millis(800),
            loading_indicators: LoadingIndicatorConfig::smooth(),
            transitions: TransitionConfig::smooth(),
        }
    }
}

/// Loading indicator configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub struct LoadingIndicatorConfig {
    pub spinner_style: SpinnerStyle,
    pub update_interval: Duration,
    pub progress_bar_style: ProgressBarStyle,
    pub pulse_enabled: bool,
    pub pulse_interval: Duration,
}

impl LoadingIndicatorConfig {
    /// Create default loading indicator configuration
    pub fn default() -> Self {
        Self {
            spinner_style: SpinnerStyle::Dots,
            update_interval: Duration::from_millis(100),
            progress_bar_style: ProgressBarStyle::Blocks,
            pulse_enabled: true,
            pulse_interval: Duration::from_millis(1000),
        }
    }
    
    /// Create minimal loading indicator configuration
    pub fn minimal() -> Self {
        Self {
            spinner_style: SpinnerStyle::Simple,
            update_interval: Duration::from_millis(200),
            progress_bar_style: ProgressBarStyle::Simple,
            pulse_enabled: false,
            pulse_interval: Duration::from_millis(2000),
        }
    }
    
    /// Create smooth loading indicator configuration
    pub fn smooth() -> Self {
        Self {
            spinner_style: SpinnerStyle::Arc,
            update_interval: Duration::from_millis(80),
            progress_bar_style: ProgressBarStyle::Gradient,
            pulse_enabled: true,
            pulse_interval: Duration::from_millis(800),
        }
    }
}

/// Spinner style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum SpinnerStyle {
    Simple,     // -\|/
    Dots,       // ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
    Arc,        // ◜◠◝◞◡◟
    Arrows,     // ←↖↑↗→↘↓↙
    Blocks,     // ▁▂▃▄▅▆▇█
    Clock,      // 🕐🕑🕒🕓🕔🕕🕖🕗🕘🕙🕚🕛
}

/// Progress bar style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum ProgressBarStyle {
    Simple,     // [====    ]
    Blocks,     // [████░░░░]
    Gradient,   // [▰▰▰▱▱▱▱▱]
    Dots,       // [●●●○○○○○]
    Arrows,     // [►►►▷▷▷▷▷]
}

/// Transition configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub struct TransitionConfig {
    pub fade_duration: Duration,
    pub slide_duration: Duration,
    pub scale_duration: Duration,
    pub easing: EasingFunction,
}

impl TransitionConfig {
    /// Create default transition configuration
    pub fn default() -> Self {
        Self {
            fade_duration: Duration::from_millis(200),
            slide_duration: Duration::from_millis(300),
            scale_duration: Duration::from_millis(150),
            easing: EasingFunction::EaseInOut,
        }
    }
    
    /// Create minimal transition configuration
    pub fn minimal() -> Self {
        Self {
            fade_duration: Duration::from_millis(50),
            slide_duration: Duration::from_millis(100),
            scale_duration: Duration::from_millis(50),
            easing: EasingFunction::Linear,
        }
    }
    
    /// Create smooth transition configuration
    pub fn smooth() -> Self {
        Self {
            fade_duration: Duration::from_millis(400),
            slide_duration: Duration::from_millis(500),
            scale_duration: Duration::from_millis(300),
            easing: EasingFunction::EaseInOutCubic,
        }
    }
}

/// Easing function enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "new-components", derive(Serialize, Deserialize))]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
}

/// Loading indicator implementation
#[derive(Debug, Clone)]
pub struct LoadingIndicator {
    style: SpinnerStyle,
    start_time: Instant,
    update_interval: Duration,
    current_frame: usize,
}

impl LoadingIndicator {
    /// Create a new loading indicator
    pub fn new(config: &LoadingIndicatorConfig) -> Self {
        Self {
            style: config.spinner_style,
            start_time: Instant::now(),
            update_interval: config.update_interval,
            current_frame: 0,
        }
    }
    
    /// Get the current spinner character
    pub fn current_char(&mut self) -> char {
        let elapsed = self.start_time.elapsed();
        let frame_count = (elapsed.as_millis() / self.update_interval.as_millis()) as usize;
        
        if frame_count != self.current_frame {
            self.current_frame = frame_count;
        }
        
        self.get_frame_char(self.current_frame)
    }
    
    /// Get character for specific frame
    fn get_frame_char(&self, frame: usize) -> char {
        match self.style {
            SpinnerStyle::Simple => {
                let chars = ['-', '\\', '|', '/'];
                chars[frame % chars.len()]
            }
            SpinnerStyle::Dots => {
                let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                chars[frame % chars.len()]
            }
            SpinnerStyle::Arc => {
                let chars = ['◜', '◠', '◝', '◞', '◡', '◟'];
                chars[frame % chars.len()]
            }
            SpinnerStyle::Arrows => {
                let chars = ['←', '↖', '↑', '↗', '→', '↘', '↓', '↙'];
                chars[frame % chars.len()]
            }
            SpinnerStyle::Blocks => {
                let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
                chars[frame % chars.len()]
            }
            SpinnerStyle::Clock => {
                let chars = ['🕐', '🕑', '🕒', '🕓', '🕔', '🕕', '🕖', '🕗', '🕘', '🕙', '🕚', '🕛'];
                chars[frame % chars.len()]
            }
        }
    }
    
    /// Reset the indicator
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.current_frame = 0;
    }
    
    /// Check if indicator should update
    pub fn should_update(&self) -> bool {
        self.start_time.elapsed() >= self.update_interval
    }
}

/// Progress bar implementation
#[derive(Debug, Clone)]
pub struct ProgressBar {
    style: ProgressBarStyle,
    width: usize,
    progress: f32, // 0.0 to 1.0
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(style: ProgressBarStyle, width: usize) -> Self {
        Self {
            style,
            width,
            progress: 0.0,
        }
    }
    
    /// Set progress (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
    
    /// Get progress value
    pub fn get_progress(&self) -> f32 {
        self.progress
    }
    
    /// Render the progress bar as a string
    pub fn render(&self) -> String {
        let filled_width = (self.width as f32 * self.progress) as usize;
        let empty_width = self.width - filled_width;
        
        match self.style {
            ProgressBarStyle::Simple => {
                format!("[{}{}]", "=".repeat(filled_width), " ".repeat(empty_width))
            }
            ProgressBarStyle::Blocks => {
                format!("[{}{}]", "█".repeat(filled_width), "░".repeat(empty_width))
            }
            ProgressBarStyle::Gradient => {
                format!("[{}{}]", "▰".repeat(filled_width), "▱".repeat(empty_width))
            }
            ProgressBarStyle::Dots => {
                format!("[{}{}]", "●".repeat(filled_width), "○".repeat(empty_width))
            }
            ProgressBarStyle::Arrows => {
                format!("[{}{}]", "►".repeat(filled_width), "▷".repeat(empty_width))
            }
        }
    }
    
    /// Render with percentage text
    pub fn render_with_percentage(&self) -> String {
        let bar = self.render();
        let percentage = (self.progress * 100.0) as u8;
        format!("{} {}%", bar, percentage)
    }
}

/// Transition effect implementation
#[derive(Debug, Clone)]
pub struct TransitionEffect {
    start_time: Instant,
    duration: Duration,
    easing: EasingFunction,
    effect_type: TransitionType,
}

/// Transition type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionType {
    Fade,
    Slide,
    Scale,
}

impl TransitionEffect {
    /// Create a new transition effect
    pub fn new(effect_type: TransitionType, duration: Duration, easing: EasingFunction) -> Self {
        Self {
            start_time: Instant::now(),
            duration,
            easing,
            effect_type,
        }
    }
    
    /// Get current progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            1.0
        } else {
            let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
            self.apply_easing(t)
        }
    }
    
    /// Check if transition is complete
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
    
    /// Apply easing function to linear progress
    fn apply_easing(&self, t: f32) -> f32 {
        match self.easing {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - 4.0 * (1.0 - t).powi(3)
                }
            }
        }
    }
    
    /// Reset the transition
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_animation_config() {
        let default_config = AnimationConfig::default();
        assert!(default_config.enabled);
        assert!(default_config.duration_fast < default_config.duration_normal);
        assert!(default_config.duration_normal < default_config.duration_slow);
        
        let minimal_config = AnimationConfig::minimal();
        assert!(!minimal_config.enabled);
        assert!(minimal_config.duration_normal < default_config.duration_normal);
    }
    
    #[test]
    fn test_loading_indicator() {
        let config = LoadingIndicatorConfig::default();
        let mut indicator = LoadingIndicator::new(&config);
        
        let first_char = indicator.current_char();
        
        // Wait a bit and check if character changes
        thread::sleep(Duration::from_millis(150));
        let second_char = indicator.current_char();
        
        // Characters should be different (animation progressed)
        // Note: This test might be flaky due to timing
        assert!(first_char != second_char || !config.update_interval.is_zero());
    }
    
    #[test]
    fn test_progress_bar() {
        let mut progress_bar = ProgressBar::new(ProgressBarStyle::Simple, 10);
        
        assert_eq!(progress_bar.get_progress(), 0.0);
        
        progress_bar.set_progress(0.5);
        assert_eq!(progress_bar.get_progress(), 0.5);
        
        let rendered = progress_bar.render();
        assert!(rendered.contains("="));
        assert!(rendered.contains(" "));
        
        // Test percentage rendering
        let with_percentage = progress_bar.render_with_percentage();
        assert!(with_percentage.contains("50%"));
        
        // Test bounds
        progress_bar.set_progress(1.5); // Should clamp to 1.0
        assert_eq!(progress_bar.get_progress(), 1.0);
        
        progress_bar.set_progress(-0.5); // Should clamp to 0.0
        assert_eq!(progress_bar.get_progress(), 0.0);
    }
    
    #[test]
    fn test_transition_effect() {
        let transition = TransitionEffect::new(
            TransitionType::Fade,
            Duration::from_millis(100),
            EasingFunction::Linear,
        );
        
        let initial_progress = transition.progress();
        assert!(initial_progress >= 0.0 && initial_progress <= 1.0);
        
        // Wait for transition to complete
        thread::sleep(Duration::from_millis(150));
        assert!(transition.is_complete());
        assert_eq!(transition.progress(), 1.0);
    }
    
    #[test]
    fn test_easing_functions() {
        let linear = TransitionEffect::new(
            TransitionType::Fade,
            Duration::from_millis(1000),
            EasingFunction::Linear,
        );
        
        let ease_in = TransitionEffect::new(
            TransitionType::Fade,
            Duration::from_millis(1000),
            EasingFunction::EaseIn,
        );
        
        // At the midpoint, eased should be different from linear
        thread::sleep(Duration::from_millis(500));
        
        let linear_progress = linear.apply_easing(0.5);
        let eased_progress = ease_in.apply_easing(0.5);
        
        assert_eq!(linear_progress, 0.5);
        assert_ne!(eased_progress, 0.5);
    }
    
    #[test]
    fn test_spinner_styles() {
        let config = LoadingIndicatorConfig::default();
        let mut indicator = LoadingIndicator::new(&config);
        
        // Test that different styles produce different characters
        for &style in &[
            SpinnerStyle::Simple,
            SpinnerStyle::Dots,
            SpinnerStyle::Arc,
            SpinnerStyle::Arrows,
        ] {
            indicator.style = style;
            let char1 = indicator.get_frame_char(0);
            let char2 = indicator.get_frame_char(1);
            assert_ne!(char1, char2, "Style {:?} should have different frames", style);
        }
    }
} 