use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub ttl: Option<Duration>,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
    
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

/// Cache eviction policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    TTL,  // Time To Live
    FIFO, // First In, First Out
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size: usize,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub cleanup_interval: Duration,
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            eviction_policy: EvictionPolicy::LRU,
            cleanup_interval: Duration::from_secs(60),
            enable_metrics: true,
        }
    }
}

/// Cache metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries: usize,
    pub hit_rate: f64,
    pub memory_usage: usize,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            entries: 0,
            hit_rate: 0.0,
            memory_usage: 0,
        }
    }
    
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.update_hit_rate();
    }
    
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.update_hit_rate();
    }
    
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }
    
    fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        }
    }
}

/// Generic cache implementation
pub struct Cache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    config: CacheConfig,
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(config: CacheConfig) -> Self {
        let cache = Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::new())),
        };
        
        // Start cleanup task
        cache.start_cleanup_task();
        
        cache
    }
    
    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                entries.remove(key);
                self.record_miss().await;
                return None;
            }
            
            entry.touch();
            self.record_hit().await;
            Some(entry.value.clone())
        } else {
            self.record_miss().await;
            None
        }
    }
    
    /// Put a value in the cache
    pub async fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, self.config.default_ttl).await;
    }
    
    /// Put a value in the cache with custom TTL
    pub async fn put_with_ttl(&self, key: K, value: V, ttl: Option<Duration>) {
        let mut entries = self.entries.write().await;
        
        // Check if we need to evict entries
        if entries.len() >= self.config.max_size && !entries.contains_key(&key) {
            self.evict_entries(&mut entries).await;
        }
        
        let entry = CacheEntry::new(value, ttl);
        entries.insert(key, entry);
        
        self.update_metrics().await;
    }
    
    /// Remove a value from the cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;
        entries.remove(key).map(|entry| entry.value)
    }
    
    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        self.update_metrics().await;
    }
    
    /// Check if the cache contains a key
    pub async fn contains_key(&self, key: &K) -> bool {
        let entries = self.entries.read().await;
        entries.get(key).is_some_and(|entry| !entry.is_expired())
    }
    
    /// Get the number of entries in the cache
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }
    
    /// Check if the cache is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Evict entries based on the eviction policy
    async fn evict_entries(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        let eviction_count = entries.len() - self.config.max_size + 1;
        
        match self.config.eviction_policy {
            EvictionPolicy::LRU => {
                self.evict_lru(entries, eviction_count).await;
            }
            EvictionPolicy::LFU => {
                self.evict_lfu(entries, eviction_count).await;
            }
            EvictionPolicy::TTL => {
                self.evict_ttl(entries, eviction_count).await;
            }
            EvictionPolicy::FIFO => {
                self.evict_fifo(entries, eviction_count).await;
            }
        }
    }
    
    /// Evict least recently used entries
    async fn evict_lru(&self, entries: &mut HashMap<K, CacheEntry<V>>, count: usize) {
        let mut sorted_entries: Vec<_> = entries.iter().map(|(k, v)| (k.clone(), v.last_accessed)).collect();
        sorted_entries.sort_by_key(|(_, last_accessed)| *last_accessed);
        
        for (key, _) in sorted_entries.iter().take(count) {
            entries.remove(key);
            self.record_eviction().await;
        }
    }
    
    /// Evict least frequently used entries
    async fn evict_lfu(&self, entries: &mut HashMap<K, CacheEntry<V>>, count: usize) {
        let mut sorted_entries: Vec<_> = entries.iter().map(|(k, v)| (k.clone(), v.access_count)).collect();
        sorted_entries.sort_by_key(|(_, access_count)| *access_count);
        
        for (key, _) in sorted_entries.iter().take(count) {
            entries.remove(key);
            self.record_eviction().await;
        }
    }
    
    /// Evict entries based on TTL
    async fn evict_ttl(&self, entries: &mut HashMap<K, CacheEntry<V>>, count: usize) {
        let mut expired_keys = Vec::new();
        
        for (key, entry) in entries.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }
        
        for key in expired_keys.iter().take(count) {
            entries.remove(key);
            self.record_eviction().await;
        }
    }
    
    /// Evict first in, first out entries
    async fn evict_fifo(&self, entries: &mut HashMap<K, CacheEntry<V>>, count: usize) {
        let mut sorted_entries: Vec<_> = entries.iter().map(|(k, v)| (k.clone(), v.created_at)).collect();
        sorted_entries.sort_by_key(|(_, created_at)| *created_at);
        
        for (key, _) in sorted_entries.iter().take(count) {
            entries.remove(key);
            self.record_eviction().await;
        }
    }
    
    /// Start the cleanup task
    fn start_cleanup_task(&self) {
        let entries = self.entries.clone();
        let cleanup_interval = self.config.cleanup_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let mut entries_guard = entries.write().await;
                let initial_count = entries_guard.len();
                
                // Remove expired entries
                entries_guard.retain(|_, entry| !entry.is_expired());
                
                let removed_count = initial_count - entries_guard.len();
                if removed_count > 0 {
                    debug!("Cleaned up {} expired cache entries", removed_count);
                }
            }
        });
    }
    
    /// Record a cache hit
    async fn record_hit(&self) {
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.record_hit();
        }
    }
    
    /// Record a cache miss
    async fn record_miss(&self) {
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.record_miss();
        }
    }
    
    /// Record an eviction
    async fn record_eviction(&self) {
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.record_eviction();
        }
    }
    
    /// Update cache metrics
    async fn update_metrics(&self) {
        if self.config.enable_metrics {
            let entries = self.entries.read().await;
            let mut metrics = self.metrics.write().await;
            metrics.entries = entries.len();
            // Note: memory_usage would need to be calculated based on actual memory usage
        }
    }
}

