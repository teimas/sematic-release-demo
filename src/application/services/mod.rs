//! Application services
//! 
//! This module provides orchestration services that coordinate multiple domains
//! to implement complex business use cases.

#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use std::sync::Arc;

// Import traits from commands module
#[cfg(feature = "new-domains")]
pub use crate::application::commands::{
    ReleaseOrchestrator, TaskManager, AiCoordinator
};

// Service implementations
pub mod release_orchestrator;
pub mod task_manager;
pub mod ai_coordinator;

// Re-exports of service implementations
#[cfg(feature = "new-domains")]
pub use release_orchestrator::*;
#[cfg(feature = "new-domains")]
pub use task_manager::*;
#[cfg(feature = "new-domains")]
pub use ai_coordinator::*;

/// Service registry for dependency injection
#[cfg(feature = "new-domains")]
pub struct ServiceRegistry {
    pub release_orchestrator: Arc<dyn ReleaseOrchestrator>,
    pub task_manager: Arc<dyn TaskManager>,
    pub ai_coordinator: Arc<dyn AiCoordinator>,
}

#[cfg(feature = "new-domains")]
impl ServiceRegistry {
    pub fn new(
        release_orchestrator: Arc<dyn ReleaseOrchestrator>,
        task_manager: Arc<dyn TaskManager>,
        ai_coordinator: Arc<dyn AiCoordinator>,
    ) -> Self {
        Self {
            release_orchestrator,
            task_manager,
            ai_coordinator,
        }
    }
} 