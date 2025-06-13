use anyhow::Result;
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use std::collections::HashMap;

use crate::types::{AppConfig, GitCommit, MondayTask};

// =============================================================================
// CORE GEMINI CLIENT
// =============================================================================

pub struct GeminiClient {
    client: Client,
}

impl GeminiClient {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let api_key = config
            .gemini_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Google Gemini API key not configured"))?
            .clone();

        // Set the API key as environment variable for genai
        std::env::set_var("GEMINI_API_KEY", &api_key);

        let client = Client::default();

        Ok(Self { client })
    }
}

// =============================================================================
// GEMINI API COMMUNICATION
// =============================================================================

impl GeminiClient {
    async fn call_gemini_with_fallback(&self, prompt: &str) -> Result<String> {
        // Try Gemini 2.5 Pro Preview first (most advanced), then fallback to 2.0 Flash
        match self.call_gemini_api(prompt, "gemini-2.5-pro-preview-06-05").await {
            Ok(response) => Ok(response),
            Err(_) => {
                eprintln!("Gemini 2.5 Pro Preview failed, trying 2.0 Flash...");
                self.call_gemini_api(prompt, "gemini-2.0-flash").await
            }
        }
    }

    async fn call_gemini_api(&self, prompt: &str, model: &str) -> Result<String> {
        let chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);

        let chat_res = self.client.exec_chat(model, chat_req, None).await?;

        let content = chat_res
            .content_text_as_str()
            .ok_or_else(|| anyhow::anyhow!("No response content from Gemini API"))?;

        Ok(content.to_string())
    }
}

// =============================================================================
// RELEASE NOTES GENERATION FEATURE
// =============================================================================

impl GeminiClient {
    pub async fn generate_release_notes(
        &self,
        version: &str,
        commits: &[GitCommit],
        monday_tasks: &[MondayTask],
    ) -> Result<String> {
        let document = self.generate_document(version, commits, monday_tasks);
        self.call_gemini_with_fallback(&document).await
    }

    pub async fn process_release_notes_document(&self, document: &str) -> Result<String> {
        // This method sends the complete structured document to Gemini for processing
        // (like the Node.js script's processWithGemini function)
        self.call_gemini_with_fallback(document).await
    }

    fn generate_document(&self, version: &str, commits: &[GitCommit], monday_tasks: &[MondayTask]) -> String {
        let mut document = String::new();
        
        // Create a mapping of task ID to details for quick lookup
        let task_details_map = self.create_task_details_map(monday_tasks);
        
        // Generate document sections
        self.add_document_header(&mut document, version, commits, monday_tasks);
        self.add_instructions_section(&mut document);
        self.add_changes_summary(&mut document, commits);
        self.add_breaking_changes_section(&mut document, commits);
        self.add_monday_tasks_section(&mut document, monday_tasks, commits, &task_details_map);
        self.add_detailed_commits_section(&mut document, commits, &task_details_map);
        self.add_template_section(&mut document);
        self.add_final_instructions(&mut document);

        document
    }
}

// =============================================================================
// COMMIT ANALYSIS FEATURE
// =============================================================================

impl GeminiClient {
    pub async fn generate_commit_description(&self, changes: &str, commit_type: Option<&str>, scope: Option<&str>, title: &str) -> Result<String> {
        let commit_type_str = commit_type.unwrap_or("general");
        let scope_str = scope.filter(|s| !s.is_empty()).unwrap_or("sistema");
        
        let prompt = format!(
            r#"Eres un desarrollador experto que debe generar una descripción técnica DETALLADA en español para un commit de git.

CONTEXTO DEL COMMIT:
- Tipo: {}
- Ámbito: {}
- Título: {}

CAMBIOS EN EL CÓDIGO:
{}

INSTRUCCIONES:
1. Analiza PROFUNDAMENTE todos los cambios mostrados en el diff
2. Explica QUÉ cambios específicos se hicieron (añadidos, modificados, eliminados)
3. Explica POR QUÉ estos cambios son importantes
4. Describe el IMPACTO técnico de estos cambios
5. Menciona archivos específicos que fueron modificados
6. Usa terminología técnica precisa en español
7. La descripción debe ser EXTENSA y DETALLADA (mínimo 200 palabras)
8. Estructura la respuesta con párrafos bien organizados

FORMATO DE RESPUESTA:
Escribe una descripción técnica completa en español, sin encabezados ni formato markdown, solo texto plano que explique detalladamente los cambios realizados."#,
            commit_type_str, scope_str, title, changes
        );

        let response = self.call_gemini_with_fallback(&prompt).await?;
        Ok(response.trim().to_string())
    }

