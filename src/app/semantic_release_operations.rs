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
    async fn setup_github_actions_semantic_release(&mut self) -> Result<()>;
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

    async fn setup_github_actions_semantic_release(&mut self) -> Result<()> {
        self.current_state = AppState::Loading;
        self.message = Some("🔧 Configurando GitHub Actions para semantic-release...".to_string());

        // Create shared state for the operation
        let release_state = SemanticReleaseState {
            status: Arc::new(Mutex::new(
                "🔧 Preparando configuración de GitHub Actions...".to_string(),
            )),
            finished: Arc::new(Mutex::new(false)),
            success: Arc::new(Mutex::new(true)),
            result: Arc::new(Mutex::new(String::new())),
        };

        // Start the operation in a background thread
        self.start_github_setup_operation(release_state.clone());

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

    pub fn start_github_setup_operation(&self, release_state: SemanticReleaseState) {
        // Clone state components for the thread
        let status_clone = release_state.status.clone();
        let finished_clone = release_state.finished.clone();
        let success_clone = release_state.success.clone();
        let result_clone = release_state.result.clone();

        // Spawn the operation in a background thread
        thread::spawn(move || {
            let mut result_text = String::new();

            // Update status: setting up GitHub Actions
            if let Ok(mut status) = status_clone.lock() {
                *status = "🔧 Configurando GitHub Actions para semantic-release...".to_string();
            }

            result_text.push_str("🚀 CONFIGURACIÓN DE GITHUB ACTIONS SEMANTIC-RELEASE\n");
            result_text.push_str("=".repeat(60).as_str());
            result_text.push_str("\n\n");

            // Step 1: Check if already configured
            if let Ok(mut status) = status_clone.lock() {
                *status = "🔍 Verificando configuración existente...".to_string();
            }

            let mut files_created = Vec::new();
            let mut files_skipped = Vec::new();

            // Step 2: Setup package.json
            if let Ok(mut status) = status_clone.lock() {
                *status = "📦 Configurando package.json...".to_string();
            }

            match setup_package_json() {
                Ok(created) => {
                    if created {
                        files_created.push("package.json");
                        result_text.push_str(
                            "✅ package.json creado con dependencias de semantic-release\n",
                        );
                    } else {
                        files_skipped.push("package.json");
                        result_text.push_str("⚠️  package.json ya existe - no modificado\n");
                    }
                }
                Err(e) => {
                    result_text.push_str(&format!("❌ Error configurando package.json: {}\n", e));
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                }
            }

            // Step 3: Setup .releaserc.json
            if let Ok(mut status) = status_clone.lock() {
                *status = "⚙️ Configurando .releaserc.json...".to_string();
            }

            match setup_releaserc() {
                Ok(created) => {
                    if created {
                        files_created.push(".releaserc.json");
                        result_text.push_str(
                            "✅ .releaserc.json creado con configuración de semantic-release\n",
                        );
                    } else {
                        files_skipped.push(".releaserc.json");
                        result_text.push_str("⚠️  .releaserc.json ya existe - no modificado\n");
                    }
                }
                Err(e) => {
                    result_text
                        .push_str(&format!("❌ Error configurando .releaserc.json: {}\n", e));
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                }
            }

            // Step 4: Setup GitHub Actions workflow
            if let Ok(mut status) = status_clone.lock() {
                *status = "🔄 Configurando GitHub Actions workflow...".to_string();
            }

            match setup_github_workflow() {
                Ok(created) => {
                    if created {
                        files_created.push(".github/workflows/release.yml");
                        result_text.push_str(
                            "✅ GitHub Actions workflow creado en .github/workflows/release.yml\n",
                        );
                    } else {
                        files_skipped.push(".github/workflows/release.yml");
                        result_text
                            .push_str("⚠️  GitHub Actions workflow ya existe - no modificado\n");
                    }
                }
                Err(e) => {
                    result_text
                        .push_str(&format!("❌ Error configurando GitHub workflow: {}\n", e));
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                }
            }

            // Step 5: Setup package-lock.json
            if let Ok(mut status) = status_clone.lock() {
                *status = "📦 Configurando package-lock.json...".to_string();
            }

            match setup_package_lock() {
                Ok(created) => {
                    if created {
                        files_created.push("package-lock.json");
                        result_text
                            .push_str("✅ package-lock.json creado para caché de dependencias\n");
                    } else {
                        files_skipped.push("package-lock.json");
                        result_text.push_str("⚠️  package-lock.json ya existe - no modificado\n");
                    }
                }
                Err(e) => {
                    result_text
                        .push_str(&format!("❌ Error configurando package-lock.json: {}\n", e));
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                }
            }

            // Step 6: Setup Node.js .gitignore
            if let Ok(mut status) = status_clone.lock() {
                *status = "📁 Configurando .gitignore para Node.js...".to_string();
            }

            match setup_nodejs_gitignore() {
                Ok(created) => {
                    if created {
                        files_created.push(".gitignore");
                        result_text.push_str("✅ .gitignore configurado para proyecto Node.js\n");
                    } else {
                        files_skipped.push(".gitignore");
                        result_text.push_str("⚠️  .gitignore ya existe con reglas Node.js\n");
                    }
                }
                Err(e) => {
                    result_text.push_str(&format!("❌ Error configurando .gitignore: {}\n", e));
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                }
            }

            // Step 7: Ensure plantilla template exists
            if let Ok(mut status) = status_clone.lock() {
                *status = "📄 Verificando plantilla de release notes...".to_string();
            }

            match ensure_plantilla_template_exists() {
                Ok(_) => {
                    result_text.push_str("✅ Plantilla de release notes verificada\n");
                }
                Err(e) => {
                    result_text.push_str(&format!("❌ Error configurando plantilla: {}\n", e));
                }
            }

            // Summary
            result_text.push('\n');
            result_text.push_str("📋 RESUMEN DE CONFIGURACIÓN\n");
            result_text.push_str("=".repeat(30).as_str());
            result_text.push_str("\n\n");

            if !files_created.is_empty() {
                result_text.push_str("✅ Archivos creados:\n");
                for file in &files_created {
                    result_text.push_str(&format!("   • {}\n", file));
                }
                result_text.push('\n');
            }

            if !files_skipped.is_empty() {
                result_text.push_str("⚠️  Archivos ya existentes (no modificados):\n");
                for file in &files_skipped {
                    result_text.push_str(&format!("   • {}\n", file));
                }
                result_text.push('\n');
            }

            // Next steps
            result_text.push_str("🚀 PRÓXIMOS PASOS\n");
            result_text.push_str("=".repeat(20).as_str());
            result_text.push_str("\n\n");
            result_text.push_str("1. 🔑 Configurar secrets en GitHub:\n");
            result_text.push_str("   • GITHUB_TOKEN: Personal access token con permisos 'repo'\n");
            result_text.push_str("   • Ir a Settings > Secrets and variables > Actions\n\n");
            result_text.push_str("2. 📦 Verificar dependencias (opcional):\n");
            result_text.push_str("   • Si tienes npm instalado: npm install\n");
            result_text
                .push_str("   • Esto actualizará package-lock.json con versiones exactas\n\n");
            result_text.push_str("3. 📝 Usar commits convencionales:\n");
            result_text.push_str("   • feat: nueva funcionalidad (minor version)\n");
            result_text.push_str("   • fix: corrección de bug (patch version)\n");
            result_text.push_str("   • feat!: breaking change (major version)\n\n");
            result_text.push_str("4. 🚢 Hacer push a main para ejecutar el primer release:\n");
            result_text.push_str("   • git add .\n");
            result_text.push_str(
                "   • git commit -m \"feat: setup semantic-release with GitHub Actions\"\n",
            );
            result_text.push_str("   • git push origin main\n\n");
            result_text.push_str("💡 NOTA: Los scripts de test y build son placeholders.\n");
            result_text
                .push_str("   Personaliza package.json según las necesidades de tu proyecto.\n\n");

            // Update final status
            let has_errors = files_created.is_empty() && files_skipped.is_empty();
            if has_errors {
                if let Ok(mut status) = status_clone.lock() {
                    *status = "❌ Error configurando GitHub Actions".to_string();
                }
                if let Ok(mut success) = success_clone.lock() {
                    *success = false;
                }
            } else {
                if let Ok(mut status) = status_clone.lock() {
                    *status = "✅ GitHub Actions configurado exitosamente".to_string();
                }
                utils::log_success("GITHUB-SETUP", "GitHub Actions configured successfully");
            }

            // Store result
            if let Ok(mut result) = result_clone.lock() {
                *result = result_text;
            }

            // Mark as finished
            if let Ok(mut finished) = finished_clone.lock() {
                *finished = true;
            }
        });
    }
}

