use anyhow::Result;
use jira_query::{Auth, Issue, JiraInstance};

use crate::types::{AppConfig, JiraTask};
use crate::utils;

// =============================================================================
// CORE JIRA CLIENT STRUCTURE
// =============================================================================

pub struct JiraClient {
    config: AppConfig,
    jira_instance: Option<JiraInstance>,
}

impl JiraClient {
    fn log_debug(&self, message: &str) {
        utils::log_debug("JIRA", message);
    }

    pub fn new(config: &AppConfig) -> Result<Self> {
        // Validate JIRA configuration
        let jira_instance = if let (Some(url), Some(username), Some(api_token)) = (
            &config.jira_url,
            &config.jira_username,
            &config.jira_api_token,
        ) {
            // Log initialization before self is fully constructed
            utils::log_debug("JIRA", &format!("🔧 JIRA Client - Initializing with URL: {}", url));
            
            let instance = JiraInstance::at(url.clone())
                .map_err(|e| anyhow::anyhow!("Failed to create JIRA instance: {}", e))?
                .authenticate(Auth::Basic {
                    user: username.clone(),
                    password: api_token.clone(),
                });
            
            Some(instance)
        } else {
            utils::log_debug("JIRA", &format!("❌ JIRA configuration incomplete - missing URL: {}, username: {}, API token: {}", 
                config.jira_url.is_none(),
                config.jira_username.is_none(), 
                config.jira_api_token.is_none()));
            None
        };

        Ok(Self {
            config: config.clone(),
            jira_instance,
        })
        }

    pub async fn search_tasks(&self, query: &str) -> Result<Vec<JiraTask>> {
        let instance = self.jira_instance.as_ref()
            .ok_or_else(|| anyhow::anyhow!("JIRA not configured properly"))?;

        // Build JQL query for the configured project
        let jql = self.build_jql_query(query);
        self.log_debug(&format!("🔍 JIRA Search - JQL Query: {}", jql));

        // Perform the search using jira_query
        match instance.search(&jql).await {
            Ok(issues) => {
                self.log_debug(&format!("✅ JIRA Search Success: Found {} issues", issues.len()));
                let mut tasks = Vec::new();
                for issue in issues {
                    match self.convert_jira_issue_to_task(issue) {
                        Ok(task) => tasks.push(task),
                        Err(e) => {
                            self.log_debug(&format!("⚠️  Failed to convert JIRA issue to task: {}", e));
                        }
                    }
                }
                Ok(tasks)
            }
            Err(e) => {
                self.log_debug(&format!("❌ JIRA Search Failed: {}", e));
                Err(anyhow::anyhow!("JIRA search failed: {}", e))
            }
        }
    }

    pub async fn get_task_details(&self, task_keys: &[String]) -> Result<Vec<JiraTask>> {
        let instance = self.jira_instance.as_ref()
            .ok_or_else(|| anyhow::anyhow!("JIRA not configured properly"))?;

        let mut tasks = Vec::new();
        let mut errors = Vec::new();
        
        for key in task_keys {
            self.log_debug(&format!("🔍 JIRA - Fetching task details for: {}", key));
            match instance.issue(key).await {
                Ok(issue) => {
                    match self.convert_jira_issue_to_task(issue) {
                        Ok(task) => {
                            self.log_debug(&format!("✅ JIRA - Successfully fetched task: {}", key));
                            tasks.push(task);
                        }
                        Err(e) => {
                            self.log_debug(&format!("❌ JIRA - Failed to convert issue {}: {}", key, e));
                            errors.push(format!("{}: {}", key, e));
                        }
                    }
                }
                Err(e) => {
                    self.log_debug(&format!("❌ JIRA - Failed to fetch task {}: {}", key, e));
                    errors.push(format!("{}: {}", key, e));
                }
            }
        }
        
        if !errors.is_empty() && tasks.is_empty() {
            return Err(anyhow::anyhow!("Failed to fetch any JIRA tasks: {}", errors.join(", ")));
        }
        
        if !errors.is_empty() {
            self.log_debug(&format!("⚠️  Some JIRA tasks failed to load: {}", errors.join(", ")));
        }
        
        Ok(tasks)
    }

    pub async fn test_connection(&self) -> Result<String> {
        let instance = self.jira_instance.as_ref()
            .ok_or_else(|| anyhow::anyhow!("JIRA configuration incomplete - missing URL, username, or API token"))?;

        // Try to search for any issue to test the connection
        let test_jql = if let Some(project_key) = &self.config.jira_project_key {
            format!("project = {} ORDER BY created DESC", project_key)
        } else {
            "ORDER BY created DESC".to_string()
        };

        self.log_debug(&format!("🔍 JIRA Connection Test - JQL: {}", test_jql));

        match instance.search(&test_jql).await {
            Ok(issues) => {
                let message = format!("✅ JIRA connection successful! Found {} issues", issues.len());
                self.log_debug(&message);
                Ok(message)
            }
            Err(e) => {
                let error_message = format!("❌ JIRA connection failed: {}", e);
                self.log_debug(&error_message);
                Err(anyhow::anyhow!(error_message))
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
        let components: Vec<String> = issue.fields.components.iter()
            .map(|c| c.name.clone())
            .collect();

        // Extract assignee information
        let assignee = issue.fields.assignee.as_ref()
            .map(|a| a.display_name.clone());

        // Extract priority
        let priority = issue.fields.priority.as_ref()
            .map(|p| p.name.clone());

        Ok(JiraTask {
            id: issue.id,
            key: issue.key,
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
            components: if components.is_empty() { None } else { Some(components) },
            labels: if issue.fields.labels.is_empty() { None } else { Some(issue.fields.labels) },
        })
    }
} 