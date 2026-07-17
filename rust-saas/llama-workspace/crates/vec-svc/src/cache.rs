//! 
//!
//! 

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::hash::Hash;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

pub struct Cache<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    data: RwLock<HashMap<K, CacheEntry<V>>>,
    ttl: Duration,
    max_size: usize,
}

impl<K, V> Cache<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    pub fn new(ttl_seconds: u64, max_size: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
            max_size,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let data = self.data.read().await;
        if let Some(entry) = data.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: K, value: V) {
        let mut data = self.data.write().await;
        
        if data.len() >= self.max_size {
            self.evict_expired(&mut data);
            if data.len() >= self.max_size {
                self.evict_lru(&mut data);
            }
        }

        data.insert(key, CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }

    pub async fn invalidate(&self, key: &K) {
        let mut data = self.data.write().await;
        data.remove(key);
    }

    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }

    pub async fn size(&self) -> usize {
        let data = self.data.read().await;
        data.len()
    }

    fn evict_expired(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        let now = Instant::now();
        data.retain(|_, v| v.expires_at > now);
    }

    fn evict_lru(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        if data.is_empty() {
            return;
        }

        let oldest_key = data
            .iter()
            .min_by_key(|(_, v)| v.expires_at)
            .map(|(k, _)| k as *const K);

        if let Some(key_ptr) = oldest_key {
            unsafe {
                let key = &*key_ptr;
                let key_clone: K = std::mem::transmute_copy(key);
                data.remove(&key_clone);
                std::mem::forget(key_clone);
            }
        }
    }
}

pub struct SearchCache {
    pub query_cache: Cache<SearchCacheKey, Vec<crate::app_state::EnhancedSearchResult>>,
    pub suggestion_cache: Cache<String, crate::search_suggestions::SuggestionResult>,
    pub document_cache: Cache<i64, crate::model::Document>,
    stats: RwLock<CacheStats>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchCacheKey {
    pub query: String,
    pub project_id: Option<i64>,
    pub top_k: usize,
    pub min_score: f32,
}

impl Hash for SearchCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.project_id.hash(state);
        self.top_k.hash(state);
        self.min_score.to_bits().hash(state);
    }
}

impl Eq for SearchCacheKey {}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl SearchCache {
    pub fn new() -> Self {
        Self {
            query_cache: Cache::new(300, 1000),
            suggestion_cache: Cache::new(600, 500),
            document_cache: Cache::new(600, 200),
            stats: RwLock::new(CacheStats::default()),
        }
    }

    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    pub async fn record_hit(&self) {
        let mut stats = self.stats.write().await;
        stats.hits += 1;
    }

    pub async fn record_miss(&self) {
        let mut stats = self.stats.write().await;
        stats.misses += 1;
    }

    pub async fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        let total = stats.hits + stats.misses;
        if total == 0 {
            0.0
        } else {
            stats.hits as f64 / total as f64
        }
    }
}

impl Default for SearchCache {
    fn default() -> Self {
        Self::new()
    }
}
