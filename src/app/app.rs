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
use log::{debug, info};

use crate::{
    config::load_config,
    git::GitRepo,
    types::{AppConfig, AppScreen, AppState, CommitForm, GeminiAnalysisState, ReleaseNotesAnalysisState, MondayTask},
    ui::{self, UIState},
};

pub struct App {
    pub config: AppConfig,
    pub current_screen: AppScreen,
    pub current_state: AppState,
    pub ui_state: UIState,
    pub commit_form: CommitForm,
    pub tasks: Vec<MondayTask>,
    pub selected_tasks: Vec<MondayTask>,
    pub message: Option<String>,
    pub should_quit: bool,
    pub preview_commit_message: String,
    pub gemini_analysis_state: Option<GeminiAnalysisState>,
    pub release_notes_analysis_state: Option<ReleaseNotesAnalysisState>,
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
            tasks: Vec::new(),
            selected_tasks: Vec::new(),
            message: None,
            should_quit: false,
            preview_commit_message: String::new(),
            gemini_analysis_state: None,
            release_notes_analysis_state: None,
        })
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
            // Check for completed Gemini analysis
            if let Some(analysis_state) = &self.gemini_analysis_state {
                let is_finished = analysis_state.finished.lock().map(|f| *f).unwrap_or(false);
                
                if is_finished {
                    // Analysis completed - update form and reset state
                    if let Ok(result) = analysis_state.result.lock() {
                        self.commit_form.description = result.clone();
                    }
                    
                    if let Ok(security) = analysis_state.security.lock() {
                        if !security.is_empty() {
                            self.commit_form.security = security.clone();
                        }
                    }
                    
                    if let Ok(breaking) = analysis_state.breaking.lock() {
                        if !breaking.is_empty() {
                            self.commit_form.breaking_change = breaking.clone();
                        }
                    }
                    
                    let success = analysis_state.success.lock().map(|s| *s).unwrap_or(false);
                    if success {
                        self.current_state = AppState::Normal;
                        self.message = Some("✅ Análisis completado - Campos actualizados automáticamente".to_string());
                    } else {
                        let status = analysis_state.status.lock().map(|s| s.clone()).unwrap_or("Error desconocido".to_string());
                        self.current_state = AppState::Error(format!("Error en análisis: {}", status));
                    }
                    
                    self.gemini_analysis_state = None;
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
            
            // Get git status for help screen
            let git_status = if self.current_screen == AppScreen::Main {
                GitRepo::new().ok().and_then(|repo| repo.get_status().ok())
            } else {
                None
            };

            terminal.draw(|f| {
                ui::draw(
                    f,
                    &self.current_screen,
                    &self.current_state,
                    &mut self.ui_state,
                    &self.commit_form,
                    &self.tasks,
                    self.message.as_deref(),
                    git_status.as_ref(),
                )
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