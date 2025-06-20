use std::sync::Arc;
use async_broadcast::{broadcast, Receiver, Sender};
use serde_json::Value;
use tokio::sync::RwLock;

use tracing::{error, info, instrument, warn};

use crate::{
    app::App,
    app::release_notes::generate_release_notes_task,
    git::repository::GitRepo,
    types::{AppState, AppConfig, GitCommit},
    error::Result,
};

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

    // General operation status
    OperationStarted { operation_id: String },
    OperationCompleted { operation_id: String },
}

/// Status of a background operation
#[derive(Debug, Clone)]
pub enum OperationStatus {
    NotStarted,
    Running,
    Completed,
    Failed,
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
                status_map.insert(operation_id_for_task.clone(), OperationStatus::Running);
            }

            // Execute the operation
            let result = operation(event_tx.clone(), operation_id_for_task.clone()).await;

            // Update final status and emit completion event
            match result {
                Ok(()) => {
                    {
                        let mut status_map = operation_status.write().await;
                        status_map.insert(operation_id_for_task.clone(), OperationStatus::Completed);
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
                        status_map.insert(operation_id_for_task.clone(), OperationStatus::Failed);
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
    #[instrument(skip(self))]
    pub async fn start_comprehensive_analysis(
        &self,
        config: &AppConfig,
        _commits: Vec<GitCommit>,
    ) -> Result<String> {
        let operation_id = format!("comprehensive_analysis_{}", uuid::Uuid::new_v4());
        
        let config_clone = config.clone();

        // Start the actual comprehensive analysis task using start_operation
        self.start_operation(
            operation_id.clone(),
            "Comprehensive AI Analysis".to_string(),
            move |event_tx, _op_id| async move {
                // Import necessary types
                use crate::git::GitRepo;
                use crate::services::GeminiClient;

                // Broadcast progress
                if let Err(e) = event_tx.broadcast(BackgroundEvent::AnalysisProgress(
                    "Analyzing git repository changes...".to_string()
                )).await {
                    warn!("Failed to broadcast analysis progress: {}", e);
                }

                // Get git changes
                let git_repo = GitRepo::new()?;
                let changes = git_repo.get_detailed_changes()?;

                // Check if there are actually any git changes to analyze
                // The function returns either actual diff content or a message about no changes
                let has_changes = !changes.trim().is_empty() && 
                    !changes.contains("No hay cambios detectados en el repositorio");

                if !has_changes {
                    if let Err(e) = event_tx.broadcast(BackgroundEvent::AnalysisError(
                        "No git changes found to analyze".to_string()
                    )).await {
                        warn!("Failed to broadcast analysis error: {}", e);
                    }
                    return Err(crate::error::SemanticReleaseError::git_error(
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "No git changes found to analyze"
                        )
                    ));
                }

                // Broadcast progress
                if let Err(e) = event_tx.broadcast(BackgroundEvent::AnalysisProgress(
                    "Connecting to Gemini AI...".to_string()
                )).await {
                    warn!("Failed to broadcast analysis progress: {}", e);
                }

                // Create Gemini client and run analysis
                let gemini_client = GeminiClient::new(&config_clone)?;

                // Broadcast progress
                if let Err(e) = event_tx.broadcast(BackgroundEvent::AnalysisProgress(
                    "Generating comprehensive commit analysis...".to_string()
                )).await {
                    warn!("Failed to broadcast analysis progress: {}", e);
                }

                let result = gemini_client
                    .generate_comprehensive_commit_analysis(&changes)
                    .await?;

                // Broadcast completion with the full result
                if let Err(e) = event_tx.broadcast(BackgroundEvent::AnalysisCompleted(result)).await {
                    warn!("Failed to broadcast analysis completion: {}", e);
                }

                Ok(())
            }
        ).await?;

        Ok(operation_id)
    }


}

impl Default for BackgroundTaskManager {
    fn default() -> Self {
        Self::new()
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


}

impl ComprehensiveAnalysisOperations for App {
    async fn handle_comprehensive_analysis(&mut self) -> Result<()> {
        // Check if already processing to avoid multiple concurrent analyses
        if matches!(self.current_state, AppState::Loading) {
            return Ok(());
        }

        // Set to loading state
        self.current_state = AppState::Loading;
        self.message = Some("🤖 Iniciando análisis completo con IA...".to_string());

        // Get commits since last tag for analysis
        let git_repo = GitRepo::new()?;
        let last_tag = git_repo.get_last_tag()?;
        let commits = git_repo.get_commits_since_tag(last_tag.as_deref())?;
        
        if let Some(tag) = &last_tag {
            info!("Running comprehensive analysis for commits since tag: {}", tag);
        } else {
            info!("No previous tag found, analyzing all commits");
        }

        // Start comprehensive analysis using background task manager
        match self.background_task_manager.start_comprehensive_analysis(&self.config, commits).await {
            Ok(_operation_id) => {
                info!("Comprehensive analysis started via BackgroundTaskManager");
            },
            Err(e) => {
                self.current_state = AppState::Error(format!("Error iniciando análisis: {}", e));
                self.message = Some(format!("❌ {}", e));
            }
        }

        Ok(())
    }
}

// Define the trait that was removed
#[allow(async_fn_in_trait)]
pub trait ComprehensiveAnalysisOperations {
    async fn handle_comprehensive_analysis(&mut self) -> Result<()>;
}
