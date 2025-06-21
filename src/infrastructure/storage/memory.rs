//! In-memory storage implementations for testing and development
//! 
//! These implementations store data in memory and are useful for testing,
//! development, and scenarios where persistence is not required.

#[cfg(feature = "new-domains")]
use crate::domains::{
    ai::{
        entities::{AnalysisResult, ConversationSession, ConversationMessage},
        value_objects::{PromptTemplate, AnalysisType, CacheConfig},
        repository::{
            PromptTemplatePort, AnalysisResultPort, ConversationPort,
            AiCachePort, TemplateUsageStats, AnalysisStatistics, CacheStatistics
        },
        errors::AiDomainError,
    },
    tasks::{
        entities::{Task, TaskTemplate, TaskAutomationRule},
        repository::{TaskRepositoryPort},
        errors::TaskManagementDomainError,
    },
};

#[cfg(feature = "new-domains")]
use async_trait::async_trait;
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc, Duration};
#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use std::sync::{Arc, RwLock};

/// In-memory storage for prompt templates
#[cfg(feature = "new-domains")]
#[derive(Default)]
pub struct InMemoryPromptTemplateStorage {
    templates: Arc<RwLock<HashMap<String, PromptTemplate>>>,
    usage_stats: Arc<RwLock<HashMap<String, TemplateUsageStats>>>,
}

#[cfg(feature = "new-domains")]
impl InMemoryPromptTemplateStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl PromptTemplatePort for InMemoryPromptTemplateStorage {
    async fn save_template(&self, template: PromptTemplate) -> Result<(), AiDomainError> {
        let mut templates = self.templates.write().unwrap();
        templates.insert(template.name().to_string(), template);
        Ok(())
    }
    
    async fn get_template(&self, name: &str) -> Result<Option<PromptTemplate>, AiDomainError> {
        let templates = self.templates.read().unwrap();
        Ok(templates.get(name).cloned())
    }
    
    async fn list_templates(&self) -> Result<Vec<PromptTemplate>, AiDomainError> {
        let templates = self.templates.read().unwrap();
        Ok(templates.values().cloned().collect())
    }
    
    async fn list_templates_by_type(&self, analysis_type: AnalysisType) -> Result<Vec<PromptTemplate>, AiDomainError> {
        let templates = self.templates.read().unwrap();
        let filtered: Vec<PromptTemplate> = templates.values()
            .filter(|template| {
                // Check if template name contains the analysis type name
                template.name().contains(&analysis_type.display_name().to_lowercase())
            })
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn update_template(&self, template: PromptTemplate) -> Result<(), AiDomainError> {
        let mut templates = self.templates.write().unwrap();
        if templates.contains_key(template.name()) {
            templates.insert(template.name().to_string(), template);
            Ok(())
        } else {
            Err(AiDomainError::InvalidPromptTemplate {
                template_name: format!("Template '{}' not found", template.name()),
            })
        }
    }
    
    async fn delete_template(&self, name: &str) -> Result<(), AiDomainError> {
        let mut templates = self.templates.write().unwrap();
        if templates.remove(name).is_some() {
            Ok(())
        } else {
            Err(AiDomainError::InvalidPromptTemplate {
                template_name: format!("Template '{}' not found", name),
            })
        }
    }
    
    async fn validate_template(&self, template: &PromptTemplate) -> Result<(), AiDomainError> {
        template.validate()
    }
    
    async fn get_template_usage_stats(&self, name: &str) -> Result<TemplateUsageStats, AiDomainError> {
        let stats = self.usage_stats.read().unwrap();
        stats.get(name).cloned().ok_or_else(|| AiDomainError::InvalidPromptTemplate {
            template_name: format!("No usage stats found for template '{}'", name),
        })
    }
}

/// In-memory storage for analysis results
#[cfg(feature = "new-domains")]
#[derive(Default)]
pub struct InMemoryAnalysisResultStorage {
    results: Arc<RwLock<HashMap<String, AnalysisResult>>>,
}

#[cfg(feature = "new-domains")]
impl InMemoryAnalysisResultStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl AnalysisResultPort for InMemoryAnalysisResultStorage {
    async fn save_result(&self, result: AnalysisResult) -> Result<(), AiDomainError> {
        let mut results = self.results.write().unwrap();
        results.insert(result.id.clone(), result);
        Ok(())
    }
    
    async fn get_result(&self, id: &str) -> Result<Option<AnalysisResult>, AiDomainError> {
        let results = self.results.read().unwrap();
        Ok(results.get(id).cloned())
    }
    
