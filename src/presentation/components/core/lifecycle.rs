//! Component Lifecycle Management
//! 
//! This module provides the ComponentManager and related infrastructure for
//! managing component lifecycles, registration, and coordination with the
//! reactive state system.

use super::{ComponentId, ComponentEvent, ComponentResult, ComponentError};
use super::base::{Component, ReactiveComponent, ComponentWrapper};
use crate::state::{StateManager, StateEvent};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{instrument, debug, warn, error};

/// Result type for mount operations
pub type MountResult = ComponentResult<()>;

/// Trait for component lifecycle management
#[async_trait]
pub trait ComponentLifecycle: Send + Sync {
    /// Called when component is first mounted
    async fn on_mount(&mut self) -> MountResult {
        debug!("Component mounted: {}", self.lifecycle_id());
        Ok(())
    }

    /// Called when component is about to be unmounted
    async fn on_unmount(&mut self) -> MountResult {
        debug!("Component unmounted: {}", self.lifecycle_id());
        Ok(())
    }

    /// Called when component becomes active/focused
    async fn on_activate(&mut self) -> MountResult {
        debug!("Component activated: {}", self.lifecycle_id());
        Ok(())
    }

    /// Called when component becomes inactive/unfocused
    async fn on_deactivate(&mut self) -> MountResult {
        debug!("Component deactivated: {}", self.lifecycle_id());
        Ok(())
    }

    /// Get component ID for lifecycle logging
    fn lifecycle_id(&self) -> &ComponentId;
}

/// Registry for tracking all registered components
#[derive(Debug)]
pub struct ComponentRegistry {
    components: HashMap<ComponentId, ComponentInfo>,
    mount_order: Vec<ComponentId>,
    focus_stack: Vec<ComponentId>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            mount_order: Vec::new(),
            focus_stack: Vec::new(),
        }
    }

    /// Register a component
    pub fn register(&mut self, id: ComponentId, info: ComponentInfo) -> ComponentResult<()> {
        if self.components.contains_key(&id) {
            return Err(ComponentError::StateError {
                message: format!("Component {} already registered", id),
            });
        }

        self.components.insert(id.clone(), info);
        debug!("Registered component: {}", id);
        Ok(())
    }

    /// Unregister a component
    pub fn unregister(&mut self, id: &ComponentId) -> ComponentResult<ComponentInfo> {
        self.components.remove(id).ok_or_else(|| ComponentError::NotFound { id: id.clone() })
    }

    /// Get component info
    pub fn get(&self, id: &ComponentId) -> Option<&ComponentInfo> {
        self.components.get(id)
    }

    /// List all registered components
    pub fn list_all(&self) -> Vec<&ComponentId> {
        self.components.keys().collect()
    }

    /// Add component to mount order
    pub fn add_to_mount_order(&mut self, id: ComponentId) {
        if !self.mount_order.contains(&id) {
            self.mount_order.push(id);
        }
    }

    /// Remove component from mount order
    pub fn remove_from_mount_order(&mut self, id: &ComponentId) {
        self.mount_order.retain(|component_id| component_id != id);
    }

    /// Get mount order
    pub fn mount_order(&self) -> &[ComponentId] {
        &self.mount_order
    }

    /// Push component to focus stack
    pub fn push_focus(&mut self, id: ComponentId) {
        // Remove if already in stack to avoid duplicates
        self.focus_stack.retain(|component_id| component_id != &id);
        self.focus_stack.push(id);
    }

    /// Pop component from focus stack
    pub fn pop_focus(&mut self) -> Option<ComponentId> {
        self.focus_stack.pop()
    }

    /// Get current focused component
    pub fn current_focus(&self) -> Option<&ComponentId> {
        self.focus_stack.last()
    }

    /// Get focus stack
    pub fn focus_stack(&self) -> &[ComponentId] {
        &self.focus_stack
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a registered component
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub id: ComponentId,
    pub component_type: String,
    pub parent_id: Option<ComponentId>,
    pub children: Vec<ComponentId>,
    pub is_mounted: bool,
    pub is_active: bool,
    pub mount_time: Option<chrono::DateTime<chrono::Utc>>,
    pub dependencies: Vec<ComponentId>,
}