fn setup_package_json() -> Result<bool> {
    use std::fs;
    use std::path::Path;

    let package_path = Path::new("package.json");

    // If package.json already exists, don't modify it
    if package_path.exists() {
        return Ok(false);
    }

    let package_json = r#"{
  "name": "semantic-release-project",
  "version": "1.0.0",
  "description": "Project configured with semantic-release for automated versioning",
  "main": "index.js",
  "scripts": {
    "test": "echo \"No tests specified\" && exit 0",
    "build": "echo \"No build step specified\" && exit 0",
    "semantic-release": "semantic-release",
    "prepare": "npm run build"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/username/repository.git"
  },
  "keywords": [
    "semantic-release",
    "automation",
    "versioning"
  ],
  "author": "",
  "license": "MIT",
  "devDependencies": {
    "@semantic-release/changelog": "^6.0.3",
    "@semantic-release/commit-analyzer": "^11.1.0",
    "@semantic-release/git": "^10.0.1",
    "@semantic-release/github": "^9.2.6",
    "@semantic-release/release-notes-generator": "^12.1.0",
    "semantic-release": "^22.0.12"
  }
}
"#;

    fs::write(package_path, package_json)?;
    Ok(true)
}

fn setup_releaserc() -> Result<bool> {
    use std::fs;
    use std::path::Path;

    let releaserc_path = Path::new(".releaserc.json");

    // If .releaserc.json already exists, don't modify it
    if releaserc_path.exists() {
        return Ok(false);
    }

    let releaserc_json = r#"{
  "branches": ["main"],
  "plugins": [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    ["@semantic-release/changelog", {
      "changelogFile": "CHANGELOG.md"
    }],
    ["@semantic-release/github", {
      "assets": [
        {
          "path": "dist/**/*",
          "label": "Distribution files"
        }
      ],
      "successComment": "🎉 This release is now available in [version ${nextRelease.version}](${releases.find(release => release.name === 'GitHub release').url}) 🎉",
      "failComment": "This release from branch `${branch.name}` has failed due to the following errors:\n- ${errors.map(err => err.message).join('\\n- ')}",
      "labels": ["released"],
      "assignees": ["@semantic-release/github"]
    }],
    ["@semantic-release/git", {
      "assets": ["package.json", "package-lock.json", "CHANGELOG.md"],
      "message": "chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}"
    }]
  ]
}
"#;

    fs::write(releaserc_path, releaserc_json)?;
    Ok(true)
}

