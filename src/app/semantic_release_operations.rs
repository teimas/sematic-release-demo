use anyhow::Result;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{
    app::App,
    types::{AppState, SemanticReleaseState},
    utils,
};

#[allow(async_fn_in_trait)]
pub trait SemanticReleaseOperations {
    async fn execute_semantic_release(&mut self, dry_run: bool) -> Result<()>;
    async fn view_last_release_info(&mut self) -> Result<()>;
    async fn view_semantic_release_config(&mut self) -> Result<()>;
    async fn get_detailed_version_info(&mut self) -> Result<()>;
}

impl SemanticReleaseOperations for App {
    async fn execute_semantic_release(&mut self, dry_run: bool) -> Result<()> {
        // Check if already processing
        if matches!(self.current_state, AppState::Loading) || self.semantic_release_state.is_some()
        {
            return Ok(());
        }

        // Set loading state immediately
        self.current_state = AppState::Loading;
        let action = if dry_run { "dry-run" } else { "release" };
        self.message = Some(format!("🚀 Ejecutando semantic-release {}...", action));

        // Create shared state for the operation
        let release_state = SemanticReleaseState {
            status: Arc::new(Mutex::new(format!(
                "📋 Preparando semantic-release {}...",
                action
            ))),
            finished: Arc::new(Mutex::new(false)),
            success: Arc::new(Mutex::new(true)),
            result: Arc::new(Mutex::new(String::new())),
        };

        // Start the operation in a background thread
        self.start_semantic_release_operation(release_state.clone(), dry_run);

        // Store the state so the main loop can poll it
        self.semantic_release_state = Some(release_state);

        Ok(())
    }

    async fn view_last_release_info(&mut self) -> Result<()> {
        self.current_state = AppState::Loading;
        self.message = Some("📋 Obteniendo información de la última versión...".to_string());

        match self.get_last_release_info().await {
            Ok(info) => {
                self.current_state = AppState::Normal;
                self.message = Some(format!("📦 Última versión: {}", info));
            }
            Err(e) => {
                utils::log_error("SEMANTIC-RELEASE", &e);
                self.current_state =
                    AppState::Error(format!("Error obteniendo información: {}", e));
            }
        }

        Ok(())
    }

    async fn view_semantic_release_config(&mut self) -> Result<()> {
        self.current_state = AppState::Loading;
        self.message = Some("⚙️ Verificando configuración de semantic-release...".to_string());

        match self.check_semantic_release_config().await {
            Ok(config_info) => {
                self.current_state = AppState::Normal;
                self.message = Some(format!("⚙️ Configuración: {}", config_info));
            }
            Err(e) => {
                utils::log_error("SEMANTIC-RELEASE", &e);
                self.current_state = AppState::Error(format!("Error en configuración: {}", e));
            }
        }

        Ok(())
    }

    async fn get_detailed_version_info(&mut self) -> Result<()> {
        self.current_state = AppState::Loading;
        self.message = Some("🔍 Analizando información de versión...".to_string());

        // Create shared state for the operation
        let release_state = SemanticReleaseState {
            status: Arc::new(Mutex::new(
                "🔍 Obteniendo información detallada de versión...".to_string(),
            )),
            finished: Arc::new(Mutex::new(false)),
            success: Arc::new(Mutex::new(true)),
            result: Arc::new(Mutex::new(String::new())),
        };

        // Start the operation in a background thread
        self.start_version_info_operation(release_state.clone());

        // Store the state so the main loop can poll it
        self.semantic_release_state = Some(release_state);

        Ok(())
    }
}

