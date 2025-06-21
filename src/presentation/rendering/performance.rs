use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::sync::Arc;

use tracing::{debug, info, warn, instrument};
use serde::{Serialize, Deserialize};

use crate::presentation::components::core::ComponentId;

/// Performance monitor for tracking rendering metrics and optimization opportunities
#[derive(Debug)]
pub struct RenderPerformanceMonitor {
    /// Frame timing history
    frame_times: VecDeque<Duration>,
    /// Component render times
    component_times: HashMap<ComponentId, ComponentPerformanceData>,
    /// Overall performance metrics
    metrics: PerformanceMetrics,
    /// Configuration for performance monitoring
    config: PerformanceConfig,
    /// Start time for current frame
    current_frame_start: Option<Instant>,
}

/// Performance data for a specific component
#[derive(Debug, Clone)]
pub struct ComponentPerformanceData {
    /// Recent render times
    render_times: VecDeque<Duration>,
    /// Total number of renders
    render_count: u64,
    /// Average render time
    average_time: Duration,
    /// Peak render time
    peak_time: Duration,
    /// Whether this component is considered expensive
    is_expensive: bool,
    /// Last time this component was rendered
    last_rendered: Instant,
}

/// Overall performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average FPS over the monitoring period
    pub average_fps: f64,
    /// Current FPS
    pub current_fps: f64,
    /// Frame time statistics
    pub frame_time_stats: TimeStatistics,
    /// Number of expensive components
    pub expensive_component_count: usize,
    /// Total memory used by render caches
    pub cache_memory_usage: usize,
    /// Performance score (0-100, higher is better)
    pub performance_score: f64,
    /// Recommendations for optimization
    pub recommendations: Vec<PerformanceRecommendation>,
}

/// Statistical data about timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStatistics {
    pub mean: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub min: Duration,
    pub max: Duration,
}

/// Performance optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub category: RecommendationCategory,
    pub severity: RecommendationSeverity,
    pub message: String,
    pub component_id: Option<ComponentId>,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    FrameRate,
    ComponentPerformance,
    MemoryUsage,
    CacheEfficiency,
    RenderOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Configuration for performance monitoring
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum number of frame times to keep in history
    pub max_frame_history: usize,
    /// Maximum number of component render times to keep
    pub max_component_history: usize,
    /// Threshold for considering a component expensive (in milliseconds)
    pub expensive_threshold_ms: u64,
    /// Target FPS for performance scoring
    pub target_fps: f64,
    /// Enable detailed component profiling
    pub enable_component_profiling: bool,
    /// Enable memory usage tracking
    pub enable_memory_tracking: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_frame_history: 300, // ~5 seconds at 60fps
            max_component_history: 100,
            expensive_threshold_ms: 16, // ~60fps
            target_fps: 60.0,
            enable_component_profiling: true,
            enable_memory_tracking: true,
        }
    }
}