    pub async fn analyze_security_risks(&self, changes: &str, commit_type: Option<&str>, scope: Option<&str>, title: &str) -> Result<String> {
        let commit_type_str = commit_type.unwrap_or("general");
        let scope_str = scope.filter(|s| !s.is_empty()).unwrap_or("sistema");
        
        let prompt = format!(
            r#"Eres un experto en seguridad informática que debe analizar cambios de código para identificar posibles riesgos de seguridad.

CONTEXTO DEL COMMIT:
- Tipo: {}
- Ámbito: {}
- Título: {}

CAMBIOS EN EL CÓDIGO:
{}

INSTRUCCIONES:
1. Analiza MINUCIOSAMENTE todos los cambios de código para identificar posibles vulnerabilidades o riesgos de seguridad
2. Busca patrones peligrosos como:
   - Inyección SQL, XSS, CSRF
   - Manejo inseguro de datos sensibles (passwords, tokens, keys)
   - Validación insuficiente de entrada
   - Exposición de información sensible
   - Configuraciones de seguridad débiles
   - Dependencias con vulnerabilidades conocidas
   - Privilegios elevados innecesarios
   - Manejo inseguro de archivos o rutas
3. Si NO encuentras riesgos de seguridad relevantes, responde EXACTAMENTE: "NA"
4. Si SÍ encuentras riesgos, describe SOLO los riesgos específicos encontrados en 1-2 líneas máximo

FORMATO DE RESPUESTA:
- Si no hay riesgos: "NA"
- Si hay riesgos: Una descripción concisa de los riesgos específicos encontrados (máximo 2 líneas)"#,
            commit_type_str, scope_str, title, changes
        );

        match self.call_gemini_with_fallback(&prompt).await {
            Ok(response) => {
                let trimmed = response.trim();
                Ok(if trimmed == "NA" || trimmed.is_empty() { 
                    String::new() 
                } else { 
                    trimmed.to_string() 
                })
            },
            Err(_) => Ok(String::new()) // Return empty if both fail
        }
    }

    pub async fn analyze_breaking_changes(&self, changes: &str, commit_type: Option<&str>, scope: Option<&str>, title: &str) -> Result<String> {
        let commit_type_str = commit_type.unwrap_or("general");
        let scope_str = scope.filter(|s| !s.is_empty()).unwrap_or("sistema");
        
        let prompt = format!(
            r#"Eres un experto en desarrollo de software que debe analizar cambios de código para identificar breaking changes (cambios que rompen compatibilidad).

CONTEXTO DEL COMMIT:
- Tipo: {}
- Ámbito: {}
- Título: {}

CAMBIOS EN EL CÓDIGO:
{}

INSTRUCCIONES:
1. Analiza CUIDADOSAMENTE todos los cambios para identificar breaking changes como:
   - Eliminación de APIs públicas, funciones, clases o métodos
   - Cambios en signatures de funciones (parámetros, tipos de retorno)
   - Modificación de contratos de interfaz
   - Cambios en formatos de datos o protocolos
   - Eliminación de configuraciones o variables de entorno
   - Cambios en comportamiento esperado de APIs existentes
   - Modificaciones de esquemas de base de datos
   - Cambios en URLs de endpoints
2. Si NO encuentras breaking changes, responde EXACTAMENTE: "NA"
3. Si SÍ encuentras breaking changes, describe SOLO los cambios específicos que rompen compatibilidad en 1-2 líneas máximo

FORMATO DE RESPUESTA:
- Si no hay breaking changes: "NA"
- Si hay breaking changes: Una descripción concisa de los cambios que rompen compatibilidad (máximo 2 líneas)"#,
            commit_type_str, scope_str, title, changes
        );

        match self.call_gemini_with_fallback(&prompt).await {
            Ok(response) => {
                let trimmed = response.trim();
                Ok(if trimmed == "NA" || trimmed.is_empty() { 
                    String::new() 
                } else { 
                    trimmed.to_string() 
                })
            },
            Err(_) => Ok(String::new()) // Return empty if both fail
        }
    }
}

