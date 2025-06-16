use anyhow::Result;
use std::fs;
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::{Utc, Local};

use crate::{
    app::App,
    git::{GitRepo, get_next_version},
    monday::MondayClient,
    gemini::GeminiClient,
    types::{AppState, ReleaseNotesAnalysisState, MondayTask},
};

pub trait ReleaseNotesOperations {
    async fn handle_release_notes_generation(&mut self) -> Result<()>;
    async fn generate_release_notes_internal_wrapper(&mut self) -> Result<()>;
    async fn generate_release_notes_with_npm_wrapper(&mut self) -> Result<()>;
}

impl ReleaseNotesOperations for App {
    async fn handle_release_notes_generation(&mut self) -> Result<()> {
        // Default behavior: generate internal release notes
        self.generate_release_notes_internal().await
    }

    async fn generate_release_notes_internal_wrapper(&mut self) -> Result<()> {
        self.current_state = crate::types::AppState::Loading;
        
        if let Err(e) = self.generate_release_notes_internal().await {
            self.current_state = crate::types::AppState::Error(e.to_string());
        } else {
            self.current_state = crate::types::AppState::Normal;
            self.message = Some("✅ Notas de versión generadas internamente exitosamente".to_string());
            self.current_screen = crate::types::AppScreen::Main;
        }
        
        Ok(())
    }

    async fn generate_release_notes_with_npm_wrapper(&mut self) -> Result<()> {
        self.current_state = crate::types::AppState::Loading;
        
        if let Err(e) = self.generate_release_notes_with_npm().await {
            self.current_state = crate::types::AppState::Error(e.to_string());
        } else {
            self.current_state = crate::types::AppState::Normal;
            self.message = Some("✅ Notas de versión generadas exitosamente".to_string());
            self.current_screen = crate::types::AppScreen::Main;
        }
        
        Ok(())
    }
}

impl App {
    pub fn generate_raw_release_notes(&self, version: &str, commits: &[crate::types::GitCommit], monday_tasks: &[crate::types::MondayTask], responsible_person: &str) -> String {
        use std::collections::HashMap;
        use std::fs;
        
        // Create a mapping of task ID to task details for quick lookup
        let _task_details_map: HashMap<String, &crate::types::MondayTask> = monday_tasks.iter()
            .map(|task| (task.id.clone(), task))
            .collect();
        
        // Group commits by type
        let _commits_by_type = self.group_commits_by_type(commits);
        
        let mut document = String::new();
        
        // Header
        document.push_str(&format!("# Datos para Generación de Notas de Versión {}\n\n", version));
        
        // General Information
        document.push_str("## Información General\n\n");
        document.push_str(&format!("- **Versión**: {}\n", version));
        document.push_str(&format!("- **Fecha**: {}\n", 
            chrono::Utc::now().format("%d/%m/%Y")));
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
        
        // Read and include template
        document.push_str("La plantilla a utilizar para generar el documento tiene que ser la siguiente. Fijate en todo lo que hay y emúlalo por completo.");
        
        match fs::read_to_string("scripts/plantilla.md") {
            Ok(plantilla_content) => {
                document.push_str(&format!("\n\n{}", plantilla_content));
                println!("✅ Plantilla cargada exitosamente: scripts/plantilla.md");
            }
            Err(e) => {
                println!("⚠️ No se pudo cargar la plantilla scripts/plantilla.md: {}", e);
                document.push_str("\n\nPor favor, utiliza el formato estándar de notas de versión de Teixo que incluye las secciones de Información para N1, Información técnica, Correcciones, Novedades (por categorías), Validación en Sandbox, Pruebas y Referencia commits.");
            }
        }
        
        document
    }
    
    // Helper methods for release notes generation
    fn group_commits_by_type<'a>(&self, commits: &'a [crate::types::GitCommit]) -> std::collections::HashMap<String, Vec<&'a crate::types::GitCommit>> {
        use std::collections::HashMap;
        let mut groups = HashMap::new();
        
        for commit in commits {
            let commit_type = commit.commit_type.as_deref().unwrap_or("other").to_string();
            groups.entry(commit_type).or_insert_with(Vec::new).push(commit);
        }
        
        groups
    }

    pub async fn generate_release_notes_internal(&mut self) -> Result<()> {
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
        let found_task_ids: HashSet<String> = monday_tasks.iter().map(|task| task.id.clone()).collect();
        for task_id in &monday_task_ids {
            if !found_task_ids.contains(task_id) {
                let mut title = "Task not found in Monday API".to_string();
                
                // Try to extract title from commits that mention this task
                for commit in &commits {
                    let task_mentioned = if let Some(scope) = &commit.scope {
                        scope.split('|').any(|id| id == task_id)
                    } else {
                        false
                    } || commit.monday_task_mentions.iter().any(|mention| &mention.id == task_id)
                      || commit.monday_tasks.contains(task_id);
                    
                    if task_mentioned {
                        for mention in &commit.monday_task_mentions {
                            if &mention.id == task_id {
                                title = mention.title.clone();
                                break;
                            }
                        }
                    }
                }
                
                // Create placeholder Monday task
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
        
        // Get version and create tag format
        let version = match get_next_version() {
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
                        Ok(None) => format!("tag-teixo-{}-1.112.0", date_str),
                        Err(_) => format!("tag-teixo-{}-1.112.0", date_str),
                    }
                }
            },
            Err(_) => {
                let date_str = Utc::now().format("%Y%m%d").to_string();
                format!("tag-teixo-{}-1.112.0", date_str)
            },
        };
        
        self.message = Some(format!("📄 Generando documento estructurado para versión {}...", version));
        
        // Generate the structured document
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

    pub async fn generate_release_notes_with_npm(&mut self) -> Result<()> {
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

} 