impl App {
    pub fn start_semantic_release_operation(
        &self,
        release_state: SemanticReleaseState,
        dry_run: bool,
    ) {
        // Clone state components for the thread
        let status_clone = release_state.status.clone();
        let finished_clone = release_state.finished.clone();
        let success_clone = release_state.success.clone();
        let result_clone = release_state.result.clone();

        // Spawn the operation in a background thread
        thread::spawn(move || {
            // Update status: checking prerequisites
            if let Ok(mut status) = status_clone.lock() {
                *status = "🔍 Verificando prerrequisitos de semantic-release...".to_string();
            }

            // Check if semantic-release is installed
            let check_cmd = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", "npx semantic-release --version"])
                    .output()
            } else {
                Command::new("npx")
                    .args(["semantic-release", "--version"])
                    .output()
            };

            match check_cmd {
                Ok(output) => {
                    if !output.status.success() {
                        if let Ok(mut status) = status_clone.lock() {
                            *status = "❌ semantic-release no está instalado".to_string();
                        }
                        if let Ok(mut success) = success_clone.lock() {
                            *success = false;
                        }
                        if let Ok(mut finished) = finished_clone.lock() {
                            *finished = true;
                        }
                        return;
                    }
                }
                Err(e) => {
                    utils::log_error("SEMANTIC-RELEASE", &e);
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error verificando semantic-release: {}", e);
                    }
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                    if let Ok(mut finished) = finished_clone.lock() {
                        *finished = true;
                    }
                    return;
                }
            }

            // Update status: executing semantic-release
            let action = if dry_run { "dry-run" } else { "release" };
            if let Ok(mut status) = status_clone.lock() {
                *status = format!("🚀 Ejecutando semantic-release {}...", action);
            }

            // Build command arguments
            let mut args = vec!["semantic-release"];
            if dry_run {
                args.push("--dry-run");
            }

            // Execute semantic-release
            let cmd_result = if cfg!(target_os = "windows") {
                let mut cmd_args = vec!["/C", "npx"];
                cmd_args.extend(args);
                Command::new("cmd")
                    .args(cmd_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
            } else {
                Command::new("npx")
                    .args(args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
            };

            match cmd_result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if output.status.success() {
                        if let Ok(mut status) = status_clone.lock() {
                            *status =
                                format!("✅ Semantic-release {} completado exitosamente", action);
                        }
                        if let Ok(mut result) = result_clone.lock() {
                            *result =
                                format!("Salida:\n{}\n\nErrores/Advertencias:\n{}", stdout, stderr);
                        }
                        utils::log_success(
                            "SEMANTIC-RELEASE",
                            &format!("Semantic-release {} completed successfully", action),
                        );
                        utils::log_debug("SEMANTIC-RELEASE", &format!("Output: {}", stdout));
                    } else {
                        if let Ok(mut status) = status_clone.lock() {
                            *status = format!("❌ Error en semantic-release {}", action);
                        }
                        if let Ok(mut result) = result_clone.lock() {
                            *result = format!("Error:\n{}\n\nSalida:\n{}", stderr, stdout);
                        }
                        if let Ok(mut success) = success_clone.lock() {
                            *success = false;
                        }
                        utils::log_error(
                            "SEMANTIC-RELEASE",
                            &format!("Semantic-release failed: {}", stderr),
                        );
                    }
                }
                Err(e) => {
                    utils::log_error("SEMANTIC-RELEASE", &e);
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error ejecutando semantic-release: {}", e);
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
    }

    async fn get_last_release_info(&self) -> Result<String> {
        // Get the last git tag
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "git describe --tags --abbrev=0"])
                .output()?
        } else {
            Command::new("git")
                .args(["describe", "--tags", "--abbrev=0"])
                .output()?
        };

