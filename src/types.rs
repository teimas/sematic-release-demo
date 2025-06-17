use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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

#[derive(Debug, Clone)]
pub struct JiraTaskMention {
    pub key: String,
    pub summary: String,
}

// Unified task interface
pub trait TaskLike {
    fn get_id(&self) -> &str;
    fn get_title(&self) -> &str;
    fn get_display_info(&self) -> String;
}

impl TaskLike for MondayTask {
    fn get_id(&self) -> &str {
        &self.id
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn get_display_info(&self) -> String {
        format!("ID: {} | Board: {} | State: {}", 
            self.id, 
            self.board_name.as_deref().unwrap_or("Unknown"),
            self.state)
    }
}

impl TaskLike for JiraTask {
    fn get_id(&self) -> &str {
        &self.id
    }
    
    fn get_title(&self) -> &str {
        &self.summary
    }
    
    fn get_display_info(&self) -> String {
        format!("Key: {} | Status: {} | Type: {}", 
            self.key,
            self.status,
            self.issue_type)
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
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


#[derive(Debug, Clone)]
#[derive(Default)]
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
    TaskSearch,
}

#[derive(Debug, Clone)]
pub enum AppState {
    Normal,
    Loading,
    Error(String),
    ConfirmingStageAll,
}



#[derive(Debug, Clone)]
pub struct ReleaseNotesAnalysisState {
    pub status: Arc<Mutex<String>>,
    pub finished: Arc<Mutex<bool>>,
    pub success: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisState {
    pub status: Arc<Mutex<String>>,
    pub finished: Arc<Mutex<bool>>,
    pub success: Arc<Mutex<bool>>,
    pub result: Arc<Mutex<serde_json::Value>>,
} 