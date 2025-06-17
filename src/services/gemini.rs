use anyhow::Result;
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;

use crate::types::AppConfig;
use crate::utils;

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
                utils::log_info("GEMINI", "Gemini 2.5 Pro Preview failed, trying 2.0 Flash...");
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
    pub async fn process_release_notes_document(&self, document: &str) -> Result<String> {
        // This method sends the complete structured document to Gemini for processing
        // (like the Node.js script's processWithGemini function)
        self.call_gemini_with_fallback(document).await
    }
}

// =============================================================================
// COMMIT ANALYSIS FEATURE
// =============================================================================

impl GeminiClient {










    pub async fn generate_comprehensive_commit_analysis(&self, changes: &str) -> Result<serde_json::Value> {
        let prompt = format!(
            r#"Eres un desarrollador experto y especialista en semantic release que debe analizar cambios de código de forma EXHAUSTIVA y generar un análisis completo de commit.

CAMBIOS EN EL CÓDIGO:
{}

INSTRUCCIONES CRÍTICAS:
Analiza MINUCIOSAMENTE los cambios proporcionados y genera un JSON con toda la información del commit. Debes ser EXTREMADAMENTE DETALLADO y PRECISO.

ANÁLISIS REQUERIDO:

1. **TÍTULO** (title):
   - Genera un título CONCISO y DESCRIPTIVO en español (máximo 50 caracteres)
   - Usa presente imperativo (ej: "añade", "corrige", "actualiza")
   - Debe explicar QUÉ se hizo específicamente
   - NO incluyas el tipo de commit (feat:, fix:, etc.)
   - Ejemplos: "añade validación de email", "corrige error de memoria"

2. **TIPO DE COMMIT** (commitType):
   - Determina el tipo según semantic release:
   - "feat": Nueva funcionalidad para el usuario final
   - "fix": Corrección de un bug o error
   - "docs": Solo cambios en documentación
   - "style": Cambios de formato sin afectar lógica
   - "refactor": Reestructuración de código sin cambios funcionales
   - "perf": Mejoras de rendimiento
   - "test": Añadir o modificar tests
   - "chore": Cambios de herramientas, configuración, build
   - "revert": Reversión de cambios anteriores

3. **DESCRIPCIÓN** (description):
   - Genera una descripción TÉCNICA EXHAUSTIVA en español (mínimo 150 palabras)
   - Explica QUÉ cambios específicos se hicieron
   - Explica POR QUÉ estos cambios son importantes
   - Describe el IMPACTO técnico y funcional
   - Menciona archivos, funciones, clases específicas modificadas
   - Incluye detalles de implementación relevantes
   - Usa terminología técnica precisa

4. **ÁMBITO/SCOPE** (scope):
   - Identifica el área del código afectada
   - Puede ser: api, ui, database, auth, config, utils, models, services, etc.
   - Si afecta múltiples áreas, usa todas y ordena por el área más importante
   - Si no hay un ámbito claro, usa "general"

5. **ANÁLISIS DE SEGURIDAD** (securityAnalysis):
   - Busca EXHAUSTIVAMENTE vulnerabilidades como:
     * Inyección SQL, XSS, CSRF
     * Manejo inseguro de datos sensibles (passwords, tokens, keys)
     * Validación insuficiente de entrada
     * Exposición de información confidencial
     * Configuraciones de seguridad débiles
     * Dependencias con vulnerabilidades
     * Privilegios elevados innecesarios
     * Manejo inseguro de archivos/rutas
   - Si NO hay riesgos: devuelve cadena vacía ""
   - Si SÍ hay riesgos: describe específicamente qué riesgos encontraste

6. **CAMBIOS QUE ROMPEN COMPATIBILIDAD** (breakingChanges):
   - Identifica breaking changes como:
     * Eliminación de APIs, funciones, clases públicas
     * Cambios en signatures (parámetros, tipos de retorno)
     * Modificación de contratos de interfaz
     * Cambios en formatos de datos o protocolos
     * Eliminación de configuraciones
     * Cambios en comportamiento esperado de APIs
     * Modificaciones de esquemas de BD
     * Cambios en URLs de endpoints
   - Si NO hay breaking changes: devuelve cadena vacía ""
   - Si SÍ hay breaking changes: describe específicamente qué se rompió

FORMATO DE RESPUESTA:
Responde ÚNICAMENTE con un JSON válido usando EXACTAMENTE esta estructura:

{{
  "title": "título conciso aquí",
  "commitType": "tipo_de_commit",
  "description": "descripción técnica exhaustiva aquí",
  "scope": "ámbito_del_código",
  "securityAnalysis": "análisis de seguridad o cadena vacía",
  "breakingChanges": "cambios que rompen compatibilidad o cadena vacía"
}}

VALIDACIONES:
- El JSON debe ser VÁLIDO y parseable
- Todos los campos son obligatorios (usa cadena vacía si no aplica)
- El título debe ser ≤ 50 caracteres
- La descripción debe ser ≥ 150 palabras
- commitType debe ser uno de los tipos válidos listados
- NO incluyas explicaciones fuera del JSON
- NO uses comillas triples ni formato markdown
- NO agregues texto antes o después del JSON"#,
            changes
        );

        let response = self.call_gemini_with_fallback(&prompt).await?;
        
        // Clean the response - remove markdown code blocks and extra text
        let cleaned_response = self.extract_json_from_response(&response);
        
        // Try to parse the JSON response
        match serde_json::from_str::<serde_json::Value>(&cleaned_response) {
            Ok(json) => {
                // Validate that all required fields are present
                if json.get("title").is_some() && 
                   json.get("commitType").is_some() && 
                   json.get("description").is_some() && 
                   json.get("scope").is_some() && 
                   json.get("securityAnalysis").is_some() && 
                   json.get("breakingChanges").is_some() {
                    Ok(json)
                } else {
                    utils::log_warning("GEMINI", "JSON response missing required fields");
                    utils::log_debug("GEMINI", &format!("Parsed JSON: {}", json));
                    // Return a fallback JSON structure
                    Ok(serde_json::json!({
                        "title": "cambios realizados en el código",
                        "commitType": "chore", 
                        "description": "Se realizaron cambios en el código del proyecto. Respuesta de Gemini incompleta.",
                        "scope": "general",
                        "securityAnalysis": "",
                        "breakingChanges": ""
                    }))
                }
            }
            Err(e) => {
                utils::log_error("GEMINI", &e);
                utils::log_debug("GEMINI", &format!("Raw response: {}", response));
                utils::log_debug("GEMINI", &format!("Cleaned response: {}", cleaned_response));
                
                // Return a fallback JSON structure
                Ok(serde_json::json!({
                    "title": "cambios realizados en el código",
                    "commitType": "chore", 
                    "description": "Se realizaron cambios en el código del proyecto. No se pudo generar un análisis detallado automáticamente.",
                    "scope": "general",
                    "securityAnalysis": "",
                    "breakingChanges": ""
                }))
            }
        }
    }