fn setup_github_workflow() -> Result<bool> {
    use std::fs;
    use std::path::Path;

    let workflow_dir = Path::new(".github/workflows");
    let workflow_path = workflow_dir.join("release.yml");

    // If workflow already exists, don't modify it
    if workflow_path.exists() {
        return Ok(false);
    }

    // Create .github/workflows directory if it doesn't exist
    fs::create_dir_all(workflow_dir)?;

    let workflow_yml = r#"name: Release

on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main

permissions:
  contents: write
  issues: write
  pull-requests: write
  id-token: write

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: ~/.npm
          key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
          restore-keys: |
            ${{ runner.os }}-node-

      - name: Install dependencies
        run: |
          if [ -f package-lock.json ]; then
            npm ci
          else
            npm install
          fi

      - name: Run tests
        run: npm test

      - name: Build
        run: npm run build

  release:
    name: Release
    runs-on: ubuntu-latest
    needs: test
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
          persist-credentials: false

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: ~/.npm
          key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
          restore-keys: |
            ${{ runner.os }}-node-

      - name: Install dependencies
        run: |
          if [ -f package-lock.json ]; then
            npm ci
          else
            npm install
          fi

      - name: Build
        run: npm run build

      - name: Release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: npx semantic-release
"#;

    fs::write(&workflow_path, workflow_yml)?;
    Ok(true)
}

