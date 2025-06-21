use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ratatui::{Frame, layout::Rect};
use tracing::{debug, instrument, trace};

use crate::presentation::components::core::{ComponentId, Component};
use crate::state::stores::AppState;
use crate::presentation::theme::AppTheme;

/// Smart renderer that tracks component changes and only re-renders dirty regions
#[derive(Debug)]
pub struct SmartRenderer {
    /// Hash of the last rendered state to detect changes
    last_state_hash: u64,
    /// Set of components that need re-rendering
    dirty_regions: HashSet<ComponentId>,
    /// Component render cache for performance
    render_cache: HashMap<ComponentId, CachedRender>,
    /// Performance metrics
    metrics: RenderMetrics,
    /// Configuration for rendering behavior
    config: SmartRendererConfig,
}

/// Cached render information for a component
#[derive(Debug, Clone)]
struct CachedRender {
    /// Hash of the component's last rendered state
    state_hash: u64,
    /// When this was last rendered
    last_rendered: Instant,
    /// Size of the area this component was rendered in
    last_area: Rect,
    /// Whether this component is expensive to render
    is_expensive: bool,
}

/// Configuration for smart rendering behavior
#[derive(Debug, Clone)]
pub struct SmartRendererConfig {
    /// Enable change detection (default: true)
    pub enable_change_detection: bool,
    /// Enable render caching (default: true)
    pub enable_caching: bool,
    /// Maximum cache age before forced re-render
    pub max_cache_age: Duration,
    /// Threshold for considering a component "expensive"
    pub expensive_render_threshold: Duration,
    /// Maximum number of cached renders
    pub max_cache_size: usize,
}

impl Default for SmartRendererConfig {
    fn default() -> Self {
        Self {
            enable_change_detection: true,
            enable_caching: true,
            max_cache_age: Duration::from_secs(5),
            expensive_render_threshold: Duration::from_millis(16), // ~60fps
            max_cache_size: 100,
        }
    }
}

/// Performance metrics for rendering
#[derive(Debug, Default)]
pub struct RenderMetrics {
    /// Total number of render calls
    pub total_renders: u64,
    /// Number of renders skipped due to caching
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Time spent rendering
    pub total_render_time: Duration,
    /// Average render time
    pub average_render_time: Duration,
    /// Number of dirty regions in last render
    pub last_dirty_count: usize,
    /// Peak memory usage for caches
    pub peak_cache_size: usize,
}

impl SmartRenderer {
    /// Create a new smart renderer with default configuration
    pub fn new() -> Self {
        Self::with_config(SmartRendererConfig::default())
    }

    /// Create a new smart renderer with custom configuration
    pub fn with_config(config: SmartRendererConfig) -> Self {
        Self {
            last_state_hash: 0,
            dirty_regions: HashSet::new(),
            render_cache: HashMap::new(),
            metrics: RenderMetrics::default(),
            config,
        }
    }

    /// Main render method - only renders if state has changed
    #[instrument(skip(self, state, frame, theme), fields(state_changed = false, dirty_count = 0))]
    pub fn render_if_changed(
        &mut self,
        state: &AppState,
        frame: &mut Frame,
        theme: &AppTheme,
    ) -> bool {
        let start_time = Instant::now();
        
        // Calculate current state hash
        let current_hash = self.calculate_state_hash(state);
        let state_changed = current_hash != self.last_state_hash;
        
        tracing::Span::current().record("state_changed", state_changed);
        
        if !state_changed && self.config.enable_change_detection {
            trace!("State unchanged, skipping render");
            self.metrics.cache_hits += 1;
            return false;
        }

        // Update metrics
        self.metrics.total_renders += 1;
        self.metrics.cache_misses += 1;
        
        // Determine dirty regions
        if state_changed {
            self.update_dirty_regions(state);
        }
        
        let dirty_count = self.dirty_regions.len();
        tracing::Span::current().record("dirty_count", dirty_count);
        self.metrics.last_dirty_count = dirty_count;
        
        // Render dirty components
        self.render_dirty_components(state, frame, theme);
        
        // Update state hash
        self.last_state_hash = current_hash;
        
        // Update timing metrics
        let render_time = start_time.elapsed();
        self.metrics.total_render_time += render_time;
        self.metrics.average_render_time = 
            self.metrics.total_render_time / self.metrics.total_renders as u32;
        
        debug!(
            "Rendered {} dirty regions in {:?}",
            dirty_count,
            render_time
        );
        
        true
    }

