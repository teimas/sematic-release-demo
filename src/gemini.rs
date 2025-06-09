use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

use crate::types::{AppConfig, GitCommit, MondayTask};

pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let api_key = config
            .gemini_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Google Gemini API key not configured"))?
            .clone();

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn generate_release_notes(
        &self,
        version: &str,
        commits: &[GitCommit],
        monday_tasks: &[MondayTask],
    ) -> Result<String> {
        let document = self.generate_document(version, commits, monday_tasks);
        
        // Try Gemini 1.5 Pro first, then fallback to 1.0 Pro
        match self.call_gemini_api(&document, "gemini-1.5-pro").await {
            Ok(response) => Ok(response),
            Err(_) => {
                eprintln!("Gemini 1.5 Pro failed, trying 1.0 Pro...");
                self.call_gemini_api(&document, "gemini-1.0-pro").await
            }
        }
    }

    pub async fn process_release_notes_document(&self, document: &str) -> Result<String> {
        // This method sends the complete structured document to Gemini for processing
        // (like the Node.js script's processWithGemini function)
        
        // Try Gemini 1.5 Pro first, then fallback to 1.0 Pro
        match self.call_gemini_api(document, "gemini-1.5-pro").await {
            Ok(response) => Ok(response),
            Err(_) => {
                eprintln!("Gemini 1.5 Pro failed, trying 1.0 Pro...");
                self.call_gemini_api(document, "gemini-1.0-pro").await
            }
        }
    }

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

        // Try Gemini 1.5 Pro first, then fallback to 1.0 Pro
        match self.call_gemini_api(&prompt, "gemini-1.5-pro").await {
            Ok(response) => Ok(response.trim().to_string()),
            Err(_) => {
                eprintln!("Gemini 1.5 Pro failed, trying 1.0 Pro...");
                let response = self.call_gemini_api(&prompt, "gemini-1.0-pro").await?;
                Ok(response.trim().to_string())
            }
        }
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

        // Try Gemini 1.5 Pro first, then fallback to 1.0 Pro
        match self.call_gemini_api(&prompt, "gemini-1.5-pro").await {
            Ok(response) => {
                let trimmed = response.trim();
                Ok(if trimmed == "NA" || trimmed.is_empty() { 
                    String::new() 
                } else { 
                    trimmed.to_string() 
                })
            },
            Err(_) => {
                eprintln!("Gemini 1.5 Pro failed, trying 1.0 Pro...");
                match self.call_gemini_api(&prompt, "gemini-1.0-pro").await {
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

        // Try Gemini 1.5 Pro first, then fallback to 1.0 Pro
        match self.call_gemini_api(&prompt, "gemini-1.5-pro").await {
            Ok(response) => {
                let trimmed = response.trim();
                Ok(if trimmed == "NA" || trimmed.is_empty() { 
                    String::new() 
                } else { 
                    trimmed.to_string() 
                })
            },
            Err(_) => {
                eprintln!("Gemini 1.5 Pro failed, trying 1.0 Pro...");
                match self.call_gemini_api(&prompt, "gemini-1.0-pro").await {
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
    }

    async fn call_gemini_api(&self, prompt: &str, model: &str) -> Result<String> {
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, self.api_key);
        
        let request_body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "topK": 40,
                "topP": 0.95,
                "maxOutputTokens": 8192
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
        }

        let result: Value = response.json().await?;
        
        if let Some(candidates) = result["candidates"].as_array() {
            if let Some(first_candidate) = candidates.first() {
                if let Some(content) = first_candidate["content"]["parts"].as_array() {
                    if let Some(text_part) = content.first() {
                        if let Some(text) = text_part["text"].as_str() {
                            return Ok(text.to_string());
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Unexpected response format from Gemini API"))
    }

    fn group_commits_by_type<'a>(&self, commits: &'a [GitCommit]) -> std::collections::HashMap<String, Vec<&'a GitCommit>> {
        let mut commits_by_type = std::collections::HashMap::new();
        
        for commit in commits {
            let commit_type = commit.commit_type.as_deref().unwrap_or("other").to_string();
            commits_by_type.entry(commit_type).or_insert_with(Vec::new).push(commit);
        }
        
        commits_by_type
    }

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

    fn generate_document(&self, version: &str, commits: &[GitCommit], monday_tasks: &[MondayTask]) -> String {
        let mut document = String::new();
        
        // Create a mapping of task ID to details for quick lookup
        let task_details_map: std::collections::HashMap<String, &MondayTask> = monday_tasks
            .iter()
            .map(|task| (task.id.clone(), task))
            .collect();
        
        // Generate the document header
        document.push_str(&format!("# Datos para Generación de Notas de Versión {}\n\n", version));
        
        document.push_str("## Información General\n\n");
        document.push_str(&format!("- **Versión**: {}\n", version));
        document.push_str(&format!("- **Fecha**: {}\n", chrono::Utc::now().format("%d/%m/%Y")));
        document.push_str(&format!("- **Total de Commits**: {}\n", commits.len()));
        document.push_str(&format!("- **Tareas de Monday relacionadas**: {}\n\n", monday_tasks.len()));
        
        // Instructions for Gemini
        document.push_str("## Instrucciones\n\n");
        document.push_str("Necesito que generes unas notas de versión detalladas en español, basadas en los datos proporcionados a continuación. ");
        document.push_str("Estas notas deben estar dirigidas a usuarios finales y equipos técnicos, destacando las nuevas funcionalidades, correcciones y mejoras. ");
        
        // Group commits by type
        let commits_by_type = self.group_commits_by_type(commits);
        
        // Summary of changes by type
        document.push_str("## Resumen de Cambios\n\n");
        
        // New features (feat)
        if let Some(feat_commits) = commits_by_type.get("feat") {
            if !feat_commits.is_empty() {
                document.push_str(&format!("### Nuevas Funcionalidades ({})\n\n", feat_commits.len()));
                for commit in feat_commits {
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
        
        // Bug fixes (fix)
        if let Some(fix_commits) = commits_by_type.get("fix") {
            if !fix_commits.is_empty() {
                document.push_str(&format!("### Correcciones ({})\n\n", fix_commits.len()));
                for commit in fix_commits {
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
        
        // Other commit types
        for (commit_type, commits_list) in &commits_by_type {
            if commit_type != "feat" && commit_type != "fix" && !commits_list.is_empty() {
                document.push_str(&format!("### {} ({})\n\n", Self::get_type_title(commit_type), commits_list.len()));
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
        
        // Breaking changes
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
        
        // Monday.com task details
        if !monday_tasks.is_empty() {
            document.push_str("## Detalles de Tareas de Monday\n\n");
            
            for task in monday_tasks {
                document.push_str(&format!("### {} (ID: {})\n\n", task.title, task.id));
                document.push_str(&format!("- **Estado**: {}\n", task.state));
                document.push_str(&format!("- **Tablero**: {} (ID: {})\n", 
                    task.board_name.as_deref().unwrap_or("N/A"), 
                    task.board_id.as_deref().unwrap_or("N/A")));
                document.push_str(&format!("- **Grupo**: {}\n", task.group_title.as_deref().unwrap_or("N/A")));
                
                // Column information
                if !task.column_values.is_empty() {
                    document.push_str("- **Detalles**:\n");
                    let relevant_columns: Vec<_> = task.column_values.iter()
                        .filter(|col| col.text.as_ref().map_or(false, |t| !t.trim().is_empty()))
                        .collect();
                    
                    if !relevant_columns.is_empty() {
                        for col in relevant_columns {
                            document.push_str(&format!("  - {}: {}\n", col.id, col.text.as_deref().unwrap_or("")));
                        }
                    } else {
                        document.push_str("  - No hay detalles adicionales disponibles\n");
                    }
                }
                
                // Recent updates
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
                
                // Related commits
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
                
                document.push('\n');
            }
        }
        
        // Complete commit details
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
            
            document.push_str("---\n\n");
        }

        // Load and append the template
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



        // Add Monday.com tasks information
        if !monday_tasks.is_empty() {
            document.push_str("## 📋 Tareas de Monday.com Relacionadas\n\n");
            for task in monday_tasks {
                document.push_str(&format!("### {} (ID: {})\n", task.title, task.id));
                document.push_str(&format!("- URL: {}\n", task.url));
                if let Some(board_name) = &task.board_name {
                    document.push_str(&format!("- Tablero: {}\n", board_name));
                }
                
                if !task.updates.is_empty() {
                    document.push_str("- Actualizaciones recientes:\n");
                    for update in task.updates.iter().take(3) {
                        if !update.body.is_empty() {
                            let body = update.body.chars().take(100).collect::<String>();
                            document.push_str(&format!("  - {}\n", body));
                        }
                    }
                }
                document.push('\n');
            }
        }

        // Add detailed commit information
        document.push_str("## 📝 Información Detallada de Commits\n\n");
        for commit in commits {
            document.push_str(&format!("### Commit: {}\n", commit.hash[..8].to_string()));
            document.push_str(&format!("- **Autor**: {} <{}>\n", commit.author_name, commit.author_email));
            document.push_str(&format!("- **Fecha**: {}\n", commit.commit_date.format("%Y-%m-%d %H:%M:%S UTC")));
            document.push_str(&format!("- **Tipo**: {}\n", commit.commit_type.as_deref().unwrap_or("unknown")));
            if let Some(scope) = &commit.scope {
                document.push_str(&format!("- **Scope**: {}\n", scope));
            }
            document.push_str(&format!("- **Descripción**: {}\n", commit.description));
            
            if !commit.test_details.is_empty() {
                document.push_str("- **Detalles de Pruebas**:\n");
                for test in &commit.test_details {
                    document.push_str(&format!("  - {}\n", test));
                }
            }
            
            if let Some(security) = &commit.security {
                document.push_str(&format!("- **Seguridad**: {}\n", security));
            }
            
            if let Some(change_id) = &commit.change_id {
                document.push_str(&format!("- **Change-Id**: {}\n", change_id));
            }
            
            document.push('\n');
        }

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

        document
    }
}

pub async fn test_gemini_connection(config: &AppConfig) -> Result<String> {
    let client = GeminiClient::new(config)?;
    
    let test_prompt = "Responde con 'Conexión exitosa con Google Gemini' si puedes leer este mensaje.";
    
    match client.call_gemini_api(test_prompt, "gemini-1.5-pro").await {
        Ok(response) => Ok(response),
        Err(_) => client.call_gemini_api(test_prompt, "gemini-1.0-pro").await,
    }
} 