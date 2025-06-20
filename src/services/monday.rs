use reqwest::Client;
use serde_json::{json, Value};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    error::{Result, SemanticReleaseError},
    types::{AppConfig, MondayColumnValue, MondayTask, MondayUpdate, MondayUser},
};

// =============================================================================
// CORE MONDAY.COM CLIENT STRUCTURE
// =============================================================================

pub struct MondayClient {
    client: Client,
    api_key: String,
    account_slug: Option<String>,
    board_id: Option<String>,
    url_template: Option<String>,
}

impl MondayClient {
    #[instrument(skip(config))]
    pub fn new(config: &AppConfig) -> Result<Self> {
        info!("Initializing Monday.com client");

        let api_key = config
            .monday_api_key
            .as_ref()
            .ok_or_else(|| {
                error!("Monday.com API key not configured");
                SemanticReleaseError::config_error("Monday.com API key not configured")
            })?
            .clone();

        let client = Self {
            client: Client::new(),
            api_key,
            account_slug: config.monday_account_slug.clone(),
            board_id: config.monday_board_id.clone(),
            url_template: config.monday_url_template.clone(),
        };

        info!(
            account_slug = ?client.account_slug,
            board_id = ?client.board_id,
            has_url_template = client.url_template.is_some(),
            "Monday.com client initialized"
        );

        Ok(client)
    }
}

// =============================================================================
// TASK SEARCH AND DISCOVERY
// =============================================================================

impl MondayClient {
    #[instrument(skip(self), fields(query = query))]
    pub async fn search_tasks(&self, query: &str) -> Result<Vec<MondayTask>> {
        info!("Searching Monday.com tasks");

        let graphql_query = self.build_search_query(query);
        debug!(
            query_type = if self.board_id.is_some() {
                "board_specific"
            } else {
                "global"
            },
            "Built GraphQL search query"
        );

        let response = self.execute_graphql_request(&graphql_query).await?;
        let result: Value = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse Monday.com search response as JSON");
            SemanticReleaseError::monday_error(e)
        })?;

        let tasks = self.parse_search_results(result)?;
        info!(
            task_count = tasks.len(),
            "Monday.com search completed successfully"
        );

        Ok(tasks)
    }

    fn build_search_query(&self, query: &str) -> Value {
        if let Some(board_id) = &self.board_id {
            // Search in specific board
            json!({
                "query": r#"
                    query ($boardId: [ID!], $limit: Int!, $queryParams: ItemsQuery) {
                        boards(ids: $boardId) {
                            name
                            items_page(limit: $limit, query_params: $queryParams) {
                                items {
                                    id
                                    name
                                    state
                                    board { id }
                                    updates(limit: 5) {
                                        id
                                        body
                                        created_at
                                        creator {
                                            id
                                            name
                                        }
                                    }
                                }
                            }
                        }
                    }
                "#,
                "variables": {
                    "boardId": [board_id],
                    "limit": 20,
                    "queryParams": {
                        "rules": [
                            {
                                "column_id": "name",
                                "operator": "contains_text",
                                "compare_value": query
                            }
                        ],
                        "operator": "and"
                    }
                }
            })
        } else {
            // Global search
            json!({
                "query": r#"
                    query ($limit: Int!, $queryParams: ItemsQuery) {
                        items_page(limit: $limit, query_params: $queryParams) {
                            items {
                                id
                                name
                                state
                                board { id name }
                                updates(limit: 5) {
                                    id
                                    body
                                    created_at
                                    creator {
                                        id
                                        name
                                    }
                                }
                            }
                        }
                    }
                "#,
                "variables": {
                    "limit": 50,
                    "queryParams": {
                        "rules": [
                            {
                                "column_id": "name",
                                "operator": "contains_text",
                                "compare_value": query
                            }
                        ],
                        "operator": "or"
                    }
                }
            })
        }
    }
}

// =============================================================================
// TASK DETAILS AND RETRIEVAL
// =============================================================================

impl MondayClient {}

// =============================================================================
// GRAPHQL REQUEST EXECUTION
// =============================================================================

impl MondayClient {
    #[instrument(skip(self, query))]
    async fn execute_graphql_request(&self, query: &Value) -> Result<reqwest::Response> {
        debug!("Executing Monday.com GraphQL request");

        let response = self
            .client
            .post("https://api.monday.com/v2")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .header("API-Version", "2024-10")
            .json(query)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Monday.com GraphQL request failed");
                SemanticReleaseError::monday_error(e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            error!(status = %status, "Monday.com API returned error status");
            return Err(SemanticReleaseError::monday_error(std::io::Error::other(
                format!("Monday.com API error: HTTP {}", status),
            )));
        }

        debug!(status = %response.status(), "Monday.com GraphQL request successful");
        Ok(response)
    }
}

// =============================================================================
// SEARCH RESULTS PARSING
// =============================================================================

impl MondayClient {
    fn parse_search_results(&self, result: Value) -> Result<Vec<MondayTask>> {
        let mut tasks = Vec::new();

        if let Some(_board_id) = &self.board_id {
            // Parse board-specific results
            tasks.extend(self.parse_board_specific_results(&result));
            debug!(
                task_count = tasks.len(),
                "Parsed board-specific search results"
            );
        } else {
            // Parse global search results
            tasks.extend(self.parse_global_search_results(&result));
            debug!(task_count = tasks.len(), "Parsed global search results");
        }

        Ok(tasks)
    }

