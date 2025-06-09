use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::{Utc, Local};
use log::{debug, info, error};

use crate::{
    config::load_config,
    gemini::GeminiClient,
    git::{GitRepo, get_next_version},
    monday::MondayClient,
    types::{AppConfig, AppScreen, AppState, CommitForm, CommitType, MondayTask, GeminiAnalysisState, ReleaseNotesAnalysisState},
    ui::{self, UIState},
};

pub struct App {
    config: AppConfig,
    current_screen: AppScreen,
    current_state: AppState,
    ui_state: UIState,
    commit_form: CommitForm,
    tasks: Vec<MondayTask>,
    selected_tasks: Vec<MondayTask>,
    message: Option<String>,
    should_quit: bool,
    preview_commit_message: String,
    gemini_analysis_state: Option<GeminiAnalysisState>,
    release_notes_analysis_state: Option<ReleaseNotesAnalysisState>,
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

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Clear error state on any key press
        if matches!(self.current_state, AppState::Error(_)) {
            self.current_state = AppState::Normal;
            return Ok(());
        }

        match (&self.current_screen, &self.ui_state.input_mode) {
            (_, ui::InputMode::Editing) => {
                self.handle_input_mode(key).await?;
            }
            (AppScreen::Main, _) => {
                self.handle_main_screen(key.code).await?;
            }
            (AppScreen::Config, _) => {
                self.handle_config_screen(key.code).await?;
            }
            (AppScreen::Commit, _) => {
                self.handle_commit_screen(key.code).await?;
            }
            (AppScreen::CommitPreview, _) => {
                self.handle_commit_preview_screen(key.code).await?;
            }
            (AppScreen::ReleaseNotes, _) => {
                self.handle_release_notes_screen(key.code).await?;
            }
            (AppScreen::TaskSearch, _) => {
                self.handle_task_search_screen(key.code).await?;
            }
            (AppScreen::TaskSelection, _) => {
                self.handle_task_selection_screen(key.code).await?;
            }
        }