    /// Force a full re-render of all components
    #[instrument(skip(self, state, frame, theme))]
    pub fn force_render(&mut self, state: &AppState, frame: &mut Frame, theme: &AppTheme) {
        debug!("Force rendering all components");
        
        // Clear cache and mark all as dirty
        self.render_cache.clear();
        self.mark_all_dirty(state);
        
        // Render everything
        self.render_dirty_components(state, frame, theme);
        
        // Update state hash
        self.last_state_hash = self.calculate_state_hash(state);
    }

    /// Mark a specific component as dirty (needs re-rendering)
    pub fn mark_dirty(&mut self, component_id: ComponentId) {
        trace!("Marking component as dirty: {:?}", component_id);
        
        // Remove from cache if present before inserting into dirty regions
        self.render_cache.remove(&component_id);
        self.dirty_regions.insert(component_id);
    }

    /// Mark all components as dirty
    fn mark_all_dirty(&mut self, state: &AppState) {
        // This would need to be implemented based on how components are tracked
        // For now, we'll clear the dirty regions and let the next render detect changes
        self.dirty_regions.clear();
        
        // Add logic here to iterate through all components in the state
        // and mark them as dirty. This depends on how components are organized.
    }

    /// Calculate a hash of the current application state
    fn calculate_state_hash(&self, state: &AppState) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        
        // Hash key state components that affect rendering
        state.mode.hash(&mut hasher);
        state.current_screen.hash(&mut hasher);
        state.is_loading.hash(&mut hasher);
        state.loading_message.hash(&mut hasher);
        state.status_message.hash(&mut hasher);
        state.should_quit.hash(&mut hasher);
        state.config_dirty.hash(&mut hasher);
        
        // Hash performance metrics for render-affecting changes
        state.performance.render_count.hash(&mut hasher);
        state.performance.state_updates.hash(&mut hasher);
        
        hasher.finish()
    }

    /// Update dirty regions based on state changes
    fn update_dirty_regions(&mut self, state: &AppState) {
        // This is a simplified implementation
        // In a real implementation, you'd compare the previous and current state
        // and determine which specific components need updating
        
        // For now, mark common UI regions as dirty when state changes
        self.dirty_regions.insert(ComponentId::new("status_bar"));
        self.dirty_regions.insert(ComponentId::new("main_content"));
        
        // Mark screen-specific components as dirty
        match state.current_screen {
            crate::state::stores::Screen::Main => {
                self.dirty_regions.insert(ComponentId::new("main_screen"));
            }
            crate::state::stores::Screen::TaskSearch => {
                self.dirty_regions.insert(ComponentId::new("task_list"));
                self.dirty_regions.insert(ComponentId::new("task_details"));
            }
            crate::state::stores::Screen::Commit => {
                self.dirty_regions.insert(ComponentId::new("commit_form"));
            }
            crate::state::stores::Screen::Config => {
                self.dirty_regions.insert(ComponentId::new("config_form"));
            }
            crate::state::stores::Screen::ReleaseNotes => {
                self.dirty_regions.insert(ComponentId::new("release_notes"));
            }
            crate::state::stores::Screen::SemanticRelease => {
                self.dirty_regions.insert(ComponentId::new("semantic_release"));
            }
            crate::state::stores::Screen::CommitPreview => {
                self.dirty_regions.insert(ComponentId::new("commit_preview"));
            }
            crate::state::stores::Screen::Help => {
                self.dirty_regions.insert(ComponentId::new("help_screen"));
            }
        }
    }

    /// Render only the dirty components
    fn render_dirty_components(&mut self, state: &AppState, frame: &mut Frame, theme: &AppTheme) {
        let dirty_components: Vec<_> = self.dirty_regions.drain().collect();
        
        for component_id in dirty_components {
            self.render_component(&component_id, state, frame, theme);
        }
        
        // Clean up old cache entries
        self.cleanup_cache();
    }

    /// Render a specific component with caching
    fn render_component(
        &mut self,
        component_id: &ComponentId,
        state: &AppState,
        frame: &mut Frame,
        theme: &AppTheme,
    ) {
        let start_time = Instant::now();
        
        // Check if we can use cached render
        if let Some(cached) = self.render_cache.get(component_id) {
            if self.config.enable_caching && 
               start_time.duration_since(cached.last_rendered) < self.config.max_cache_age {
                trace!("Using cached render for component: {:?}", component_id);
                return;
            }
        }
        
        // Perform actual rendering
        // This would need to be implemented based on your component system
        trace!("Rendering component: {:?}", component_id);
        
        // For now, this is a placeholder - you'd call the actual component render method here
        // component.render(frame, area, theme);
        
        let render_time = start_time.elapsed();
        
        // Update cache
        if self.config.enable_caching {
            let cached_render = CachedRender {
                state_hash: self.calculate_component_hash(component_id, state),
                last_rendered: start_time,
                last_area: frame.area(),
                is_expensive: render_time > self.config.expensive_render_threshold,
            };
            
            self.render_cache.insert(component_id.clone(), cached_render);
        }
        
        trace!(
            "Rendered component {:?} in {:?}",
            component_id,
            render_time
        );
    }

    /// Calculate hash for a specific component's state
    fn calculate_component_hash(&self, component_id: &ComponentId, state: &AppState) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        component_id.hash(&mut hasher);
        
        // Add component-specific state hashing logic here
        // This would depend on how component state is stored
        
        hasher.finish()
    }

    /// Clean up old cache entries
    fn cleanup_cache(&mut self) {
        let now = Instant::now();
        let max_age = self.config.max_cache_age;
        
        // Remove expired entries
        self.render_cache.retain(|_, cached| {
            now.duration_since(cached.last_rendered) < max_age
        });
        
        // Enforce cache size limit
        if self.render_cache.len() > self.config.max_cache_size {
            // Remove oldest entries (simple LRU)
            let mut entries: Vec<_> = self.render_cache.iter()
                .map(|(id, cached)| (id.clone(), cached.last_rendered))
                .collect();
            
            entries.sort_by_key(|(_, time)| *time);
            
            let to_remove = entries.len() - self.config.max_cache_size;
            for (id, _) in entries.into_iter().take(to_remove) {
                self.render_cache.remove(&id);
            }
        }
        
        // Update peak cache size metric
        self.metrics.peak_cache_size = self.metrics.peak_cache_size.max(self.render_cache.len());
    }

    /// Get current performance metrics
    pub fn metrics(&self) -> &RenderMetrics {
        &self.metrics
    }

    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = RenderMetrics::default();
    }

    /// Get current configuration
    pub fn config(&self) -> &SmartRendererConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SmartRendererConfig) {
        self.config = config;
        
        // Clear cache if caching was disabled
        if !self.config.enable_caching {
            self.render_cache.clear();
        }
    }
}

