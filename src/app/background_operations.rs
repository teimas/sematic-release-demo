use std::sync::{Arc, Mutex};
use std::thread;

use crate::{
    app::App,
    git::repository::GitRepo,
    services::gemini::GeminiClient,
    types::{AppState, ComprehensiveAnalysisState},
    utils,
};

use crate::error::{Result, SemanticReleaseError};
use async_broadcast::{broadcast, Receiver, Sender};
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{error, info, instrument, warn};

/// Events emitted by background operations
#[derive(Debug, Clone)]
pub enum BackgroundEvent {
    // Release notes analysis events
    ReleaseNotesProgress(String),
    ReleaseNotesCompleted(Value),
    ReleaseNotesError(String),

    // Comprehensive analysis events
    AnalysisProgress(String),
    AnalysisCompleted(Value),
    AnalysisError(String),

    // Semantic release events
    SemanticReleaseProgress(String),
    SemanticReleaseCompleted(String),
    SemanticReleaseError(String),

    // General operation status
    OperationStarted { operation_id: String, description: String },
    OperationCompleted { operation_id: String },
    OperationCancelled { operation_id: String },
}

/// Status of a background operation
#[derive(Debug, Clone)]
pub enum OperationStatus {
    NotStarted,
    Running { progress: String },
    Completed { result: String },
    Failed { error: String },
    Cancelled,
}

/// Manages background operations with async channels
#[derive(Debug)]
pub struct BackgroundTaskManager {
    /// Event channel for broadcasting operation updates
    event_tx: Sender<BackgroundEvent>,
    event_rx: Receiver<BackgroundEvent>,
    
    /// Current operation statuses
    operation_status: Arc<RwLock<std::collections::HashMap<String, OperationStatus>>>,
    
    /// Active task handles for cancellation
    active_tasks: Arc<RwLock<std::collections::HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl BackgroundTaskManager {
    /// Create a new background task manager
    pub fn new() -> Self {
        let (event_tx, event_rx) = broadcast(1000);
        
        Self {
            event_tx,
            event_rx,
            operation_status: Arc::new(RwLock::new(std::collections::HashMap::new())),
            active_tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get a receiver for background events
    pub fn subscribe(&self) -> Receiver<BackgroundEvent> {
        self.event_rx.clone()
    }

    /// Get the current status of an operation
    #[instrument(skip(self))]
    pub async fn get_status(&self, operation_id: &str) -> Option<OperationStatus> {
        let status_map = self.operation_status.read().await;
        status_map.get(operation_id).cloned()
    }

    /// Start a new background operation
    #[instrument(skip(self, operation))]
    pub async fn start_operation<F, Fut>(
        &self,
        operation_id: String,
        description: String,
        operation: F,
    ) -> Result<()>
    where
        F: FnOnce(Sender<BackgroundEvent>, String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        // Set initial status
        {
            let mut status_map = self.operation_status.write().await;
            status_map.insert(operation_id.clone(), OperationStatus::NotStarted);
        }

        // Emit started event
        if let Err(e) = self.event_tx.broadcast(BackgroundEvent::OperationStarted {
            operation_id: operation_id.clone(),
            description: description.clone(),
        }).await {
            warn!("Failed to broadcast operation started event: {}", e);
        }

        // Create and store task handle
        let event_tx = self.event_tx.clone();
        let operation_status = self.operation_status.clone();
        let active_tasks = self.active_tasks.clone();
        let operation_id_for_task = operation_id.clone();

        let task = tokio::spawn(async move {
            // Update status to running
            {
                let mut status_map = operation_status.write().await;
                status_map.insert(operation_id_for_task.clone(), OperationStatus::Running {
                    progress: "Starting...".to_string(),
                });
            }

            // Execute the operation
            let result = operation(event_tx.clone(), operation_id_for_task.clone()).await;

            // Update final status and emit completion event
            match result {
                Ok(()) => {
                    {
                        let mut status_map = operation_status.write().await;
                        status_map.insert(operation_id_for_task.clone(), OperationStatus::Completed {
                            result: "Operation completed successfully".to_string(),
                        });
                    }
                    
                    if let Err(e) = event_tx.broadcast(BackgroundEvent::OperationCompleted {
                        operation_id: operation_id_for_task.clone(),
                    }).await {
                        warn!("Failed to broadcast operation completed event: {}", e);
                    }
                }
                Err(error) => {
                    let error_msg = error.to_string();
                    {
                        let mut status_map = operation_status.write().await;
                        status_map.insert(operation_id_for_task.clone(), OperationStatus::Failed {
                            error: error_msg.clone(),
                        });
                    }
                    
                    error!("Background operation '{}' failed: {}", operation_id_for_task, error_msg);
                }
            }

            // Remove from active tasks
            {
                let mut tasks = active_tasks.write().await;
                tasks.remove(&operation_id_for_task);
            }
        });

        // Store the task handle
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(operation_id, task);
        }

        Ok(())
    }

    /// Start release notes generation as a background task
    #[instrument(skip(self, config, commits))]
    pub async fn start_release_notes_generation(
        &self,
        config: &crate::types::AppConfig,
        commits: Vec<crate::types::GitCommit>,
    ) -> Result<String> {
        let operation_id = format!("release_notes_{}", uuid::Uuid::new_v4());
        let config_clone = config.clone();
        let operation_desc = "Release notes generation".to_string();

        self.start_operation(
            operation_id.clone(),
            operation_desc,
            move |event_tx, op_id| {
                let config = config_clone;
                let commits = commits;
                async move {
                    // Import the async function we already have
                    use crate::app::release_notes::generate_release_notes_task;
                    
                    match generate_release_notes_task(event_tx, op_id, config, commits).await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            error!("Release notes generation failed: {}", e);
                            Err(e)
                        }
                    }
                }
            }
        ).await?;

        Ok(operation_id)
    }

