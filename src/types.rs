use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::app::background_operations::{BackgroundEvent, OperationStatus, BackgroundTaskManager};
use async_broadcast::Receiver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MondayTask {
    pub id: String,
    pub title: String,
    pub board_id: Option<String>,
    pub board_name: Option<String>,
    pub url: String,
    pub state: String,
    pub updates: Vec<MondayUpdate>,
    pub group_title: Option<String>,
    pub column_values: Vec<MondayColumnValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MondayColumnValue {
    pub id: String,
    pub column_type: String,
    pub text: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MondayUpdate {
    pub id: String,
    pub body: String,
    pub created_at: String,
    pub creator: Option<MondayUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MondayUser {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTask {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub issue_type: String,
    pub status: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub reporter: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub project_key: String,
    pub project_name: String,
    pub components: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUser {
    pub account_id: String,
    pub display_name: String,
    pub email_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTaskMention {
    pub key: String,
    pub summary: String,
}

// Unified task interface
pub trait TaskLike {
    fn get_id(&self) -> &str;
    fn get_title(&self) -> &str;
}

impl TaskLike for MondayTask {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_title(&self) -> &str {
        &self.title
    }
}

impl TaskLike for JiraTask {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_title(&self) -> &str {
        &self.summary
    }
}

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub description: String,
    pub author_name: String,
    pub author_email: String,
    pub commit_date: chrono::DateTime<chrono::FixedOffset>,
    pub commit_type: Option<String>,
    pub scope: Option<String>,
    pub body: String,
    pub breaking_changes: Vec<String>,
    pub test_details: Vec<String>,
    pub security: Option<String>,
    pub monday_tasks: Vec<String>,
    pub monday_task_mentions: Vec<MondayTaskMention>,
    pub jira_tasks: Vec<String>,
    pub jira_task_mentions: Vec<JiraTaskMention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MondayTaskMention {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommitType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Chore,
    Revert,
}

impl CommitType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommitType::Feat => "feat",
            CommitType::Fix => "fix",
            CommitType::Docs => "docs",
            CommitType::Style => "style",
            CommitType::Refactor => "refactor",
            CommitType::Perf => "perf",
            CommitType::Test => "test",
            CommitType::Chore => "chore",
            CommitType::Revert => "revert",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CommitType::Feat => "A new feature",
            CommitType::Fix => "A bug fix",
            CommitType::Docs => "Documentation only changes",
            CommitType::Style => "Code style changes (formatting, etc)",
            CommitType::Refactor => "Code changes that neither fix bugs nor add features",
            CommitType::Perf => "Performance improvements",
            CommitType::Test => "Adding or fixing tests",
            CommitType::Chore => "Changes to the build process or auxiliary tools",
            CommitType::Revert => "Revert to a commit",
        }
    }

    pub fn all() -> Vec<CommitType> {
        vec![
            CommitType::Feat,
            CommitType::Fix,
            CommitType::Docs,
            CommitType::Style,
            CommitType::Refactor,
            CommitType::Perf,
            CommitType::Test,
            CommitType::Chore,
            CommitType::Revert,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub monday_api_key: Option<String>,
    pub monday_account_slug: Option<String>,
    pub monday_board_id: Option<String>,
    pub monday_url_template: Option<String>,
    pub jira_url: Option<String>,
    pub jira_username: Option<String>,
    pub jira_api_token: Option<String>,
    pub jira_project_key: Option<String>,
    pub gemini_token: Option<String>,
}

impl AppConfig {
    pub fn is_monday_configured(&self) -> bool {
        self.monday_api_key.is_some() && self.monday_account_slug.is_some()
    }

    pub fn is_jira_configured(&self) -> bool {
        self.jira_url.is_some() && self.jira_username.is_some() && self.jira_api_token.is_some()
    }

    pub fn get_task_system(&self) -> TaskSystem {
        if self.is_monday_configured() {
            TaskSystem::Monday
        } else if self.is_jira_configured() {
            TaskSystem::Jira
        } else {
            TaskSystem::None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskSystem {
    Monday,
    Jira,
    None,
}

#[derive(Debug, Clone, Default)]
pub struct CommitForm {
    pub commit_type: Option<CommitType>,
    pub scope: String,
    pub title: String,
    pub description: String,
    pub breaking_change: String,
    pub test_details: String,
    pub security: String,
    pub migraciones_lentas: String,
    pub partes_a_ejecutar: String,
    pub selected_tasks: Vec<MondayTask>, // Unified interface for now
    pub selected_monday_tasks: Vec<MondayTask>,
    pub selected_jira_tasks: Vec<JiraTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Main,
    Config,
    Commit,
    CommitPreview,
    ReleaseNotes,
    SemanticRelease,
    TaskSearch,
}

#[derive(Debug, Clone)]
pub enum AppState {
    Normal,
    Loading,
    Error(String),
    ConfirmingStageAll,
}

// Modern async-friendly background operation state
#[derive(Debug, Clone)]
pub struct AsyncOperationState {
    pub operation_id: String,
    pub task_manager: Arc<BackgroundTaskManager>,
}

impl AsyncOperationState {
    pub fn new(operation_id: String, task_manager: Arc<BackgroundTaskManager>) -> Self {
        Self {
            operation_id,
            task_manager,
        }
    }

    pub async fn get_status(&self) -> Option<OperationStatus> {
        self.task_manager.get_status(&self.operation_id).await
    }

    pub fn subscribe_to_events(&self) -> Receiver<BackgroundEvent> {
        self.task_manager.subscribe()
    }

    pub async fn cancel(&self) -> crate::error::Result<()> {
        self.task_manager.cancel_operation(&self.operation_id).await
    }
}

// Keep SemanticReleaseState for UI display compatibility
#[derive(Debug, Clone)]
pub struct SemanticReleaseState {
    pub status: Arc<Mutex<String>>,
    pub finished: Arc<Mutex<bool>>,
    pub success: Arc<Mutex<bool>>,
    pub result: Arc<Mutex<String>>,
}

impl Default for SemanticReleaseState {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new("Ready".to_string())),
            finished: Arc::new(Mutex::new(false)),
            success: Arc::new(Mutex::new(false)),
            result: Arc::new(Mutex::new(String::new())),
        }
    }
}

// Modern async equivalents - these replace the legacy structs completely
#[derive(Debug, Clone)]
pub struct AsyncReleaseNotesState {
    pub operation_state: AsyncOperationState,
}

impl AsyncReleaseNotesState {
    pub fn new(task_manager: Arc<BackgroundTaskManager>) -> Self {
        let operation_id = format!("release_notes_{}", uuid::Uuid::new_v4());
        Self {
            operation_state: AsyncOperationState::new(operation_id, task_manager),
        }
    }

    pub async fn is_running(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Running { .. })
        )
    }

    pub async fn is_finished(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Completed { .. } | OperationStatus::Failed { .. } | OperationStatus::Cancelled)
        )
    }

    pub async fn is_successful(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Completed { .. })
        )
    }
}

