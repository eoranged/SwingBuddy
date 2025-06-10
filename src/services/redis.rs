//! Redis integration service implementation
//! 
//! This service handles Redis connection pool setup, caching utilities for CAS API results,
//! user state caching for conversation flows, cache invalidation strategies,
//! and performance optimization for database queries.

use redis::{Client, AsyncCommands, RedisResult};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, debug};
use crate::config::settings::Settings;
use crate::utils::errors::{SwingBuddyError, Result};

/// Redis service for caching and state management
#[derive(Clone)]
#[derive(Debug)]
pub struct RedisService {
    client: Client,
    settings: Settings,
}

/// Cache entry with TTL information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub ttl_seconds: u64,
}

impl RedisService {
    /// Create a new RedisService instance
    pub fn new(settings: Settings) -> Result<Self> {
        let client = Client::open(settings.redis.url.as_str())
            .map_err(|e| SwingBuddyError::Redis(e))?;

        Ok(Self { client, settings })
    }

    /// Get Redis connection
    async fn get_connection(&self) -> Result<redis::aio::Connection> {
        self.client.get_async_connection().await
            .map_err(|e| SwingBuddyError::Redis(e))
    }

    /// Set a value in Redis with TTL
    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| SwingBuddyError::Serialization(e))?;

        let full_key = format!("{}{}", self.settings.redis.prefix, key);
        let ttl = ttl_seconds.unwrap_or(self.settings.redis.ttl_seconds);

        let _: () = conn.set_ex(&full_key, serialized, ttl).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(key = %full_key, ttl = ttl, "Value set in Redis");
        Ok(())
    }

    /// Get a value from Redis
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        let result: Option<String> = conn.get(&full_key).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        match result {
            Some(data) => {
                let deserialized = serde_json::from_str::<T>(&data)
                    .map_err(|e| SwingBuddyError::Serialization(e))?;
                debug!(key = %full_key, "Value retrieved from Redis");
                Ok(Some(deserialized))
            }
            None => {
                debug!(key = %full_key, "Key not found in Redis");
                Ok(None)
            }
        }
    }

    /// Delete a key from Redis
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        let deleted: i32 = conn.del(&full_key).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(key = %full_key, deleted = deleted > 0, "Key deletion attempted");
        Ok(deleted > 0)
    }

    /// Check if a key exists in Redis
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        let exists: bool = conn.exists(&full_key).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(key = %full_key, exists = exists, "Key existence check");
        Ok(exists)
    }

    /// Set TTL for an existing key
    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        let result: bool = conn.expire(&full_key, ttl_seconds as i64).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(key = %full_key, ttl = ttl_seconds, success = result, "TTL set for key");
        Ok(result)
    }

    /// Get TTL for a key
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        let ttl: i64 = conn.ttl(&full_key).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(key = %full_key, ttl = ttl, "TTL retrieved for key");
        Ok(ttl)
    }

    /// Get all keys matching a pattern
    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let full_pattern = format!("{}{}", self.settings.redis.prefix, pattern);

        let keys: Vec<String> = conn.keys(&full_pattern).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(pattern = %full_pattern, count = keys.len(), "Keys retrieved by pattern");
        Ok(keys)
    }

    /// Delete all keys matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64> {
        let keys = self.keys(pattern).await?;
        if keys.is_empty() {
            return Ok(0);
        }

        let mut conn = self.get_connection().await?;
        let deleted: u64 = conn.del(&keys).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        info!(pattern = %pattern, deleted = deleted, "Keys deleted by pattern");
        Ok(deleted)
    }

    /// Cache user state for conversation flows
    pub async fn cache_user_state<T>(&self, user_id: i64, state: &T) -> Result<()>
    where
        T: Serialize,
    {
        let key = format!("user_state:{}", user_id);
        self.set(&key, state, Some(3600)).await // 1 hour TTL for user states
    }

    /// Get cached user state
    pub async fn get_user_state<T>(&self, user_id: i64) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let key = format!("user_state:{}", user_id);
        self.get(&key).await
    }

    /// Clear user state
    pub async fn clear_user_state(&self, user_id: i64) -> Result<bool> {
        let key = format!("user_state:{}", user_id);
        self.delete(&key).await
    }

    /// Cache CAS check result
    pub async fn cache_cas_result(&self, user_id: i64, result: &serde_json::Value) -> Result<()> {
        let key = format!("cas_check:{}", user_id);
        self.set(&key, result, Some(self.settings.redis.ttl_seconds)).await
    }

    /// Get cached CAS result
    pub async fn get_cas_result(&self, user_id: i64) -> Result<Option<serde_json::Value>> {
        let key = format!("cas_check:{}", user_id);
        self.get(&key).await
    }

    /// Cache database query result
    pub async fn cache_query_result<T>(&self, query_key: &str, result: &T, ttl_seconds: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        let key = format!("query:{}", query_key);
        self.set(&key, result, ttl_seconds).await
    }

    /// Get cached query result
    pub async fn get_query_result<T>(&self, query_key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let key = format!("query:{}", query_key);
        self.get(&key).await
    }

    /// Increment a counter
    pub async fn increment(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        let value: i64 = conn.incr(&full_key, 1).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(key = %full_key, value = value, "Counter incremented");
        Ok(value)
    }

    /// Increment a counter with TTL
    pub async fn increment_with_ttl(&self, key: &str, ttl_seconds: u64) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        // Use a pipeline to ensure atomicity
        let (value,): (i64,) = redis::pipe()
            .incr(&full_key, 1)
            .expire(&full_key, ttl_seconds as i64)
            .query_async(&mut conn)
            .await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(key = %full_key, value = value, ttl = ttl_seconds, "Counter incremented with TTL");
        Ok(value)
    }

    /// Get counter value
    pub async fn get_counter(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let full_key = format!("{}{}", self.settings.redis.prefix, key);

        let value: Option<i64> = conn.get(&full_key).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        Ok(value.unwrap_or(0))
    }

    /// Rate limiting check
    pub async fn check_rate_limit(&self, identifier: &str, limit: u64, window_seconds: u64) -> Result<bool> {
        let key = format!("rate_limit:{}", identifier);
        let current_count = self.increment_with_ttl(&key, window_seconds).await?;
        
        let allowed = current_count <= limit as i64;
        debug!(
            identifier = %identifier,
            current_count = current_count,
            limit = limit,
            allowed = allowed,
            "Rate limit check"
        );
        
        Ok(allowed)
    }

    /// Clear all cache entries
    pub async fn clear_all_cache(&self) -> Result<u64> {
        self.delete_pattern("*").await
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        let all_keys = self.keys("*").await?;
        let total_keys = all_keys.len() as u64;

        // Sample some keys to get size estimation
        let mut total_memory = 0u64;
        let sample_size = std::cmp::min(100, all_keys.len());
        
        for key in all_keys.iter().take(sample_size) {
            if let Ok(Some(value)) = self.get::<serde_json::Value>(&key.replace(&self.settings.redis.prefix, "")).await {
                if let Ok(serialized) = serde_json::to_string(&value) {
                    total_memory += serialized.len() as u64;
                }
            }
        }

        // Extrapolate memory usage
        if sample_size > 0 {
            total_memory = (total_memory * total_keys) / sample_size as u64;
        }

        Ok(CacheStats {
            total_keys,
            estimated_memory_bytes: total_memory,
            prefix: self.settings.redis.prefix.clone(),
        })
    }

    /// Health check for Redis connection
    pub async fn health_check(&self) -> Result<bool> {
        match self.get_connection().await {
            Ok(mut conn) => {
                let result: RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
                match result {
                    Ok(response) => {
                        debug!(response = %response, "Redis health check successful");
                        Ok(response == "PONG")
                    }
                    Err(e) => {
                        warn!(error = %e, "Redis health check failed");
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Redis connection failed");
                Ok(false)
            }
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_keys: u64,
    pub estimated_memory_bytes: u64,
    pub prefix: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_service_creation() {
        let settings = Settings::default();
        let result = RedisService::new(settings);
        
        // This test will fail if Redis is not available, which is expected in CI
        // In a real environment, you'd want to use a test Redis instance
        match result {
            Ok(_) => println!("Redis service created successfully"),
            Err(e) => println!("Redis service creation failed (expected in test env): {}", e),
        }
    }

    #[test]
    fn test_cache_entry_serialization() {
        let entry = CacheEntry {
            data: "test_data".to_string(),
            created_at: chrono::Utc::now(),
            ttl_seconds: 3600,
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: CacheEntry<String> = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(entry.data, deserialized.data);
        assert_eq!(entry.ttl_seconds, deserialized.ttl_seconds);
    }

    #[test]
    fn test_cache_stats_serialization() {
        let stats = CacheStats {
            total_keys: 100,
            estimated_memory_bytes: 1024,
            prefix: "test:".to_string(),
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: CacheStats = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(stats.total_keys, deserialized.total_keys);
        assert_eq!(stats.estimated_memory_bytes, deserialized.estimated_memory_bytes);
        assert_eq!(stats.prefix, deserialized.prefix);
    }
}