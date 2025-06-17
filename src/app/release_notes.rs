use anyhow::Result;
use std::fs;
use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::Utc;

use crate::{
    app::App,
    git::{GitRepo, get_next_version},
    services::MondayClient,
    services::GeminiClient,
    types::{AppState, ReleaseNotesAnalysisState, MondayTask, AppConfig},
};

pub trait ReleaseNotesOperations {
    async fn handle_release_notes_generation(&mut self) -> Result<()>;
    async fn generate_release_notes_with_npm_wrapper(&mut self) -> Result<()>;
}

impl ReleaseNotesOperations for App {
    async fn handle_release_notes_generation(&mut self) -> Result<()> {
        // Check if already processing to avoid multiple concurrent analyses
        if matches!(self.current_state, AppState::Loading) || self.release_notes_analysis_state.is_some() {
            return Ok(());
        }
        
        // IMMEDIATELY set loading state and create analysis state
        self.current_state = AppState::Loading;
        self.message = Some("🚀 Iniciando generación de notas de versión...".to_string());
        
        // Create shared state for the analysis
        let analysis_state = ReleaseNotesAnalysisState {
            status: Arc::new(Mutex::new("📋 Obteniendo commits desde la última versión...".to_string())),
            finished: Arc::new(Mutex::new(false)),
            success: Arc::new(Mutex::new(true)),
        };
        
        // Start the analysis in a background thread
        self.start_release_notes_analysis(analysis_state.clone());
        
        // Store the analysis state so the main loop can poll it
        self.release_notes_analysis_state = Some(analysis_state);
        
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
    pub fn start_release_notes_analysis(&self, analysis_state: ReleaseNotesAnalysisState) {
        // Clone data needed for the thread
        let config_clone = self.config.clone();
        
        // Clone analysis state components
        let status_clone = analysis_state.status.clone();
        let finished_clone = analysis_state.finished.clone();
        let success_clone = analysis_state.success.clone();

        // Spawn the analysis in a background thread
        thread::spawn(move || {
            // Create a temporary App-like structure for the background thread
            let temp_app = TempAppForBackground {
                config: config_clone,
            };
            
            // Run the release notes generation
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
            
            rt.block_on(async {
                temp_app.run_release_notes_generation(status_clone, finished_clone, success_clone).await;
            });
        });
    }

    pub fn generate_raw_release_notes(&self, version: &str, commits: &[crate::types::GitCommit], monday_tasks: &[crate::types::MondayTask], responsible_person: &str) -> String {
        use std::collections::HashMap;
        use std::fs;
        
        // Create a mapping of task ID to task details for quick lookup
        let task_details_map: HashMap<String, &crate::types::MondayTask> = monday_tasks.iter()
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
        
        // Add changes summary section
        self.add_changes_summary_to_document(&mut document, &commits_by_type, commits);
        
        // Add breaking changes section
        self.add_breaking_changes_to_document(&mut document, commits);
        
        // Add Monday tasks section
        self.add_monday_tasks_to_document(&mut document, monday_tasks, commits, &task_details_map);
        
        // Add detailed commits section
        self.add_detailed_commits_to_document(&mut document, commits, &task_details_map);
        
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

    fn add_changes_summary_to_document(&self, document: &mut String, commits_by_type: &std::collections::HashMap<String, Vec<&crate::types::GitCommit>>, _commits: &[crate::types::GitCommit]) {
        document.push_str("## Resumen de Cambios\n\n");
        
        // Add feat commits
        self.add_commits_section_to_document(document, commits_by_type, "feat", "Nuevas Funcionalidades");
        
        // Add fix commits  
        self.add_commits_section_to_document(document, commits_by_type, "fix", "Correcciones");
        
        // Add other commit types
        for (commit_type, commits_list) in commits_by_type {
            if commit_type != "feat" && commit_type != "fix" && !commits_list.is_empty() {
                let section_title = self.get_type_title(commit_type);
                self.add_commits_section_to_document(document, commits_by_type, commit_type, section_title);
            }
        }
    }

    fn add_commits_section_to_document(&self, document: &mut String, commits_by_type: &std::collections::HashMap<String, Vec<&crate::types::GitCommit>>, commit_type: &str, section_title: &str) {
        if let Some(commits_list) = commits_by_type.get(commit_type) {
            if !commits_list.is_empty() {
                document.push_str(&format!("### {} ({})\n\n", section_title, commits_list.len()));
                for commit in commits_list {
                    let description = &commit.description;
                    
                    document.push_str(&format!("- **{}** [{:.7}] - {} <{}> ({})\n", 
                        description,
                        commit.hash, 
                        commit.author_name, 
                        commit.author_email, 
                        commit.commit_date.format("%a %b %d %H:%M:%S %Y %z")));
                    
                    if !commit.body.is_empty() {
                        document.push_str(&format!("  - Detalles: {}\n", self.format_multiline_text(&commit.body)));
                    }
                }
                document.push('\n');
            }
        }
    }

    fn add_breaking_changes_to_document(&self, document: &mut String, commits: &[crate::types::GitCommit]) {
        let breaking_changes: Vec<&crate::types::GitCommit> = commits.iter()
            .filter(|c| !c.breaking_changes.is_empty())
            .collect();
            
        if !breaking_changes.is_empty() {
            document.push_str("## Cambios que Rompen Compatibilidad\n\n");
            for commit in breaking_changes {
                let description = &commit.description;
                
                document.push_str(&format!("- **{}** [{:.7}] - {} <{}> ({})\n", 
                    description,
                    commit.hash, 
                    commit.author_name, 
                    commit.author_email, 
                    commit.commit_date.format("%a %b %d %H:%M:%S %Y %z")));
                    
                for breaking_change in &commit.breaking_changes {
                    document.push_str(&format!("  - Detalles: {}\n", breaking_change));
            }
            }
            document.push('\n');
        }
    }

    fn add_monday_tasks_to_document(&self, document: &mut String, monday_tasks: &[crate::types::MondayTask], commits: &[crate::types::GitCommit], _task_details_map: &std::collections::HashMap<String, &crate::types::MondayTask>) {
        if !monday_tasks.is_empty() {
            document.push_str("## Detalles de Tareas de Monday\n\n");
            
            for task in monday_tasks {
                document.push_str(&format!("### {} (ID: {})\n\n", task.title, task.id));
                document.push_str(&format!("- **Estado**: {}\n", task.state));
                
                if let Some(board_name) = &task.board_name {
                    if !board_name.is_empty() {
                        document.push_str(&format!("- **Tablero**: {} (ID: {})\n", 
                            board_name, 
                            task.board_id.as_deref().unwrap_or("")));
                    }
                }
                
                if let Some(group_title) = &task.group_title {
                    if !group_title.is_empty() {
                        document.push_str(&format!("- **Grupo**: {}\n", group_title));
                    }
                }
                
                // Add column values
                self.add_task_column_values_to_document(document, task);
                
                // Add updates
                self.add_task_updates_to_document(document, task);
        
                // Add related commits
                self.add_related_commits_to_document(document, task, commits);
                
                document.push('\n');
            }
        }
    }

    fn add_task_column_values_to_document(&self, document: &mut String, task: &crate::types::MondayTask) {
        if !task.column_values.is_empty() {
            document.push_str("- **Detalles**:\n");
            for column in &task.column_values {
                if let Some(text) = &column.text {
                    if !text.is_empty() && text != "-" {
                        document.push_str(&format!("  - {}: {}\n", column.column_type, text));
                    }
                }
            }
        }
    }

    fn add_task_updates_to_document(&self, document: &mut String, task: &crate::types::MondayTask) {
        if !task.updates.is_empty() {
            document.push_str("- **Actualizaciones Recientes**:\n");
            for update in task.updates.iter().take(3) {
                let date = &update.created_at;
                let author = if let Some(creator) = &update.creator {
                    &creator.name
        } else {
                    "Unknown"
                };
                let body_preview = if update.body.len() > 100 {
                    format!("{}...", &update.body[..100])
                    } else {
                    update.body.clone()
                };
                document.push_str(&format!("  - {} por {}: {}\n", date, author, body_preview));
            }
        }
    }

    fn add_related_commits_to_document(&self, document: &mut String, task: &crate::types::MondayTask, commits: &[crate::types::GitCommit]) {
        let related_commits: Vec<&crate::types::GitCommit> = commits
            .iter()
            .filter(|commit| {
                // Check if task ID is in commit scope
                if let Some(scope) = &commit.scope {
                    if scope.split('|').any(|id| id == task.id) {
                        return true;
                    }
                }
                
                // Check if task ID is in monday_tasks
                if commit.monday_tasks.contains(&task.id) {
                    return true;
                }
                
                // Check monday_task_mentions
                commit.monday_task_mentions.iter().any(|mention| mention.id == task.id)
            })
            .collect();

        if !related_commits.is_empty() {
            document.push_str("- **Commits Relacionados**:\n");
            for commit in related_commits {
                let commit_type = commit.commit_type.as_deref().unwrap_or("other");
                let description = &commit.description;
                
                document.push_str(&format!("  - {}: {} [{:.7}]\n", 
                    commit_type, description, commit.hash));
            }
        }
    }

    fn add_detailed_commits_to_document(&self, document: &mut String, commits: &[crate::types::GitCommit], task_details_map: &std::collections::HashMap<String, &crate::types::MondayTask>) {
        document.push_str("## Detalles Completos de Commits\n\n");
        
        for commit in commits {
            let commit_type = commit.commit_type.as_deref().unwrap_or("other");
            let scope = commit.scope.as_deref().unwrap_or("");
            let description = &commit.description;
            
            // Header
            document.push_str(&format!("### {}", 
                if scope.is_empty() {
                    format!("{}: {} [{:.7}]", commit_type, description, commit.hash)
                } else {
                    format!("{}({}): {} [{:.7}]", commit_type, scope, description, commit.hash)
                }
            ));
            document.push('\n');
            document.push('\n');
            
            // Author and date
            document.push_str(&format!("**Autor**: {} <{}>\n", commit.author_name, commit.author_email));
            document.push_str(&format!("**Fecha**: {}\n\n", commit.commit_date.format("%a %b %d %H:%M:%S %Y %z")));
            
            // Body/Description
            if !commit.body.is_empty() {
                document.push_str(&format!("{}\n\n", commit.body));
            }
            
            // Test details
            if !commit.test_details.is_empty() {
                document.push_str("**Pruebas**:\n");
                for test in &commit.test_details {
                    document.push_str(&format!("- {}\n", test));
                }
                document.push('\n');
            }
            
            // Security information
            if let Some(security) = &commit.security {
                document.push_str(&format!("**Seguridad**: {}\n\n", security));
            }
            
            // Monday tasks
            self.add_commit_monday_tasks_to_document(document, commit, task_details_map);
            
            document.push_str("---\n\n");
        }
    }

    fn add_commit_monday_tasks_to_document(&self, document: &mut String, commit: &crate::types::GitCommit, task_details_map: &std::collections::HashMap<String, &crate::types::MondayTask>) {
        let mut all_task_ids = std::collections::HashSet::new();
        
        // Collect task IDs from various sources
        if let Some(scope) = &commit.scope {
            for id in scope.split('|') {
                if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
                    all_task_ids.insert(id.to_string());
                }
            }
        }
        
        for task_id in &commit.monday_tasks {
            all_task_ids.insert(task_id.clone());
        }
        
        for mention in &commit.monday_task_mentions {
            all_task_ids.insert(mention.id.clone());
        }
        
        if !all_task_ids.is_empty() {
            document.push_str("**Tareas relacionadas**:\n");
            for task_id in &all_task_ids {
                if let Some(task) = task_details_map.get(task_id) {
                    document.push_str(&format!("- {} (ID: {}, Estado: {})\n", 
                        task.title, task.id, task.state));
                        } else {
                    document.push_str(&format!("- Task ID: {} (Detalles no disponibles)\n", task_id));
                }
            }
            document.push('\n');
        }
    }

    fn format_multiline_text(&self, text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn get_type_title(&self, commit_type: &str) -> &str {
        match commit_type {
            "feat" => "Nuevas Funcionalidades",
            "fix" => "Correcciones", 
            "docs" => "Documentación",
            "style" => "Estilos",
            "refactor" => "Refactorizaciones",
            "perf" => "Mejoras de Rendimiento",
            "test" => "Pruebas",
            "chore" => "Tareas de Mantenimiento",
            "ci" => "Integración Continua",
            "build" => "Compilación",
            "revert" => "Reversiones",
            _ => "Otros Cambios",
        }
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

// Helper struct for background processing
struct TempAppForBackground {
    config: AppConfig,
}

impl TempAppForBackground {
    async fn run_release_notes_generation(
        &self,
        status_clone: Arc<Mutex<String>>,
        finished_clone: Arc<Mutex<bool>>,
        success_clone: Arc<Mutex<bool>>,
    ) {
        // Update status: getting git repository
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
        if let Ok(mut status) = status_clone.lock() {
            *status = "🏷️ Obteniendo última etiqueta del repositorio...".to_string();
        }
        
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
        
        if let Ok(mut status) = status_clone.lock() {
            *status = "📊 Analizando commits desde la última versión...".to_string();
        }
        
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
        
        if let Ok(mut status) = status_clone.lock() {
            *status = format!("📊 Se encontraron {} commits para analizar", commits.len());
        }
        
        // Extract Monday.com task IDs from commits
        if let Ok(mut status) = status_clone.lock() {
            *status = "🔍 Extrayendo IDs de tareas de Monday.com...".to_string();
        }
        
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
        
        if let Ok(mut status) = status_clone.lock() {
            *status = format!("🔍 Obteniendo detalles de {} tareas de Monday.com...", monday_task_ids.len());
        }
        
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
                        Ok(tasks) => {
                            if let Ok(mut status) = status_clone.lock() {
                                *status = format!("✅ Obtenidos detalles de {} tareas de Monday.com", tasks.len());
                            }
                            tasks
                        },
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
        if let Ok(mut status) = status_clone.lock() {
            *status = "🏷️ Generando información de versión...".to_string();
        }
        
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
        
        if let Ok(mut status) = status_clone.lock() {
            *status = format!("📄 Generando documento estructurado para versión {}...", version);
        }
        
        // Generate the structured document
        let temp_app_helper = App::new_for_background(&self.config);
        let structured_document = temp_app_helper.generate_raw_release_notes(&version, &commits, &monday_tasks, &responsible_person);
        
        // Create output directory
        if let Err(e) = fs::create_dir_all("release-notes") {
            eprintln!("Warning: Could not create release-notes directory: {}", e);
        }
        
        // Generate filenames
        let date_str = Utc::now().format("%Y-%m-%d").to_string();
        let structured_filename = format!("release-notes/release-notes-{}_SCRIPT_WITH_ENTER_KEY.md", date_str);
        let gemini_filename = format!("release-notes/release-notes-{}_GEMINI.md", date_str);
        
        // Save the structured document first
        if let Ok(mut status) = status_clone.lock() {
            *status = "💾 Guardando documento estructurado...".to_string();
        }
        
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
        
        if let Ok(mut status) = status_clone.lock() {
            *status = "🤖 Enviando documento a Google Gemini API...".to_string();
        }
        
        // Try to process with Gemini
        match GeminiClient::new(&self.config) {
            Ok(gemini_client) => {
                match gemini_client.process_release_notes_document(&structured_document).await {
                    Ok(gemini_response) => {
                        if let Ok(mut status) = status_clone.lock() {
                            *status = "💾 Guardando respuesta de Gemini...".to_string();
                        }
                        
                        // Save the Gemini-processed version
                        if let Err(e) = fs::write(&gemini_filename, &gemini_response) {
                            eprintln!("⚠️ Error guardando respuesta de Gemini: {}", e);
                            if let Ok(mut status) = status_clone.lock() {
                                *status = format!(
                                    "✅ Documento estructurado generado: {}\n⚠️ Error guardando versión de Gemini",
                                    structured_filename
                                );
                            }
                        } else {
                            if let Ok(mut status) = status_clone.lock() {
                                *status = format!(
                                    "✅ Notas de versión generadas exitosamente:\n📄 Documento estructurado: {}\n🤖 Versión procesada por Gemini: {}",
                                    structured_filename, gemini_filename
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ Error procesando con Gemini: {}", e);
                        if let Ok(mut status) = status_clone.lock() {
                            *status = format!(
                                "⚠️ Gemini falló, pero se generó el documento estructurado:\n📄 Documento estructurado: {}\n💡 Ejecuta el script de Node.js para procesamiento con Gemini",
                                structured_filename
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ Error configurando Gemini: {}", e);
                if let Ok(mut status) = status_clone.lock() {
                    *status = format!(
                        "⚠️ Gemini no configurado, solo se generó el documento estructurado:\n📄 Documento estructurado: {}\n💡 Configura el token de Gemini para procesamiento IA",
                        structured_filename
                    );
                }
            }
        }
        
        // Mark as finished
        if let Ok(mut finished) = finished_clone.lock() {
            *finished = true;
        }
    }
} 