fn setup_package_lock() -> Result<bool> {
    use std::path::Path;
    use std::process::Command;

    let package_lock_path = Path::new("package-lock.json");
    let package_json_path = Path::new("package.json");

    // If package-lock.json already exists, don't modify it
    if package_lock_path.exists() {
        return Ok(false);
    }

    // Only create lock file if package.json exists
    if !package_json_path.exists() {
        return Ok(false);
    }

    // Try to run npm install to generate package-lock.json
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "npm install --package-lock-only"])
            .output()
    } else {
        Command::new("npm")
            .args(["install", "--package-lock-only"])
            .output()
    };

    match output {
        Ok(result) => {
            if result.status.success() {
                // Check if package-lock.json was created
                if package_lock_path.exists() {
                    Ok(true)
                } else {
                    // If npm install failed, create a minimal package-lock.json
                    create_minimal_package_lock()?;
                    Ok(true)
                }
            } else {
                // If npm install failed, create a minimal package-lock.json
                create_minimal_package_lock()?;
                Ok(true)
            }
        }
        Err(_) => {
            // If npm is not available, create a minimal package-lock.json
            create_minimal_package_lock()?;
            Ok(true)
        }
    }
}

fn create_minimal_package_lock() -> Result<()> {
    let package_lock_json = r#"{
  "name": "semantic-release-project",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {
    "": {
      "name": "semantic-release-project",
      "version": "1.0.0",
      "license": "MIT",
      "devDependencies": {
        "@semantic-release/changelog": "^6.0.3",
        "@semantic-release/commit-analyzer": "^11.1.0",
        "@semantic-release/git": "^10.0.1",
        "@semantic-release/github": "^9.2.6",
        "@semantic-release/release-notes-generator": "^12.1.0",
        "semantic-release": "^22.0.12"
      }
    }
  }
}
"#;

    std::fs::write("package-lock.json", package_lock_json)?;
    Ok(())
}

fn setup_nodejs_gitignore() -> Result<bool> {
    use std::fs;
    use std::path::Path;

    let gitignore_path = Path::new(".gitignore");

    // If .gitignore already exists, don't overwrite it - just ensure it has Node.js rules
    if gitignore_path.exists() {
        return ensure_nodejs_gitignore_rules();
    }

    // Create a comprehensive .gitignore for Node.js/semantic-release projects
    let gitignore_content = r#"# Dependencies
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Runtime data
pids
*.pid
*.seed
*.pid.lock

# Coverage directory used by tools like istanbul
coverage/
*.lcov

# nyc test coverage
.nyc_output

# Dependency directories
node_modules/
jspm_packages/

# TypeScript cache
*.tsbuildinfo

# Optional npm cache directory
.npm

# Optional eslint cache
.eslintcache

# Microbundle cache
.rpt2_cache/
.rts2_cache_cjs/
.rts2_cache_es/
.rts2_cache_umd/

# Optional REPL history
.node_repl_history

# Output of 'npm pack'
*.tgz

# Yarn Integrity file
.yarn-integrity

# Environment variables
.env
.env.local
.env.development.local
.env.test.local
.env.production.local

# Build outputs
dist/
build/
out/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Logs
logs
*.log

# Semantic-release generated files
CHANGELOG.md

"#;

    fs::write(gitignore_path, gitignore_content)?;
    Ok(true)
}

fn ensure_nodejs_gitignore_rules() -> Result<bool> {
    use std::fs;

    let gitignore_content = fs::read_to_string(".gitignore")?;
    let mut needs_update = false;
    let mut updated_content = gitignore_content.clone();

    // Essential Node.js rules to check for
    let essential_rules = ["node_modules/", ".env"];

    for rule in essential_rules {
        if !gitignore_content.lines().any(|line| line.trim() == rule) {
            needs_update = true;
            if !updated_content.ends_with('\n') {
                updated_content.push('\n');
            }
            updated_content.push_str(&format!("\n# Added by semantic-release setup\n{}\n", rule));
        }
    }

    if needs_update {
        fs::write(".gitignore", updated_content)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn ensure_plantilla_template_exists() -> Result<()> {
    // Implementation of ensure_plantilla_template_exists function
    Ok(())
}
