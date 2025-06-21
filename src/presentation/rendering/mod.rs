// Phase 3: Performance & UX Enhancement - Rendering Optimization
// 
// This module implements smart rendering with change detection, 
// dirty region tracking, and performance optimizations.

pub mod smart_renderer;
pub mod performance;
pub mod effects;

// Re-exports for convenience
pub use smart_renderer::*;
pub use performance::*;
pub use effects::*; 