    // Helper method to extract JSON from Gemini response that might be wrapped in markdown
    fn extract_json_from_response(&self, response: &str) -> String {
        let response = response.trim();
        
        // Case 1: Response is wrapped in markdown code blocks
        if let Some(start) = response.find("```json") {
            // Look for the closing ``` after the opening ```json
            let search_start = start + 7; // Skip past "```json"
            if search_start < response.len() {
                if let Some(end_offset) = response[search_start..].find("```") {
                    let end_pos = search_start + end_offset;
                    // Safely extract content between ```json and ```
                    if start + 7 <= end_pos && end_pos <= response.len() {
                        return response[start + 7..end_pos].trim().to_string();
                    }
                }
            }
        }
        
        // Case 2: Response is wrapped in regular code blocks
        if let Some(start) = response.find("```") {
            let search_start = start + 3; // Skip past "```"
            if search_start < response.len() {
                if let Some(end_offset) = response[search_start..].find("```") {
                    let end_pos = search_start + end_offset;
                    // Safely extract content between ``` and ```
                    if start + 3 <= end_pos && end_pos <= response.len() {
                        return response[start + 3..end_pos].trim().to_string();
                    }
                }
            }
        }
        
        // Case 3: Response contains JSON between braces
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if end > start && end < response.len() {
                    return response[start..=end].trim().to_string();
                }
            }
        }
        
        // Case 4: Return as-is if no special formatting detected
        response.to_string()
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