    /// Start comprehensive analysis as a background task
    #[instrument(skip(self, config, commits))]
    pub async fn start_comprehensive_analysis(
        &self,
        config: &crate::types::AppConfig,
        commits: Vec<crate::types::GitCommit>,
    ) -> Result<String> {
        let operation_id = format!("comprehensive_analysis_{}", uuid::Uuid::new_v4());
        let config_clone = config.clone();
        let operation_desc = "Comprehensive analysis".to_string();

        self.start_operation(
            operation_id.clone(),
            operation_desc,
            move |event_tx, op_id| {
                let config = config_clone;
                let commits = commits;
                async move {
                    // Placeholder for comprehensive analysis task
                    // This should be implemented similar to generate_release_notes_task
                    
                    // Emit progress
                    if let Err(e) = event_tx.broadcast(crate::app::background_operations::BackgroundEvent::AnalysisProgress(
                        "Starting comprehensive analysis...".to_string()
                    )).await {
                        warn!("Failed to emit progress: {}", e);
                    }

                    // TODO: Implement actual comprehensive analysis here
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                    
                    // Emit completion with sample data
                    let result = serde_json::json!({
                        "title": "Example analysis result",
                        "commitType": "feat",
                        "description": "Analysis completed",
                        "scope": "general"
                    });
                    
                    if let Err(e) = event_tx.broadcast(crate::app::background_operations::BackgroundEvent::AnalysisCompleted(result)).await {
                        warn!("Failed to emit completion: {}", e);
                    }

                    Ok(())
                }
            }
        ).await?;

        Ok(operation_id)
    }

    /// Cancel a background operation
    #[instrument(skip(self))]
    pub async fn cancel_operation(&self, operation_id: &str) -> Result<()> {
        let mut tasks = self.active_tasks.write().await;
        
        if let Some(task) = tasks.remove(operation_id) {
            task.abort();
            
            // Update status
            {
                let mut status_map = self.operation_status.write().await;
                status_map.insert(operation_id.to_string(), OperationStatus::Cancelled);
            }

            // Emit cancellation event
            if let Err(e) = self.event_tx.broadcast(BackgroundEvent::OperationCancelled {
                operation_id: operation_id.to_string(),
            }).await {
                warn!("Failed to broadcast operation cancelled event: {}", e);
            }

            info!("Cancelled background operation: {}", operation_id);
            Ok(())
        } else {
            Err(SemanticReleaseError::operation_error(
                format!("No active operation found with ID: {}", operation_id)
            ))
        }
    }

    /// Cancel all active operations
    #[instrument(skip(self))]
    pub async fn cancel_all_operations(&self) -> Result<()> {
        let task_ids: Vec<String> = {
            let tasks = self.active_tasks.read().await;
            tasks.keys().cloned().collect()
        };

        for operation_id in task_ids {
            if let Err(e) = self.cancel_operation(&operation_id).await {
                warn!("Failed to cancel operation {}: {}", operation_id, e);
            }
        }

        info!("Cancelled all background operations");
        Ok(())
    }

    /// Get list of active operation IDs
    pub async fn get_active_operations(&self) -> Vec<String> {
        let tasks = self.active_tasks.read().await;
        tasks.keys().cloned().collect()
    }

