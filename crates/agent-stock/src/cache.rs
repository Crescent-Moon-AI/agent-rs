//! Caching layer for stock data to reduce API calls

use cached::{Cached, TimedCache};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Cache key for stock data requests
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Stock symbol
    pub symbol: String,
    /// API endpoint or operation type
    pub endpoint: String,
    /// Additional parameters as JSON string
    pub params: String,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(symbol: impl Into<String>, endpoint: impl Into<String>, params: impl Serialize) -> Self {
        Self {
            symbol: symbol.into(),
            endpoint: endpoint.into(),
            params: serde_json::to_string(&params).unwrap_or_default(),
        }
    }
}

/// Thread-safe cache for stock data
pub struct StockCache {
    cache: Arc<RwLock<TimedCache<CacheKey, serde_json::Value>>>,
}

impl StockCache {
    /// Create a new cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(TimedCache::with_lifespan(ttl))),
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &CacheKey) -> Option<serde_json::Value> {
        let mut cache = self.cache.write().await;
        cache.cache_get(key).cloned()
    }

    /// Insert a value into the cache
    pub async fn insert(&self, key: CacheKey, value: serde_json::Value) {
        let mut cache = self.cache.write().await;
        let _ = cache.cache_set(key, value);
    }

    /// Get or fetch a value using the provided fetcher function
    ///
    /// If the value exists in cache, it's returned immediately.
    /// Otherwise, the fetcher function is called and the result is cached.
    pub async fn get_or_fetch<F, Fut, E>(
        &self,
        key: CacheKey,
        fetcher: F,
    ) -> Result<serde_json::Value, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value, E>>,
    {
        // Try to get from cache first
        if let Some(value) = self.get(&key).await {
            tracing::debug!("Cache hit for key: {:?}", key);
            return Ok(value);
        }

        tracing::debug!("Cache miss for key: {:?}", key);

        // Fetch the value
        let value = fetcher().await?;

        // Store in cache
        self.insert(key, value.clone()).await;

        Ok(value)
    }

    /// Invalidate a specific cache entry
    pub async fn invalidate(&self, key: &CacheKey) {
        let mut cache = self.cache.write().await;
        let _ = cache.cache_remove(key);
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.cache_clear();
    }

    /// Get the number of cached entries
    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.cache_size()
    }

    /// Check if the cache is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

impl Clone for StockCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Multi-tiered cache system for different data types
pub struct CacheManager {
    /// Cache for real-time data (quotes, prices) with short TTL
    pub realtime: StockCache,
    /// Cache for fundamental data with longer TTL
    pub fundamental: StockCache,
    /// Cache for news data with medium TTL
    pub news: StockCache,
}

impl CacheManager {
    /// Create a new cache manager with specified TTLs
    pub fn new(
        realtime_ttl: Duration,
        fundamental_ttl: Duration,
        news_ttl: Duration,
    ) -> Self {
        Self {
            realtime: StockCache::new(realtime_ttl),
            fundamental: StockCache::new(fundamental_ttl),
            news: StockCache::new(news_ttl),
        }
    }

    /// Create a default cache manager
    pub fn default_config() -> Self {
        Self::new(
            Duration::from_secs(60),    // 1 minute for realtime
            Duration::from_secs(3600),  // 1 hour for fundamental
            Duration::from_secs(300),   // 5 minutes for news
        )
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.realtime.clear().await;
        self.fundamental.clear().await;
        self.news.clear().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_creation() {
        let key = CacheKey::new("AAPL", "quote", serde_json::json!({"foo": "bar"}));
        assert_eq!(key.symbol, "AAPL");
        assert_eq!(key.endpoint, "quote");
        assert!(key.params.contains("foo"));
    }

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = StockCache::new(Duration::from_secs(60));
        let key = CacheKey::new("AAPL", "quote", serde_json::json!({}));
        let value = serde_json::json!({"price": 150.0});

        cache.insert(key.clone(), value.clone()).await;

        let retrieved = cache.get(&key).await;
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_cache_get_or_fetch() {
        let cache = StockCache::new(Duration::from_secs(60));
        let key = CacheKey::new("AAPL", "quote", serde_json::json!({}));
        let value = serde_json::json!({"price": 150.0});

        let mut call_count = 0;
        let fetcher = || {
            call_count += 1;
            async { Ok::<_, String>(value.clone()) }
        };

        // First call should execute fetcher
        let result = cache.get_or_fetch(key.clone(), fetcher).await.unwrap();
        assert_eq!(result, value);
        assert_eq!(call_count, 1);

        // Second call should use cache
        let result = cache.get_or_fetch(key.clone(), || async {
            call_count += 1;
            Ok::<_, String>(value.clone())
        }).await.unwrap();
        assert_eq!(result, value);
        assert_eq!(call_count, 1); // Should not have incremented
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = StockCache::new(Duration::from_secs(60));
        let key = CacheKey::new("AAPL", "quote", serde_json::json!({}));
        let value = serde_json::json!({"price": 150.0});

        cache.insert(key.clone(), value).await;
        assert!(cache.get(&key).await.is_some());

        cache.invalidate(&key).await;
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = StockCache::new(Duration::from_secs(60));

        for i in 0..5 {
            let key = CacheKey::new(format!("STOCK{}", i), "quote", serde_json::json!({}));
            cache.insert(key, serde_json::json!({"price": i})).await;
        }

        assert_eq!(cache.len().await, 5);

        cache.clear().await;
        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let manager = CacheManager::default_config();

        let key = CacheKey::new("AAPL", "quote", serde_json::json!({}));
        let value = serde_json::json!({"price": 150.0});

        manager.realtime.insert(key.clone(), value.clone()).await;
        manager.fundamental.insert(key.clone(), value.clone()).await;
        manager.news.insert(key.clone(), value.clone()).await;

        assert_eq!(manager.realtime.len().await, 1);
        assert_eq!(manager.fundamental.len().await, 1);
        assert_eq!(manager.news.len().await, 1);

        manager.clear_all().await;

        assert!(manager.realtime.is_empty().await);
        assert!(manager.fundamental.is_empty().await);
        assert!(manager.news.is_empty().await);
    }
}
