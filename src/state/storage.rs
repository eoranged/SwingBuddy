//! State storage implementation
//! 
//! This module handles persistence of conversation state using Redis,
//! including serialization, deserialization, expiration, and cleanup.

use std::time::Duration;
use redis::{AsyncCommands, RedisResult};
use serde_json;
use tracing::{debug, warn, error, info};
use crate::utils::errors::{SwingBuddyError, Result};
use crate::config::RedisConfig;
use super::context::ConversationContext;

/// Redis-based state storage manager
#[derive(Clone)]
pub struct StateStorage {
    /// Redis connection manager
    connection_manager: redis::aio::ConnectionManager,
    /// Redis configuration
    config: RedisConfig,
}

impl StateStorage {
    /// Create a new state storage instance
    pub async fn new(config: RedisConfig) -> Result<Self> {
        let client = redis::Client::open(config.url.as_str())?;
        let connection_manager = redis::aio::ConnectionManager::new(client).await?;
        
        Ok(Self {
            connection_manager,
            config,
        })
    }

    /// Save conversation context to Redis
    pub async fn save_context(&self, context: &ConversationContext) -> Result<()> {
        let key = self.get_context_key(context.user_id);
        debug!(user_id = context.user_id, key = %key, scenario = ?context.scenario,
               step = ?context.step, "Saving context to Redis");
        
        let serialized = match serde_json::to_string(context) {
            Ok(data) => {
                debug!(user_id = context.user_id, data_length = data.len(), "Context serialized successfully");
                data
            },
            Err(e) => {
                error!(user_id = context.user_id, error = %e, "Failed to serialize context");
                return Err(e.into());
            }
        };
        
        let mut conn = self.connection_manager.clone();
        
        // Set the context with TTL
        let ttl_seconds = if let Some(expires_at) = context.expires_at {
            let now = chrono::Utc::now();
            let duration = expires_at - now;
            std::cmp::max(duration.num_seconds(), 60) as u64 // Minimum 60 seconds
        } else {
            self.config.ttl_seconds
        };

        match conn.set_ex::<_, _, ()>(&key, serialized, ttl_seconds).await {
            Ok(_) => {
                debug!(user_id = context.user_id, ttl_seconds = ttl_seconds, "Context saved to Redis successfully");
                Ok(())
            },
            Err(e) => {
                error!(user_id = context.user_id, error = %e, "Failed to save context to Redis");
                Err(e.into())
            }
        }
    }

    /// Load conversation context from Redis
    pub async fn load_context(&self, user_id: i64) -> Result<Option<ConversationContext>> {
        let key = self.get_context_key(user_id);
        debug!(user_id = user_id, key = %key, "Loading context from Redis");
        
        let mut conn = self.connection_manager.clone();
        
        let serialized: Option<String> = match conn.get::<&str, Option<String>>(&key).await {
            Ok(data) => {
                debug!(user_id = user_id, has_data = data.is_some(), "Redis GET result");
                data
            },
            Err(e) => {
                error!(user_id = user_id, error = %e, "Failed to get context from Redis");
                return Err(e.into());
            }
        };
        
        match serialized {
            Some(data) => {
                debug!(user_id = user_id, data_length = data.len(), "Deserializing context data");
                let context: ConversationContext = match serde_json::from_str::<ConversationContext>(&data) {
                    Ok(ctx) => {
                        debug!(user_id = user_id, scenario = ?ctx.scenario, step = ?ctx.step,
                               "Context deserialized successfully");
                        ctx
                    },
                    Err(e) => {
                        error!(user_id = user_id, error = %e, "Failed to deserialize context");
                        return Err(e.into());
                    }
                };
                
                // Check if context has expired
                if context.is_expired() {
                    warn!(user_id = user_id, expires_at = ?context.expires_at, "Context has expired, removing");
                    self.delete_context(user_id).await?;
                    return Ok(None);
                }
                
                debug!(user_id = user_id, scenario = ?context.scenario, step = ?context.step,
                       "Context loaded successfully");
                Ok(Some(context))
            }
            None => {
                debug!(user_id = user_id, "No context found in Redis");
                Ok(None)
            }
        }
    }

    /// Delete conversation context from Redis
    pub async fn delete_context(&self, user_id: i64) -> Result<()> {
        let key = self.get_context_key(user_id);
        let mut conn = self.connection_manager.clone();
        
        let deleted: u32 = conn.del(&key).await?;
        
        if deleted > 0 {
            debug!("Deleted context for user {}", user_id);
        } else {
            debug!("No context to delete for user {}", user_id);
        }
        
        Ok(())
    }