    /// Helper to broadcast progress updates
    pub async fn broadcast_progress(&self, operation_id: &str, progress: String) {
        // Update internal status
        {
            let mut status_map = self.operation_status.write().await;
            status_map.insert(operation_id.to_string(), OperationStatus::Running {
                progress: progress.clone(),
            });
        }

        // Broadcast appropriate event based on operation type
        let event = if operation_id.starts_with("release_notes") {
            BackgroundEvent::ReleaseNotesProgress(progress)
        } else if operation_id.starts_with("comprehensive_analysis") {
            BackgroundEvent::AnalysisProgress(progress)
        } else if operation_id.starts_with("semantic_release") {
            BackgroundEvent::SemanticReleaseProgress(progress)
        } else {
            // Generic progress - could extend this
            return;
        };

        if let Err(e) = self.event_tx.broadcast(event).await {
            warn!("Failed to broadcast progress event: {}", e);
        }
    }

    /// Helper to broadcast completion with result
    pub async fn broadcast_completion(&self, operation_id: &str, result: Value) {
        let event = if operation_id.starts_with("comprehensive_analysis") {
            BackgroundEvent::AnalysisCompleted(result)
        } else if operation_id.starts_with("release_notes") {
            BackgroundEvent::ReleaseNotesCompleted(result)
        } else if operation_id.starts_with("semantic_release") {
            if let Some(result_str) = result.as_str() {
                BackgroundEvent::SemanticReleaseCompleted(result_str.to_string())
            } else {
                return;
            }
        } else {
            return;
        };

        if let Err(e) = self.event_tx.broadcast(event).await {
            warn!("Failed to broadcast completion event: {}", e);
        }
    }

    /// Helper to broadcast errors
    pub async fn broadcast_error(&self, operation_id: &str, error: String) {
        let event = if operation_id.starts_with("release_notes") {
            BackgroundEvent::ReleaseNotesError(error)
        } else if operation_id.starts_with("comprehensive_analysis") {
            BackgroundEvent::AnalysisError(error)
        } else if operation_id.starts_with("semantic_release") {
            BackgroundEvent::SemanticReleaseError(error)
        } else {
            return;
        };

        if let Err(e) = self.event_tx.broadcast(event).await {
            warn!("Failed to broadcast error event: {}", e);
        }
    }
}

