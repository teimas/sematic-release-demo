use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

use crate::types::{AppConfig, MondayTask, MondayUpdate, MondayUser, MondayColumnValue};

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
    pub fn new(config: &AppConfig) -> Result<Self> {
        let api_key = config
            .monday_api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Monday.com API key not configured"))?
            .clone();

        Ok(Self {
            client: Client::new(),
            api_key,
            account_slug: config.monday_account_slug.clone(),
            board_id: config.monday_board_id.clone(),
            url_template: config.monday_url_template.clone(),
        })
    }
}

// =============================================================================
// TASK SEARCH AND DISCOVERY
// =============================================================================

impl MondayClient {
    pub async fn search_tasks(&self, query: &str) -> Result<Vec<MondayTask>> {
        let graphql_query = self.build_search_query(query);

        let response = self.execute_graphql_request(&graphql_query).await?;
        let result: Value = response.json().await?;
        
        self.parse_search_results(result)
    }

    pub async fn comprehensive_search(&self, query: &str) -> Result<Vec<MondayTask>> {
        // First do the regular search
        let mut tasks = self.search_tasks(query).await?;
        
        // If no results found, try a broader search with different casing
        if tasks.is_empty() {
            let lower_query = query.to_lowercase();
            let upper_query = query.to_uppercase();
            
            // Try lowercase
            if let Ok(lower_tasks) = self.search_tasks(&lower_query).await {
                tasks.extend(lower_tasks);
            }
            
            // Try uppercase if still no results
            if tasks.is_empty() {
                if let Ok(upper_tasks) = self.search_tasks(&upper_query).await {
                    tasks.extend(upper_tasks);
                }
            }
        }
        
        // Deduplicate tasks by ID
        tasks.sort_by(|a, b| a.id.cmp(&b.id));
        tasks.dedup_by(|a, b| a.id == b.id);
        
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

impl MondayClient {
    pub async fn get_task_details(&self, task_ids: &[String]) -> Result<Vec<MondayTask>> {
        let ids: Vec<&str> = task_ids.iter().map(|s| s.as_str()).collect();
        
        let graphql_query = self.build_task_details_query(&ids);
        let response = self.execute_graphql_request(&graphql_query).await?;
        let result: Value = response.json().await?;
        
        self.parse_task_details_results(result)
    }

    fn build_task_details_query(&self, ids: &[&str]) -> Value {
        json!({
            "query": r#"
                query ($itemIds: [ID!]) {
                    items(ids: $itemIds) {
                        id
                        name
                        state
                        board { id name }
                        group { id title }
                        column_values {
                            id
                            type
                            text
                            value
                        }
                        updates(limit: 15) {
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
            "#,
            "variables": {
                "itemIds": ids
            }
        })
    }

    fn parse_task_details_results(&self, result: Value) -> Result<Vec<MondayTask>> {
        if let Some(items) = result["data"]["items"].as_array() {
            let tasks = items
                .iter()
                .filter_map(|item| self.parse_task_item(item))
                .collect();
            Ok(tasks)
        } else {
            Ok(Vec::new())
        }
    }
}

// =============================================================================
// GRAPHQL REQUEST EXECUTION
// =============================================================================

impl MondayClient {
    async fn execute_graphql_request(&self, query: &Value) -> Result<reqwest::Response> {
        let response = self
            .client
            .post("https://api.monday.com/v2")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .header("API-Version", "2024-10")
            .json(query)
            .send()
            .await?;

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
        } else {
            // Parse global search results
            tasks.extend(self.parse_global_search_results(&result));
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
                                    tasks.push(task);
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
                            tasks.push(task);
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
            format!("https://{}.monday.com/boards/{}/pulses/{}", slug, board_id, item_id)
        } else {
            format!("https://monday.com/boards/{}/pulses/{}", board_id, item_id)
        }
    }
}

// =============================================================================
// CONNECTION TESTING AND VALIDATION
// =============================================================================

impl MondayClient {
    pub async fn test_connection(&self) -> Result<String> {
        let query = json!({
            "query": "query { me { name email } }"
        });

        let response = self.execute_graphql_request(&query).await?;
        let result: Value = response.json().await?;

        self.parse_connection_test_result(result)
    }

    fn parse_connection_test_result(&self, result: Value) -> Result<String> {
        if let Some(me) = result["data"]["me"].as_object() {
            let name = me["name"].as_str().unwrap_or("Unknown");
            let email = me["email"].as_str().unwrap_or("Unknown");
            Ok(format!("{} ({})", name, email))
        } else {
            Err(anyhow::anyhow!("Failed to get user information"))
        }
    }
} 