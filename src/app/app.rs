use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

use crate::{
    config::load_config,
    git::GitRepo,
    types::{AppConfig, AppScreen, AppState, CommitForm, ReleaseNotesAnalysisState, ComprehensiveAnalysisState, SemanticReleaseState, MondayTask, JiraTask},
    ui::UIState,
};

pub struct App {
    pub config: AppConfig,
    pub current_screen: AppScreen,
    pub current_state: AppState,
    pub ui_state: UIState,
    pub commit_form: CommitForm,
    pub monday_tasks: Vec<MondayTask>,
    pub jira_tasks: Vec<JiraTask>,
    pub selected_monday_tasks: Vec<MondayTask>,
    pub selected_jira_tasks: Vec<JiraTask>,
    pub message: Option<String>,
    pub should_quit: bool,
    pub preview_commit_message: String,
    pub release_notes_analysis_state: Option<ReleaseNotesAnalysisState>,
    pub comprehensive_analysis_state: Option<ComprehensiveAnalysisState>,
    pub semantic_release_state: Option<SemanticReleaseState>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = load_config().unwrap_or_default();
        
        Ok(Self {
            config,
            current_screen: AppScreen::Main,
            current_state: AppState::Normal,
            ui_state: UIState::default(),
            commit_form: CommitForm::default(),
            monday_tasks: Vec::new(),
            jira_tasks: Vec::new(),
            selected_monday_tasks: Vec::new(),
            selected_jira_tasks: Vec::new(),
            message: None,
            should_quit: false,
            preview_commit_message: String::new(),
            release_notes_analysis_state: None,
            comprehensive_analysis_state: None,
            semantic_release_state: None,
        })
    }

    pub fn new_for_background(config: &AppConfig) -> Self {
        Self {
            config: config.clone(),
            current_screen: AppScreen::Main,
            current_state: AppState::Normal,
            ui_state: UIState::default(),
            commit_form: CommitForm::default(),
            monday_tasks: Vec::new(),
            jira_tasks: Vec::new(),
            selected_monday_tasks: Vec::new(),
            selected_jira_tasks: Vec::new(),
            message: None,
            should_quit: false,
            preview_commit_message: String::new(),
            release_notes_analysis_state: None,
            comprehensive_analysis_state: None,
            semantic_release_state: None,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {

            
            // Check for completed Release Notes analysis
            if let Some(analysis_state) = &self.release_notes_analysis_state {
                let is_finished = analysis_state.finished.lock().map(|f| *f).unwrap_or(false);
                
                if is_finished {
                    let success = analysis_state.success.lock().map(|s| *s).unwrap_or(false);
                    if success {
                        self.current_state = AppState::Normal;
                        self.message = Some("✅ Notas de versión generadas internamente exitosamente".to_string());
                        self.current_screen = AppScreen::Main;
                    } else {
                        let status = analysis_state.status.lock().map(|s| s.clone()).unwrap_or("Error desconocido".to_string());
                        self.current_state = AppState::Error(format!("Error en generación: {}", status));
                    }
                    
                    self.release_notes_analysis_state = None;
                } else {
                    // Update status message if it changed
                    if let Ok(status) = analysis_state.status.lock() {
                        let current_message = self.message.as_deref().unwrap_or("");
                        if *status != current_message {
                            self.message = Some(status.clone());
                        }
                    }
                }
            }
            
            // Check for completed Comprehensive Analysis
            if let Some(analysis_state) = &self.comprehensive_analysis_state {
                let is_finished = analysis_state.finished.lock().map(|f| *f).unwrap_or(false);
                
                if is_finished {
                    let success = analysis_state.success.lock().map(|s| *s).unwrap_or(false);
                    if success {
                        // Extract results from JSON and populate form
                        if let Ok(result) = analysis_state.result.lock() {
                            // Parse and populate all fields from the JSON response
                            if let Some(title) = result.get("title").and_then(|v| v.as_str()) {
                                if !title.is_empty() {
                                    self.commit_form.title = title.to_string();
                                }
                            }
                            
                            if let Some(commit_type) = result.get("commitType").and_then(|v| v.as_str()) {
                                if !commit_type.is_empty() {
                                    use crate::types::CommitType;
                                    let commit_type_enum = match commit_type {
                                        "feat" => Some(CommitType::Feat),
                                        "fix" => Some(CommitType::Fix),
                                        "docs" => Some(CommitType::Docs),
                                        "style" => Some(CommitType::Style),
                                        "refactor" => Some(CommitType::Refactor),
                                        "perf" => Some(CommitType::Perf),
                                        "test" => Some(CommitType::Test),
                                        "chore" => Some(CommitType::Chore),
                                        "revert" => Some(CommitType::Revert),
                                        _ => None,
                                    };
                                    
                                    if let Some(ct) = commit_type_enum {
                                        self.commit_form.commit_type = Some(ct.clone());
                                        // Update UI state to reflect the selected commit type
                                        let commit_types = CommitType::all();
                                        if let Some(index) = commit_types.iter().position(|t| *t == ct) {
                                            self.ui_state.selected_commit_type = index;
                                        }
                                    }
                                }
                            }
                            
                            if let Some(description) = result.get("description").and_then(|v| v.as_str()) {
                                if !description.is_empty() {
                                    self.commit_form.description = description.to_string();
                                }
                            }
                            
                            if let Some(scope) = result.get("scope").and_then(|v| v.as_str()) {
                                if !scope.is_empty() && scope != "general" {
                                    self.commit_form.scope = scope.to_string();
                                }
                            }
                            
                            if let Some(security) = result.get("securityAnalysis").and_then(|v| v.as_str()) {
                                if !security.is_empty() {
                                    self.commit_form.security = security.to_string();
                                }
                            }
                            
                            if let Some(breaking) = result.get("breakingChanges").and_then(|v| v.as_str()) {
                                if !breaking.is_empty() {
                                    self.commit_form.breaking_change = breaking.to_string();
                                }
                            }
                        }
                        
                        self.current_state = AppState::Normal;
                        self.message = Some("✅ Análisis completo completado - Todos los campos actualizados automáticamente".to_string());
                    } else {
                        let status = analysis_state.status.lock().map(|s| s.clone()).unwrap_or("Error desconocido".to_string());
                        self.current_state = AppState::Error(format!("Error en análisis completo: {}", status));
                    }
                    
                    self.comprehensive_analysis_state = None;
                } else {
                    // Update status message if it changed
                    if let Ok(status) = analysis_state.status.lock() {
                        let current_message = self.message.as_deref().unwrap_or("");
                        if *status != current_message {
                            self.message = Some(status.clone());
                        }
                    }
                }
            }
            
            // Check for completed Semantic Release operation
            if let Some(release_state) = &self.semantic_release_state {
                let is_finished = release_state.finished.lock().map(|f| *f).unwrap_or(false);
                
                if is_finished {
                    let _success = release_state.success.lock().map(|s| *s).unwrap_or(false);
                    // Stay on semantic release screen to show results
                    self.current_state = AppState::Normal;
                    let status = release_state.status.lock().map(|s| s.clone()).unwrap_or("Completado".to_string());
                    self.message = Some(status);
                    // Don't clear the state or change screen - keep results visible
                } else {
                    // Update status message if it changed
                    if let Ok(status) = release_state.status.lock() {
                        let current_message = self.message.as_deref().unwrap_or("");
                        if *status != current_message {
                            self.message = Some(status.clone());
                        }
                    }
                }
            }
            
            // Get git status for help screen
            let git_status = if self.current_screen == AppScreen::Main {
                GitRepo::new().ok().and_then(|repo| repo.get_status().ok())
            } else {
                None
            };

            terminal.draw(|f| {
                crate::ui::draw(
                    f,
                    &self.current_screen,
                    &self.current_state,
                    &mut self.ui_state,
                    &self.commit_form,
                    &self.monday_tasks,
                    &self.jira_tasks,
                    &self.config,
                    self.message.as_deref(),
                    git_status.as_ref(),
                    self.semantic_release_state.as_ref(),
                );
            })?;

            if self.should_quit {
                break;
            }

            // Use timeout to allow animation updates during loading
            let timeout = if matches!(self.current_state, AppState::Loading) {
                std::time::Duration::from_millis(100) // 10 FPS for smooth animation
            } else {
                std::time::Duration::from_millis(1000) // 1 FPS when not loading
            };

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        use crate::app::event_handlers::EventHandlers;
        self.handle_key_event_impl(key).await
    }
} 