impl ComponentInfo {
    pub fn new(id: ComponentId, component_type: String) -> Self {
        Self {
            id,
            component_type,
            parent_id: None,
            children: Vec::new(),
            is_mounted: false,
            is_active: false,
            mount_time: None,
            dependencies: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: ComponentId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<ComponentId>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn mark_mounted(&mut self) {
        self.is_mounted = true;
        self.mount_time = Some(chrono::Utc::now());
    }

    pub fn mark_unmounted(&mut self) {
        self.is_mounted = false;
        self.mount_time = None;
    }

    pub fn mark_active(&mut self) {
        self.is_active = true;
    }

    pub fn mark_inactive(&mut self) {
        self.is_active = false;
    }

    pub fn add_child(&mut self, child_id: ComponentId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    pub fn remove_child(&mut self, child_id: &ComponentId) {
        self.children.retain(|id| id != child_id);
    }
}

/// Central component manager that coordinates all component lifecycle operations
pub struct ComponentManager {
    registry: Arc<RwLock<ComponentRegistry>>,
    state_manager: Arc<StateManager>,
    event_sender: broadcast::Sender<ComponentEvent>,
    shutdown_signal: Arc<RwLock<bool>>,
}

impl ComponentManager {
    /// Create a new component manager
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self {
            registry: Arc::new(RwLock::new(ComponentRegistry::new())),
            state_manager,
            event_sender,
            shutdown_signal: Arc::new(RwLock::new(false)),
        }
    }

    /// Register and mount a component
    #[instrument(skip(self, component))]
    pub async fn mount_component<C>(&self, mut component: C) -> ComponentResult<()>
    where
        C: Component + ComponentLifecycle + Send + 'static,
    {
        let component_id = component.lifecycle_id().clone();
        debug!("Mounting component: {}", component_id);

        // Check dependencies first
        self.check_dependencies(&component_id).await?;

        // Register component
        let info = ComponentInfo::new(component_id.clone(), std::any::type_name::<C>().to_string());
        {
            let mut registry = self.registry.write().await;
            registry.register(component_id.clone(), info)?;
            registry.add_to_mount_order(component_id.clone());
        }

        // Call mount lifecycle
        if let Err(e) = component.on_mount().await {
            error!("Failed to mount component {}: {}", component_id, e);
            // Cleanup on failure
            let mut registry = self.registry.write().await;
            let _ = registry.unregister(&component_id);
            return Err(e);
        }

        // Mark as mounted
        {
            let mut registry = self.registry.write().await;
            if let Some(info) = registry.components.get_mut(&component_id) {
                info.mark_mounted();
            }
        }

        // Emit mount event
        let mount_event = ComponentEvent::FocusGained { component_id: component_id.clone() };
        let _ = self.event_sender.send(mount_event);

        debug!("Successfully mounted component: {}", component_id);
        Ok(())
    }

    /// Unmount a component
    #[instrument(skip(self))]
    pub async fn unmount_component(&self, component_id: &ComponentId) -> ComponentResult<()> {
        debug!("Unmounting component: {}", component_id);

        // Get component info and check if it's mounted
        let info = {
            let registry = self.registry.read().await;
            registry.get(component_id).cloned()
        };

        let mut component_info = info.ok_or_else(|| ComponentError::NotFound { 
            id: component_id.clone() 
        })?;

        if !component_info.is_mounted {
            warn!("Attempting to unmount already unmounted component: {}", component_id);
            return Ok(());
        }

        // Unmount children first
        for child_id in component_info.children.clone() {
            if let Err(e) = Box::pin(self.unmount_component(&child_id)).await {
                warn!("Failed to unmount child component {}: {}", child_id, e);
            }
        }

        // Remove from focus if currently focused
        {
            let mut registry = self.registry.write().await;
            if registry.current_focus() == Some(component_id) {
                registry.pop_focus();
            }
        }

        // Mark as unmounted
        {
            let mut registry = self.registry.write().await;
            if let Some(info) = registry.components.get_mut(component_id) {
                info.mark_unmounted();
            }
            registry.remove_from_mount_order(component_id);
        }

        // Emit unmount event
        let unmount_event = ComponentEvent::FocusLost { component_id: component_id.clone() };
        let _ = self.event_sender.send(unmount_event);

        debug!("Successfully unmounted component: {}", component_id);
        Ok(())
    }

    /// Activate/focus a component
    #[instrument(skip(self))]
    pub async fn activate_component(&self, component_id: &ComponentId) -> ComponentResult<()> {
        debug!("Activating component: {}", component_id);

        // Check if component exists and is mounted
        {
            let registry = self.registry.read().await;
            let info = registry.get(component_id)
                .ok_or_else(|| ComponentError::NotFound { id: component_id.clone() })?;
            
            if !info.is_mounted {
                return Err(ComponentError::StateError {
                    message: format!("Cannot activate unmounted component: {}", component_id),
                });
            }
        }

        // Deactivate currently focused component
        if let Some(current_focus) = self.get_current_focus().await {
            if &current_focus != component_id {
                self.deactivate_component(&current_focus).await?;
            }
        }

        // Activate the component
        {
            let mut registry = self.registry.write().await;
            if let Some(info) = registry.components.get_mut(component_id) {
                info.mark_active();
            }
            registry.push_focus(component_id.clone());
        }

        // Emit activation event
        let activation_event = ComponentEvent::FocusGained { component_id: component_id.clone() };
        let _ = self.event_sender.send(activation_event);

        debug!("Successfully activated component: {}", component_id);
        Ok(())
    }

    /// Deactivate a component
    #[instrument(skip(self))]
    pub async fn deactivate_component(&self, component_id: &ComponentId) -> ComponentResult<()> {
        debug!("Deactivating component: {}", component_id);

        // Mark as inactive
        {
            let mut registry = self.registry.write().await;
            if let Some(info) = registry.components.get_mut(component_id) {
                info.mark_inactive();
            }
        }

        // Emit deactivation event
        let deactivation_event = ComponentEvent::FocusLost { component_id: component_id.clone() };
        let _ = self.event_sender.send(deactivation_event);

        debug!("Successfully deactivated component: {}", component_id);
        Ok(())
    }

    /// Get currently focused component
    pub async fn get_current_focus(&self) -> Option<ComponentId> {
        let registry = self.registry.read().await;
        registry.current_focus().cloned()
    }

    /// Get all mounted components
    pub async fn get_mounted_components(&self) -> Vec<ComponentId> {
        let registry = self.registry.read().await;
        registry.components.values()
            .filter(|info| info.is_mounted)
            .map(|info| info.id.clone())
            .collect()
    }

    /// Check if component dependencies are satisfied
    async fn check_dependencies(&self, component_id: &ComponentId) -> ComponentResult<()> {
        let registry = self.registry.read().await;
        if let Some(info) = registry.get(component_id) {
            for dep_id in &info.dependencies {
                if let Some(dep_info) = registry.get(dep_id) {
                    if !dep_info.is_mounted {
                        return Err(ComponentError::StateError {
                            message: format!("Dependency {} is not mounted for component {}", dep_id, component_id),
                        });
                    }
                } else {
                    return Err(ComponentError::NotFound { id: dep_id.clone() });
                }
            }
        }
        Ok(())
    }

    /// Subscribe to component events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ComponentEvent> {
        self.event_sender.subscribe()
    }

    /// Emit a component event
    pub async fn emit_event(&self, event: ComponentEvent) {
        let _ = self.event_sender.send(event);
    }

    /// Shutdown the component manager and all components
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> ComponentResult<()> {
        debug!("Shutting down component manager");

        // Set shutdown signal
        {
            let mut shutdown = self.shutdown_signal.write().await;
            *shutdown = true;
        }

        // Unmount all components in reverse mount order
        let mount_order = {
            let registry = self.registry.read().await;
            registry.mount_order().to_vec()
        };

        for component_id in mount_order.into_iter().rev() {
            if let Err(e) = self.unmount_component(&component_id).await {
                warn!("Failed to unmount component {} during shutdown: {}", component_id, e);
            }
        }

        debug!("Component manager shutdown complete");
        Ok(())
    }

    /// Check if shutdown has been requested
    pub async fn is_shutdown_requested(&self) -> bool {
        let shutdown = self.shutdown_signal.read().await;
        *shutdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateConfig;

    #[tokio::test]
    async fn test_component_registry() {
        let mut registry = ComponentRegistry::new();
        
        let id = ComponentId::new("test_component");
        let info = ComponentInfo::new(id.clone(), "TestComponent".to_string());
        
        assert!(registry.register(id.clone(), info).is_ok());
        assert!(registry.get(&id).is_some());
        
        registry.add_to_mount_order(id.clone());
        assert_eq!(registry.mount_order(), &[id.clone()]);
        
        registry.push_focus(id.clone());
        assert_eq!(registry.current_focus(), Some(&id));
        
        let removed_info = registry.unregister(&id);
        assert!(removed_info.is_ok());
        assert!(registry.get(&id).is_none());
    }

    #[tokio::test]
    async fn test_component_info() {
        let id = ComponentId::new("test");
        let mut info = ComponentInfo::new(id.clone(), "TestComponent".to_string());
        
        assert!(!info.is_mounted);
        assert!(!info.is_active);
        
        info.mark_mounted();
        assert!(info.is_mounted);
        assert!(info.mount_time.is_some());
        
        info.mark_active();
        assert!(info.is_active);
        
        let child_id = ComponentId::new("child");
        info.add_child(child_id.clone());
        assert!(info.children.contains(&child_id));
        
        info.remove_child(&child_id);
        assert!(!info.children.contains(&child_id));
    }

    #[tokio::test]
    async fn test_component_manager_creation() {
        let state_manager = Arc::new(StateManager::new());
        state_manager.initialize().await.unwrap();
        
        let component_manager = ComponentManager::new(state_manager);
        
        assert!(component_manager.get_current_focus().await.is_none());
        assert!(component_manager.get_mounted_components().await.is_empty());
    }

    #[tokio::test]
    async fn test_component_manager_shutdown() {
        let state_manager = Arc::new(StateManager::new());
        state_manager.initialize().await.unwrap();
        
        let component_manager = ComponentManager::new(state_manager);
        
        assert!(!component_manager.is_shutdown_requested().await);
        
        component_manager.shutdown().await.unwrap();
        
        assert!(component_manager.is_shutdown_requested().await);
    }
} 