        Ok(())
    }

    async fn handle_main_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.ui_state.selected_tab = (self.ui_state.selected_tab + 1) % 4;
            }
            KeyCode::BackTab => {
                self.ui_state.selected_tab = if self.ui_state.selected_tab == 0 {
                    3
                } else {
                    self.ui_state.selected_tab - 1
                };
            }
            KeyCode::Enter => {
                match self.ui_state.selected_tab {
                    0 => self.current_screen = AppScreen::Commit,
                    1 => self.current_screen = AppScreen::ReleaseNotes,
                    2 => self.current_screen = AppScreen::Config,
                    3 => {}, // Help - stay here
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_config_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_commit_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            KeyCode::Char('s') => {
                // Search for Monday.com tasks
                self.current_screen = AppScreen::TaskSearch;
                self.ui_state.current_input.clear();
                self.tasks.clear(); // Clear previous search results
                self.ui_state.selected_tab = 0; // Reset selection
                self.message = Some("Search screen - Press 'i' or '/' to start typing".to_string());
            }
            KeyCode::Char('c') => {
                // Generate commit preview
                self.preview_commit_message = self.build_commit_message();
                self.current_screen = AppScreen::CommitPreview;
                self.ui_state.input_mode = crate::ui::InputMode::Editing;
                self.ui_state.current_input = self.preview_commit_message.clone();
                self.ui_state.cursor_position = self.preview_commit_message.len();
                self.message = Some("Review and edit your commit message. Press Ctrl+C to commit, Esc to cancel".to_string());
            }
            KeyCode::Char('t') => {
                // Toggle task management mode
                self.ui_state.task_management_mode = !self.ui_state.task_management_mode;
                if self.ui_state.task_management_mode {
                    self.ui_state.selected_tab = 0; // Reset selection
                    self.message = Some("Task management mode ON - Use ↑↓ to navigate, Delete/Space to remove tasks".to_string());
                } else {
                    self.message = Some("Task management mode OFF".to_string());
                }
            }

            KeyCode::Char('r') => {
                // Generate/regenerate commit description with security and breaking changes analysis
                // Check if already processing to avoid multiple concurrent analyses
                if matches!(self.current_state, AppState::Loading) || self.gemini_analysis_state.is_some() {
                    return Ok(());
                }
                
                // IMMEDIATELY set loading state and create analysis state
                self.current_state = AppState::Loading;
                self.message = Some("🚀 Iniciando análisis inteligente con Gemini AI...".to_string());
                
                // Create shared state for the analysis
                let analysis_state = GeminiAnalysisState {
                    status: Arc::new(Mutex::new("🔍 Analizando cambios en el repositorio...".to_string())),
                    finished: Arc::new(Mutex::new(false)),
                    success: Arc::new(Mutex::new(true)),
                    result: Arc::new(Mutex::new(String::new())),
                    security: Arc::new(Mutex::new(String::new())),
                    breaking: Arc::new(Mutex::new(String::new())),
                };
                
                // Start the analysis in a background thread
                self.start_gemini_analysis(analysis_state.clone());
                
                // Store the analysis state so the main loop can poll it
                self.gemini_analysis_state = Some(analysis_state);
            }
            KeyCode::Tab => {
                // If we're editing, save the current field first
                if self.ui_state.input_mode == crate::ui::InputMode::Editing {
                    self.save_current_field();
                }
                
                // Navigate to next field
                use crate::ui::CommitField;
                self.ui_state.current_field = match self.ui_state.current_field {
                    CommitField::Type => CommitField::Scope,
                    CommitField::Scope => CommitField::Title,
                    CommitField::Title => CommitField::Description,
                    CommitField::Description => CommitField::BreakingChange,
                    CommitField::BreakingChange => CommitField::TestDetails,
                    CommitField::TestDetails => CommitField::Security,
                    CommitField::Security => CommitField::MigracionesLentas,
                    CommitField::MigracionesLentas => CommitField::PartesAEjecutar,
                    CommitField::PartesAEjecutar => CommitField::SelectedTasks,
                    CommitField::SelectedTasks => CommitField::Type,
                };
                
                // Auto-enter edit mode for text fields
                if !matches!(self.ui_state.current_field, CommitField::Type | CommitField::SelectedTasks) {
                    self.ui_state.input_mode = crate::ui::InputMode::Editing;
                    self.ui_state.current_input = match self.ui_state.current_field {
                        crate::ui::CommitField::Scope => self.commit_form.scope.clone(),
                        crate::ui::CommitField::Title => self.commit_form.title.clone(),
                        crate::ui::CommitField::Description => self.commit_form.description.clone(),
                        crate::ui::CommitField::BreakingChange => self.commit_form.breaking_change.clone(),
                        crate::ui::CommitField::TestDetails => self.commit_form.test_details.clone(),
                        crate::ui::CommitField::Security => self.commit_form.security.clone(),
                        crate::ui::CommitField::MigracionesLentas => self.commit_form.migraciones_lentas.clone(),
                        crate::ui::CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar.clone(),
                        _ => String::new(),
                    };
                    // Set cursor position to end of text
                    self.ui_state.cursor_position = self.ui_state.current_input.len();
                } else {
                    // Exiting edit mode when moving to commit type field
                    self.ui_state.input_mode = crate::ui::InputMode::Normal;
                    self.ui_state.current_input.clear();
                }
            }
            KeyCode::BackTab => {
                // If we're editing, save the current field first
                if self.ui_state.input_mode == crate::ui::InputMode::Editing {
                    self.save_current_field();
                }
                
                // Navigate to previous field
                use crate::ui::CommitField;
                self.ui_state.current_field = match self.ui_state.current_field {
                    CommitField::Type => CommitField::SelectedTasks,
                    CommitField::Scope => CommitField::Type,
                    CommitField::Title => CommitField::Scope,
                    CommitField::Description => CommitField::Title,
                    CommitField::BreakingChange => CommitField::Description,
                    CommitField::TestDetails => CommitField::BreakingChange,
                    CommitField::Security => CommitField::TestDetails,
                    CommitField::MigracionesLentas => CommitField::Security,
                    CommitField::PartesAEjecutar => CommitField::MigracionesLentas,
                    CommitField::SelectedTasks => CommitField::PartesAEjecutar,
                };
                
                // Auto-enter edit mode for text fields
                if !matches!(self.ui_state.current_field, CommitField::Type | CommitField::SelectedTasks) {
                    self.ui_state.input_mode = crate::ui::InputMode::Editing;
                    self.ui_state.current_input = match self.ui_state.current_field {
                        crate::ui::CommitField::Scope => self.commit_form.scope.clone(),
                        crate::ui::CommitField::Title => self.commit_form.title.clone(),
                        crate::ui::CommitField::Description => self.commit_form.description.clone(),
                        crate::ui::CommitField::BreakingChange => self.commit_form.breaking_change.clone(),
                        crate::ui::CommitField::TestDetails => self.commit_form.test_details.clone(),
                        crate::ui::CommitField::Security => self.commit_form.security.clone(),
                        crate::ui::CommitField::MigracionesLentas => self.commit_form.migraciones_lentas.clone(),
                        crate::ui::CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar.clone(),
                        _ => String::new(),
                    };
                    // Set cursor position to end of text
                    self.ui_state.cursor_position = self.ui_state.current_input.len();
                } else {
                    // Exiting edit mode when moving to commit type field
                    self.ui_state.input_mode = crate::ui::InputMode::Normal;
                    self.ui_state.current_input.clear();
                }
            }
            KeyCode::Up => {
                if self.ui_state.task_management_mode {
                    // Navigate through selected tasks in task management mode
                    if !self.selected_tasks.is_empty() && self.ui_state.selected_tab > 0 {
                        self.ui_state.selected_tab -= 1;
                    }
                } else {
                    match self.ui_state.current_field {
                        crate::ui::CommitField::Type => {
                            if self.ui_state.selected_commit_type > 0 {
                                self.ui_state.selected_commit_type -= 1;
                            }
                        }
                        crate::ui::CommitField::SelectedTasks => {
                            // Navigate through selected tasks
                            if !self.selected_tasks.is_empty() && self.ui_state.selected_tab > 0 {
                                self.ui_state.selected_tab -= 1;
                            }
                        }
                        _ => {
                            // In text fields, handle cursor movement within text (will be handled by input_mode if editing)
                        }
                    }
                }
            }
            KeyCode::Down => {
                if self.ui_state.task_management_mode {
                    // Navigate through selected tasks in task management mode
                    if !self.selected_tasks.is_empty() && self.ui_state.selected_tab < self.selected_tasks.len().saturating_sub(1) {
                        self.ui_state.selected_tab += 1;
                    }
                } else {
                    match self.ui_state.current_field {
                        crate::ui::CommitField::Type => {
                            let max_types = CommitType::all().len();
                            if self.ui_state.selected_commit_type < max_types - 1 {
                                self.ui_state.selected_commit_type += 1;
                            }
                        }
                        crate::ui::CommitField::SelectedTasks => {
                            // Navigate through selected tasks
                            if !self.selected_tasks.is_empty() && self.ui_state.selected_tab < self.selected_tasks.len().saturating_sub(1) {
                                self.ui_state.selected_tab += 1;
                            }
                        }
                        _ => {
                            // In text fields, handle cursor movement within text (will be handled by input_mode if editing)
                        }
                    }
                }
            }
            KeyCode::Enter => {
                match self.ui_state.current_field {
                    crate::ui::CommitField::Type => {
                        // Select commit type
                        let commit_types = CommitType::all();
                        if let Some(selected_type) = commit_types.get(self.ui_state.selected_commit_type) {
                            self.commit_form.commit_type = Some(selected_type.clone());
                        }
                    }
                    crate::ui::CommitField::SelectedTasks => {
                        // Activate task management mode when on selected tasks field
                        self.ui_state.task_management_mode = true;
                        self.message = Some("Task management mode ON. Use Up/Down to navigate tasks, Delete/r/Space to remove".to_string());
                    }

                    _ => {
                        // Enter edit mode for text fields
                        self.ui_state.input_mode = crate::ui::InputMode::Editing;
                        self.ui_state.current_input = match self.ui_state.current_field {
                            crate::ui::CommitField::Scope => self.commit_form.scope.clone(),
                            crate::ui::CommitField::Title => self.commit_form.title.clone(),
                            crate::ui::CommitField::Description => self.commit_form.description.clone(),
                            crate::ui::CommitField::BreakingChange => self.commit_form.breaking_change.clone(),
                            crate::ui::CommitField::TestDetails => self.commit_form.test_details.clone(),
                            crate::ui::CommitField::Security => self.commit_form.security.clone(),
                            crate::ui::CommitField::MigracionesLentas => self.commit_form.migraciones_lentas.clone(),
                            crate::ui::CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar.clone(),
                            _ => String::new(),
                        };
                        // Set cursor position to end of text
                        self.ui_state.cursor_position = self.ui_state.current_input.len();
                    }
                                }
            }
            KeyCode::Delete | KeyCode::Char(' ') => {
                // Handle task deletion in task management mode OR when on SelectedTasks field
                if (self.ui_state.task_management_mode || self.ui_state.current_field == crate::ui::CommitField::SelectedTasks) && !self.selected_tasks.is_empty() && self.ui_state.selected_tab < self.selected_tasks.len() {
                    self.selected_tasks.remove(self.ui_state.selected_tab);
                    
                    // Adjust selected_tab if we're now past the end
                    if self.ui_state.selected_tab >= self.selected_tasks.len() && !self.selected_tasks.is_empty() {
                        self.ui_state.selected_tab = self.selected_tasks.len() - 1;
                    }
                    
                    self.update_task_selection();
                    self.message = Some("Task removed from selection".to_string());
                }
            }
 
            _ => {}
        }
        Ok(())
    }

    async fn handle_commit_preview_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Cancel commit and go back to commit screen
                self.current_screen = AppScreen::Commit;
                self.ui_state.input_mode = crate::ui::InputMode::Normal;
                self.ui_state.current_input.clear();
                self.message = Some("Commit cancelled".to_string());
            }

            _ => {
                // Handle normal text editing in the preview
                // The input mode handling will take care of character input
            }
        }
        Ok(())
    }

    async fn handle_release_notes_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::Main;
            }
            KeyCode::Enter => {
                // Generate release notes internally with modal overlay (non-blocking)
                // Check if already processing to avoid multiple concurrent generations
                if matches!(self.current_state, AppState::Loading) || self.release_notes_analysis_state.is_some() {
                    return Ok(());
                }
                
                // IMMEDIATELY set loading state and create analysis state
                self.current_state = AppState::Loading;
                self.message = Some("🚀 Iniciando generación de notas de versión...".to_string());
                
                // Create shared state for the analysis
                let analysis_state = ReleaseNotesAnalysisState {
                    status: Arc::new(Mutex::new("📋 Preparando generación de notas de versión...".to_string())),
                    finished: Arc::new(Mutex::new(false)),
                    success: Arc::new(Mutex::new(true)),
                };
                
                // Start the analysis in a background thread
                self.start_release_notes_analysis(analysis_state.clone());
                
                // Store the analysis state so the main loop can poll it
                self.release_notes_analysis_state = Some(analysis_state);
            }
            KeyCode::Char('i') => {
                // Generate release notes internally (using Rust code instead of npm script)
                self.current_state = AppState::Loading;
                
                if let Err(e) = self.generate_release_notes_internal().await {
                    self.current_state = AppState::Error(e.to_string());
                } else {
                    self.current_state = AppState::Normal;
                    self.message = Some("✅ Notas de versión generadas internamente exitosamente".to_string());
                    self.current_screen = AppScreen::Main;
                }
            }
            KeyCode::Char('o') => {
                self.current_state = AppState::Loading;
                
                if let Err(e) = self.generate_release_notes_with_npm().await {
                    self.current_state = AppState::Error(e.to_string());
                } else {
                    self.current_state = AppState::Normal;
                    self.message = Some("✅ Notas de versión generadas exitosamente".to_string());
                    self.current_screen = AppScreen::Main;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_task_search_screen(&mut self, key: KeyCode) -> Result<()> {
        // Check if we're in search input mode (typing in the search box)
        let in_search_input = self.ui_state.input_mode == crate::ui::InputMode::Editing;
        
        // Debug: log key events and current mode
        self.message = Some(format!("DEBUG: Key: {:?}, Mode: {:?}, Input: '{}'", key, self.ui_state.input_mode, self.ui_state.current_input));
        
        if in_search_input {
            // When typing in search box, handle text input
            match key {
                KeyCode::Esc => {
                    // Exit search input mode
                    self.ui_state.input_mode = crate::ui::InputMode::Normal;
                    self.message = Some("Exited search mode".to_string());
                }
                KeyCode::Enter => {
                    // Submit search
                    self.message = Some(format!("DEBUG: Starting search with query: '{}'", self.ui_state.current_input));
                    if !self.ui_state.current_input.is_empty() {
                        self.current_state = AppState::Loading;
                        match self.search_monday_tasks(&self.ui_state.current_input).await {
                            Ok(tasks) => {
                                self.tasks = tasks;
                                self.ui_state.selected_tab = 0;
                                self.current_state = AppState::Normal;
                                // Exit input mode after search
                                self.ui_state.input_mode = crate::ui::InputMode::Normal;
                                // Show feedback message
                                self.message = Some(format!("DEBUG: Search completed! Found {} tasks", self.tasks.len()));
                            }
                            Err(e) => {
                                self.current_state = AppState::Error(format!("DEBUG: Search failed: {}", e));
                            }
                        }
                    } else {
                        self.message = Some("DEBUG: Search query is empty".to_string());
                    }
                }
                KeyCode::Char(c) => {
                    // Add any character to search (including 'r', 'q', etc.)
                    self.ui_state.current_input.push(c);
                }
                KeyCode::Backspace => {
                    self.ui_state.current_input.pop();
                }
                _ => {}
            }
        } else {
            // When not typing in search box, handle navigation and commands
            match key {
                KeyCode::Char('q') => {
                    self.current_screen = AppScreen::Commit;
                }
                KeyCode::Esc => {
                    // Clear search results but keep selected tasks
                    self.tasks.clear();
                    self.ui_state.current_input.clear();
                    self.ui_state.focused_search_index = 0;
                    self.message = Some("Search cleared".to_string());
                }
                KeyCode::Char('i') | KeyCode::Char('/') => {
                    // Enter search input mode
                    self.ui_state.input_mode = crate::ui::InputMode::Editing;
                    self.message = Some("DEBUG: Entered edit mode - you can now type".to_string());
                }
                KeyCode::Enter => {
                    // If there's text in search box, search; otherwise enter input mode
                    if !self.ui_state.current_input.is_empty() {
                        self.current_state = AppState::Loading;
                        match self.search_monday_tasks(&self.ui_state.current_input).await {
                            Ok(tasks) => {
                                self.tasks = tasks;
                                self.ui_state.selected_tab = 0;
                                self.current_state = AppState::Normal;
                                // Show feedback message
                                self.message = Some(format!("Found {} tasks", self.tasks.len()));
                            }
                            Err(e) => {
                                self.current_state = AppState::Error(e.to_string());
                            }
                        }
                    } else {
                        self.ui_state.input_mode = crate::ui::InputMode::Editing;
                    }
                }
                KeyCode::Up => {
                    if !self.tasks.is_empty() {
                        // Navigate through search results when there are search results
                        if self.ui_state.focused_search_index > 0 {
                            self.ui_state.focused_search_index -= 1;
                        }
                    } else if !self.selected_tasks.is_empty() {
                        // Navigate through selected tasks when no search results
                        if self.ui_state.selected_tab > 0 {
                            self.ui_state.selected_tab -= 1;
                        }
                    }
                }
                KeyCode::Down => {
                    if !self.tasks.is_empty() {
                        // Navigate through search results when there are search results
                        if self.ui_state.focused_search_index < self.tasks.len().saturating_sub(1) {
                            self.ui_state.focused_search_index += 1;
                        }
                    } else if !self.selected_tasks.is_empty() {
                        // Navigate through selected tasks when no search results
                        if self.ui_state.selected_tab < self.selected_tasks.len().saturating_sub(1) {
                            self.ui_state.selected_tab += 1;
                        }
                    }
                }
                KeyCode::Delete | KeyCode::Char('r') => {
                    if !self.tasks.is_empty() {
                        // Remove the currently focused search result from selected tasks (if it's selected)
                        if let Some(task) = self.tasks.get(self.ui_state.focused_search_index) {
                            if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                                self.selected_tasks.remove(pos);
                                self.update_task_selection();
                                self.message = Some("Task removed from selection".to_string());
                            } else {
                                self.message = Some("Task is not selected".to_string());
                            }
                        }
                    } else if !self.selected_tasks.is_empty() && self.ui_state.selected_tab < self.selected_tasks.len() {
                        // Remove task from selected tasks list (when navigating selected tasks)
                        self.selected_tasks.remove(self.ui_state.selected_tab);
                        
                        // Adjust selected_tab if we're now past the end
                        if self.ui_state.selected_tab >= self.selected_tasks.len() && !self.selected_tasks.is_empty() {
                            self.ui_state.selected_tab = self.selected_tasks.len() - 1;
                        }
                        
                        self.update_task_selection();
                        self.message = Some("Task removed from selection".to_string());
                    }
                }
                KeyCode::Char(' ') => {
                    if !self.tasks.is_empty() {
                        // Toggle the currently focused search result
                        if let Some(task) = self.tasks.get(self.ui_state.focused_search_index) {
                            if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                                self.selected_tasks.remove(pos);
                                self.message = Some("Task deselected".to_string());
                            } else {
                                self.selected_tasks.push(task.clone());
                                self.message = Some("Task selected".to_string());
                            }
                            self.update_task_selection();
                        }
                    } else if !self.selected_tasks.is_empty() {
                        // When navigating selected tasks, space removes them (same as Delete/r)
                        if self.ui_state.selected_tab < self.selected_tasks.len() {
                            self.selected_tasks.remove(self.ui_state.selected_tab);
                            
                            // Adjust selected_tab if we're now past the end
                            if self.ui_state.selected_tab >= self.selected_tasks.len() && !self.selected_tasks.is_empty() {
                                self.ui_state.selected_tab = self.selected_tasks.len() - 1;
                            }
                            
                            self.update_task_selection();
                            self.message = Some("Task removed from selection".to_string());
                        }
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    // Numeric selection: 1-9 for tasks 0-8, 0 for task 9
                    let index = if c == '0' { 9 } else { (c as usize) - ('1' as usize) };
                    if let Some(task) = self.tasks.get(index) {
                        if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                            self.selected_tasks.remove(pos);
                        } else {
                            self.selected_tasks.push(task.clone());
                        }
                        self.update_task_selection();
                    }
                }
                KeyCode::Backspace => {
                    // Clear search if in normal mode
                    self.ui_state.current_input.clear();
                    self.tasks.clear();
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_task_selection_screen(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = AppScreen::TaskSearch;
            }
            KeyCode::Up => {
                if self.ui_state.selected_tab > 0 {
                    self.ui_state.selected_tab -= 1;
                }
            }
            KeyCode::Down => {
                if self.ui_state.selected_tab < self.tasks.len().saturating_sub(1) {
                    self.ui_state.selected_tab += 1;
                }
            }
            KeyCode::Char(' ') => {
                // Toggle task selection
                if let Some(task) = self.tasks.get(self.ui_state.selected_tab) {
                    if let Some(pos) = self.selected_tasks.iter().position(|t| t.id == task.id) {
                        self.selected_tasks.remove(pos);
                    } else {
                        self.selected_tasks.push(task.clone());
                    }
                }
            }
            KeyCode::Enter => {
                // Confirm selection and return to commit screen
                self.commit_form.selected_tasks = self.selected_tasks.clone();
                
                // Generate scope from selected task IDs
                let task_ids: Vec<String> = self.selected_tasks.iter().map(|t| t.id.clone()).collect();
                if !task_ids.is_empty() {
                    self.commit_form.scope = task_ids.join("|");
                }
                
                self.current_screen = AppScreen::Commit;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_input_mode(&mut self, key: KeyEvent) -> Result<()> {
        // Handle CommitPreview screen differently - it's just a text editor
        if self.current_screen == AppScreen::CommitPreview {
            return self.handle_commit_preview_text_editing(key).await;
        }

        match key.code {
            KeyCode::Esc => {
                // Cancel editing without saving
                self.ui_state.input_mode = ui::InputMode::Normal;
                self.ui_state.current_input.clear();
            }
            KeyCode::Enter => {
                // Special handling for TaskSearch screen - trigger search
                if self.current_screen == AppScreen::TaskSearch {
                    // Submit search
                    self.message = Some(format!("DEBUG: Starting search with query: '{}'", self.ui_state.current_input));
                    if !self.ui_state.current_input.is_empty() {
                        self.current_state = AppState::Loading;
                        match self.search_monday_tasks(&self.ui_state.current_input).await {
                            Ok(tasks) => {
                                self.tasks = tasks;
                                self.ui_state.selected_tab = 0;
                                self.current_state = AppState::Normal;
                                // Exit input mode after search
                                self.ui_state.input_mode = crate::ui::InputMode::Normal;
                                // Show feedback message
                                self.message = Some(format!("DEBUG: Search completed! Found {} tasks", self.tasks.len()));
                            }
                            Err(e) => {
                                self.current_state = AppState::Error(format!("DEBUG: Search failed: {}", e));
                            }
                        }
                    } else {
                        self.message = Some("DEBUG: Search query is empty".to_string());
                    }
                } else {
                    // Regular commit form field handling
                    log::debug!("Enter pressed, is_multiline: {}", self.is_multiline_field());
                    
                    if self.is_multiline_field() {
                        // In multiline fields, Enter always adds a new line
                        log::debug!("Adding newline in multiline field");
                        self.ui_state.current_input.push('\n');
                        self.ui_state.cursor_position = self.ui_state.current_input.len();
                    } else {
                        // In single-line fields, Enter does nothing (use Tab to navigate)
                        log::debug!("Enter ignored in single-line field - use Tab to navigate");
                    }
                }
            }
            KeyCode::Tab => {
                // Save current field and move to next field
                self.save_current_field();
                self.ui_state.current_input.clear();
                
                // Navigate to next field
                use crate::ui::CommitField;
                self.ui_state.current_field = match self.ui_state.current_field {
                    CommitField::Type => CommitField::Scope,
                    CommitField::Scope => CommitField::Title,
                    CommitField::Title => CommitField::Description,
                    CommitField::Description => CommitField::BreakingChange,
                    CommitField::BreakingChange => CommitField::TestDetails,
                    CommitField::TestDetails => CommitField::Security,
                    CommitField::Security => CommitField::MigracionesLentas,
                    CommitField::MigracionesLentas => CommitField::PartesAEjecutar,
                    CommitField::PartesAEjecutar => CommitField::SelectedTasks,
                    CommitField::SelectedTasks => CommitField::Type,
                };
                
                // Auto-enter edit mode for text fields, exit for commit type and selected tasks
                if !matches!(self.ui_state.current_field, CommitField::Type | CommitField::SelectedTasks) {
                    self.ui_state.current_input = match self.ui_state.current_field {
                        crate::ui::CommitField::Scope => self.commit_form.scope.clone(),
                        crate::ui::CommitField::Title => self.commit_form.title.clone(),
                        crate::ui::CommitField::Description => self.commit_form.description.clone(),
                        crate::ui::CommitField::BreakingChange => self.commit_form.breaking_change.clone(),
                        crate::ui::CommitField::TestDetails => self.commit_form.test_details.clone(),
                        crate::ui::CommitField::Security => self.commit_form.security.clone(),
                        crate::ui::CommitField::MigracionesLentas => self.commit_form.migraciones_lentas.clone(),
                        crate::ui::CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar.clone(),
                        _ => String::new(),
                    };
                    // Set cursor position to end of text
                    self.ui_state.cursor_position = self.ui_state.current_input.len();
                    // Stay in editing mode
                } else {
                    // Exit edit mode when moving to commit type field
                    self.ui_state.input_mode = ui::InputMode::Normal;
                }
            }
            KeyCode::BackTab => {
                // Save current field and move to previous field
                self.save_current_field();
                self.ui_state.current_input.clear();
                
                // Navigate to previous field
                use crate::ui::CommitField;
                self.ui_state.current_field = match self.ui_state.current_field {
                    CommitField::Type => CommitField::SelectedTasks,
                    CommitField::Scope => CommitField::Type,
                    CommitField::Title => CommitField::Scope,
                    CommitField::Description => CommitField::Title,
                    CommitField::BreakingChange => CommitField::Description,
                    CommitField::TestDetails => CommitField::BreakingChange,
                    CommitField::Security => CommitField::TestDetails,
                    CommitField::MigracionesLentas => CommitField::Security,
                    CommitField::PartesAEjecutar => CommitField::MigracionesLentas,
                    CommitField::SelectedTasks => CommitField::PartesAEjecutar,
                };
                
                // Auto-enter edit mode for text fields, exit for commit type and selected tasks
                if !matches!(self.ui_state.current_field, CommitField::Type | CommitField::SelectedTasks) {
                    self.ui_state.current_input = match self.ui_state.current_field {
                        crate::ui::CommitField::Scope => self.commit_form.scope.clone(),
                        crate::ui::CommitField::Title => self.commit_form.title.clone(),
                        crate::ui::CommitField::Description => self.commit_form.description.clone(),
                        crate::ui::CommitField::BreakingChange => self.commit_form.breaking_change.clone(),
                        crate::ui::CommitField::TestDetails => self.commit_form.test_details.clone(),
                        crate::ui::CommitField::Security => self.commit_form.security.clone(),
                        crate::ui::CommitField::MigracionesLentas => self.commit_form.migraciones_lentas.clone(),
                        crate::ui::CommitField::PartesAEjecutar => self.commit_form.partes_a_ejecutar.clone(),
                        _ => String::new(),
                    };
                    // Set cursor position to end of text
                    self.ui_state.cursor_position = self.ui_state.current_input.len();
                    // Stay in editing mode
                } else {
                    // Exit edit mode when moving to commit type field
                    self.ui_state.input_mode = ui::InputMode::Normal;
                }
            }
            KeyCode::Up => {
                // Move cursor up within text for multiline fields
                if self.is_multiline_field() {
                    self.move_cursor_up();
                }
            }
            KeyCode::Down => {
                // Move cursor down within text for multiline fields
                if self.is_multiline_field() {
                    self.move_cursor_down();
                }
            }
            KeyCode::Left => {
                // Move cursor left within text (handle UTF-8 properly)
                let text = &self.ui_state.current_input;
                if self.ui_state.cursor_position > 0 {
                    // Find the previous character boundary
                    let mut new_pos = self.ui_state.cursor_position.saturating_sub(1);
                    while new_pos > 0 && !text.is_char_boundary(new_pos) {
                        new_pos -= 1;
                    }
                    self.ui_state.cursor_position = new_pos;
                }
            }
            KeyCode::Right => {
                // Move cursor right within text (handle UTF-8 properly)
                let text = &self.ui_state.current_input;
                if self.ui_state.cursor_position < text.len() {
                    // Find the next character boundary
                    let mut new_pos = self.ui_state.cursor_position + 1;
                    while new_pos < text.len() && !text.is_char_boundary(new_pos) {
                        new_pos += 1;
                    }
                    self.ui_state.cursor_position = new_pos.min(text.len());
                }
            }
            KeyCode::Home => {
                // Move to beginning of line
                self.move_cursor_to_line_start();
            }
            KeyCode::End => {
                // Move to end of line
                self.move_cursor_to_line_end();
            }
            KeyCode::Char(c) => {
                // Insert character at cursor position (UTF-8 safe)
                let text = &self.ui_state.current_input;
                let cursor_pos = self.ui_state.cursor_position.min(text.len());
                
                // Ensure we're at a valid character boundary
                if text.is_char_boundary(cursor_pos) {
                    self.ui_state.current_input.insert(cursor_pos, c);
                    self.ui_state.cursor_position = cursor_pos + c.len_utf8();
                } else {
                    // Move to the end if we're not at a valid boundary
                    self.ui_state.current_input.push(c);
                    self.ui_state.cursor_position = self.ui_state.current_input.len();
                }
            }
            KeyCode::Backspace => {
                // Delete character before cursor (UTF-8 safe)
                let text = &self.ui_state.current_input;
                if self.ui_state.cursor_position > 0 {
                    let cursor_pos = self.ui_state.cursor_position.min(text.len());
                    
                    // Find the previous character boundary
                    let mut prev_pos = cursor_pos.saturating_sub(1);
                    while prev_pos > 0 && !text.is_char_boundary(prev_pos) {
                        prev_pos -= 1;
                    }
                    
                    if text.is_char_boundary(prev_pos) {
                        self.ui_state.current_input.remove(prev_pos);
                        self.ui_state.cursor_position = prev_pos;
                    }
                }
            }
            KeyCode::Delete => {
                // Delete character at cursor (UTF-8 safe)
                let text = &self.ui_state.current_input;
                let cursor_pos = self.ui_state.cursor_position.min(text.len());
                
                if cursor_pos < text.len() && text.is_char_boundary(cursor_pos) {
                    self.ui_state.current_input.remove(cursor_pos);
                    // cursor_position stays the same since we deleted the character at the cursor
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_commit_preview_text_editing(&mut self, key: KeyEvent) -> Result<()> {
        // Check for Ctrl+C first (before general character handling)
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            // Ctrl+C: confirm and create commit
            self.preview_commit_message = self.ui_state.current_input.clone();
            if let Err(e) = self.create_commit_with_message(&self.preview_commit_message).await {
                self.current_state = AppState::Error(e.to_string());
            } else {
                self.message = Some("Commit created successfully!".to_string());
                self.current_screen = AppScreen::Main;
                self.ui_state.input_mode = crate::ui::InputMode::Normal;
                self.ui_state.current_input.clear();
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                // Cancel commit and go back to commit screen
                self.current_screen = AppScreen::Commit;
                self.ui_state.input_mode = crate::ui::InputMode::Normal;
                self.ui_state.current_input.clear();
                self.message = Some("Commit cancelled".to_string());
            }
            KeyCode::Enter => {
                // Add new line in commit message
                self.ui_state.current_input.push('\n');
                self.ui_state.cursor_position = self.ui_state.current_input.len();
            }
            KeyCode::Left => {
                // Move cursor left (UTF-8 safe)
                let text = &self.ui_state.current_input;
                if self.ui_state.cursor_position > 0 {
                    // Find the previous character boundary
                    let mut new_pos = self.ui_state.cursor_position.saturating_sub(1);
                    while new_pos > 0 && !text.is_char_boundary(new_pos) {
                        new_pos -= 1;
                    }
                    self.ui_state.cursor_position = new_pos;
                }
            }
            KeyCode::Right => {
                // Move cursor right (UTF-8 safe)
                let text = &self.ui_state.current_input;
                if self.ui_state.cursor_position < text.len() {
                    // Find the next character boundary
                    let mut new_pos = self.ui_state.cursor_position + 1;
                    while new_pos < text.len() && !text.is_char_boundary(new_pos) {
                        new_pos += 1;
                    }
                    self.ui_state.cursor_position = new_pos.min(text.len());
                }
            }
            KeyCode::Up => {
                // Move cursor up one line
                self.move_cursor_up();
            }
            KeyCode::Down => {
                // Move cursor down one line
                self.move_cursor_down();
            }
            KeyCode::Home => {
                // Move to beginning of line
                self.move_cursor_to_line_start();
            }
            KeyCode::End => {
                // Move to end of line
                self.move_cursor_to_line_end();
            }
            KeyCode::Char(c) => {
                // Insert character at cursor position (UTF-8 safe)
                let text = &self.ui_state.current_input;
                let cursor_pos = self.ui_state.cursor_position.min(text.len());
                
                // Ensure we're at a valid character boundary
                if text.is_char_boundary(cursor_pos) {
                    self.ui_state.current_input.insert(cursor_pos, c);
                    self.ui_state.cursor_position = cursor_pos + c.len_utf8();
                } else {
                    // Move to the end if we're not at a valid boundary
                    self.ui_state.current_input.push(c);
                    self.ui_state.cursor_position = self.ui_state.current_input.len();
                }
            }
            KeyCode::Backspace => {
                // Delete character before cursor (UTF-8 safe)
                let text = &self.ui_state.current_input;
                if self.ui_state.cursor_position > 0 {
                    let cursor_pos = self.ui_state.cursor_position.min(text.len());
                    
                    // Find the previous character boundary
                    let mut prev_pos = cursor_pos.saturating_sub(1);
                    while prev_pos > 0 && !text.is_char_boundary(prev_pos) {
                        prev_pos -= 1;
                    }
                    
                    if text.is_char_boundary(prev_pos) {
                        self.ui_state.current_input.remove(prev_pos);
                        self.ui_state.cursor_position = prev_pos;
                    }
                }
            }
            KeyCode::Delete => {
                // Delete character at cursor (UTF-8 safe)
                let text = &self.ui_state.current_input;
                let cursor_pos = self.ui_state.cursor_position.min(text.len());
                
                if cursor_pos < text.len() && text.is_char_boundary(cursor_pos) {
                    self.ui_state.current_input.remove(cursor_pos);
                    // cursor_position stays the same since we deleted the character at the cursor
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn move_cursor_up(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        // Ensure we're at a valid character boundary
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        // Find the current line start
        let text_before_cursor = &text[..cursor_pos];
        let current_line_start = text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
        let current_column = cursor_pos - current_line_start;
        
        if current_line_start > 0 {
            // Find the previous line start
            let prev_line_end = current_line_start - 1; // Position of the '\n' before current line
            let text_before_prev_line = &text[..prev_line_end];
            let prev_line_start = text_before_prev_line.rfind('\n').map_or(0, |pos| pos + 1);
            let prev_line_length = prev_line_end - prev_line_start;
            
            // Move to the same column in the previous line, or end of line if shorter
            let new_column = current_column.min(prev_line_length);
            self.ui_state.cursor_position = prev_line_start + new_column;
        }
    }

    fn move_cursor_down(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        // Ensure we're at a valid character boundary
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        // Find the current line start and end
        let text_before_cursor = &text[..cursor_pos];
        let current_line_start = text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
        let current_column = cursor_pos - current_line_start;
        
        let text_after_cursor = &text[cursor_pos..];
        if let Some(current_line_end_offset) = text_after_cursor.find('\n') {
            let current_line_end = cursor_pos + current_line_end_offset;
            let next_line_start = current_line_end + 1;
            
            if next_line_start < text.len() {
                // Find the next line end
                let text_after_next_line = &text[next_line_start..];
                let next_line_end = text_after_next_line.find('\n')
                    .map_or(text.len(), |pos| next_line_start + pos);
                let next_line_length = next_line_end - next_line_start;
                
                // Move to the same column in the next line, or end of line if shorter
                let new_column = current_column.min(next_line_length);
                self.ui_state.cursor_position = next_line_start + new_column;
            }
        }
    }

    fn move_cursor_to_line_start(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        // Ensure we're at a valid character boundary
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        let text_before_cursor = &text[..cursor_pos];
        let line_start = text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
        self.ui_state.cursor_position = line_start;
    }

    fn move_cursor_to_line_end(&mut self) {
        let text = &self.ui_state.current_input;
        let cursor_pos = self.ui_state.cursor_position.min(text.len());
        
        // Ensure we're at a valid character boundary
        if !text.is_char_boundary(cursor_pos) {
            self.ui_state.cursor_position = text.len();
            return;
        }
        
        let text_after_cursor = &text[cursor_pos..];
        let line_end = text_after_cursor.find('\n')
            .map_or(text.len(), |pos| cursor_pos + pos);
        self.ui_state.cursor_position = line_end;
    }

    fn is_multiline_field(&self) -> bool {
        use crate::ui::CommitField;
        matches!(
            self.ui_state.current_field,
            CommitField::Description | CommitField::TestDetails | CommitField::Security | CommitField::MigracionesLentas | CommitField::PartesAEjecutar
        )
    }

    fn update_task_selection(&mut self) {
        // Update commit form with latest selections
        self.commit_form.selected_tasks = self.selected_tasks.clone();
        
        // Update scope with task IDs
        let task_ids: Vec<String> = self.selected_tasks.iter().map(|t| t.id.clone()).collect();
        self.commit_form.scope = if task_ids.is_empty() {
            String::new()
        } else {
            task_ids.join("|")
        };
    }

    fn save_current_field(&mut self) {
        // Save the input based on current field
        match self.ui_state.current_field {
            crate::ui::CommitField::Scope => {
                self.commit_form.scope = self.ui_state.current_input.clone();
            }
            crate::ui::CommitField::Title => {
                self.commit_form.title = self.ui_state.current_input.clone();
            }
            crate::ui::CommitField::Description => {
                self.commit_form.description = self.ui_state.current_input.clone();
            }
            crate::ui::CommitField::BreakingChange => {
                self.commit_form.breaking_change = self.ui_state.current_input.clone();
            }
            crate::ui::CommitField::TestDetails => {
                self.commit_form.test_details = self.ui_state.current_input.clone();
            }
            crate::ui::CommitField::Security => {
                self.commit_form.security = self.ui_state.current_input.clone();
            }
            crate::ui::CommitField::MigracionesLentas => {
                self.commit_form.migraciones_lentas = self.ui_state.current_input.clone();
            }
            crate::ui::CommitField::PartesAEjecutar => {
                self.commit_form.partes_a_ejecutar = self.ui_state.current_input.clone();
            }
            _ => {}
        }
    }

    async fn search_monday_tasks(&self, query: &str) -> Result<Vec<MondayTask>> {
        // Write debug to file
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut debug_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .unwrap_or_else(|_| std::fs::File::create("debug.log").unwrap());
            
        writeln!(debug_file, "DEBUG: search_monday_tasks called with query: '{}'", query).ok();
        
        let client = MondayClient::new(&self.config)?;
        writeln!(debug_file, "DEBUG: MondayClient created successfully").ok();
        
        let result = client.comprehensive_search(query).await;
        match &result {
            Ok(tasks) => {
                writeln!(debug_file, "DEBUG: Search returned {} tasks", tasks.len()).ok();
                for (i, task) in tasks.iter().enumerate().take(3) {
                    writeln!(debug_file, "DEBUG: Task {}: {} ({})", i, task.title, task.id).ok();
                }
            }
            Err(e) => {
                writeln!(debug_file, "DEBUG: Search failed with error: {}", e).ok();
            }
        }
        
        result
    }

    #[allow(dead_code)]
    async fn create_commit(&self) -> Result<()> {
        info!("Creating commit...");
        
        // Validate commit form
        if self.commit_form.commit_type.is_none() {
            error!("No commit type selected");
            return Err(anyhow::anyhow!("Please select a commit type"));
        }
        
        if self.commit_form.title.trim().is_empty() {
            error!("No commit title provided");
            return Err(anyhow::anyhow!("Please enter a commit title"));
        }
        
        debug!("Initializing git repository...");
        let git_repo = match GitRepo::new() {
            Ok(repo) => {
                info!("Git repository initialized successfully");
                repo
            }
            Err(e) => {
                error!("Failed to initialize git repository: {}", e);
                return Err(anyhow::anyhow!("Git repository error: {}. Make sure you're in a git repository.", e));
            }
        };
        
        // Build commit message
        let commit_message = self.build_commit_message();
        info!("Built commit message:\n{}", commit_message);
        
        // Create the commit
        debug!("Creating git commit...");
        match git_repo.create_commit(&commit_message) {
            Ok(_) => {
                info!("Commit created successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to create commit: {}", e);
                Err(anyhow::anyhow!("Failed to create commit: {}. Make sure you have staged changes or there are changes to commit.", e))
            }
        }
    }

    async fn create_commit_with_message(&self, message: &str) -> Result<()> {
        debug!("Creating commit with custom message");
        
        debug!("Initializing git repository...");
        let git_repo = match GitRepo::new() {
            Ok(repo) => {
                info!("Git repository initialized successfully");
                repo
            }
            Err(e) => {
                error!("Failed to initialize git repository: {}", e);
                return Err(anyhow::anyhow!("Git repository error: {}. Make sure you're in a git repository.", e));
            }
        };
        
        info!("Creating commit with message:\n{}", message);
        
        // Create the commit
        debug!("Creating git commit...");
        match git_repo.create_commit(message) {
            Ok(_) => {
                info!("Commit created successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to create commit: {}", e);
                Err(anyhow::anyhow!("Failed to create commit: {}. Make sure you have staged changes or there are changes to commit.", e))
            }
        }
    }

    fn build_commit_message(&self) -> String {
        let mut message = String::new();
        
        // Type and scope
        if let Some(commit_type) = &self.commit_form.commit_type {
            message.push_str(commit_type.as_str());
            
            if !self.commit_form.scope.is_empty() {
                message.push_str(&format!("({})", self.commit_form.scope));
            }
            
            message.push_str(": ");
        }
        
        // Title
        message.push_str(&self.commit_form.title);
        
        // Body
        if !self.commit_form.description.is_empty() {
            message.push_str("\n\n");
            message.push_str(&self.commit_form.description);
        }
        
        // Breaking changes
        if !self.commit_form.breaking_change.is_empty() {
            message.push_str("\n\nBREAKING CHANGE: ");
            message.push_str(&self.commit_form.breaking_change);
        }
        
        // Test details
        if !self.commit_form.test_details.is_empty() {
            message.push_str("\n\nTest Details: ");
            message.push_str(&self.commit_form.test_details);
        }
        
        // Security
        if !self.commit_form.security.is_empty() {
            message.push_str("\n\nSecurity: ");
            message.push_str(&self.commit_form.security);
        }
        
        // Migraciones Lentas
        if !self.commit_form.migraciones_lentas.is_empty() {
            message.push_str("\n\nMigraciones Lentas: ");
            message.push_str(&self.commit_form.migraciones_lentas);
        }
        
        // Partes a Ejecutar
        if !self.commit_form.partes_a_ejecutar.is_empty() {
            message.push_str("\n\nPartes a Ejecutar: ");
            message.push_str(&self.commit_form.partes_a_ejecutar);
        }
        
        // Monday.com tasks
        if !self.commit_form.selected_tasks.is_empty() {
            message.push_str("\n\nMONDAY TASKS:\n");
            for task in &self.commit_form.selected_tasks {
                message.push_str(&format!("- {} (ID: {}) - {}\n", task.title, task.id, task.url));
            }
        }
        
        message
    }

    async fn generate_release_notes_internal(&mut self) -> Result<()> {
        use chrono::Utc;
        use crate::git::GitRepo;
        use crate::monday::MondayClient;
        use crate::gemini::GeminiClient;
        use std::fs;
        use std::collections::HashSet;

        self.message = Some("📋 Obteniendo commits desde la última versión...".to_string());
        
        // Get git repository
        let git_repo = GitRepo::new()
            .map_err(|e| anyhow::anyhow!("Error accediendo al repositorio: {}", e))?;
        
        // Get last tag and commits since then
        let last_tag = git_repo.get_last_tag()
            .map_err(|e| anyhow::anyhow!("Error obteniendo última etiqueta: {}", e))?;
        
        let commits = git_repo.get_commits_since_tag(last_tag.as_deref())
            .map_err(|e| anyhow::anyhow!("Error obteniendo commits: {}", e))?;
        
        if commits.is_empty() {
            self.message = Some("⚠️ No se encontraron commits desde la última versión".to_string());
            return Ok(());
        }
        
        self.message = Some(format!("📊 Se encontraron {} commits para analizar", commits.len()));
        
        // Extract Monday.com task IDs from commits
        let mut monday_task_ids = HashSet::new();
        for commit in &commits {
            // Check scope for task IDs
            if let Some(scope) = &commit.scope {
                for id in scope.split('|') {
                    if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
                        monday_task_ids.insert(id.to_string());
                    }
                }
            }
            
            // Check monday task mentions
            for mention in &commit.monday_task_mentions {
                monday_task_ids.insert(mention.id.clone());
            }
            
            // Check monday_tasks field
            for task_id in &commit.monday_tasks {
                monday_task_ids.insert(task_id.clone());
            }
        }
        
        self.message = Some(format!("🔍 Obteniendo detalles de {} tareas de Monday.com...", monday_task_ids.len()));
        
        // Extract responsible person from most recent commit author
        let responsible_person = if !commits.is_empty() {
            commits[0].author_name.clone()
        } else {
            "".to_string()
        };
        
        // Get Monday.com task details
        let mut monday_tasks = if !monday_task_ids.is_empty() {
            match MondayClient::new(&self.config) {
                Ok(client) => {
                    let task_ids: Vec<String> = monday_task_ids.iter().cloned().collect();
                    match client.get_task_details(&task_ids).await {
                        Ok(tasks) => tasks,
                        Err(e) => {
                            eprintln!("⚠️ Error obteniendo detalles de Monday.com: {}", e);
                            Vec::new()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Error conectando con Monday.com: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        
        // Create placeholder tasks for IDs that couldn't be fetched from Monday API
        // This ensures all task IDs found in commits are represented
        let found_task_ids: HashSet<String> = monday_tasks.iter().map(|task| task.id.clone()).collect();
        for task_id in &monday_task_ids {
            if !found_task_ids.contains(task_id) {
                // Create a placeholder task based on commit information
                let mut title = "Task not found in Monday API".to_string();
                let mut support_bee_links = Vec::new();
                
                // Try to extract title from commits that mention this task
                for commit in &commits {
                    let task_mentioned = if let Some(scope) = &commit.scope {
                        scope.split('|').any(|id| id == task_id)
                    } else {
                        false
                    } || commit.monday_task_mentions.iter().any(|mention| &mention.id == task_id)
                      || commit.monday_tasks.contains(task_id);
                    
                    if task_mentioned {
                        // Extract title from commit message or monday task mentions
                        for mention in &commit.monday_task_mentions {
                            if &mention.id == task_id {
                                title = mention.title.clone();
                                break;
                            }
                        }
                        
                        // Extract SupportBee links from commit message using regex
                        use regex::Regex;
                        if let Ok(re) = Regex::new(r"https://teimas\.supportbee\.com/tickets/[0-9]+") {
                            let commit_text = format!("{} {}", commit.subject, commit.body);
                    for mat in re.find_iter(&commit_text) {
                                let link = mat.as_str().to_string();
                                if !support_bee_links.contains(&link) {
                                    support_bee_links.push(link);
                                }
                            }
                        }
                    }
                }
                
                // Create placeholder Monday task
                use crate::types::MondayTask;
                let placeholder_task = MondayTask {
                    id: task_id.clone(),
                    title,
                    board_id: Some("".to_string()),
                    board_name: Some("".to_string()),
                    url: "".to_string(),
                    state: "active".to_string(),
                    updates: Vec::new(),
                    group_title: Some("".to_string()),
                    column_values: Vec::new(),
                };
                
                monday_tasks.push(placeholder_task);
            }
        }
        
        // Get version and create tag format like Node.js script does  
        let version = match crate::git::get_next_version() {
            Ok(v) => {
                if v != "next version" && v != "próxima versión" && !v.is_empty() {
                    // Create tag format like "tag-teixo-20250416-1.112.0"
                    let date_str = Utc::now().format("%Y%m%d").to_string();
                    format!("tag-teixo-{}-{}", date_str, v)
                } else {
                    // Create a fallback version with current date and incremental number
                    let date_str = Utc::now().format("%Y%m%d").to_string();
                    match git_repo.get_last_tag() {
                        Ok(Some(tag)) => {
                            // Try to extract version number from last tag and increment
                            if let Some(version_part) = tag.split('-').next_back() {
                                if let Ok(mut version_num) = version_part.parse::<f32>() {
                                    version_num += 0.001;
                                    format!("tag-teixo-{}-{:.3}", date_str, version_num)
                                } else {
                                    format!("tag-teixo-{}-1.112.0", date_str)
                                }
                            } else {
                                format!("tag-teixo-{}-1.112.0", date_str)
                            }
                        },
                        Ok(None) => format!("tag-teixo-{}-1.112.0", date_str),
                        Err(_) => format!("tag-teixo-{}-1.112.0", date_str),
                    }
                }
            },
            Err(_) => {
                // Create a fallback version with current date
                let date_str = Utc::now().format("%Y%m%d").to_string();
                format!("tag-teixo-{}-1.112.0", date_str)
            },
        };
        
        self.message = Some(format!("📄 Generando documento estructurado para versión {}...", version));
        
        // Generate the structured document (like Node.js script does)
        let structured_document = self.generate_raw_release_notes(&version, &commits, &monday_tasks, &responsible_person);
        
        // Create output directory
        if let Err(e) = fs::create_dir_all("release-notes") {
            eprintln!("Warning: Could not create release-notes directory: {}", e);
        }
        
        // Generate filenames
        let date_str = Utc::now().format("%Y-%m-%d").to_string();
        let structured_filename = format!("release-notes/release-notes-{}_SCRIPT.md", date_str);
        let gemini_filename = format!("release-notes/release-notes-{}_GEMINI.md", date_str);
        
        // Save the structured document first
        if let Err(e) = fs::write(&structured_filename, &structured_document) {
            return Err(anyhow::anyhow!("Error guardando documento estructurado: {}", e));
        }
        
        self.message = Some("🤖 Enviando documento a Google Gemini API...".to_string());
        
        // Try to process with Gemini
        match GeminiClient::new(&self.config) {
            Ok(gemini_client) => {
                match gemini_client.process_release_notes_document(&structured_document).await {
                    Ok(gemini_response) => {
                        // Save the Gemini-processed version
                        if let Err(e) = fs::write(&gemini_filename, &gemini_response) {
                            eprintln!("⚠️ Error guardando respuesta de Gemini: {}", e);
                        } else {
                            self.message = Some(format!(
                                "✅ Notas de versión generadas exitosamente:\n📄 Documento estructurado: {}\n🤖 Versión procesada por Gemini: {}",
                                structured_filename, gemini_filename
                            ));
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ Error procesando con Gemini: {}", e);
                        self.message = Some(format!(
                            "⚠️ Gemini falló, pero se generó el documento estructurado:\n📄 Documento estructurado: {}\n💡 Ejecuta el script de Node.js para procesamiento con Gemini",
                            structured_filename
                        ));
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ Error configurando Gemini: {}", e);
                self.message = Some(format!(
                    "⚠️ Gemini no configurado, solo se generó el documento estructurado:\n📄 Documento estructurado: {}\n💡 Configura el token de Gemini para procesamiento IA",
                    structured_filename
                ));
                return Ok(());
            }
        }
        
        // Fallback message if we get here
        self.message = Some(format!(
            "📄 Documento estructurado generado: {}\n⚠️ No se pudo procesar con Gemini",
            structured_filename
        ));
        
        Ok(())
    }

    async fn generate_release_notes_with_npm(&mut self) -> Result<()> {
        use std::process::{Command, Stdio};
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        // Shared state for communication between thread and UI
        let npm_status = Arc::new(Mutex::new(String::from("🚀 Iniciando npm run release-notes...")));
        let npm_finished = Arc::new(Mutex::new(false));
        let npm_success = Arc::new(Mutex::new(true));
        
        // Clone for the thread
        let status_clone = npm_status.clone();
        let finished_clone = npm_finished.clone();
        let success_clone = npm_success.clone();
        
        // Update initial status
        self.message = Some("🚀 Iniciando npm run release-notes...".to_string());
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Spawn npm command in background thread
        thread::spawn(move || {
            // Update status
            if let Ok(mut status) = status_clone.lock() {
                *status = "🌐 Ejecutando comando npm...".to_string();
            }
            
            let output = Command::new("npm")
                .args(["run", "release-notes"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();
                
            match output {
                Ok(output) => {
                    // Update status with some output info
                    if let Ok(mut status) = status_clone.lock() {
                        if output.status.success() {
                            *status = "✅ npm run release-notes completado exitosamente".to_string();
                        } else {
                            *status = format!("❌ npm falló con código: {}", output.status.code().unwrap_or(-1));
                            if let Ok(mut success) = success_clone.lock() {
                                *success = false;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error ejecutando npm: {}", e);
                    }
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                }
            }
            
            // Mark as finished
            if let Ok(mut finished) = finished_clone.lock() {
                *finished = true;
            }
        });

        // Poll for updates and keep UI responsive
        let mut current_status = String::new();
        loop {
            // Check if npm is finished
            let is_finished = {
                npm_finished.lock().map(|f| *f).unwrap_or(false)
            };
            
            // Update status message if it changed
            if let Ok(status) = npm_status.lock() {
                if *status != current_status {
                    current_status = status.clone();
                    self.message = Some(current_status.clone());
                }
            }
            
            if is_finished {
                break;
            }
            
            // Yield control to UI with short sleep
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        
        // Check if npm succeeded
        let success = npm_success.lock().map(|s| *s).unwrap_or(false);
        if !success {
            return Err(anyhow::anyhow!("{}", current_status));
        }

        Ok(())
    }

    fn generate_raw_release_notes(&self, version: &str, commits: &[crate::types::GitCommit], monday_tasks: &[MondayTask], responsible_person: &str) -> String {
        use chrono::Utc;
        use std::collections::HashMap;
        use std::fs;
        
        // Create a mapping of task ID to task details for quick lookup
        let task_details_map: HashMap<String, &MondayTask> = monday_tasks.iter()
            .map(|task| (task.id.clone(), task))
            .collect();
        
        // Group commits by type
        let commits_by_type = self.group_commits_by_type(commits);
        
        let mut document = String::new();
        
        // Header
        document.push_str(&format!("# Datos para Generación de Notas de Versión {}\n\n", version));
        
        // General Information
        document.push_str("## Información General\n\n");
        document.push_str(&format!("- **Versión**: {}\n", version));
        document.push_str(&format!("- **Fecha**: {}\n", 
            Utc::now().format("%d/%m/%Y")));
        document.push_str(&format!("- **Total de Commits**: {}\n", commits.len()));
        document.push_str(&format!("- **Tareas de Monday relacionadas**: {}\n\n", monday_tasks.len()));
        
        // Instructions for Gemini
        document.push_str("## Instrucciones CRÍTICAS\n\n");
        document.push_str("DEBES seguir EXACTAMENTE la plantilla que se proporciona al final de este documento. ");
        document.push_str("NO crees un resumen o documento libre. COPIA la estructura de la plantilla y RELLENA cada sección. ");
        document.push_str("OBLIGATORIO: \n");
        document.push_str(&format!("1. El responsable del despliegue es: {} - úsalo en la sección 'Responsable despliegue'.\n", responsible_person));
        document.push_str("2. Para las tareas de Monday.com, usa SIEMPRE el formato 'm' + ID (ej: m8817155664).\n");
        document.push_str("3. En la tabla 'Información para N1', incluye TODAS las tareas BUG con SupportBee links.\n");
        document.push_str("4. Para las secciones Correcciones y Proyectos especiales, usa solo las tareas con labels BUG y PE.\n");
        document.push_str("5. En las tablas de validación, incluye descripciones específicas basadas en el título de cada tarea.\n");
        document.push_str("6. Incluye TODOS los commits en 'Referencia commits' con el formato exacto mostrado.\n");
        document.push_str(&format!("7. Usa el título: '# Actualización Teixo versión {}'.\n", version));
        document.push_str("8. Si una tabla está vacía en la plantilla, déjala vacía pero manténla.\n");
        document.push_str("CRÍTICO: No inventes información, usa solo los datos proporcionados.\n\n");
        
        // Summary of changes by type
        document.push_str("## Resumen de Cambios\n\n");
        
        // New Features (feat)
        if let Some(feat_commits) = commits_by_type.get("feat") {
            if !feat_commits.is_empty() {
                document.push_str(&format!("### Nuevas Funcionalidades ({})\n\n", feat_commits.len()));
                for commit in feat_commits {
                    document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                        commit.description,
                        commit.hash.chars().take(7).collect::<String>(),
                        &commit.author_name,
                        &commit.author_email,
                        commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                    ));
                    if !commit.body.trim().is_empty() {
                        document.push_str(&format!("  - Detalles: {}\n", self.format_multiline_text(&commit.body)));
                    }
                }
                document.push('\n');
            }
        }
        
        // Bug Fixes (fix)
        if let Some(fix_commits) = commits_by_type.get("fix") {
            if !fix_commits.is_empty() {
                document.push_str(&format!("### Correcciones ({})\n\n", fix_commits.len()));
                for commit in fix_commits {
                    document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                        commit.description,
                        commit.hash.chars().take(7).collect::<String>(),
                        &commit.author_name,
                        &commit.author_email,
                        commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                    ));
                    if !commit.body.trim().is_empty() {
                        document.push_str(&format!("  - Detalles: {}\n", self.format_multiline_text(&commit.body)));
                    }
                }
                document.push('\n');
            }
        }
        
        // Other types of commits
        let other_types: Vec<&String> = commits_by_type.keys()
            .filter(|&type_name| !["feat", "fix"].contains(&type_name.as_str()))
            .collect();
        
        for type_name in other_types {
            if let Some(type_commits) = commits_by_type.get(type_name) {
                if !type_commits.is_empty() {
                    document.push_str(&format!("### {} ({})\n\n", self.get_type_title(type_name), type_commits.len()));
                    for commit in type_commits {
                        document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                            commit.description,
                            commit.hash.chars().take(7).collect::<String>(),
                            &commit.author_name,
                            &commit.author_email,
                            commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                        ));
                        if !commit.body.trim().is_empty() {
                            document.push_str(&format!("  - Detalles: {}\n", self.format_multiline_text(&commit.body)));
                        }
                    }
                    document.push('\n');
                }
            }
        }
        
        // Breaking changes
        let breaking_commits: Vec<&crate::types::GitCommit> = commits.iter()
            .filter(|commit| !commit.breaking_changes.is_empty())
            .collect();
        
        if !breaking_commits.is_empty() {
            document.push_str("## Cambios que Rompen Compatibilidad\n\n");
            for commit in breaking_commits {
                document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                    commit.description,
                    commit.hash.chars().take(7).collect::<String>(),
                    &commit.author_name,
                    &commit.author_email,
                    commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                ));
                document.push_str(&format!("  - Detalles: {}\n", 
                    commit.breaking_changes.join(" | ")));
            }
            document.push('\n');
        }
        
        // Monday.com task details
        if !monday_tasks.is_empty() {
            document.push_str("## Detalles de Tareas de Monday\n\n");
            
            for task in monday_tasks {
                document.push_str(&format!("### {} (ID: {})\n\n", task.title, task.id));
                document.push_str(&format!("- **Estado**: {}\n", task.state));
                document.push_str(&format!("- **Tablero**: {} (ID: {})\n", 
                    task.board_name.as_deref().unwrap_or("N/A"), 
                    task.board_id.as_deref().unwrap_or("N/A")));
                document.push_str(&format!("- **Grupo**: {}\n", 
                    task.group_title.as_deref().unwrap_or("N/A")));
                
                // Column values
                if !task.column_values.is_empty() {
                    document.push_str("- **Detalles**:\n");
                    let relevant_columns: Vec<&crate::types::MondayColumnValue> = task.column_values.iter()
                        .filter(|col| col.text.is_some() && !col.text.as_ref().unwrap().trim().is_empty())
                        .collect();
                    
                    if !relevant_columns.is_empty() {
                        for col in relevant_columns {
                            document.push_str(&format!("  - {}: {}\n", 
                                col.id, 
                                col.text.as_deref().unwrap_or("")));
                        }
                    } else {
                        document.push_str("  - No hay detalles adicionales disponibles\n");
                    }
                }
                
                // SupportBee links extracted from Monday task column values (texto field)
                let mut supportbee_links = Vec::new();
                for col in &task.column_values {
                    if col.id == "texto" {
                        if let Some(text) = &col.text {
                            let supportbee_regex = regex::Regex::new(r"https?://[^\s,]*teimas\.supportbee[^\s,]*").unwrap();
                            for mat in supportbee_regex.find_iter(text) {
                                let link = mat.as_str().to_string();
                                if !supportbee_links.contains(&link) {
                                    supportbee_links.push(link);
                                }
                            }
                        }
                    }
                }
                
                if !supportbee_links.is_empty() {
                    document.push_str("- **Enlaces SupportBee**:\n");
                    for link in supportbee_links {
                        document.push_str(&format!("  - {}\n", link));
                    }
                }
                
                // Recent updates (Actualizaciones Recientes)
                if !task.updates.is_empty() {
                    document.push_str("- **Actualizaciones Recientes**:\n");
                    
                    // Show the 3 most recent updates
                    for update in task.updates.iter().take(3) {
                        // Format the date from ISO string to DD/MM/YYYY format
                        let date = if let Ok(parsed_date) = update.created_at.parse::<chrono::DateTime<chrono::Utc>>() {
                            parsed_date.format("%d/%m/%Y").to_string()
                        } else if let Some(date_part) = update.created_at.split('T').next() {
                            // If we can't parse the full datetime, try to extract just the date part
                            if let Ok(parsed_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                                parsed_date.format("%d/%m/%Y").to_string()
                            } else {
                                update.created_at.clone()
                            }
                        } else {
                            update.created_at.clone()
                        };
                        
                        let creator_name = update.creator.as_ref()
                            .map(|c| c.name.as_str())
                            .unwrap_or("Usuario");
                        
                        // Truncate the body to 100 characters max
                        let body_preview = if update.body.len() > 100 {
                            format!("{}...", &update.body[..100])
                        } else {
                            update.body.clone()
                        };
                        
                        document.push_str(&format!("  - {} por {}: {}\n", date, creator_name, body_preview));
                    }
                }
                
                // Related commits
                let related_commits: Vec<&crate::types::GitCommit> = commits.iter()
                    .filter(|commit| {
                        // Check scope
                        if let Some(scope) = &commit.scope {
                            if scope.split('|').any(|id| id == task.id) {
                                return true;
                            }
                        }
                        
                        // Check monday_task_mentions
                        commit.monday_task_mentions.iter().any(|mention| mention.id == task.id)
                    })
                    .collect();
                
                if !related_commits.is_empty() {
                    document.push_str("- **Commits Relacionados**:\n");
                    for commit in related_commits {
                        document.push_str(&format!("  - {}: {} [{}]\n", 
                            commit.commit_type.as_deref().unwrap_or("other"),
                            commit.description,
                            commit.hash.chars().take(7).collect::<String>()));
                    }
                }
                
                document.push('\n');
            }
        }
        
        // Complete commit details
        document.push_str("## Detalles Completos de Commits\n\n");
        
        for commit in commits {
            let scope_part = if let Some(scope) = &commit.scope {
                format!("({})", scope)
            } else {
                String::new()
            };
            
            document.push_str(&format!("### {}{}: {} [{}]\n\n", 
                commit.commit_type.as_deref().unwrap_or("other"),
                scope_part,
                commit.description,
                commit.hash.chars().take(7).collect::<String>()));
            
            document.push_str(&format!("**Autor**: {} <{}>\n", 
                &commit.author_name,
                &commit.author_email));
            document.push_str(&format!("**Fecha**: {}\n\n", 
                commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")));
            
            if !commit.body.trim().is_empty() {
                document.push_str(&format!("{}\n\n", self.format_multiline_text(&commit.body)));
            }
            
            if !commit.test_details.is_empty() {
                document.push_str("**Pruebas**:\n");
                for test in &commit.test_details {
                    document.push_str(&format!("- {}\n", test));
                }
                document.push('\n');
            }
            
            if let Some(security) = &commit.security {
                if security != "NA" {
                    document.push_str(&format!("**Seguridad**: {}\n\n", security));
                }
            }
            
            if !commit.monday_task_mentions.is_empty() {
                document.push_str("**Tareas relacionadas**:\n");
                
                for mention in &commit.monday_task_mentions {
                    let task_details = task_details_map.get(&mention.id);
                    let task_name = task_details.map(|t| t.title.as_str()).unwrap_or(&mention.title);
                    let task_state = task_details.map(|t| t.state.as_str()).unwrap_or("Desconocido");
                    
                    document.push_str(&format!("- {} (ID: {}, Estado: {})\n", 
                        task_name, mention.id, task_state));
                }
                
                document.push('\n');
            }
            
            document.push_str("---\n\n");
        }

        document.push_str("La plantilla a utilizar para generar el documento tiene que ser la siguiente. Fijate en todo lo que hay y emúlalo por completo.");
        
        // Read and include the template content (like Node.js does)
        match fs::read_to_string("scripts/plantilla.md") {
            Ok(plantilla_content) => {
                document.push_str(&format!("\n\n{}", plantilla_content));
                println!("✅ Plantilla cargada exitosamente: scripts/plantilla.md");
            }
            Err(e) => {
                println!("⚠️ No se pudo cargar la plantilla scripts/plantilla.md: {}", e);
                // If we can't load the template, at least add the instruction to use the original template format
                document.push_str("\n\nPor favor, utiliza el formato estándar de notas de versión de Teixo que incluye las secciones de Información para N1, Información técnica, Correcciones, Novedades (por categorías), Validación en Sandbox, Pruebas y Referencia commits.");
            }
        }
        
        document
    }
    
    // Helper methods for release notes generation
    fn group_commits_by_type<'a>(&self, commits: &'a [crate::types::GitCommit]) -> std::collections::HashMap<String, Vec<&'a crate::types::GitCommit>> {
        let mut groups = std::collections::HashMap::new();
        
        for commit in commits {
            let commit_type = commit.commit_type.as_deref().unwrap_or("other").to_string();
            groups.entry(commit_type).or_insert_with(Vec::new).push(commit);
        }
        
        groups
    }
    
    fn format_multiline_text(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }
        
        text.split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" | ")
    }
    
    fn get_type_title(&self, commit_type: &str) -> String {
        match commit_type {
            "feat" => "Nuevas Funcionalidades".to_string(),
            "fix" => "Correcciones".to_string(),
            "docs" => "Documentación".to_string(),
            "style" => "Estilo".to_string(),
            "refactor" => "Refactorizaciones".to_string(),
            "perf" => "Mejoras de Rendimiento".to_string(),
            "test" => "Pruebas".to_string(),
            "build" => "Construcción".to_string(),
            "ci" => "Integración Continua".to_string(),
            "chore" => "Tareas".to_string(),
            "revert" => "Reversiones".to_string(),
            _ => {
                let first_char = commit_type.chars().next().unwrap().to_uppercase().collect::<String>();
                format!("{}{}", first_char, &commit_type[1..])
            }
        }
    }

    // CLI methods for direct command usage
    pub async fn commit_flow(&self) -> Result<()> {
        println!("🚀 Semantic Release TUI - Commit Flow");
        println!("This would open the TUI commit interface");
        Ok(())
    }

    pub async fn generate_release_notes(&self) -> Result<()> {
        println!("📝 Generating release notes...");
        
        // Get version
        let version = get_next_version().unwrap_or_else(|_| "next".to_string());
        println!("📦 Version: {}", version);
        
        // Get git repository and commits
        let git_repo = GitRepo::new()?;
        let last_tag = git_repo.get_last_tag()?;
        let commits = git_repo.get_commits_since_tag(last_tag.as_deref())?;
        
        println!("📋 Found {} commits since last tag", commits.len());
        
        // Extract Monday task IDs from commits
        let mut task_ids = Vec::new();
        for commit in &commits {
            // Add task IDs from monday_tasks field
            task_ids.extend(commit.monday_tasks.clone());
            
            // Add task IDs from monday_task_mentions
            for mention in &commit.monday_task_mentions {
                task_ids.push(mention.id.clone());
            }
            
            // Also check the scope for task IDs (pipe-separated)
            if let Some(scope) = &commit.scope {
                let scope_ids: Vec<String> = scope.split('|')
                    .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
                    .map(|id| id.to_string())
                    .collect();
                task_ids.extend(scope_ids);
            }
        }
        task_ids.sort();
        task_ids.dedup();
        
        // Get Monday task details
        let monday_tasks = if !task_ids.is_empty() && self.config.monday_api_key.is_some() {
            println!("🔍 Fetching Monday.com task details...");
            let client = MondayClient::new(&self.config)?;
            client.get_task_details(&task_ids).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        
        println!("📋 Found {} related Monday.com tasks", monday_tasks.len());
        
        // Extract responsible person from most recent commit author
        let responsible_person = if !commits.is_empty() {
            commits[0].author_name.clone()
        } else {
            "".to_string()
        };
        
        // Create release notes directory
        let release_notes_dir = Path::new("release-notes");
        if !release_notes_dir.exists() {
            fs::create_dir_all(release_notes_dir)?;
        }
        
        // Generate structured document
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let script_file = release_notes_dir.join(format!("release-notes-{}_SCRIPT.md", date));
        let ai_file = release_notes_dir.join(format!("release-notes-{}_GEMINI.md", date));
        
        let script_content = self.generate_raw_release_notes(&version, &commits, &monday_tasks, &responsible_person);
        fs::write(&script_file, &script_content)?;
        println!("✅ Script release notes saved to: {}", script_file.display());
        
        // Generate AI release notes if Gemini is configured
        if self.config.gemini_token.is_some() {
            println!("🤖 Generating AI-powered release notes...");
            let gemini_client = GeminiClient::new(&self.config)?;
            match gemini_client.generate_release_notes(&version, &commits, &monday_tasks).await {
                Ok(ai_content) => {
                    fs::write(&ai_file, &ai_content)?;
                    println!("✅ AI release notes saved to: {}", ai_file.display());
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate AI release notes: {}", e);
                }
            }
        } else {
            println!("⚠️  Google Gemini not configured. Skipping AI generation.");
        }
        
        Ok(())
    }

    pub async fn search_tasks(&self, query: &str) -> Result<()> {
        println!("🔍 Searching Monday.com tasks for: {}", query);
        
        let client = MondayClient::new(&self.config)?;
        let tasks = client.search_tasks(query).await?;
        
        println!("📋 Found {} tasks:", tasks.len());
        for task in tasks {
            println!("  • {} [{}] (ID: {})", task.title, task.state.to_uppercase(), task.id);
            println!("    URL: {}", task.url);
            if let Some(board_name) = task.board_name {
                println!("    Board: {}", board_name);
            }
            println!();
        }
        
        Ok(())
    }

    // Debug methods for troubleshooting
    pub async fn debug_monday(&self) -> Result<()> {
        println!("🔍 Debug: Testing Monday.com connection...");
        
        if self.config.monday_api_key.is_none() {
            println!("❌ No Monday.com API key configured");
            return Ok(());
        }
        
        println!("✅ Monday.com API key: Configured");
        println!("✅ Account slug: {}", self.config.monday_account_slug.as_deref().unwrap_or("Not set"));
        
        let client = MondayClient::new(&self.config)?;
        match client.test_connection().await {
            Ok(response) => {
                println!("✅ Monday.com connection: SUCCESS");
                println!("📋 Response: {}", response);
            }
            Err(e) => {
                println!("❌ Monday.com connection: FAILED");
                println!("🔍 Error details: {}", e);
            }
        }
        
        Ok(())
    }

    pub async fn debug_gemini(&self) -> Result<()> {
        println!("🤖 Debug: Testing Gemini AI connection...");
        
        if self.config.gemini_token.is_none() {
            println!("❌ No Gemini API token configured");
            return Ok(());
        }
        
        println!("✅ Gemini API token: Configured");
        
        match crate::gemini::test_gemini_connection(&self.config).await {
            Ok(response) => {
                println!("✅ Gemini connection: SUCCESS");
                println!("🤖 Response: {}", response);
            }
            Err(e) => {
                println!("❌ Gemini connection: FAILED");
                println!("🔍 Error details: {}", e);
            }
        }
        
        Ok(())
    }

    pub async fn debug_git(&self) -> Result<()> {
        println!("📂 Debug: Testing Git repository...");
        
        match GitRepo::new() {
            Ok(repo) => {
                println!("✅ Git repository: Found");
                
                match repo.get_current_branch() {
                    Ok(branch) => println!("📍 Current branch: {}", branch),
                    Err(e) => println!("❌ Could not get current branch: {}", e),
                }
                
                match repo.get_status() {
                    Ok(status) => {
                        println!("📊 Repository status:");
                        println!("  • Modified files: {}", status.modified.len());
                        println!("  • Staged files: {}", status.staged.len());
                        println!("  • Untracked files: {}", status.untracked.len());
                        
                        if !status.staged.is_empty() {
                            println!("📝 Staged files:");
                            for file in &status.staged {
                                println!("    + {}", file);
                            }
                        }
                        
                        if !status.modified.is_empty() {
                            println!("📝 Modified files:");
                            for file in &status.modified {
                                println!("    ~ {}", file);
                            }
                        }
                    }
                    Err(e) => println!("❌ Could not get repository status: {}", e),
                }
            }
            Err(e) => {
                println!("❌ Git repository: NOT FOUND");
                println!("🔍 Error: {}", e);
                println!("💡 Make sure you're in a git repository directory");
            }
        }
        
        Ok(())
    }

    pub async fn debug_commit(&self) -> Result<()> {
        println!("💾 Debug: Testing commit creation...");
        
        // Check git repository
        println!("\n1. Checking Git repository...");
        let git_repo = match GitRepo::new() {
            Ok(repo) => {
                println!("✅ Git repository found");
                repo
            }
            Err(e) => {
                println!("❌ Git repository error: {}", e);
                return Ok(());
            }
        };
        
        // Check repository status
        println!("\n2. Checking repository status...");
        match git_repo.get_status() {
            Ok(status) => {
                if status.staged.is_empty() && status.modified.is_empty() {
                    println!("⚠️  No changes to commit");
                    println!("💡 Try making some changes and staging them with 'git add .'");
                    return Ok(());
                }
                
                if status.staged.is_empty() {
                    println!("⚠️  No staged changes found");
                    println!("💡 Stage your changes with 'git add .' first");
                    
                    println!("\n📝 Available modified files:");
                    for file in &status.modified {
                        println!("    ~ {}", file);
                    }
                    return Ok(());
                }
                
                println!("✅ Found {} staged changes", status.staged.len());
                for file in &status.staged {
                    println!("    + {}", file);
                }
            }
            Err(e) => {
                println!("❌ Could not check repository status: {}", e);
                return Ok(());
            }
        }
        
        // Test commit message building
        println!("\n3. Testing commit message...");
        if self.commit_form.commit_type.is_none() {
            println!("⚠️  No commit type selected in current form");
        } else {
            println!("✅ Commit type: {:?}", self.commit_form.commit_type);
        }
        
        if self.commit_form.title.trim().is_empty() {
            println!("⚠️  No commit title in current form");
        } else {
            println!("✅ Commit title: {}", self.commit_form.title);
        }
        
        let commit_message = self.build_commit_message();
        println!("\n📝 Generated commit message:");
        println!("---");
        println!("{}", commit_message);
        println!("---");
        
        println!("\n✅ Debug complete! Use this information to troubleshoot commit issues.");
        
        Ok(())
    }

    fn start_gemini_analysis(&self, analysis_state: GeminiAnalysisState) {
        use crate::git::GitRepo;
        use crate::gemini::GeminiClient;
        use std::thread;

        // Clone data needed for the thread
        let config_clone = self.config.clone();
        let commit_type = self.commit_form.commit_type.as_ref().map(|ct| ct.as_str().to_string());
        let scope = if self.commit_form.scope.is_empty() { None } else { Some(self.commit_form.scope.clone()) };
        let title = self.commit_form.title.clone();
        
        // Clone analysis state components
        let status_clone = analysis_state.status.clone();
        let finished_clone = analysis_state.finished.clone();
        let success_clone = analysis_state.success.clone();
        let result_clone = analysis_state.result.clone();
        let security_clone = analysis_state.security.clone();
        let breaking_clone = analysis_state.breaking.clone();

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
                    *result = "No hay cambios para describir.".to_string();
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

            // Update status: generating description and analyzing security
            if let Ok(mut status) = status_clone.lock() {
                *status = "📝 Generando descripción y analizando seguridad...".to_string();
            }

            // Make the async Gemini calls in a blocking context
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
            
            let commit_type_ref = commit_type.as_deref();
            let scope_ref = scope.as_deref();
            
            // Run three Gemini analyses in parallel
            let results = rt.block_on(async {
                let description_future = gemini_client.generate_commit_description(&changes, commit_type_ref, scope_ref, &title);
                let security_future = gemini_client.analyze_security_risks(&changes, commit_type_ref, scope_ref, &title);
                let breaking_future = gemini_client.analyze_breaking_changes(&changes, commit_type_ref, scope_ref, &title);
                
                tokio::join!(description_future, security_future, breaking_future)
            });
            
            // Handle the results
            match results.0 {
                Ok(description) => {
                    if let Ok(mut result) = result_clone.lock() {
                        *result = description;
                    }
                }
                Err(e) => {
                    // Fallback to a basic description
                    if let Ok(mut result) = result_clone.lock() {
                        *result = "Cambios realizados en el código del proyecto.".to_string();
                    }
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("⚠️ Gemini falló en descripción: {}", e);
                    }
                }
            }
            
            // Handle security analysis result
            if let Ok(security) = results.1 {
                if !security.is_empty() {
                    if let Ok(mut sec) = security_clone.lock() {
                        *sec = security;
                    }
                }
            }
            
            // Handle breaking changes analysis result
            if let Ok(breaking) = results.2 {
                if !breaking.is_empty() {
                    if let Ok(mut brk) = breaking_clone.lock() {
                        *brk = breaking;
                    }
                }
            }
            
            // Update final status
            if let Ok(mut status) = status_clone.lock() {
                *status = "✅ Análisis completado exitosamente".to_string();
            }
            
            // Mark as finished
            if let Ok(mut finished) = finished_clone.lock() {
                *finished = true;
            }
        });
    }

    fn start_release_notes_analysis(&self, analysis_state: ReleaseNotesAnalysisState) {
        use crate::git::GitRepo;
        use crate::monday::MondayClient;
        use crate::gemini::GeminiClient;
        use std::thread;
        use std::fs;
        use std::collections::HashSet;
        use chrono::Utc;

        // Clone data needed for the thread
        let config_clone = self.config.clone();
        
        // Clone analysis state components
        let status_clone = analysis_state.status.clone();
        let finished_clone = analysis_state.finished.clone();
        let success_clone = analysis_state.success.clone();

        // Spawn the analysis in a background thread
        thread::spawn(move || {
            // Update status: getting commits
            if let Ok(mut status) = status_clone.lock() {
                *status = "📋 Obteniendo commits desde la última versión...".to_string();
            }
            
            // Get git repository
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
            
            // Get last tag and commits since then
            let last_tag = match git_repo.get_last_tag() {
                Ok(tag) => tag,
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error obteniendo última etiqueta: {}", e);
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
            
            let commits = match git_repo.get_commits_since_tag(last_tag.as_deref()) {
                Ok(commits) => commits,
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error obteniendo commits: {}", e);
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
            
            if commits.is_empty() {
                if let Ok(mut status) = status_clone.lock() {
                    *status = "⚠️ No se encontraron commits desde la última versión".to_string();
                }
                if let Ok(mut finished) = finished_clone.lock() {
                    *finished = true;
                }
                return;
            }
            
            // Update status: analyzing commits
            if let Ok(mut status) = status_clone.lock() {
                *status = format!("📊 Se encontraron {} commits para analizar", commits.len());
            }
            
            // Extract Monday.com task IDs from commits
            let mut monday_task_ids = HashSet::new();
            for commit in &commits {
                // Check scope for task IDs
                if let Some(scope) = &commit.scope {
                    for id in scope.split('|') {
                        if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
                            monday_task_ids.insert(id.to_string());
                        }
                    }
                }
                
                // Check monday task mentions
                for mention in &commit.monday_task_mentions {
                    monday_task_ids.insert(mention.id.clone());
                }
                
                // Check monday_tasks field
                for task_id in &commit.monday_tasks {
                    monday_task_ids.insert(task_id.clone());
                }
            }
            
            // Update status: getting Monday tasks
            if let Ok(mut status) = status_clone.lock() {
                *status = format!("🔍 Obteniendo detalles de {} tareas de Monday.com...", monday_task_ids.len());
            }
            
            // Extract responsible person from most recent commit author
            let responsible_person = if !commits.is_empty() {
                commits[0].author_name.clone()
            } else {
                "".to_string()
            };
            
            // Get Monday.com task details using a blocking runtime
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
            
            let mut monday_tasks = if !monday_task_ids.is_empty() {
                match MondayClient::new(&config_clone) {
                    Ok(client) => {
                        let task_ids: Vec<String> = monday_task_ids.iter().cloned().collect();
                        match rt.block_on(client.get_task_details(&task_ids)) {
                            Ok(tasks) => tasks,
                            Err(_) => Vec::new()
                        }
                    }
                    Err(_) => Vec::new()
                }
            } else {
                Vec::new()
            };
            
            // Create placeholder tasks for missing IDs (same logic as original)
            let found_task_ids: HashSet<String> = monday_tasks.iter().map(|task| task.id.clone()).collect();
            for task_id in &monday_task_ids {
                if !found_task_ids.contains(task_id) {
                    let mut title = "Task not found in Monday API".to_string();
                    
                    // Try to extract title from commits that mention this task
                    for commit in &commits {
                        for mention in &commit.monday_task_mentions {
                            if &mention.id == task_id {
                                title = mention.title.clone();
                                break;
                            }
                        }
                    }
                    
                    // Create placeholder Monday task
                    use crate::types::MondayTask;
                    let placeholder_task = MondayTask {
                        id: task_id.clone(),
                        title,
                        board_id: Some("".to_string()),
                        board_name: Some("".to_string()),
                        url: "".to_string(),
                        state: "active".to_string(),
                        updates: Vec::new(),
                        group_title: Some("".to_string()),
                        column_values: Vec::new(),
                    };
                    
                    monday_tasks.push(placeholder_task);
                }
            }
            
            // Update status: generating version
            if let Ok(mut status) = status_clone.lock() {
                *status = "🏷️ Generando versión y estructura...".to_string();
            }
            
            // Get version and create tag format (same logic as original)
            let version = match crate::git::get_next_version() {
                Ok(v) => {
                    if v != "next version" && v != "próxima versión" && !v.is_empty() {
                        let date_str = Utc::now().format("%Y%m%d").to_string();
                        format!("tag-teixo-{}-{}", date_str, v)
                    } else {
                        let date_str = Utc::now().format("%Y%m%d").to_string();
                        match git_repo.get_last_tag() {
                            Ok(Some(tag)) => {
                                if let Some(version_part) = tag.split('-').next_back() {
                                    if let Ok(mut version_num) = version_part.parse::<f32>() {
                                        version_num += 0.001;
                                        format!("tag-teixo-{}-{:.3}", date_str, version_num)
                                    } else {
                                        format!("tag-teixo-{}-1.112.0", date_str)
                                    }
                                } else {
                                    format!("tag-teixo-{}-1.112.0", date_str)
                                }
                            },
                            _ => format!("tag-teixo-{}-1.112.0", date_str),
                        }
                    }
                },
                Err(_) => {
                    let date_str = Utc::now().format("%Y%m%d").to_string();
                    format!("tag-teixo-{}-1.112.0", date_str)
                },
            };
            
            // Update status: generating document
            if let Ok(mut status) = status_clone.lock() {
                *status = format!("📄 Generando documento estructurado para versión {}...", version);
            }
            
            // Generate the structured document (using embedded function logic)
            let structured_document = Self::generate_raw_release_notes_static(&version, &commits, &monday_tasks, &responsible_person);
            
            // Create output directory and save files
            if let Err(_) = fs::create_dir_all("release-notes") {
                // Continue even if directory creation fails
            }
            
            let date_str = Utc::now().format("%Y-%m-%d").to_string();
            let structured_filename = format!("release-notes/release-notes-{}_SCRIPT_WITH_LETTER_O.md", date_str);
            let gemini_filename = format!("release-notes/release-notes-{}_GEMINI_WITH_LETTER_O.md", date_str);
            
            // Save the structured document
            if let Err(e) = fs::write(&structured_filename, &structured_document) {
                if let Ok(mut status) = status_clone.lock() {
                    *status = format!("❌ Error guardando documento estructurado: {}", e);
                }
                if let Ok(mut success) = success_clone.lock() {
                    *success = false;
                }
                if let Ok(mut finished) = finished_clone.lock() {
                    *finished = true;
                }
                return;
            }
            
            // Update status: processing with Gemini
            if let Ok(mut status) = status_clone.lock() {
                *status = "🤖 Enviando documento a Google Gemini API...".to_string();
            }
            
            // Try to process with Gemini
            match GeminiClient::new(&config_clone) {
                Ok(gemini_client) => {
                    match rt.block_on(gemini_client.process_release_notes_document(&structured_document)) {
                        Ok(gemini_response) => {
                            // Save the Gemini-processed version
                            if let Err(_) = fs::write(&gemini_filename, &gemini_response) {
                                // Continue even if saving Gemini version fails
                            }
                            
                            if let Ok(mut status) = status_clone.lock() {
                                *status = format!(
                                    "✅ Notas de versión generadas exitosamente:\n📄 Documento estructurado: {}\n🤖 Versión procesada por Gemini: {}",
                                    structured_filename, gemini_filename
                                );
                            }
                        }
                        Err(_) => {
                            if let Ok(mut status) = status_clone.lock() {
                                *status = format!(
                                    "⚠️ Gemini falló, pero se generó el documento estructurado:\n📄 Documento estructurado: {}",
                                    structured_filename
                                );
                            }
                        }
                    }
                }
                Err(_) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!(
                            "⚠️ Gemini no configurado, solo se generó el documento estructurado:\n📄 Documento estructurado: {}",
                            structured_filename
                        );
                    }
                }
            }
            
            // Mark as finished
            if let Ok(mut finished) = finished_clone.lock() {
                *finished = true;
            }
        });
    }
    
    // Static version of generate_raw_release_notes for use in background thread
    fn generate_raw_release_notes_static(version: &str, commits: &[crate::types::GitCommit], monday_tasks: &[crate::types::MondayTask], responsible_person: &str) -> String {
        use chrono::Utc;
        use std::collections::HashMap;
        use std::fs;
        
        // Create a mapping of task ID to task details for quick lookup
        let task_details_map: HashMap<String, &crate::types::MondayTask> = monday_tasks.iter()
            .map(|task| (task.id.clone(), task))
            .collect();
        
        // Group commits by type
        let mut commits_by_type: HashMap<String, Vec<&crate::types::GitCommit>> = HashMap::new();
        for commit in commits {
            let commit_type = commit.commit_type.as_deref().unwrap_or("other").to_string();
            commits_by_type.entry(commit_type).or_insert_with(Vec::new).push(commit);
        }
        
        let mut document = String::new();
        
        // Header
        document.push_str(&format!("# Datos para Generación de Notas de Versión {}\n\n", version));
        
        // General Information
        document.push_str("## Información General\n\n");
        document.push_str(&format!("- **Versión**: {}\n", version));
        document.push_str(&format!("- **Fecha**: {}\n", 
            Utc::now().format("%d/%m/%Y")));
        document.push_str(&format!("- **Total de Commits**: {}\n", commits.len()));
        document.push_str(&format!("- **Tareas de Monday relacionadas**: {}\n\n", monday_tasks.len()));
        
        // Instructions for Gemini
        document.push_str("## Instrucciones CRÍTICAS\n\n");
        document.push_str("DEBES seguir EXACTAMENTE la plantilla que se proporciona al final de este documento. ");
        document.push_str("NO crees un resumen o documento libre. COPIA la estructura de la plantilla y RELLENA cada sección. ");
        document.push_str("OBLIGATORIO: \n");
        document.push_str(&format!("1. El responsable del despliegue es: {} - úsalo en la sección 'Responsable despliegue'.\n", responsible_person));
        document.push_str("2. Para las tareas de Monday.com, usa SIEMPRE el formato 'm' + ID (ej: m8817155664).\n");
        document.push_str("3. En la tabla 'Información para N1', incluye TODAS las tareas BUG con SupportBee links.\n");
        document.push_str("4. Para las secciones Correcciones y Proyectos especiales, usa solo las tareas con labels BUG y PE.\n");
        document.push_str("5. En las tablas de validación, incluye descripciones específicas basadas en el título de cada tarea.\n");
        document.push_str("6. Incluye TODOS los commits en 'Referencia commits' con el formato exacto mostrado.\n");
        document.push_str(&format!("7. Usa el título: '# Actualización Teixo versión {}'.\n", version));
        document.push_str("8. Si una tabla está vacía en la plantilla, déjala vacía pero manténla.\n");
        document.push_str("CRÍTICO: No inventes información, usa solo los datos proporcionados.\n\n");
        
        // Summary of changes by type
        document.push_str("## Resumen de Cambios\n\n");
        
        // New Features (feat)
        if let Some(feat_commits) = commits_by_type.get("feat") {
            if !feat_commits.is_empty() {
                document.push_str(&format!("### Nuevas Funcionalidades ({})\n\n", feat_commits.len()));
                for commit in feat_commits {
                    document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                        commit.description,
                        commit.hash.chars().take(7).collect::<String>(),
                        &commit.author_name,
                        &commit.author_email,
                        commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                    ));
                    if !commit.body.trim().is_empty() {
                        document.push_str(&format!("  - Detalles: {}\n", Self::format_multiline_text_static(&commit.body)));
                    }
                }
                document.push('\n');
            }
        }
        
        // Bug Fixes (fix)
        if let Some(fix_commits) = commits_by_type.get("fix") {
            if !fix_commits.is_empty() {
                document.push_str(&format!("### Correcciones ({})\n\n", fix_commits.len()));
                for commit in fix_commits {
                    document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                        commit.description,
                        commit.hash.chars().take(7).collect::<String>(),
                        &commit.author_name,
                        &commit.author_email,
                        commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                    ));
                    if !commit.body.trim().is_empty() {
                        document.push_str(&format!("  - Detalles: {}\n", Self::format_multiline_text_static(&commit.body)));
                    }
                }
                document.push('\n');
            }
        }
        
        // Other types of commits
        let other_types: Vec<&String> = commits_by_type.keys()
            .filter(|&type_name| !["feat", "fix"].contains(&type_name.as_str()))
            .collect();
        
        for type_name in other_types {
            if let Some(type_commits) = commits_by_type.get(type_name) {
                if !type_commits.is_empty() {
                    document.push_str(&format!("### {} ({})\n\n", Self::get_type_title_static(type_name), type_commits.len()));
                    for commit in type_commits {
                        document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                            commit.description,
                            commit.hash.chars().take(7).collect::<String>(),
                            &commit.author_name,
                            &commit.author_email,
                            commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                        ));
                        if !commit.body.trim().is_empty() {
                            document.push_str(&format!("  - Detalles: {}\n", Self::format_multiline_text_static(&commit.body)));
                        }
                    }
                    document.push('\n');
                }
            }
        }
        
        // Breaking changes
        let breaking_commits: Vec<&crate::types::GitCommit> = commits.iter()
            .filter(|commit| !commit.breaking_changes.is_empty())
            .collect();
        
        if !breaking_commits.is_empty() {
            document.push_str("## Cambios que Rompen Compatibilidad\n\n");
            for commit in breaking_commits {
                document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                    commit.description,
                    commit.hash.chars().take(7).collect::<String>(),
                    &commit.author_name,
                    &commit.author_email,
                    commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")
                ));
                document.push_str(&format!("  - Detalles: {}\n", 
                    commit.breaking_changes.join(" | ")));
            }
            document.push('\n');
        }
        
        // Monday.com task details
        if !monday_tasks.is_empty() {
            document.push_str("## Detalles de Tareas de Monday\n\n");
            
            for task in monday_tasks {
                document.push_str(&format!("### {} (ID: {})\n\n", task.title, task.id));
                document.push_str(&format!("- **Estado**: {}\n", task.state));
                document.push_str(&format!("- **Tablero**: {} (ID: {})\n", 
                    task.board_name.as_deref().unwrap_or("N/A"), 
                    task.board_id.as_deref().unwrap_or("N/A")));
                document.push_str(&format!("- **Grupo**: {}\n", 
                    task.group_title.as_deref().unwrap_or("N/A")));
                
                // Column values
                if !task.column_values.is_empty() {
                    document.push_str("- **Detalles**:\n");
                    let relevant_columns: Vec<&crate::types::MondayColumnValue> = task.column_values.iter()
                        .filter(|col| col.text.is_some() && !col.text.as_ref().unwrap().trim().is_empty())
                        .collect();
                    
                    if !relevant_columns.is_empty() {
                        for col in relevant_columns {
                            document.push_str(&format!("  - {}: {}\n", 
                                col.id, 
                                col.text.as_deref().unwrap_or("")));
                        }
                    } else {
                        document.push_str("  - No hay detalles adicionales disponibles\n");
                    }
                }
                
                // SupportBee links extracted from Monday task column values (texto field)
                let mut supportbee_links = Vec::new();
                for col in &task.column_values {
                    if col.id == "texto" {
                        if let Some(text) = &col.text {
                            let supportbee_regex = regex::Regex::new(r"https?://[^\s,]*teimas\.supportbee[^\s,]*").unwrap();
                            for mat in supportbee_regex.find_iter(text) {
                                let link = mat.as_str().to_string();
                                if !supportbee_links.contains(&link) {
                                    supportbee_links.push(link);
                                }
                            }
                        }
                    }
                }
                
                if !supportbee_links.is_empty() {
                    document.push_str("- **Enlaces SupportBee**:\n");
                    for link in supportbee_links {
                        document.push_str(&format!("  - {}\n", link));
                    }
                }
                
                // Recent updates (Actualizaciones Recientes)
                if !task.updates.is_empty() {
                    document.push_str("- **Actualizaciones Recientes**:\n");
                    
                    // Show the 3 most recent updates
                    for update in task.updates.iter().take(3) {
                        // Format the date from ISO string to DD/MM/YYYY format
                        let date = if let Ok(parsed_date) = update.created_at.parse::<chrono::DateTime<chrono::Utc>>() {
                            parsed_date.format("%d/%m/%Y").to_string()
                        } else if let Some(date_part) = update.created_at.split('T').next() {
                            // If we can't parse the full datetime, try to extract just the date part
                            if let Ok(parsed_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                                parsed_date.format("%d/%m/%Y").to_string()
                            } else {
                                update.created_at.clone()
                            }
                        } else {
                            update.created_at.clone()
                        };
                        
                        let creator_name = update.creator.as_ref()
                            .map(|c| c.name.as_str())
                            .unwrap_or("Usuario");
                        
                        // Truncate the body to 100 characters max
                        let body_preview = if update.body.len() > 100 {
                            format!("{}...", &update.body[..100])
                        } else {
                            update.body.clone()
                        };
                        
                        document.push_str(&format!("  - {} por {}: {}\n", date, creator_name, body_preview));
                    }
                }
                
                // Related commits
                let related_commits: Vec<&crate::types::GitCommit> = commits.iter()
                    .filter(|commit| {
                        // Check scope
                        if let Some(scope) = &commit.scope {
                            if scope.split('|').any(|id| id == task.id) {
                                return true;
                            }
                        }
                        
                        // Check monday_task_mentions
                        commit.monday_task_mentions.iter().any(|mention| mention.id == task.id)
                    })
                    .collect();
                
                if !related_commits.is_empty() {
                    document.push_str("- **Commits Relacionados**:\n");
                    for commit in related_commits {
                        document.push_str(&format!("  - {}: {} [{}]\n", 
                            commit.commit_type.as_deref().unwrap_or("other"),
                            commit.description,
                            commit.hash.chars().take(7).collect::<String>()));
                    }
                }
                
                document.push('\n');
            }
        }
        
        // Complete commit details
        document.push_str("## Detalles Completos de Commits\n\n");
        
        for commit in commits {
            let scope_part = if let Some(scope) = &commit.scope {
                format!("({})", scope)
            } else {
                String::new()
            };
            
            document.push_str(&format!("### {}{}: {} [{}]\n\n", 
                commit.commit_type.as_deref().unwrap_or("other"),
                scope_part,
                commit.description,
                commit.hash.chars().take(7).collect::<String>()));
            
            document.push_str(&format!("**Autor**: {} <{}>\n", 
                &commit.author_name,
                &commit.author_email));
            document.push_str(&format!("**Fecha**: {}\n\n", 
                commit.commit_date.with_timezone(&Local).format("%a %b %d %H:%M:%S %Y %z")));
            
            if !commit.body.trim().is_empty() {
                document.push_str(&format!("{}\n\n", Self::format_multiline_text_static(&commit.body)));
            }
            
            if !commit.test_details.is_empty() {
                document.push_str("**Pruebas**:\n");
                for test in &commit.test_details {
                    document.push_str(&format!("- {}\n", test));
                }
                document.push('\n');
            }
            
            if let Some(security) = &commit.security {
                if security != "NA" {
                    document.push_str(&format!("**Seguridad**: {}\n\n", security));
                }
            }
            
            if !commit.monday_task_mentions.is_empty() {
                document.push_str("**Tareas relacionadas**:\n");
                
                for mention in &commit.monday_task_mentions {
                    let task_details = task_details_map.get(&mention.id);
                    let task_name = task_details.map(|t| t.title.as_str()).unwrap_or(&mention.title);
                    let task_state = task_details.map(|t| t.state.as_str()).unwrap_or("Desconocido");
                    
                    document.push_str(&format!("- {} (ID: {}, Estado: {})\n", 
                        task_name, mention.id, task_state));
                }
                
                document.push('\n');
            }
            
            document.push_str("---\n\n");
        }

        document.push_str("La plantilla a utilizar para generar el documento tiene que ser la siguiente. Fijate en todo lo que hay y emúlalo por completo.");
        
        // Read and include the template content (like Node.js does)
        match fs::read_to_string("scripts/plantilla.md") {
            Ok(plantilla_content) => {
                document.push_str(&format!("\n\n{}", plantilla_content));
                println!("✅ Plantilla cargada exitosamente: scripts/plantilla.md");
            }
            Err(e) => {
                println!("⚠️ No se pudo cargar la plantilla scripts/plantilla.md: {}", e);
                // If we can't load the template, at least add the instruction to use the original template format
                document.push_str("\n\nPor favor, utiliza el formato estándar de notas de versión de Teixo que incluye las secciones de Información para N1, Información técnica, Correcciones, Novedades (por categorías), Validación en Sandbox, Pruebas y Referencia commits.");
            }
        }
        
        document
    }
    
    // Static helper methods for the static function
    fn format_multiline_text_static(text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }
        
        text.split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" | ")
    }
    
    fn get_type_title_static(commit_type: &str) -> String {
        match commit_type {
            "feat" => "Nuevas Funcionalidades".to_string(),
            "fix" => "Correcciones".to_string(),
            "docs" => "Documentación".to_string(),
            "style" => "Estilo".to_string(),
            "refactor" => "Refactorizaciones".to_string(),
            "perf" => "Mejoras de Rendimiento".to_string(),
            "test" => "Pruebas".to_string(),
            "build" => "Construcción".to_string(),
            "ci" => "Integración Continua".to_string(),
            "chore" => "Tareas".to_string(),
            "revert" => "Reversiones".to_string(),
            _ => {
                let first_char = commit_type.chars().next().unwrap().to_uppercase().collect::<String>();
                format!("{}{}", first_char, &commit_type[1..])
            }
        }
    }
} 