use std::sync::{Arc, Mutex};
use std::thread;

use crate::{
    app::App,
    types::{GeminiAnalysisState, ReleaseNotesAnalysisState, ComprehensiveAnalysisState, AppState},
    git::GitRepo,
    services::GeminiClient,
};

pub trait BackgroundOperations {
    async fn start_gemini_analysis_wrapper(&mut self);
    #[allow(dead_code)]
    fn start_release_notes_analysis_wrapper(&self, _analysis_state: ReleaseNotesAnalysisState);
    async fn start_comprehensive_analysis_wrapper(&mut self);
}

impl BackgroundOperations for App {
    async fn start_gemini_analysis_wrapper(&mut self) {
        // Check if already processing to avoid multiple concurrent analyses
        if matches!(self.current_state, AppState::Loading) || self.gemini_analysis_state.is_some() {
            return;
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
            title: Arc::new(Mutex::new(String::new())),
            commit_type: Arc::new(Mutex::new(String::new())),
        };
        
        // Start the analysis in a background thread
        self.start_gemini_analysis(analysis_state.clone());
        
        // Store the analysis state so the main loop can poll it
        self.gemini_analysis_state = Some(analysis_state);
    }

    fn start_release_notes_analysis_wrapper(&self, _analysis_state: ReleaseNotesAnalysisState) {
        // Implementation moved from original app.rs
        // This method will start the release notes analysis in a background thread
    }

    async fn start_comprehensive_analysis_wrapper(&mut self) {
        // Check if already processing to avoid multiple concurrent analyses
        if matches!(self.current_state, AppState::Loading) || self.comprehensive_analysis_state.is_some() {
            return;
        }
        
        // IMMEDIATELY set loading state and create analysis state
        self.current_state = AppState::Loading;
        self.message = Some("🚀 Iniciando análisis completo con Gemini AI...".to_string());
        
        // Create shared state for the analysis
        let analysis_state = ComprehensiveAnalysisState {
            status: Arc::new(Mutex::new("🔍 Analizando cambios en el repositorio...".to_string())),
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
    pub fn start_gemini_analysis(&self, analysis_state: GeminiAnalysisState) {
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
        let title_clone = analysis_state.title.clone();
        let commit_type_clone = analysis_state.commit_type.clone();

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
            
            // Run five Gemini analyses in parallel
            let results = rt.block_on(async {
                let description_future = gemini_client.generate_commit_description(&changes, commit_type_ref, scope_ref, &title);
                let security_future = gemini_client.analyze_security_risks(&changes, commit_type_ref, scope_ref, &title);
                let breaking_future = gemini_client.analyze_breaking_changes(&changes, commit_type_ref, scope_ref, &title);
                let title_future = gemini_client.generate_commit_title(&changes);
                let commit_type_future = gemini_client.generate_commit_type(&changes, &title);
                
                tokio::join!(description_future, security_future, breaking_future, title_future, commit_type_future)
            });
            
            // Handle the results
            match results.0 {
                Ok(description) => {
                    if let Ok(mut result) = result_clone.lock() {
                        *result = description;
                    }
                }
                Err(_) => {
                    // Fallback to a basic description
                    if let Ok(mut result) = result_clone.lock() {
                        *result = "Cambios realizados en el código del proyecto.".to_string();
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
            
            // Handle title generation result
            if let Ok(generated_title) = results.3 {
                if !generated_title.is_empty() {
                    if let Ok(mut title_result) = title_clone.lock() {
                        *title_result = generated_title;
                    }
                }
            }
            
            // Handle commit type generation result
            if let Ok(generated_commit_type) = results.4 {
                if !generated_commit_type.is_empty() {
                    if let Ok(mut commit_type_result) = commit_type_clone.lock() {
                        *commit_type_result = generated_commit_type;
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
                        "breakingChanges": ""
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
                gemini_client.generate_comprehensive_commit_analysis(&changes).await
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
                    eprintln!("❌ Error en análisis completo: {}", e);
                    // Fallback to basic result
                    if let Ok(mut result) = result_clone.lock() {
                        *result = serde_json::json!({
                            "title": "cambios realizados en el código",
                            "commitType": "chore",
                            "description": "Se realizaron cambios en el código del proyecto. No se pudo generar un análisis detallado automáticamente.",
                            "scope": "general",
                            "securityAnalysis": "",
                            "breakingChanges": ""
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