//! Cache storage implementations
//! 
//! Various cache implementations including Redis, in-memory, and file-based caches.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Simple in-memory cache with TTL support
#[derive(Clone)]
pub struct MemoryCache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K, V> MemoryCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }
    
    pub fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }
    
    pub fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let expires_at = Instant::now() + ttl;
        let entry = CacheEntry { value, expires_at };
        
        let mut data = self.data.write().unwrap();
        data.insert(key, entry);
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().unwrap();
        
        if let Some(entry) = data.get(key) {
            if Instant::now() < entry.expires_at {
                Some(entry.value.clone())
            } else {
                // Entry has expired, remove it
                data.remove(key);
                None
            }
        } else {
            None
        }
    }
    
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().unwrap();
        data.remove(key).map(|entry| entry.value)
    }
    
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }
    
    pub fn cleanup_expired(&self) -> usize {
        let mut data = self.data.write().unwrap();
        let now = Instant::now();
        let old_size = data.len();
        
        data.retain(|_, entry| now < entry.expires_at);
        
        old_size - data.len()
    }
    
    pub fn size(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }
}

impl<K, V> Default for MemoryCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(3600)) // 1 hour default TTL
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub default_ttl: Duration,
    pub max_size: Option<usize>,
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_size: Some(10000), // 10k entries max
            cleanup_interval: Duration::from_secs(300), // cleanup every 5 minutes
        }
    }
} 