impl Default for BackgroundTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to run an operation with timeout
pub async fn run_with_timeout<F, T>(
    operation: F,
    timeout_duration: Duration,
    operation_name: &str,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match timeout(timeout_duration, operation).await {
        Ok(result) => result,
        Err(_) => {
            error!("Operation '{}' timed out after {:?}", operation_name, timeout_duration);
            Err(SemanticReleaseError::operation_error(
                format!("Operation '{}' timed out", operation_name)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_background_task_manager() {
        let manager = BackgroundTaskManager::new();
        let mut receiver = manager.subscribe();

        // Start a simple operation
        manager.start_operation(
            "test_op".to_string(),
            "Test operation".to_string(),
            |event_tx, operation_id| async move {
                // Simulate some work
                sleep(Duration::from_millis(100)).await;
                Ok(())
            }
        ).await.unwrap();

        // Should receive started event
        let event = receiver.recv().await.unwrap();
        match event {
            BackgroundEvent::OperationStarted { operation_id, .. } => {
                assert_eq!(operation_id, "test_op");
            }
            _ => panic!("Expected OperationStarted event"),
        }

        // Should eventually receive completed event
        let mut completed = false;
        while let Ok(event) = tokio::time::timeout(Duration::from_secs(1), receiver.recv()).await {
            if let Ok(BackgroundEvent::OperationCompleted { operation_id }) = event {
                assert_eq!(operation_id, "test_op");
                completed = true;
                break;
            }
        }
        assert!(completed, "Operation should have completed");
    }

    #[tokio::test]
    async fn test_operation_cancellation() {
        let manager = BackgroundTaskManager::new();
        
        // Start a long-running operation
        manager.start_operation(
            "long_op".to_string(),
            "Long operation".to_string(),
            |_event_tx, _operation_id| async move {
                sleep(Duration::from_secs(10)).await; // Long operation
                Ok(())
            }
        ).await.unwrap();

        // Cancel it immediately
        manager.cancel_operation("long_op").await.unwrap();

        // Check status
        let status = manager.get_status("long_op").await.unwrap();
        matches!(status, OperationStatus::Cancelled);
    }
}

#[allow(async_fn_in_trait)]
pub trait BackgroundOperations {
    async fn start_comprehensive_analysis_wrapper(&mut self);
}

impl BackgroundOperations for App {
    async fn start_comprehensive_analysis_wrapper(&mut self) {
        // Check if already processing to avoid multiple concurrent analyses
        if matches!(self.current_state, AppState::Loading)
            || self.comprehensive_analysis_state.is_some()
        {
            return;
        }

        // IMMEDIATELY set loading state and create analysis state
        self.current_state = AppState::Loading;
        self.message = Some("🚀 Iniciando análisis completo con Gemini AI...".to_string());

        // Create shared state for the analysis
        let analysis_state = ComprehensiveAnalysisState {
            status: Arc::new(Mutex::new(
                "🔍 Analizando cambios en el repositorio...".to_string(),
            )),
            finished: Arc::new(Mutex::new(false)),
            success: Arc::new(Mutex::new(true)),
            result: Arc::new(Mutex::new(serde_json::Value::Null)),
        };

        // Start the analysis in a background thread
        self.start_comprehensive_analysis(analysis_state.clone());

        // Store the analysis state so the main loop can poll it
        self.comprehensive_analysis_state = Some(analysis_state);
    }
}

impl App {
    pub fn start_comprehensive_analysis(&self, analysis_state: ComprehensiveAnalysisState) {
        // Clone data needed for the thread
        let config_clone = self.config.clone();

        // Clone analysis state components
        let status_clone = analysis_state.status.clone();
        let finished_clone = analysis_state.finished.clone();
        let success_clone = analysis_state.success.clone();
        let result_clone = analysis_state.result.clone();

        // Spawn the analysis in a background thread
        thread::spawn(move || {
            // Update status: analyzing changes
            if let Ok(mut status) = status_clone.lock() {
                *status = "🔍 Analizando cambios en el repositorio...".to_string();
            }

            // Get git changes
            let git_repo = match GitRepo::new() {
                Ok(repo) => repo,
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error accediendo al repositorio: {}", e);
                    }
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                    if let Ok(mut finished) = finished_clone.lock() {
                        *finished = true;
                    }
                    return;
                }
            };

            let changes = match git_repo.get_detailed_changes() {
                Ok(changes) => changes,
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error obteniendo cambios: {}", e);
                    }
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                    if let Ok(mut finished) = finished_clone.lock() {
                        *finished = true;
                    }
                    return;
                }
            };

            // If no meaningful changes, skip Gemini call
            if changes.trim() == "No hay cambios detectados en el repositorio." {
                if let Ok(mut result) = result_clone.lock() {
                    *result = serde_json::json!({
                        "title": "sin cambios",
                        "commitType": "chore",
                        "description": "No hay cambios para describir.",
                        "scope": "general",
                        "securityAnalysis": "",
                        "breakingChanges": "",
                        "testAnalysis": ""
                    });
                }
                if let Ok(mut status) = status_clone.lock() {
                    *status = "✅ No hay cambios para describir".to_string();
                }
                if let Ok(mut finished) = finished_clone.lock() {
                    *finished = true;
                }
                return;
            }

            // Update status: connecting to Gemini
            if let Ok(mut status) = status_clone.lock() {
                *status = "🌐 Conectando con Gemini AI...".to_string();
            }

            // Create Gemini client
            let gemini_client = match GeminiClient::new(&config_clone) {
                Ok(client) => client,
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error conectando con Gemini: {}", e);
                    }
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                    if let Ok(mut finished) = finished_clone.lock() {
                        *finished = true;
                    }
                    return;
                }
            };

            // Update status: generating comprehensive analysis
            if let Ok(mut status) = status_clone.lock() {
                *status = "🧠 Generando análisis completo de commit...".to_string();
            }

            // Make the async Gemini call in a blocking context
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error creando runtime: {}", e);
                    }
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                    if let Ok(mut finished) = finished_clone.lock() {
                        *finished = true;
                    }
                    return;
                }
            };

            // Run the comprehensive analysis
            let result = rt.block_on(async {
                gemini_client
                    .generate_comprehensive_commit_analysis(&changes)
                    .await
            });

            // Handle the result
            match result {
                Ok(json_result) => {
                    if let Ok(mut result) = result_clone.lock() {
                        *result = json_result;
                    }
                    if let Ok(mut status) = status_clone.lock() {
                        *status = "✅ Análisis completo completado exitosamente".to_string();
                    }
                }
                Err(e) => {
                    // Log error to debug file instead of screen
                    utils::log_error("BACKGROUND", &e);
                    // Fallback to basic result
                    if let Ok(mut result) = result_clone.lock() {
                        *result = serde_json::json!({
                            "title": "cambios realizados en el código",
                            "commitType": "chore",
                            "description": "Se realizaron cambios en el código del proyecto. No se pudo generar un análisis detallado automáticamente.",
                            "scope": "general",
                            "securityAnalysis": "",
                            "breakingChanges": "",
                            "testAnalysis": ""
                        });
                    }
                    if let Ok(mut status) = status_clone.lock() {
                        *status = "⚠️ Análisis completado con resultado básico".to_string();
                    }
                }
            }

            // Mark as finished
            if let Ok(mut finished) = finished_clone.lock() {
                *finished = true;
            }
        });
    }
}