// =============================================================================
// DOCUMENT GENERATION HELPERS
// =============================================================================

impl GeminiClient {
    fn create_task_details_map<'a>(&self, monday_tasks: &'a [MondayTask]) -> HashMap<String, &'a MondayTask> {
        monday_tasks
            .iter()
            .map(|task| (task.id.clone(), task))
            .collect()
    }

    fn add_document_header(&self, document: &mut String, version: &str, commits: &[GitCommit], monday_tasks: &[MondayTask]) {
        document.push_str(&format!("# Datos para Generación de Notas de Versión {}\n\n", version));
        
        document.push_str("## Información General\n\n");
        document.push_str(&format!("- **Versión**: {}\n", version));
        document.push_str(&format!("- **Fecha**: {}\n", chrono::Utc::now().format("%d/%m/%Y")));
        document.push_str(&format!("- **Total de Commits**: {}\n", commits.len()));
        document.push_str(&format!("- **Tareas de Monday relacionadas**: {}\n\n", monday_tasks.len()));
    }

    fn add_instructions_section(&self, document: &mut String) {
        document.push_str("## Instrucciones\n\n");
        document.push_str("Necesito que generes unas notas de versión detalladas en español, basadas en los datos proporcionados a continuación. ");
        document.push_str("Estas notas deben estar dirigidas a usuarios finales y equipos técnicos, destacando las nuevas funcionalidades, correcciones y mejoras. ");
    }

    fn add_template_section(&self, document: &mut String) {
        document.push_str("La plantilla a utilizar para generar el documento tiene que ser la siguiente. Fijate en todo lo que hay y emúlalo por completo.\n\n");
        
        match std::fs::read_to_string("scripts/plantilla.md") {
            Ok(template_content) => {
                document.push_str(&template_content);
            }
            Err(e) => {
                eprintln!("⚠️ No se pudo cargar la plantilla: {}", e);
                document.push_str("No se pudo cargar la plantilla de formato.\n");
            }
        }
    }

    fn add_final_instructions(&self, document: &mut String) {
        document.push_str("\n---\n\n");
        document.push_str("**INSTRUCCIONES PARA GEMINI:**\n");
        document.push_str("Genera unas notas de versión profesionales en español que incluyan:\n");
        document.push_str("1. Un resumen ejecutivo de la versión\n");
        document.push_str("2. Lista organizada de nuevas funcionalidades\n");
        document.push_str("3. Lista de correcciones y mejoras\n");
        document.push_str("4. Cambios que rompen compatibilidad (si los hay)\n");
        document.push_str("5. Información relevante de las tareas de Monday.com\n");
        document.push_str("6. Cualquier información adicional que consideres relevante\n\n");
        document.push_str("El tono debe ser profesional pero accesible, dirigido a desarrolladores y stakeholders técnicos.\n");
    }
}

// =============================================================================
// COMMITS SECTION GENERATION
// =============================================================================

impl GeminiClient {
    fn add_changes_summary(&self, document: &mut String, commits: &[GitCommit]) {
        let commits_by_type = self.group_commits_by_type(commits);
        
        document.push_str("## Resumen de Cambios\n\n");
        
        self.add_commits_section(document, &commits_by_type, "feat", "Nuevas Funcionalidades");
        self.add_commits_section(document, &commits_by_type, "fix", "Correcciones");
        
        // Other commit types
        for (commit_type, commits_list) in &commits_by_type {
            if commit_type != "feat" && commit_type != "fix" && !commits_list.is_empty() {
                self.add_commits_section(document, &commits_by_type, commit_type, Self::get_type_title(commit_type));
            }
        }
    }