impl RenderPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self::with_config(PerformanceConfig::default())
    }

    /// Create a new performance monitor with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(config.max_frame_history),
            component_times: HashMap::new(),
            metrics: PerformanceMetrics::default(),
            config,
            current_frame_start: None,
        }
    }

    /// Start timing a new frame
    #[instrument(skip(self))]
    pub fn start_frame(&mut self) {
        self.current_frame_start = Some(Instant::now());
    }

    /// End timing the current frame and update metrics
    #[instrument(skip(self))]
    pub fn end_frame(&mut self) {
        if let Some(start_time) = self.current_frame_start.take() {
            let frame_time = start_time.elapsed();
            
            // Add to frame time history
            self.frame_times.push_back(frame_time);
            if self.frame_times.len() > self.config.max_frame_history {
                self.frame_times.pop_front();
            }
            
            // Update metrics
            self.update_frame_metrics();
            
            // Check for performance issues
            self.check_performance_issues(frame_time);
        }
    }

    /// Start timing a component render
    #[instrument(skip(self))]
    pub fn start_component_render(&mut self, component_id: &ComponentId) -> ComponentRenderTimer {
        ComponentRenderTimer {
            component_id: component_id.clone(),
            start_time: Instant::now(),
            monitor: self as *mut Self,
        }
    }

    /// Record a component render time
    fn record_component_time(&mut self, component_id: ComponentId, render_time: Duration) {
        if !self.config.enable_component_profiling {
            return;
        }

        let data = self.component_times.entry(component_id.clone()).or_insert_with(|| {
            ComponentPerformanceData {
                render_times: VecDeque::with_capacity(self.config.max_component_history),
                render_count: 0,
                average_time: Duration::ZERO,
                peak_time: Duration::ZERO,
                is_expensive: false,
                last_rendered: Instant::now(),
            }
        });

        // Update component data
        data.render_times.push_back(render_time);
        if data.render_times.len() > self.config.max_component_history {
            data.render_times.pop_front();
        }

        data.render_count += 1;
        data.peak_time = data.peak_time.max(render_time);
        data.last_rendered = Instant::now();

        // Calculate average
        let total: Duration = data.render_times.iter().sum();
        data.average_time = total / data.render_times.len() as u32;

        // Check if expensive
        data.is_expensive = data.average_time.as_millis() > self.config.expensive_threshold_ms as u128;

        if data.is_expensive {
            debug!(
                "Component {:?} is expensive: avg={:?}, peak={:?}",
                component_id, data.average_time, data.peak_time
            );
        }
    }

    /// Update frame-level metrics
    fn update_frame_metrics(&mut self) {
        if self.frame_times.is_empty() {
            return;
        }

        // Calculate FPS
        let recent_frames = self.frame_times.iter().rev().take(60).cloned().collect::<Vec<_>>();
        if !recent_frames.is_empty() {
            let total_time: Duration = recent_frames.iter().sum();
            self.metrics.current_fps = recent_frames.len() as f64 / total_time.as_secs_f64();
        }

        // Calculate average FPS
        let total_time: Duration = self.frame_times.iter().sum();
        self.metrics.average_fps = self.frame_times.len() as f64 / total_time.as_secs_f64();

        // Calculate frame time statistics
        self.metrics.frame_time_stats = self.calculate_time_statistics(&self.frame_times);

        // Count expensive components
        self.metrics.expensive_component_count = self.component_times
            .values()
            .filter(|data| data.is_expensive)
            .count();

        // Calculate performance score
        self.metrics.performance_score = self.calculate_performance_score();

        // Generate recommendations
        self.metrics.recommendations = self.generate_recommendations();
    }

    /// Calculate statistical data for a set of durations
    fn calculate_time_statistics(&self, times: &VecDeque<Duration>) -> TimeStatistics {
        if times.is_empty() {
            return TimeStatistics {
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                min: Duration::ZERO,
                max: Duration::ZERO,
            };
        }

        let mut sorted_times: Vec<Duration> = times.iter().cloned().collect();
        sorted_times.sort();

        let len = sorted_times.len();
        let total: Duration = sorted_times.iter().sum();

        TimeStatistics {
            mean: total / len as u32,
            median: sorted_times[len / 2],
            p95: sorted_times[(len as f64 * 0.95) as usize],
            p99: sorted_times[(len as f64 * 0.99) as usize],
            min: sorted_times[0],
            max: sorted_times[len - 1],
        }
    }

    /// Calculate overall performance score (0-100)
    fn calculate_performance_score(&self) -> f64 {
        let mut score = 100.0;

        // FPS score (50% weight)
        let fps_ratio = self.metrics.current_fps / self.config.target_fps;
        let fps_score = (fps_ratio.min(1.0) * 50.0).max(0.0);
        score = fps_score;

        // Frame time consistency (25% weight)
        let frame_time_ms = self.metrics.frame_time_stats.mean.as_millis() as f64;
        let target_frame_time_ms = 1000.0 / self.config.target_fps;
        let consistency_score = if frame_time_ms <= target_frame_time_ms {
            25.0
        } else {
            25.0 * (target_frame_time_ms / frame_time_ms).max(0.0)
        };
        score += consistency_score;

        // Component performance (25% weight)
        let expensive_ratio = self.metrics.expensive_component_count as f64 / 
            self.component_times.len().max(1) as f64;
        let component_score = 25.0 * (1.0 - expensive_ratio).max(0.0);
        score += component_score;

        score.min(100.0).max(0.0)
    }

    /// Generate performance recommendations
    fn generate_recommendations(&self) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();

        // FPS recommendations
        if self.metrics.current_fps < self.config.target_fps * 0.8 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::FrameRate,
                severity: if self.metrics.current_fps < self.config.target_fps * 0.5 {
                    RecommendationSeverity::Critical
                } else {
                    RecommendationSeverity::High
                },
                message: format!(
                    "Low frame rate detected: {:.1} FPS (target: {:.1} FPS)",
                    self.metrics.current_fps, self.config.target_fps
                ),
                component_id: None,
                suggested_action: "Consider enabling render caching or reducing visual complexity".to_string(),
            });
        }

        // Component performance recommendations
        for (component_id, data) in &self.component_times {
            if data.is_expensive {
                recommendations.push(PerformanceRecommendation {
                    category: RecommendationCategory::ComponentPerformance,
                    severity: if data.average_time.as_millis() > 50 {
                        RecommendationSeverity::High
                    } else {
                        RecommendationSeverity::Medium
                    },
                    message: format!(
                        "Component {:?} has slow render time: {:?} average",
                        component_id, data.average_time
                    ),
                    component_id: Some(component_id.clone()),
                    suggested_action: "Consider caching or optimizing this component's render logic".to_string(),
                });
            }
        }

        // Memory recommendations
        if self.config.enable_memory_tracking && self.metrics.cache_memory_usage > 10_000_000 { // 10MB
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::MemoryUsage,
                severity: RecommendationSeverity::Medium,
                message: format!(
                    "High cache memory usage: {} bytes",
                    self.metrics.cache_memory_usage
                ),
                component_id: None,
                suggested_action: "Consider reducing cache size limits or implementing cache eviction".to_string(),
            });
        }

        recommendations
    }

    /// Check for immediate performance issues
    fn check_performance_issues(&self, frame_time: Duration) {
        let frame_time_ms = frame_time.as_millis();
        
        if frame_time_ms > 100 { // Very slow frame
            warn!(
                "Very slow frame detected: {}ms (target: {}ms)",
                frame_time_ms,
                (1000.0 / self.config.target_fps) as u64
            );
        } else if frame_time_ms > 50 { // Moderately slow frame
            debug!(
                "Slow frame detected: {}ms",
                frame_time_ms
            );
        }
    }

    /// Get current performance metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Get component performance data
    pub fn component_data(&self, component_id: &ComponentId) -> Option<&ComponentPerformanceData> {
        self.component_times.get(component_id)
    }

    /// Get all component performance data
    pub fn all_component_data(&self) -> &HashMap<ComponentId, ComponentPerformanceData> {
        &self.component_times
    }

    /// Reset all metrics and history
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.component_times.clear();
        self.metrics = PerformanceMetrics::default();
    }

    /// Export performance data for analysis
    pub fn export_data(&self) -> serde_json::Value {
        serde_json::json!({
            "metrics": self.metrics,
            "frame_count": self.frame_times.len(),
            "component_count": self.component_times.len(),
            "config": {
                "target_fps": self.config.target_fps,
                "expensive_threshold_ms": self.config.expensive_threshold_ms,
            }
        })
    }
}