    /// Check if context exists for a user
    pub async fn context_exists(&self, user_id: i64) -> Result<bool> {
        let key = self.get_context_key(user_id);
        let mut conn = self.connection_manager.clone();
        
        let exists: bool = conn.exists(&key).await?;
        Ok(exists)
    }

    /// Extend the TTL of a context
    pub async fn extend_context_ttl(&self, user_id: i64, additional_seconds: u64) -> Result<bool> {
        let key = self.get_context_key(user_id);
        let mut conn = self.connection_manager.clone();
        
        // Get current TTL
        let current_ttl: i64 = conn.ttl(&key).await?;
        
        if current_ttl > 0 {
            let new_ttl = current_ttl as u64 + additional_seconds;
            let result: bool = conn.expire(&key, new_ttl as i64).await?;
            
            if result {
                debug!("Extended TTL for user {} to {}s", user_id, new_ttl);
            }
            
            Ok(result)
        } else {
            // Key doesn't exist or has no expiry
            Ok(false)
        }
    }

    /// Get all active user contexts (for cleanup/monitoring)
    pub async fn get_active_users(&self) -> Result<Vec<i64>> {
        let pattern = format!("{}context:*", self.config.prefix);
        let mut conn = self.connection_manager.clone();
        
        let keys: Vec<String> = conn.keys(&pattern).await?;
        
        let mut user_ids = Vec::new();
        for key in keys {
            if let Some(user_id_str) = key.strip_prefix(&format!("{}context:", self.config.prefix)) {
                if let Ok(user_id) = user_id_str.parse::<i64>() {
                    user_ids.push(user_id);
                }
            }
        }
        
        debug!("Found {} active user contexts", user_ids.len());
        Ok(user_ids)
    }

    /// Clean up expired contexts
    pub async fn cleanup_expired_contexts(&self) -> Result<u32> {
        let active_users = self.get_active_users().await?;
        let mut cleaned_count = 0;
        
        for user_id in active_users {
            if let Ok(Some(context)) = self.load_context(user_id).await {
                if context.is_expired() {
                    self.delete_context(user_id).await?;
                    cleaned_count += 1;
                }
            }
        }
        
        if cleaned_count > 0 {
            info!("Cleaned up {} expired contexts", cleaned_count);
        }
        
        Ok(cleaned_count)
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let active_users = self.get_active_users().await?;
        let total_contexts = active_users.len();
        
        let mut expired_contexts = 0;
        let mut scenarios_count = std::collections::HashMap::new();
        
        for user_id in &active_users {
            if let Ok(Some(context)) = self.load_context(*user_id).await {
                if context.is_expired() {
                    expired_contexts += 1;
                } else if let Some(scenario) = &context.scenario {
                    *scenarios_count.entry(scenario.clone()).or_insert(0) += 1;
                }
            }
        }
        
        Ok(StorageStats {
            total_contexts,
            expired_contexts,
            active_contexts: total_contexts - expired_contexts,
            scenarios_count,
        })
    }

    /// Backup all contexts to a JSON string
    pub async fn backup_contexts(&self) -> Result<String> {
        let active_users = self.get_active_users().await?;
        let mut contexts = Vec::new();
        
        for user_id in active_users {
            if let Ok(Some(context)) = self.load_context(user_id).await {
                if !context.is_expired() {
                    contexts.push(context);
                }
            }
        }
        
        let backup = serde_json::to_string_pretty(&contexts)?;
        info!("Created backup of {} contexts", contexts.len());
        
        Ok(backup)
    }

    /// Restore contexts from a JSON backup
    pub async fn restore_contexts(&self, backup_data: &str) -> Result<u32> {
        let contexts: Vec<ConversationContext> = serde_json::from_str(backup_data)?;
        let mut restored_count = 0;
        
        for context in contexts {
            if !context.is_expired() {
                self.save_context(&context).await?;
                restored_count += 1;
            }
        }
        
        info!("Restored {} contexts from backup", restored_count);
        Ok(restored_count)
    }

    /// Get the Redis key for a user's context
    fn get_context_key(&self, user_id: i64) -> String {
        format!("{}context:{}", self.config.prefix, user_id)
    }

    /// Test Redis connection
    pub async fn test_connection(&self) -> Result<()> {
        let mut conn = self.connection_manager.clone();
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }

    /// Get Redis connection info
    pub async fn get_connection_info(&self) -> Result<ConnectionInfo> {
        let mut conn = self.connection_manager.clone();
        
        // Get Redis info
        let info: String = redis::cmd("INFO").arg("server").query_async(&mut conn).await?;
        let mut redis_version = "unknown".to_string();
        
        for line in info.lines() {
            if line.starts_with("redis_version:") {
                redis_version = line.split(':').nth(1).unwrap_or("unknown").to_string();
                break;
            }
        }
        
        Ok(ConnectionInfo {
            redis_version,
            url: self.config.url.clone(),
            prefix: self.config.prefix.clone(),
            default_ttl: self.config.ttl_seconds,
        })
    }
}

impl std::fmt::Debug for StateStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateStorage")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// Storage statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageStats {
    pub total_contexts: usize,
    pub active_contexts: usize,
    pub expired_contexts: usize,
    pub scenarios_count: std::collections::HashMap<String, u32>,
}

/// Connection information
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionInfo {
    pub redis_version: String,
    pub url: String,
    pub prefix: String,
    pub default_ttl: u64,
}

/// State storage manager with automatic cleanup
#[derive(Debug)]
pub struct StateStorageManager {
    storage: StateStorage,
    cleanup_interval: Duration,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl StateStorageManager {
    /// Create a new state storage manager with automatic cleanup
    pub async fn new(config: RedisConfig, cleanup_interval: Duration) -> Result<Self> {
        let storage = StateStorage::new(config).await?;
        
        Ok(Self {
            storage,
            cleanup_interval,
            cleanup_handle: None,
        })
    }

    /// Start automatic cleanup task
    pub fn start_cleanup(&mut self) {
        if self.cleanup_handle.is_some() {
            warn!("Cleanup task is already running");
            return;
        }

        let storage = self.storage.clone();
        let interval = self.cleanup_interval;
        
        let handle = tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(interval);
            
            loop {
                cleanup_interval.tick().await;
                
                match storage.cleanup_expired_contexts().await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Cleanup task removed {} expired contexts", count);
                        }
                    }
                    Err(e) => {
                        error!("Cleanup task failed: {}", e);
                    }
                }
            }
        });
        
        self.cleanup_handle = Some(handle);
        info!("Started automatic cleanup task with interval {:?}", self.cleanup_interval);
    }

    /// Stop automatic cleanup task
    pub fn stop_cleanup(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
            info!("Stopped automatic cleanup task");
        }
    }

    /// Get reference to the storage
    pub fn storage(&self) -> &StateStorage {
        &self.storage
    }
}

impl Drop for StateStorageManager {
    fn drop(&mut self) {
        self.stop_cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;

    fn create_test_config() -> RedisConfig {
        RedisConfig {
            url: "redis://localhost:6379".to_string(),
            prefix: "test_swingbuddy:".to_string(),
            ttl_seconds: 3600,
        }
    }

    #[tokio::test]
    async fn test_context_save_load() {
        let config = create_test_config();
        let storage = StateStorage::new(config).await.unwrap();
        
        let mut context = ConversationContext::new(123);
        context.start_scenario("test", "step1").unwrap();
        context.set_data("key", "value").unwrap();
        
        // Save context
        storage.save_context(&context).await.unwrap();
        
        // Load context
        let loaded = storage.load_context(123).await.unwrap();
        assert!(loaded.is_some());
        
        let loaded_context = loaded.unwrap();
        assert_eq!(loaded_context.user_id, 123);
        assert_eq!(loaded_context.scenario, Some("test".to_string()));
        assert_eq!(loaded_context.step, Some("step1".to_string()));
        assert_eq!(loaded_context.get_string("key"), Some("value".to_string()));
        
        // Cleanup
        storage.delete_context(123).await.unwrap();
    }

    #[tokio::test]
    async fn test_context_expiry() {
        let config = create_test_config();
        let storage = StateStorage::new(config).await.unwrap();
        
        let mut context = ConversationContext::new(456);
        context.start_scenario("test", "step1").unwrap();
        
        // Set expiry in the past
        context.set_expiry(chrono::Utc::now() - chrono::Duration::hours(1));
        
        // Save context
        storage.save_context(&context).await.unwrap();
        
        // Try to load - should return None due to expiry
        let loaded = storage.load_context(456).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_context_deletion() {
        let config = create_test_config();
        let storage = StateStorage::new(config).await.unwrap();
        
        let context = ConversationContext::new(789);
        
        // Save context
        storage.save_context(&context).await.unwrap();
        
        // Verify it exists
        assert!(storage.context_exists(789).await.unwrap());
        
        // Delete context
        storage.delete_context(789).await.unwrap();
        
        // Verify it's gone
        assert!(!storage.context_exists(789).await.unwrap());
    }
}