    fn add_commits_section(&self, document: &mut String, commits_by_type: &HashMap<String, Vec<&GitCommit>>, commit_type: &str, section_title: &str) {
        if let Some(commits_list) = commits_by_type.get(commit_type) {
            if !commits_list.is_empty() {
                document.push_str(&format!("### {} ({})\n\n", section_title, commits_list.len()));
                for commit in commits_list {
                    document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                        commit.description, 
                        &commit.hash[..7], 
                        commit.author_name, 
                        commit.author_email, 
                        commit.commit_date.format("%Y-%m-%d")));
                    if !commit.body.is_empty() {
                        document.push_str(&format!("  - Detalles: {}\n", Self::format_multiline_text(&commit.body)));
                    }
                }
                document.push('\n');
            }
        }
    }

    fn add_breaking_changes_section(&self, document: &mut String, commits: &[GitCommit]) {
        let breaking_changes: Vec<&GitCommit> = commits.iter().filter(|c| !c.breaking_changes.is_empty()).collect();
        if !breaking_changes.is_empty() {
            document.push_str("## Cambios que Rompen Compatibilidad\n\n");
            for commit in breaking_changes {
                document.push_str(&format!("- **{}** [{}] - {} <{}> ({})\n", 
                    commit.description, 
                    &commit.hash[..7], 
                    commit.author_name, 
                    commit.author_email, 
                    commit.commit_date.format("%Y-%m-%d")));
                for breaking_change in &commit.breaking_changes {
                    document.push_str(&format!("  - Detalles: {}\n", breaking_change));
                }
            }
            document.push('\n');
        }
    }

    fn add_detailed_commits_section(&self, document: &mut String, commits: &[GitCommit], task_details_map: &HashMap<String, &MondayTask>) {
        document.push_str("## Detalles Completos de Commits\n\n");
        
        for commit in commits {
            document.push_str(&format!("### {}{}: {} [{}]\n\n", 
                commit.commit_type.as_deref().unwrap_or("other"),
                commit.scope.as_ref().map(|s| format!("({})", s)).unwrap_or_default(),
                commit.description,
                &commit.hash[..7]));
            
            document.push_str(&format!("**Autor**: {} <{}>\n", commit.author_name, commit.author_email));
            document.push_str(&format!("**Fecha**: {}\n\n", commit.commit_date.format("%Y-%m-%d %H:%M:%S UTC")));
            
            if !commit.body.is_empty() {
                document.push_str(&format!("{}\n\n", Self::format_multiline_text(&commit.body)));
            }
            
            if !commit.test_details.is_empty() {
                document.push_str("**Pruebas**:\n");
                for test in &commit.test_details {
                    document.push_str(&format!("- {}\n", test));
                }
                document.push('\n');
            }
            
            if let Some(security) = &commit.security {
                document.push_str(&format!("**Seguridad**: {}\n\n", security));
            }
            
            self.add_commit_monday_tasks(document, commit, task_details_map);
            
            document.push_str("---\n\n");
        }
    }

    fn add_commit_monday_tasks(&self, document: &mut String, commit: &GitCommit, task_details_map: &HashMap<String, &MondayTask>) {
        if !commit.monday_task_mentions.is_empty() {
            document.push_str("**Tareas relacionadas**:\n");
            
            for mention in &commit.monday_task_mentions {
                let task_details = task_details_map.get(&mention.id);
                let task_name = task_details.map(|t| t.title.as_str()).unwrap_or(&mention.title);
                let task_state = task_details.map(|t| t.state.as_str()).unwrap_or("Desconocido");
                
                document.push_str(&format!("- {} (ID: {}, Estado: {})\n", task_name, mention.id, task_state));
            }
            
            document.push('\n');
        }
    }

    fn group_commits_by_type<'a>(&self, commits: &'a [GitCommit]) -> HashMap<String, Vec<&'a GitCommit>> {
        let mut commits_by_type = HashMap::new();
        
        for commit in commits {
            let commit_type = commit.commit_type.as_deref().unwrap_or("other").to_string();
            commits_by_type.entry(commit_type).or_insert_with(Vec::new).push(commit);
        }
        
        commits_by_type
    }
}

// =============================================================================
// MONDAY TASKS SECTION GENERATION
// =============================================================================

