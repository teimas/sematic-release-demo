//! Dependency Injection Container
//! 
//! This module provides a centralized dependency injection container that wires
//! together all the architectural layers of the application.

#[cfg(feature = "new-domains")]
use std::sync::Arc;

#[cfg(feature = "new-domains")]
use crate::infrastructure::{
    storage::{
        InMemoryTaskStorage, InMemoryPromptTemplateStorage,
        InMemoryAnalysisResultStorage, InMemoryAiCache,
    },
    external::{
        ai_providers::{GeminiAiProvider, MockAiProvider},
        task_systems::{JiraAdapter, MockTaskSystemAdapter},
    },
    events::{InMemoryEventBus},
};

/// Configuration for the dependency injection container
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone)]
pub struct DiContainerConfig {
    pub use_mock_ai: bool,
    pub use_mock_tasks: bool,
    pub gemini_api_key: Option<String>,
    pub jira_url: Option<String>,
    pub jira_token: Option<String>,
}

#[cfg(feature = "new-domains")]
impl Default for DiContainerConfig {
    fn default() -> Self {
        Self {
            use_mock_ai: true,
            use_mock_tasks: true,
            gemini_api_key: None,
            jira_url: None,
            jira_token: None,
        }
    }
}

/// Dependency injection container that wires up all application dependencies
#[cfg(feature = "new-domains")]
pub struct DiContainer {
    pub task_storage: Arc<InMemoryTaskStorage>,
    pub ai_cache: Arc<InMemoryAiCache>,
    pub event_bus: Arc<InMemoryEventBus>,
}

#[cfg(feature = "new-domains")]
impl DiContainer {
    /// Creates a new dependency injection container with production dependencies
    pub fn new(config: DiContainerConfig) -> Self {
        // Create infrastructure adapters
        let task_storage = Arc::new(InMemoryTaskStorage::new());
        let ai_cache = Arc::new(InMemoryAiCache::with_default_config());
        let event_bus = Arc::new(InMemoryEventBus::new());
        
        Self {
            task_storage,
            ai_cache,
            event_bus,
        }
    }
    
    /// Creates a container for testing with mock dependencies
    pub fn new_for_testing() -> Self {
        Self::new(DiContainerConfig::default())
    }
}
