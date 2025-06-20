use jira_query::{Auth, Issue, JiraInstance};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    error::{Result, SemanticReleaseError},
    types::{AppConfig, JiraTask},
};

// =============================================================================
// CORE JIRA CLIENT STRUCTURE
// =============================================================================

pub struct JiraClient {
    config: AppConfig,
    jira_instance: Option<JiraInstance>,
}

impl JiraClient {
    #[instrument(skip(config))]
    pub fn new(config: &AppConfig) -> Result<Self> {
        info!("Initializing JIRA client");
        
        // Validate JIRA configuration
        let jira_instance = if let (Some(url), Some(username), Some(api_token)) = (
            &config.jira_url,
            &config.jira_username,
            &config.jira_api_token,
        ) {
            info!(url = %url, username = %username, "Creating JIRA client instance");

            let instance = JiraInstance::at(url.clone())
                .map_err(|e| {
                    error!(url = %url, error = %e, "Failed to create JIRA instance");
                    SemanticReleaseError::jira_error(e)
                })?
                .authenticate(Auth::Basic {
                    user: username.clone(),
                    password: api_token.clone(),
                });

            Some(instance)
        } else {
            let missing_fields = [
                ("url", config.jira_url.is_none()),
                ("username", config.jira_username.is_none()),
                ("api_token", config.jira_api_token.is_none()),
            ]
            .iter()
            .filter(|(_, is_missing)| *is_missing)
            .map(|(field, _)| *field)
            .collect::<Vec<_>>();

            warn!(missing_fields = ?missing_fields, "JIRA configuration incomplete");
            None
        };

        let client = Self {
            config: config.clone(),
            jira_instance,
        };

        info!(
            configured = client.jira_instance.is_some(),
            project_key = ?config.jira_project_key,
            "JIRA client initialized"
        );

        Ok(client)
    }

    #[instrument(skip(self), fields(query = query))]
    pub async fn search_tasks(&self, query: &str) -> Result<Vec<JiraTask>> {
        info!("Searching JIRA tasks");
        
        let instance = self
            .jira_instance
            .as_ref()
            .ok_or_else(|| {
                error!("JIRA search attempted but client not configured");
                SemanticReleaseError::config_error("JIRA not configured properly - missing URL, username, or API token")
            })?;

        // Build JQL query for the configured project
        let jql = self.build_jql_query(query);
        debug!(jql = %jql, "Built JQL query for search");

        // Perform the search using jira_query
        match instance.search(&jql).await {
            Ok(issues) => {
                info!(issue_count = issues.len(), "JIRA search completed successfully");
                
                let mut tasks = Vec::new();
                let mut conversion_errors = Vec::new();
                
                for issue in issues {
                    match self.convert_jira_issue_to_task(issue) {
                        Ok(task) => {
                            debug!(task_key = %task.key, "Successfully converted JIRA issue to task");
                            tasks.push(task);
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to convert JIRA issue to task");
                            conversion_errors.push(e.to_string());
                        }
                    }
                }

                if !conversion_errors.is_empty() {
                    warn!(
                        conversion_errors = conversion_errors.len(),
                        successful_tasks = tasks.len(),
                        "Some JIRA issues failed to convert"
                    );
                }

                Ok(tasks)
            }
            Err(e) => {
                error!(jql = %jql, error = %e, "JIRA search failed");
                Err(SemanticReleaseError::jira_error(e))
            }
        }
    }

    #[instrument(skip(self), fields(task_count = task_keys.len()))]
    pub async fn get_task_details(&self, task_keys: &[String]) -> Result<Vec<JiraTask>> {
        info!("Fetching JIRA task details");
        
        let instance = self
            .jira_instance
            .as_ref()
            .ok_or_else(|| {
                error!("JIRA task details fetch attempted but client not configured");
                SemanticReleaseError::config_error("JIRA not configured properly - missing URL, username, or API token")
            })?;

        let mut tasks = Vec::new();
        let mut errors = Vec::new();

        for key in task_keys {
            debug!(task_key = %key, "Fetching JIRA task details");
            
            match instance.issue(key).await {
                Ok(issue) => match self.convert_jira_issue_to_task(issue) {
                    Ok(task) => {
                        info!(task_key = %key, "Successfully fetched JIRA task");
                        tasks.push(task);
                    }
                    Err(e) => {
                        warn!(task_key = %key, error = %e, "Failed to convert JIRA issue");
                        errors.push(format!("{}: {}", key, e));
                    }
                },
                Err(e) => {
                    warn!(task_key = %key, error = %e, "Failed to fetch JIRA task");
                    errors.push(format!("{}: {}", key, e));
                }
            }
        }

        if !errors.is_empty() && tasks.is_empty() {
            error!(error_count = errors.len(), "Failed to fetch any JIRA tasks");
            return Err(SemanticReleaseError::jira_error(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to fetch any JIRA tasks: {}", errors.join(", "))
                )
            ));
        }