impl Default for RenderPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_fps: 0.0,
            current_fps: 0.0,
            frame_time_stats: TimeStatistics {
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                min: Duration::ZERO,
                max: Duration::ZERO,
            },
            expensive_component_count: 0,
            cache_memory_usage: 0,
            performance_score: 100.0,
            recommendations: Vec::new(),
        }
    }
}

/// RAII timer for component rendering
pub struct ComponentRenderTimer {
    component_id: ComponentId,
    start_time: Instant,
    monitor: *mut RenderPerformanceMonitor,
}

impl Drop for ComponentRenderTimer {
    fn drop(&mut self) {
        let render_time = self.start_time.elapsed();
        
        // Safety: This is safe because the timer lifetime is tied to the monitor
        unsafe {
            if !self.monitor.is_null() {
                (*self.monitor).record_component_time(self.component_id.clone(), render_time);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = RenderPerformanceMonitor::new();
        assert_eq!(monitor.frame_times.len(), 0);
        assert_eq!(monitor.component_times.len(), 0);
    }

    #[test]
    fn test_frame_timing() {
        let mut monitor = RenderPerformanceMonitor::new();
        
        monitor.start_frame();
        thread::sleep(Duration::from_millis(16)); // Simulate 60fps frame
        monitor.end_frame();
        
        assert_eq!(monitor.frame_times.len(), 1);
        assert!(monitor.metrics.current_fps > 0.0);
    }

    #[test]
    fn test_component_timing() {
        let mut monitor = RenderPerformanceMonitor::new();
        let component_id = ComponentId::new("test_component");
        
        {
            let _timer = monitor.start_component_render(&component_id);
            thread::sleep(Duration::from_millis(1));
        } // Timer drops here, recording the time
        
        assert!(monitor.component_times.contains_key(&component_id));
        let data = monitor.component_times.get(&component_id).unwrap();
        assert_eq!(data.render_count, 1);
        assert!(data.average_time > Duration::ZERO);
    }

    #[test]
    fn test_performance_score_calculation() {
        let mut monitor = RenderPerformanceMonitor::with_config(PerformanceConfig {
            target_fps: 60.0,
            ..Default::default()
        });
        
        // Simulate good performance
        for _ in 0..60 {
            monitor.frame_times.push_back(Duration::from_millis(16)); // 60fps
        }
        
        monitor.update_frame_metrics();
        assert!(monitor.metrics.performance_score > 80.0);
    }
} 