    async fn list_results_by_type(&self, analysis_type: AnalysisType, limit: usize, offset: usize) -> Result<Vec<AnalysisResult>, AiDomainError> {
        let results = self.results.read().unwrap();
        let filtered: Vec<AnalysisResult> = results.values()
            .filter(|result| result.analysis_type == analysis_type)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn list_recent_results(&self, limit: usize) -> Result<Vec<AnalysisResult>, AiDomainError> {
        let results = self.results.read().unwrap();
        let mut sorted_results: Vec<AnalysisResult> = results.values().cloned().collect();
        sorted_results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted_results.truncate(limit);
        Ok(sorted_results)
    }
    
    async fn search_results(&self, query: &str, limit: usize) -> Result<Vec<AnalysisResult>, AiDomainError> {
        let results = self.results.read().unwrap();
        let filtered: Vec<AnalysisResult> = results.values()
            .filter(|result| {
                result.result_data.as_ref()
                    .map(|data| data.contains(query))
                    .unwrap_or(false)
            })
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn update_result(&self, result: AnalysisResult) -> Result<(), AiDomainError> {
        let mut results = self.results.write().unwrap();
        results.insert(result.id.clone(), result);
        Ok(())
    }
    
    async fn cleanup_old_results(&self, older_than: DateTime<Utc>) -> Result<usize, AiDomainError> {
        let mut results = self.results.write().unwrap();
        let old_count = results.len();
        results.retain(|_, result| result.created_at > older_than);
        Ok(old_count - results.len())
    }
    
    async fn get_analysis_statistics(&self, period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Result<AnalysisStatistics, AiDomainError> {
        let results = self.results.read().unwrap();
        let period_results: Vec<&AnalysisResult> = results.values()
            .filter(|result| result.created_at >= period_start && result.created_at <= period_end)
            .collect();
        
        let total_analyses = period_results.len() as u64;
        let successful_analyses = period_results.iter()
            .filter(|result| result.is_successful())
            .count() as u64;
        let failed_analyses = total_analyses - successful_analyses;
        
        let average_confidence = if !period_results.is_empty() {
            period_results.iter()
                .filter_map(|result| result.confidence_score)
                .sum::<f32>() / period_results.len() as f32
        } else {
            0.0
        };
        
        let total_cost = period_results.iter()
            .filter_map(|result| result.metadata.cost_estimate)
            .sum();
        
        let total_tokens = period_results.iter()
            .map(|result| result.metadata.tokens_consumed as u64)
            .sum();
        
        let analyses_by_type: HashMap<AnalysisType, u64> = period_results.iter()
            .fold(HashMap::new(), |mut acc, result| {
                *acc.entry(result.analysis_type.clone()).or_insert(0) += 1;
                acc
            });
        
        Ok(AnalysisStatistics {
            total_analyses,
            successful_analyses,
            failed_analyses,
            average_confidence,
            total_cost,
            total_tokens,
            average_response_time: Duration::seconds(1), // Placeholder
            analyses_by_type,
        })
    }
}

/// In-memory cache implementation
#[cfg(feature = "new-domains")]
#[derive(Default)]
pub struct InMemoryAiCache {
    cache: Arc<RwLock<HashMap<String, (AnalysisResult, DateTime<Utc>)>>>,
    config: CacheConfig,
}

#[cfg(feature = "new-domains")]
impl InMemoryAiCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(CacheConfig::default())
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl AiCachePort for InMemoryAiCache {
    async fn get_cached_result(&self, cache_key: &str) -> Result<Option<AnalysisResult>, AiDomainError> {
        let cache = self.cache.read().unwrap();
        if let Some((result, timestamp)) = cache.get(cache_key) {
            // Check if the cache entry is still valid
            let now = Utc::now();
            if now - *timestamp < self.config.ttl {
                Ok(Some(result.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    async fn cache_result(&self, cache_key: &str, result: AnalysisResult, _ttl: Duration) -> Result<(), AiDomainError> {
        let mut cache = self.cache.write().unwrap();
        cache.insert(cache_key.to_string(), (result, Utc::now()));
        Ok(())
    }
    
    async fn invalidate_cache(&self, pattern: &str) -> Result<usize, AiDomainError> {
        let mut cache = self.cache.write().unwrap();
        let old_size = cache.len();
        cache.retain(|key, _| !key.contains(pattern));
        Ok(old_size - cache.len())
    }
    
    async fn get_cache_statistics(&self) -> Result<CacheStatistics, AiDomainError> {
        let cache = self.cache.read().unwrap();
        let total_entries = cache.len() as u64;
        
        // Calculate cache size (rough estimate)
        let cache_size_bytes = total_entries * 1024; // Rough estimate
        
        let now = Utc::now();
        let oldest_entry_age = cache.values()
            .map(|(_, timestamp)| now - *timestamp)
            .min();
        
        Ok(CacheStatistics {
            total_entries,
            hit_ratio: 0.8, // Placeholder
            miss_ratio: 0.2, // Placeholder
            expired_entries: 0,
            cache_size_bytes,
            oldest_entry_age,
        })
    }
    
    async fn cleanup_expired_entries(&self) -> Result<usize, AiDomainError> {
        let mut cache = self.cache.write().unwrap();
        let old_size = cache.len();
        let now = Utc::now();
        
        cache.retain(|_, (_, timestamp)| now - *timestamp < self.config.ttl);
        
        Ok(old_size - cache.len())
    }
    
    fn generate_cache_key(&self, request: &crate::domains::ai::value_objects::AnalysisRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request.analysis_type.hash(&mut hasher);
        request.input_data.hash(&mut hasher);
        // Add context to hash for more specific caching
        for (key, value) in &request.context {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }
        
        format!("analysis_{:x}", hasher.finish())
    }
}

/// In-memory task storage implementation
#[cfg(feature = "new-domains")]
#[derive(Default)]
pub struct InMemoryTaskStorage {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

#[cfg(feature = "new-domains")]
impl InMemoryTaskStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "new-domains")]
#[async_trait]
impl TaskRepositoryPort for InMemoryTaskStorage {
    async fn create_task(&self, task: &Task) -> Result<(), TaskManagementDomainError> {
        let mut tasks = self.tasks.write().unwrap();
        tasks.insert(task.id.as_str().to_string(), task.clone());
        Ok(())
    }
    
    async fn get_task(&self, id: &crate::domains::tasks::value_objects::TaskId) -> Result<Option<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        Ok(tasks.get(id.as_str()).cloned())
    }
    
    async fn update_task(&self, task: &Task) -> Result<(), TaskManagementDomainError> {
        let mut tasks = self.tasks.write().unwrap();
        tasks.insert(task.id.as_str().to_string(), task.clone());
        Ok(())
    }
    
    async fn delete_task(&self, id: &crate::domains::tasks::value_objects::TaskId) -> Result<(), TaskManagementDomainError> {
        let mut tasks = self.tasks.write().unwrap();
        if tasks.remove(id.as_str()).is_some() {
            Ok(())
        } else {
            Err(TaskManagementDomainError::TaskNotFound {
                task_id: id.as_str().to_string(),
            })
        }
    }
    
    async fn get_tasks_by_system(
        &self,
        system: &crate::domains::tasks::value_objects::TaskSystem,
    ) -> Result<Vec<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| task.id.system() == system)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn get_tasks_by_status(
        &self,
        status: &crate::domains::tasks::value_objects::TaskStatus,
    ) -> Result<Vec<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| &task.status == status)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn get_tasks_by_assignee(
        &self,
        assignee: &crate::domains::tasks::value_objects::TaskAssignee,
    ) -> Result<Vec<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| task.assignee.as_ref() == Some(assignee))
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn get_tasks_by_label(
        &self,
        label: &str,
    ) -> Result<Vec<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| task.labels.iter().any(|l| l == label))
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn get_overdue_tasks(&self) -> Result<Vec<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        let now = chrono::Utc::now();
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| {
                task.due_date.map(|due| due < now).unwrap_or(false)
            })
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn get_tasks_updated_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| task.updated_at > since)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn search_tasks(
        &self,
        query: &str,
        system: Option<&crate::domains::tasks::value_objects::TaskSystem>,
    ) -> Result<Vec<Task>, TaskManagementDomainError> {
        let tasks = self.tasks.read().unwrap();
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| {
                let matches_query = task.title.contains(query) || 
                    task.description.as_ref().map(|d| d.contains(query)).unwrap_or(false);
                let matches_system = system.map(|s| task.id.system() == s).unwrap_or(true);
                matches_query && matches_system
            })
            .cloned()
            .collect();
        Ok(filtered)
    }
}