#[derive(Debug, Clone)]
pub struct AsyncComprehensiveAnalysisState {
    pub operation_state: AsyncOperationState,
}

impl AsyncComprehensiveAnalysisState {
    pub fn new(task_manager: Arc<BackgroundTaskManager>) -> Self {
        let operation_id = format!("comprehensive_analysis_{}", uuid::Uuid::new_v4());
        Self {
            operation_state: AsyncOperationState::new(operation_id, task_manager),
        }
    }

    pub async fn is_running(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Running { .. })
        )
    }

    pub async fn is_finished(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Completed { .. } | OperationStatus::Failed { .. } | OperationStatus::Cancelled)
        )
    }

    pub async fn is_successful(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Completed { .. })
        )
    }
}

#[derive(Debug, Clone)]
pub struct AsyncSemanticReleaseState {
    pub operation_state: AsyncOperationState,
}

impl AsyncSemanticReleaseState {
    pub fn new(task_manager: Arc<BackgroundTaskManager>) -> Self {
        let operation_id = format!("semantic_release_{}", uuid::Uuid::new_v4());
        Self {
            operation_state: AsyncOperationState::new(operation_id, task_manager),
        }
    }

    pub async fn is_running(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Running { .. })
        )
    }

    pub async fn is_finished(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Completed { .. } | OperationStatus::Failed { .. } | OperationStatus::Cancelled)
        )
    }

    pub async fn is_successful(&self) -> bool {
        matches!(
            self.operation_state.get_status().await,
            Some(OperationStatus::Completed { .. })
        )
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub next_version: String,
    pub current_version: Option<String>,
    pub version_type: VersionType,
    pub commit_count: usize,
    pub has_unreleased_changes: bool,
    pub dry_run_output: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VersionType {
    Major,
    Minor,
    Patch,
    None,
    Unknown,
}

impl std::fmt::Display for VersionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionType::Major => write!(f, "Major"),
            VersionType::Minor => write!(f, "Minor"),
            VersionType::Patch => write!(f, "Patch"),
            VersionType::None => write!(f, "No Release"),
            VersionType::Unknown => write!(f, "Unknown"),
        }
    }
}
