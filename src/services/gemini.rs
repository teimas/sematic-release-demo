use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    error::{Result, SemanticReleaseError},
    types::AppConfig,
};

// =============================================================================
// CORE GEMINI CLIENT
// =============================================================================

pub struct GeminiClient {
    client: Client,
}

impl GeminiClient {
    #[instrument(skip(config))]
    pub fn new(config: &AppConfig) -> Result<Self> {
        info!("Initializing Gemini AI client");

        let api_key = config
            .gemini_token
            .as_ref()
            .ok_or_else(|| {
                error!("Google Gemini API key not configured");
                SemanticReleaseError::config_error("Google Gemini API key not configured")
            })?
            .clone();

        // Set the API key as environment variable for genai
        std::env::set_var("GEMINI_API_KEY", &api_key);

        let client = Client::default();

        info!("Gemini AI client initialized successfully");
        Ok(Self { client })
    }
}

// =============================================================================
// GEMINI API COMMUNICATION
// =============================================================================

impl GeminiClient {
    #[instrument(skip(self), fields(prompt_len = prompt.len()))]
    async fn call_gemini_with_fallback(&self, prompt: &str) -> Result<String> {
        debug!("Attempting Gemini API call with fallback strategy");

        // Try Gemini 2.5 Pro Preview first (most advanced), then fallback to 2.0 Flash
        match self
            .call_gemini_api(prompt, "gemini-2.5-pro-preview-06-05")
            .await
        {
            Ok(response) => {
                info!(
                    model = "gemini-2.5-pro-preview-06-05",
                    "Gemini API call successful"
                );
                Ok(response)
            }
            Err(e) => {
                warn!(
                    model = "gemini-2.5-pro-preview-06-05",
                    error = %e,
                    "Gemini 2.5 Pro Preview failed, trying 2.0 Flash"
                );

                let fallback_response = self.call_gemini_api(prompt, "gemini-2.0-flash").await?;
                info!(model = "gemini-2.0-flash", "Gemini API fallback successful");
                Ok(fallback_response)
            }
        }
    }

    #[instrument(skip(self), fields(model = model, prompt_len = prompt.len()))]
    async fn call_gemini_api(&self, prompt: &str, model: &str) -> Result<String> {
        debug!(model = model, "Making Gemini API request");

        let chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);

        let chat_res = self
            .client
            .exec_chat(model, chat_req, None)
            .await
            .map_err(|e| {
                error!(model = model, error = %e, "Gemini API request failed");
                SemanticReleaseError::ai_error("Gemini", e)
            })?;

        let content = chat_res.content_text_as_str().ok_or_else(|| {
            error!(model = model, "Gemini API returned no response content");
            SemanticReleaseError::ai_error(
                "Gemini",
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "No response content from Gemini API",
                ),
            )
        })?;

        debug!(
            model = model,
            response_len = content.len(),
            "Gemini API response received"
        );

        Ok(content.to_string())
    }
}

// =============================================================================
// RELEASE NOTES GENERATION FEATURE
// =============================================================================

impl GeminiClient {
    #[instrument(skip(self), fields(document_len = document.len()))]
    pub async fn process_release_notes_document(&self, document: &str) -> Result<String> {
        info!("Processing release notes document with Gemini");

        // This method sends the complete structured document to Gemini for processing
        // (like the Node.js script's processWithGemini function)
        let result = self.call_gemini_with_fallback(document).await?;

        info!(
            input_len = document.len(),
            output_len = result.len(),
            "Release notes processing completed"
        );

        Ok(result)
    }
}

// =============================================================================
// COMMIT ANALYSIS FEATURE
// =============================================================================