    fn parse_board_specific_results(&self, result: &Value) -> Vec<MondayTask> {
        let mut tasks = Vec::new();

        if let Some(boards) = result["data"]["boards"].as_array() {
            for board in boards {
                if let Some(items_page) = board["items_page"].as_object() {
                    if let Some(items) = items_page["items"].as_array() {
                        for item in items {
                            if let Some(task) = self.parse_task_item(item) {
                                if task.state == "active" {
                                    debug!(task_id = %task.id, "Found active Monday.com task");
                                    tasks.push(task);
                                } else {
                                    debug!(task_id = %task.id, state = %task.state, "Skipping non-active Monday.com task");
                                }
                            }
                        }
                    }
                }
            }
        }

        tasks
    }

    fn parse_global_search_results(&self, result: &Value) -> Vec<MondayTask> {
        let mut tasks = Vec::new();

        if let Some(items_page) = result["data"]["items_page"].as_object() {
            if let Some(items) = items_page["items"].as_array() {
                for item in items {
                    if let Some(task) = self.parse_task_item(item) {
                        if task.state == "active" {
                            debug!(task_id = %task.id, "Found active Monday.com task");
                            tasks.push(task);
                        } else {
                            debug!(task_id = %task.id, state = %task.state, "Skipping non-active Monday.com task");
                        }
                    }
                }
            }
        }

        tasks
    }
}

// =============================================================================
// TASK ITEM PARSING AND CONSTRUCTION
// =============================================================================

impl MondayClient {
    fn parse_task_item(&self, item: &Value) -> Option<MondayTask> {
        let id = item["id"].as_str()?.to_string();
        let title = item["name"].as_str()?.to_string();
        let state = item["state"].as_str().unwrap_or("unknown").to_string();

        let board_id = item["board"]["id"].as_str().map(|s| s.to_string());
        let board_name = item["board"]["name"].as_str().map(|s| s.to_string());
        let group_title = item["group"]["title"].as_str().map(|s| s.to_string());

        let url = self.generate_task_url(board_id.as_deref().unwrap_or(""), &id);
        let updates = self.parse_task_updates(item);
        let column_values = self.parse_task_column_values(item);

        Some(MondayTask {
            id,
            title,
            board_id,
            board_name,
            url,
            state,
            updates,
            group_title,
            column_values,
        })
    }

    fn parse_task_updates(&self, item: &Value) -> Vec<MondayUpdate> {
        if let Some(updates_array) = item["updates"].as_array() {
            updates_array
                .iter()
                .filter_map(|update| self.parse_update(update))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn parse_task_column_values(&self, item: &Value) -> Vec<MondayColumnValue> {
        if let Some(columns_array) = item["column_values"].as_array() {
            columns_array
                .iter()
                .filter_map(|column| self.parse_column_value(column))
                .collect()
        } else {
            Vec::new()
        }
    }
}

// =============================================================================
// INDIVIDUAL COMPONENT PARSING
// =============================================================================

impl MondayClient {
    fn parse_update(&self, update: &Value) -> Option<MondayUpdate> {
        let id = update["id"].as_str()?.to_string();
        let body = update["body"].as_str().unwrap_or("").to_string();
        let created_at = update["created_at"].as_str().unwrap_or("").to_string();

        let creator = update["creator"].as_object().map(|creator_obj| MondayUser {
            id: creator_obj["id"].as_str().unwrap_or("").to_string(),
            name: creator_obj["name"].as_str().unwrap_or("").to_string(),
        });

        Some(MondayUpdate {
            id,
            body,
            created_at,
            creator,
        })
    }

    fn parse_column_value(&self, column: &Value) -> Option<MondayColumnValue> {
        let id = column["id"].as_str()?.to_string();
        let column_type = column["type"].as_str().unwrap_or("").to_string();
        let text = column["text"].as_str().map(|s| s.to_string());
        let value = column["value"].as_str().map(|s| s.to_string());

        Some(MondayColumnValue {
            id,
            column_type,
            text,
            value,
        })
    }
}

// =============================================================================
// URL GENERATION AND UTILITIES
// =============================================================================

impl MondayClient {
    fn generate_task_url(&self, board_id: &str, item_id: &str) -> String {
        if let Some(template) = &self.url_template {
            template
                .replace("{board_id}", board_id)
                .replace("{item_id}", item_id)
        } else if let Some(slug) = &self.account_slug {
            format!(
                "https://{}.monday.com/boards/{}/pulses/{}",
                slug, board_id, item_id
            )
        } else {
            format!("https://monday.com/boards/{}/pulses/{}", board_id, item_id)
        }
    }
}

// =============================================================================
// CONNECTION TESTING AND VALIDATION
// =============================================================================

impl MondayClient {
    #[instrument(skip(self))]
    pub async fn test_connection(&self) -> Result<String> {
        info!("Testing Monday.com connection");

        let query = json!({
            "query": "query { me { name email } }"
        });

        let response = self.execute_graphql_request(&query).await?;
        let result: Value = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse Monday.com connection test response as JSON");
            SemanticReleaseError::monday_error(e)
        })?;

        let user_info = self.parse_connection_test_result(result)?;
        info!(user_info = %user_info, "Monday.com connection test successful");

        Ok(user_info)
    }

    fn parse_connection_test_result(&self, result: Value) -> Result<String> {
        if let Some(me) = result["data"]["me"].as_object() {
            let name = me["name"].as_str().unwrap_or("Unknown");
            let email = me["email"].as_str().unwrap_or("Unknown");
            Ok(format!("{} ({})", name, email))
        } else {
            error!(response = ?result, "Monday.com connection test returned invalid response structure");
            Err(SemanticReleaseError::monday_error(std::io::Error::other(
                "Failed to get user information from Monday.com API",
            )))
        }
    }
}