impl GeminiClient {
    fn add_monday_tasks_section(&self, document: &mut String, monday_tasks: &[MondayTask], commits: &[GitCommit], _task_details_map: &HashMap<String, &MondayTask>) {
        if monday_tasks.is_empty() {
            return;
        }

        document.push_str("## Detalles de Tareas de Monday\n\n");
        
        for task in monday_tasks {
            document.push_str(&format!("### {} (ID: {})\n\n", task.title, task.id));
            document.push_str(&format!("- **Estado**: {}\n", task.state));
            document.push_str(&format!("- **Tablero**: {} (ID: {})\n", 
                task.board_name.as_deref().unwrap_or("N/A"), 
                task.board_id.as_deref().unwrap_or("N/A")));
            document.push_str(&format!("- **Grupo**: {}\n", task.group_title.as_deref().unwrap_or("N/A")));
            
            self.add_task_column_values(document, task);
            self.add_task_updates(document, task);
            self.add_related_commits(document, task, commits);
            
            document.push('\n');
        }
    }

    fn add_task_column_values(&self, document: &mut String, task: &MondayTask) {
        if !task.column_values.is_empty() {
            document.push_str("- **Detalles**:\n");
            let relevant_columns: Vec<_> = task.column_values.iter()
                .filter(|col| col.text.as_ref().is_some_and(|t| !t.trim().is_empty()))
                .collect();
            
            if !relevant_columns.is_empty() {
                for col in relevant_columns {
                    document.push_str(&format!("  - {}: {}\n", col.id, col.text.as_deref().unwrap_or("")));
                }
            } else {
                document.push_str("  - No hay detalles adicionales disponibles\n");
            }
        }
    }

    fn add_task_updates(&self, document: &mut String, task: &MondayTask) {
        if !task.updates.is_empty() {
            document.push_str("- **Actualizaciones Recientes**:\n");
            
            for update in task.updates.iter().take(3) {
                let date = update.created_at.split('T').next().unwrap_or(&update.created_at);
                let creator_name = update.creator.as_ref().map(|c| c.name.as_str()).unwrap_or("Usuario");
                let body_preview = if update.body.len() > 100 {
                    format!("{}...", &update.body[..100])
                } else {
                    update.body.clone()
                };
                document.push_str(&format!("  - {} por {}: {}\n", date, creator_name, body_preview));
            }
        }
    }

    fn add_related_commits(&self, document: &mut String, task: &MondayTask, commits: &[GitCommit]) {
        let related_commits: Vec<&GitCommit> = commits.iter().filter(|commit| {
            // Check scope for task ID
            if let Some(scope) = &commit.scope {
                if scope.split('|').any(|id| id == task.id) {
                    return true;
                }
            }
            
            // Check Monday task mentions
            commit.monday_task_mentions.iter().any(|mention| mention.id == task.id)
        }).collect();
        
        if !related_commits.is_empty() {
            document.push_str("- **Commits Relacionados**:\n");
            for commit in related_commits {
                document.push_str(&format!("  - {}: {} [{}]\n", 
                    commit.commit_type.as_deref().unwrap_or("other"), 
                    commit.description, 
                    &commit.hash[..7]));
            }
        }
    }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

impl GeminiClient {
    fn format_multiline_text(text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn get_type_title(commit_type: &str) -> &str {
        match commit_type {
            "feat" => "Nuevas Funcionalidades",
            "fix" => "Correcciones",
            "docs" => "Documentación",
            "style" => "Estilo",
            "refactor" => "Refactorizaciones",
            "perf" => "Mejoras de Rendimiento",
            "test" => "Pruebas",
            "build" => "Construcción",
            "ci" => "Integración Continua",
            "chore" => "Tareas",
            "revert" => "Reversiones",
            _ => "Otros Cambios",
        }
    }
}

// =============================================================================
// PUBLIC UTILITY FUNCTIONS
// =============================================================================

pub async fn test_gemini_connection(config: &AppConfig) -> Result<String> {
    let client = GeminiClient::new(config)?;
    
    let test_prompt = "Responde con 'Conexión exitosa con Google Gemini' si puedes leer este mensaje.";
    
    match client.call_gemini_api(test_prompt, "gemini-2.5-pro-preview-06-05").await {
        Ok(response) => Ok(response),
        Err(_) => client.call_gemini_api(test_prompt, "gemini-2.0-flash").await,
    }
} 