impl GeminiClient {
    #[instrument(skip(self), fields(changes_len = changes.len()))]
    pub async fn generate_comprehensive_commit_analysis(
        &self,
        changes: &str,
    ) -> Result<serde_json::Value> {
        info!("Generating comprehensive commit analysis with Gemini");

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

7. **ANÁLISIS DE PRUEBAS** (testAnalysis):
   - Recomienda pruebas manuales específicas que una persona debería realizar para verificar los cambios
   - Incluye casos de prueba concretos y pasos detallados
   - Considera diferentes escenarios: casos normales, casos extremos, casos de error
   - Menciona qué funcionalidades específicas probar basándose en los cambios realizados
   - Incluye verificaciones de integración si es aplicable
   - Sugiere datos de prueba específicos si es necesario
   - Estructura las recomendaciones como una lista clara y accionable
   - Si no hay necesidad de pruebas manuales específicas: devuelve cadena vacía ""
   - RESPONDE EN ESPAÑOL

FORMATO DE RESPUESTA:
Responde ÚNICAMENTE con un JSON válido usando EXACTAMENTE esta estructura:

{{
  "title": "título conciso aquí",
  "commitType": "tipo_de_commit",
  "description": "descripción técnica exhaustiva aquí",
  "scope": "ámbito_del_código",
  "securityAnalysis": "análisis de seguridad o cadena vacía",
  "breakingChanges": "cambios que rompen compatibilidad o cadena vacía",
  "testAnalysis": "recomendaciones de pruebas manuales en español o cadena vacía"
}}

VALIDACIONES:
- El JSON debe ser VÁLIDO y parseable
- Todos los campos son obligatorios (usa cadena vacía si no aplica)
- El título debe ser ≤ 50 caracteres
- La descripción debe ser ≥ 150 palabras
- commitType debe ser uno de los tipos válidos listados
- testAnalysis debe estar en español y ser específico para los cambios realizados
- NO incluyas explicaciones fuera del JSON
- NO uses comillas triples ni formato markdown
- NO agregues texto antes o después del JSON"#,
            changes
        );

        debug!(prompt_len = prompt.len(), "Built commit analysis prompt");

        let response = self.call_gemini_with_fallback(&prompt).await?;

        // Clean the response - remove markdown code blocks and extra text
        let cleaned_response = self.extract_json_from_response(&response);
        debug!(
            raw_response_len = response.len(),
            cleaned_response_len = cleaned_response.len(),
            "Cleaned Gemini response"
        );

        // Try to parse the JSON response
        match serde_json::from_str::<serde_json::Value>(&cleaned_response) {
            Ok(json) => {
                // Validate that all required fields are present
                if json.get("title").is_some()
                    && json.get("commitType").is_some()
                    && json.get("description").is_some()
                    && json.get("scope").is_some()
                    && json.get("securityAnalysis").is_some()
                    && json.get("breakingChanges").is_some()
                    && json.get("testAnalysis").is_some()
                {
                    info!("Commit analysis completed successfully");
                    Ok(json)
                } else {
                    warn!("Gemini JSON response missing required fields, using fallback");
                    debug!(parsed_json = ?json, "Incomplete JSON response");

                    // Return a fallback JSON structure
                    Ok(serde_json::json!({
                        "title": "cambios realizados en el código",
                        "commitType": "chore",
                        "description": "Se realizaron cambios en el código del proyecto. Respuesta de Gemini incompleta.",
                        "scope": "general",
                        "securityAnalysis": "",
                        "breakingChanges": "",
                        "testAnalysis": ""
                    }))
                }
            }
            Err(e) => {
                error!(
                    parse_error = %e,
                    raw_response_preview = %response.chars().take(200).collect::<String>(),
                    cleaned_response_preview = %cleaned_response.chars().take(200).collect::<String>(),
                    "Failed to parse Gemini JSON response, using fallback"
                );

                // Return a fallback JSON structure
                Ok(serde_json::json!({
                    "title": "cambios realizados en el código",
                    "commitType": "chore",
                    "description": "Se realizaron cambios en el código del proyecto. No se pudo generar un análisis detallado automáticamente.",
                    "scope": "general",
                    "securityAnalysis": "",
                    "breakingChanges": "",
                    "testAnalysis": ""
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

#[instrument(skip(config))]
pub async fn test_gemini_connection(config: &AppConfig) -> Result<String> {
    info!("Testing Gemini connection");

    let client = GeminiClient::new(config)?;

    let test_prompt =
        "Responde con 'Conexión exitosa con Google Gemini' si puedes leer este mensaje.";

    match client
        .call_gemini_api(test_prompt, "gemini-2.5-pro-preview-06-05")
        .await
    {
        Ok(response) => {
            info!(
                model = "gemini-2.5-pro-preview-06-05",
                "Gemini connection test successful"
            );
            Ok(response)
        }
        Err(_e) => {
            warn!("Gemini 2.5 Pro Preview failed during connection test, trying fallback");
            let fallback_response = client
                .call_gemini_api(test_prompt, "gemini-2.0-flash")
                .await?;

            info!(
                model = "gemini-2.0-flash",
                "Gemini connection test successful with fallback"
            );
            Ok(fallback_response)
        }
    }
}
