use crate::error::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
};
use ratatui::{
    backend::Backend,
    Terminal,
};
use tracing::{info, instrument};

use crate::{
    app::background_operations::BackgroundTaskManager,
    config::load_config,
    types::{
        AppConfig, AppScreen, AppState, CommitForm, JiraTask,
        MondayTask, SemanticReleaseState,
    },
    ui::UIState,
};

#[derive(Debug)]
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
    
    // Modern async background operations
    pub background_task_manager: BackgroundTaskManager,
    
    // Keep semantic_release_state for UI results display
    pub semantic_release_state: Option<SemanticReleaseState>,
}

impl App {
    #[instrument]
    pub async fn new() -> Result<Self> {
        info!("Initializing new app instance");
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
            
            // Initialize modern async background operations
            background_task_manager: BackgroundTaskManager::new(),
            
            // Keep for UI display
            semantic_release_state: None,
        })
    }

    #[instrument(skip_all)]
    pub async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        info!("Starting main application loop");
        
        // Subscribe to background events
        let mut event_rx = self.background_task_manager.subscribe();
        
        loop {
            // Handle background task events (modern async approach)
            while let Ok(event) = event_rx.try_recv() {
                use crate::app::background_operations::BackgroundEvent;
                match event {
                    BackgroundEvent::ReleaseNotesProgress(status) => {
                        self.message = Some(format!("🔄 {}", status));
                    }
                    BackgroundEvent::ReleaseNotesCompleted(result) => {
                        // Extract and display results
                        if let Some(notes) = result.get("notes").and_then(|v| v.as_str()) {
                            self.message = Some("✅ Notas de versión generadas exitosamente".to_string());
                            self.current_state = AppState::Normal;
                            self.current_screen = AppScreen::Main;
                            tracing::info!("Release notes completed: {} characters", notes.len());
                        } else {
                            // Handle case where result is a direct string
                            self.message = Some("✅ Notas de versión generadas exitosamente".to_string());
                            self.current_state = AppState::Normal;
                            self.current_screen = AppScreen::Main;
                            tracing::info!("Release notes completed: {:?}", result);
                        }
                    }
                    BackgroundEvent::ReleaseNotesError(error) => {
                        self.current_state = AppState::Error(format!("Error en generación: {}", error));
                        self.message = Some(format!("❌ {}", error));
                    }
                    BackgroundEvent::AnalysisProgress(status) => {
                        self.message = Some(format!("🤖 {}", status));
                    }
                    BackgroundEvent::AnalysisCompleted(result) => {
                        self.current_state = AppState::Normal;
                        self.message = Some("✅ Análisis completado - Formulario poblado automáticamente".to_string());
                        
                        // Populate commit form with analysis results
                        if let Some(title) = result.get("title").and_then(|v| v.as_str()) {
                            if !title.is_empty() {
                                self.commit_form.title = title.to_string();
                                // Update the UI textarea as well
                                self.ui_state.title_textarea.select_all();
                                self.ui_state.title_textarea.delete_str(self.ui_state.title_textarea.lines().join("\n").len());
                                self.ui_state.title_textarea.insert_str(title);
                            }
                        }

                        if let Some(scope) = result.get("scope").and_then(|v| v.as_str()) {
                            if !scope.is_empty() {
                                self.commit_form.scope = scope.to_string();
                                // Update the UI textarea as well
                                self.ui_state.scope_textarea.select_all();
                                self.ui_state.scope_textarea.delete_str(self.ui_state.scope_textarea.lines().join("\n").len());
                                self.ui_state.scope_textarea.insert_str(scope);
                            }
                        }

                        if let Some(description) = result.get("description").and_then(|v| v.as_str()) {
                            if !description.is_empty() {
                                self.commit_form.description = description.to_string();
                                // Update the UI textarea as well
                                self.ui_state.description_textarea.select_all();
                                self.ui_state.description_textarea.delete_str(self.ui_state.description_textarea.lines().join("\n").len());
                                self.ui_state.description_textarea.insert_str(description);
                            }
                        }

                        if let Some(commit_type) = result.get("commitType").and_then(|v| v.as_str()) {
                            // Parse the commit type string into CommitType enum
                            let parsed_commit_type = match commit_type {
                                "feat" => Some(crate::types::CommitType::Feat),
                                "fix" => Some(crate::types::CommitType::Fix),
                                "docs" => Some(crate::types::CommitType::Docs),
                                "style" => Some(crate::types::CommitType::Style),
                                "refactor" => Some(crate::types::CommitType::Refactor),
                                "perf" => Some(crate::types::CommitType::Perf),
                                "test" => Some(crate::types::CommitType::Test),
                                "chore" => Some(crate::types::CommitType::Chore),
                                "revert" => Some(crate::types::CommitType::Revert),
                                _ => Some(crate::types::CommitType::Feat), // Default fallback
                            };
                            
                            self.commit_form.commit_type = parsed_commit_type;
                            
                            // Update the selected commit type in UI
                            let commit_types = vec!["feat", "fix", "docs", "style", "refactor", "perf", "test", "chore"];
                            if let Some(index) = commit_types.iter().position(|&t| t == commit_type) {
                                self.ui_state.selected_commit_type = index;
                            }
                        }

                        if let Some(security) = result.get("securityAnalysis").and_then(|v| v.as_str()) {
                            if !security.is_empty() && security != "N/A" {
                                self.commit_form.security = security.to_string();
                                // Update the UI textarea as well
                                self.ui_state.security_textarea.select_all();
                                self.ui_state.security_textarea.delete_str(self.ui_state.security_textarea.lines().join("\n").len());
                                self.ui_state.security_textarea.insert_str(security);
                            }
                        }

                        if let Some(breaking) = result.get("breakingChanges").and_then(|v| v.as_str()) {
                            if !breaking.is_empty() && breaking != "N/A" {
                                self.commit_form.breaking_change = breaking.to_string();
                                // Update the UI textarea as well
                                self.ui_state.breaking_change_textarea.select_all();
                                self.ui_state.breaking_change_textarea.delete_str(self.ui_state.breaking_change_textarea.lines().join("\n").len());
                                self.ui_state.breaking_change_textarea.insert_str(breaking);
                            }
                        }

                        if let Some(test_details) = result.get("testAnalysis").and_then(|v| v.as_str()) {
                            if !test_details.is_empty() && test_details != "N/A" {
                                self.commit_form.test_details = test_details.to_string();
                                // Update the UI textarea as well
                                self.ui_state.test_details_textarea.select_all();
                                self.ui_state.test_details_textarea.delete_str(self.ui_state.test_details_textarea.lines().join("\n").len());
                                self.ui_state.test_details_textarea.insert_str(test_details);
                            }
                        }

                        tracing::info!("Analysis completed and form populated with comprehensive data");
                    }
                    BackgroundEvent::AnalysisError(error) => {
                        self.current_state = AppState::Error(format!("Error en análisis: {}", error));
                        self.message = Some(format!("❌ {}", error));
                    }
                    BackgroundEvent::SemanticReleaseProgress(status) => {
                        self.message = Some(format!("🚀 {}", status));
                    }
                    BackgroundEvent::SemanticReleaseCompleted(result) => {
                        self.current_state = AppState::Normal;
                        self.message = Some("✅ Semantic release completado".to_string());
                        tracing::info!("Semantic release completed: {}", result);
                    }
                    BackgroundEvent::SemanticReleaseError(error) => {
                        self.current_state = AppState::Error(format!("Error en semantic release: {}", error));
                        self.message = Some(format!("❌ {}", error));
                    }
                    BackgroundEvent::OperationStarted { operation_id, description: _ } => {
                        self.current_state = AppState::Loading;
                        tracing::info!("Operation started: {}", operation_id);
                    }
                    BackgroundEvent::OperationCompleted { operation_id } => {
                        self.current_state = AppState::Normal;
                        tracing::info!("Operation completed: {}", operation_id);
                    }
                    BackgroundEvent::OperationCancelled { operation_id } => {
                        self.message = Some("❌ Operación cancelada".to_string());
                        self.current_state = AppState::Normal;
                        tracing::info!("Operation cancelled: {}", operation_id);
                    }
                }
            }

            // Draw UI
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
                    None, // git_status - not needed in the simplified version
                    self.semantic_release_state.as_ref(),
                );
            })?;

            // Handle input events
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        use crate::app::event_handlers::EventHandlers;
                        self.handle_key_event_impl(key).await?;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        info!("Application loop ended");
        Ok(())
    }

    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        use crate::app::event_handlers::EventHandlers;
        self.handle_key_event_impl(key).await
    }

    pub async fn run(mut self) -> Result<()> {
        use crossterm::{
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use ratatui::{
            backend::CrosstermBackend,
            Terminal,
        };
        use std::io;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        result
    }
}