impl Default for SmartRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::stores::AppState;

    #[test]
    fn test_smart_renderer_creation() {
        let renderer = SmartRenderer::new();
        assert_eq!(renderer.last_state_hash, 0);
        assert!(renderer.dirty_regions.is_empty());
        assert!(renderer.render_cache.is_empty());
    }

    #[test]
    fn test_mark_dirty() {
        let mut renderer = SmartRenderer::new();
        let component_id = ComponentId::new("test_component");
        
        renderer.mark_dirty(component_id.clone());
        assert!(renderer.dirty_regions.contains(&component_id));
    }

    #[test]
    fn test_state_hash_consistency() {
        let renderer = SmartRenderer::new();
        let state = AppState::builder().build();
        
        let hash1 = renderer.calculate_state_hash(&state);
        let hash2 = renderer.calculate_state_hash(&state);
        
        assert_eq!(hash1, hash2, "State hash should be consistent for same state");
    }

    #[test]
    fn test_cache_cleanup() {
        let mut renderer = SmartRenderer::with_config(SmartRendererConfig {
            max_cache_size: 2,
            ..Default::default()
        });
        
        // Add entries beyond cache size
        renderer.render_cache.insert(
            ComponentId::new("comp1"),
            CachedRender {
                state_hash: 1,
                last_rendered: Instant::now(),
                last_area: Rect::default(),
                is_expensive: false,
            }
        );
        
        renderer.render_cache.insert(
            ComponentId::new("comp2"),
            CachedRender {
                state_hash: 2,
                last_rendered: Instant::now(),
                last_area: Rect::default(),
                is_expensive: false,
            }
        );
        
        renderer.render_cache.insert(
            ComponentId::new("comp3"),
            CachedRender {
                state_hash: 3,
                last_rendered: Instant::now(),
                last_area: Rect::default(),
                is_expensive: false,
            }
        );
        
        renderer.cleanup_cache();
        
        assert!(renderer.render_cache.len() <= 2, "Cache should respect size limit");
    }
} 