/// Specialized caches for common use cases
pub type StringCache = Cache<String, String>;
pub type JsonCache = Cache<String, serde_json::Value>;
pub type BinaryCache = Cache<String, Vec<u8>>;

/// Cache manager for coordinating multiple caches
pub struct CacheManager {
    caches: Arc<RwLock<HashMap<String, Box<dyn CacheInterface + Send + Sync>>>>,
}

pub trait CacheInterface {
    fn get_name(&self) -> &str;
    fn get_metrics(&self) -> CacheMetrics;
    fn clear(&self);
}

impl<K, V> CacheInterface for Arc<Cache<K, V>>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get_name(&self) -> &str {
        "generic"
    }
    
    fn get_metrics(&self) -> CacheMetrics {
        // This would need to be async in a real implementation
        CacheMetrics::new()
    }
    
    fn clear(&self) {
        // This would need to be async in a real implementation
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new cache
    pub async fn create_cache<K, V>(
        &self,
        name: String,
        config: CacheConfig,
    ) -> Arc<Cache<K, V>>
    where
        K: Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let cache = Arc::new(Cache::new(config));
        let mut caches = self.caches.write().await;
        // Note: This is simplified - in practice, you'd need to handle the trait object differently
        caches.insert(name, Box::new(cache.clone()));
        cache
    }
    
    /// Get cache metrics for all caches
    pub async fn get_all_metrics(&self) -> HashMap<String, CacheMetrics> {
        let caches = self.caches.read().await;
        let mut metrics = HashMap::new();
        
        for (name, cache) in caches.iter() {
            metrics.insert(name.clone(), cache.get_metrics());
        }
        
        metrics
    }
    
    /// Clear all caches
    pub async fn clear_all(&self) {
        let caches = self.caches.read().await;
        for cache in caches.values() {
            cache.clear();
        }
    }
}

/// Cache decorator for adding additional functionality
pub struct CachedFunction<F, K, V> {
    function: F,
    cache: Arc<Cache<K, V>>,
    key_generator: Box<dyn Fn(&K) -> K + Send + Sync>,
}

impl<F, K, V> CachedFunction<F, K, V>
where
    F: Fn(K) -> std::pin::Pin<Box<dyn std::future::Future<Output = V> + Send>> + Send + Sync,
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(
        function: F,
        cache: Arc<Cache<K, V>>,
        key_generator: Box<dyn Fn(&K) -> K + Send + Sync>,
    ) -> Self {
        Self {
            function,
            cache,
            key_generator,
        }
    }
    
    pub async fn call(&self, input: K) -> V {
        let cache_key = (self.key_generator)(&input);
        
        // Try to get from cache first
        if let Some(cached_value) = self.cache.get(&cache_key).await {
            return cached_value;
        }
        
        // If not in cache, compute the value
        let value = (self.function)(input).await;
        
        // Store in cache
        self.cache.put(cache_key, value.clone()).await;
        
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig::default();
        let cache: Cache<String, String> = Cache::new(config);
        
        // Test put and get
        cache.put("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        
        // Test miss
        assert_eq!(cache.get(&"key2".to_string()).await, None);
        
        // Test remove
        cache.remove(&"key1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }
    
    #[tokio::test]
    async fn test_cache_ttl() {
        let config = CacheConfig {
            default_ttl: Some(Duration::from_millis(100)),
            ..Default::default()
        };
        let cache: Cache<String, String> = Cache::new(config);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }
    
    #[tokio::test]
    async fn test_cache_metrics() {
        let config = CacheConfig {
            enable_metrics: true,
            ..Default::default()
        };
        let cache: Cache<String, String> = Cache::new(config);
        
        // Generate some hits and misses
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.get(&"key1".to_string()).await; // hit
        cache.get(&"key2".to_string()).await; // miss
        
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate, 0.5);
    }
}