        if output.status.success() {
            let last_tag = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Get commit count since last tag
            let commit_count_output = Command::new("git")
                .args(["rev-list", "--count", &format!("{}..HEAD", last_tag)])
                .output()?;

            let commit_count_str = String::from_utf8_lossy(&commit_count_output.stdout);
            let commit_count = commit_count_str.trim();

            if commit_count == "0" {
                Ok(format!(
                    "{} (sin cambios desde la última versión)",
                    last_tag
                ))
            } else {
                Ok(format!(
                    "{} ({} commits desde entonces)",
                    last_tag, commit_count
                ))
            }
        } else {
            Ok("No hay versiones anteriores encontradas".to_string())
        }
    }

    async fn check_semantic_release_config(&self) -> Result<String> {
        // Check if package.json exists
        if !std::path::Path::new("package.json").exists() {
            return Ok("❌ package.json no encontrado".to_string());
        }

        // Check if .releaserc.json or semantic-release config exists
        let has_releaserc = std::path::Path::new(".releaserc.json").exists();
        let has_releaserc_js = std::path::Path::new(".releaserc.js").exists();
        let has_release_config = std::path::Path::new("release.config.js").exists();

        // Check package.json for semantic-release dependency
        let package_json = std::fs::read_to_string("package.json")?;
        let has_semantic_release_dep = package_json.contains("semantic-release");

        let mut config_info = Vec::new();

        if has_semantic_release_dep {
            config_info.push("✅ semantic-release en package.json");
        } else {
            config_info.push("❌ semantic-release no está en package.json");
        }

        if has_releaserc {
            config_info.push("✅ .releaserc.json encontrado");
        } else if has_releaserc_js {
            config_info.push("✅ .releaserc.js encontrado");
        } else if has_release_config {
            config_info.push("✅ release.config.js encontrado");
        } else {
            config_info.push("⚠️ No se encontró configuración específica (usará defaults)");
        }

        // Check if we're in a git repository
        let git_check = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output();

        match git_check {
            Ok(output) => {
                if output.status.success() {
                    config_info.push("✅ Repositorio Git válido");
                } else {
                    config_info.push("❌ No es un repositorio Git");
                }
            }
            Err(_) => {
                config_info.push("❌ Git no disponible");
            }
        }

        Ok(config_info.join(", "))
    }

    pub fn start_version_info_operation(&self, release_state: SemanticReleaseState) {
        use crate::git::repository::get_version_info;

        // Clone state components for the thread
        let status_clone = release_state.status.clone();
        let finished_clone = release_state.finished.clone();
        let success_clone = release_state.success.clone();
        let result_clone = release_state.result.clone();

        // Spawn the operation in a background thread
        thread::spawn(move || {
            // Update status: analyzing version
            if let Ok(mut status) = status_clone.lock() {
                *status = "📊 Analizando información de versión...".to_string();
            }

            match get_version_info() {
                Ok(version_info) => {
                    let mut result_text = String::new();

                    // Current version section
                    result_text.push_str("📦 INFORMACIÓN DE VERSIÓN\n");
                    result_text.push_str("=".repeat(50).as_str());
                    result_text.push_str("\n\n");

                    if let Some(current) = &version_info.current_version {
                        result_text.push_str(&format!("🏷️  Versión actual: {}\n", current));
                    } else {
                        result_text.push_str("🏷️  Versión actual: Sin versiones anteriores\n");
                    }

                    result_text.push_str(&format!(
                        "🚀 Próxima versión: {}\n",
                        version_info.next_version
                    ));
                    result_text.push_str(&format!(
                        "📊 Tipo de release: {}\n",
                        version_info.version_type
                    ));
                    result_text.push_str(&format!(
                        "📈 Commits desde última versión: {}\n",
                        version_info.commit_count
                    ));

                    if version_info.has_unreleased_changes {
                        result_text.push_str("✅ Hay cambios para publicar\n");
                    } else {
                        result_text.push_str("⚠️  No hay cambios para publicar\n");
                    }

                    result_text.push('\n');
                    result_text.push_str("🔍 ANÁLISIS DETALLADO\n");
                    result_text.push_str("=".repeat(50).as_str());
                    result_text.push_str("\n\n");
                    result_text.push_str(&version_info.dry_run_output);

                    if let Ok(mut status) = status_clone.lock() {
                        *status = "✅ Análisis de versión completado".to_string();
                    }
                    if let Ok(mut result) = result_clone.lock() {
                        *result = result_text;
                    }
                    utils::log_success("VERSION-INFO", "Version analysis completed successfully");
                }
                Err(e) => {
                    utils::log_error("VERSION-INFO", &e);
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error analizando versión: {}", e);
                    }
                    if let Ok(mut result) = result_clone.lock() {
                        *result = format!("Error: {}", e);
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
    }
}