/// Stub implementations for other storage ports to enable compilation
#[cfg(feature = "new-domains")]
#[derive(Default)]
pub struct StubConversationStorage;

#[cfg(feature = "new-domains")]
#[async_trait]
impl ConversationPort for StubConversationStorage {
    async fn save_conversation(&self, _conversation: ConversationSession) -> Result<(), AiDomainError> {
        Ok(())
    }
    
    async fn get_conversation(&self, _id: &str) -> Result<Option<ConversationSession>, AiDomainError> {
        Ok(None)
    }
    
    async fn list_recent_conversations(&self, _limit: usize) -> Result<Vec<ConversationSession>, AiDomainError> {
        Ok(vec![])
    }
    
    async fn update_conversation(&self, _conversation: ConversationSession) -> Result<(), AiDomainError> {
        Ok(())
    }
    
    async fn delete_conversation(&self, _id: &str) -> Result<(), AiDomainError> {
        Ok(())
    }
    
    async fn add_message(&self, _conversation_id: &str, _message: ConversationMessage) -> Result<(), AiDomainError> {
        Ok(())
    }
    
    async fn get_messages(&self, _conversation_id: &str, _limit: usize, _offset: usize) -> Result<Vec<ConversationMessage>, AiDomainError> {
        Ok(vec![])
    }
    
    async fn search_conversations(&self, _query: &str, _limit: usize) -> Result<Vec<ConversationSession>, AiDomainError> {
        Ok(vec![])
    }
} 