        if !errors.is_empty() {
            warn!(
                successful_tasks = tasks.len(),
                failed_tasks = errors.len(),
                "Some JIRA tasks failed to load"
            );
        }

        info!(
            successful_tasks = tasks.len(),
            failed_tasks = errors.len(),
            "JIRA task details fetch completed"
        );

        Ok(tasks)
    }

    #[instrument(skip(self))]
    pub async fn test_connection(&self) -> Result<String> {
        info!("Testing JIRA connection");
        
        let instance = self.jira_instance.as_ref().ok_or_else(|| {
            error!("JIRA connection test attempted but client not configured");
            SemanticReleaseError::config_error(
                "JIRA configuration incomplete - missing URL, username, or API token"
            )
        })?;

        // Try to search for any issue to test the connection
        let test_jql = if let Some(project_key) = &self.config.jira_project_key {
            format!("project = {} ORDER BY created DESC", project_key)
        } else {
            "ORDER BY created DESC".to_string()
        };

        debug!(test_jql = %test_jql, "Testing JIRA connection with JQL query");

        match instance.search(&test_jql).await {
            Ok(issues) => {
                let message = format!(
                    "✅ JIRA connection successful! Found {} issues",
                    issues.len()
                );
                info!(issue_count = issues.len(), "JIRA connection test successful");
                Ok(message)
            }
            Err(e) => {
                error!(test_jql = %test_jql, error = %e, "JIRA connection test failed");
                Err(SemanticReleaseError::jira_error(e))
            }
        }
    }

    // =============================================================================
    // HELPER METHODS
    // =============================================================================

    fn build_jql_query(&self, query: &str) -> String {
        let mut jql_parts = Vec::new();

        // Add project filter if configured
        if let Some(project_key) = &self.config.jira_project_key {
            jql_parts.push(format!("project = {}", project_key));
        }

        // Add text search if query is not empty
        if !query.trim().is_empty() {
            // Search in summary, description, and comments
            let text_search = format!(
                "(summary ~ \"{}\" OR description ~ \"{}\" OR comment ~ \"{}\")",
                query, query, query
            );
            jql_parts.push(text_search);
        }

        // Combine parts with AND
        if jql_parts.is_empty() {
            "ORDER BY created DESC".to_string()
        } else {
            format!("{} ORDER BY created DESC", jql_parts.join(" AND "))
        }
    }

    fn convert_jira_issue_to_task(&self, issue: Issue) -> Result<JiraTask> {
        // Extract components into a string
        let components: Vec<String> = issue
            .fields
            .components
            .iter()
            .map(|c| c.name.clone())
            .collect();

        // Extract assignee information
        let assignee = issue
            .fields
            .assignee
            .as_ref()
            .map(|a| a.display_name.clone());

        // Extract priority
        let priority = issue.fields.priority.as_ref().map(|p| p.name.clone());

        Ok(JiraTask {
            id: issue.id,
            key: issue.key.clone(),
            summary: issue.fields.summary,
            description: issue.fields.description,
            status: issue.fields.status.name,
            issue_type: issue.fields.issuetype.name,
            priority,
            assignee,
            reporter: Some(issue.fields.reporter.display_name),
            project_key: issue.fields.project.key,
            project_name: issue.fields.project.name,
            created: Some(issue.fields.created.to_rfc3339()),
            updated: Some(issue.fields.updated.to_rfc3339()),
            components: if components.is_empty() {
                None
            } else {
                Some(components)
            },
            labels: if issue.fields.labels.is_empty() {
                None
            } else {
                Some(issue.fields.labels)
            },
        })
    }
}

