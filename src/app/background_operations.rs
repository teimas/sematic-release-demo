use std::sync::{Arc, Mutex};
use std::thread;

use crate::{
    app::App,
    git::GitRepo,
    services::GeminiClient,
    types::{AppState, ComprehensiveAnalysisState},
    utils,
};

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
