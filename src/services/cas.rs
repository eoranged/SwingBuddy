//! CAS (Combot Anti-Spam) service implementation
//! 
//! This service handles CAS API integration for spam protection,
//! including HTTP client setup, response parsing, caching logic,
//! rate limiting, and error handling.

use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};
use redis::AsyncCommands;
use crate::config::settings::Settings;
use crate::utils::errors::{SwingBuddyError, CasError, Result};

/// CAS API response structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CasResponse {
    pub ok: bool,
    pub result: Option<CasApiResult>,
}

/// CAS check result structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CasApiResult {
    pub offenses: u32,
    pub messages: Vec<String>,
    pub time_added: Option<String>,
}

/// CAS check result with caching info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCasResult {
    pub is_banned: bool,
    pub offenses: u32,
    pub messages: Vec<String>,
    pub time_added: Option<String>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

/// CAS service for spam protection
#[derive(Clone)]
#[derive(Debug)]
pub struct CasService {
    client: Client,
    redis_client: redis::Client,
    settings: Settings,
}

impl CasService {
    /// Create a new CasService instance
    pub fn new(redis_client: redis::Client, settings: Settings) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(settings.cas.timeout_seconds))
            .user_agent("SwingBuddy-Bot/1.0")
            .build()
            .map_err(|e| SwingBuddyError::Http(e))?;

        Ok(Self {
            client,
            redis_client,
            settings,
        })
    }

    /// Check if user is banned according to CAS
    pub async fn check_user(&self, user_id: i64) -> Result<CachedCasResult> {
        debug!(user_id = user_id, "Checking user against CAS");

        // First check cache
        if let Some(cached_result) = self.get_cached_result(user_id).await? {
            debug!(user_id = user_id, "Found cached CAS result");
            return Ok(cached_result);
        }

        // Make API request
        let result = self.make_cas_request(user_id).await?;
        
        // Cache the result
        self.cache_result(user_id, &result).await?;

        Ok(result)
    }

    /// Force refresh CAS check (bypass cache)
    pub async fn force_check_user(&self, user_id: i64) -> Result<CachedCasResult> {
        info!(user_id = user_id, "Force checking user against CAS (bypassing cache)");

        let result = self.make_cas_request(user_id).await?;
        
        // Update cache with new result
        self.cache_result(user_id, &result).await?;

        Ok(result)
    }

    /// Check multiple users in batch
    pub async fn check_users_batch(&self, user_ids: Vec<i64>) -> Result<Vec<(i64, CachedCasResult)>> {
        debug!(count = user_ids.len(), "Batch checking users against CAS");

        let mut results = Vec::new();
        
        // Process in chunks to avoid overwhelming the API
        for chunk in user_ids.chunks(10) {
            let mut chunk_results = Vec::new();
            
            for &user_id in chunk {
                match self.check_user(user_id).await {
                    Ok(result) => chunk_results.push((user_id, result)),
                    Err(e) => {
                        warn!(user_id = user_id, error = %e, "Failed to check user against CAS");
                        // Continue with other users even if one fails
                    }
                }
                
                // Small delay between requests to be respectful to the API
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            results.extend(chunk_results);
        }

        Ok(results)
    }

    /// Get cached result from Redis
    async fn get_cached_result(&self, user_id: i64) -> Result<Option<CachedCasResult>> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        let cache_key = format!("{}cas:check:{}", self.settings.redis.prefix, user_id);
        
        let cached_data: Option<String> = conn.get(&cache_key).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        if let Some(data) = cached_data {
            match serde_json::from_str::<CachedCasResult>(&data) {
                Ok(result) => {
                    // Check if cache is still valid (not older than TTL)
                    let cache_age = chrono::Utc::now() - result.checked_at;
                    if cache_age.num_seconds() < self.settings.redis.ttl_seconds as i64 {
                        return Ok(Some(result));
                    } else {
                        // Cache expired, remove it
                        let _: () = conn.del(&cache_key).await
                            .map_err(|e| SwingBuddyError::Redis(e))?;
                    }
                }
                Err(e) => {
                    warn!(user_id = user_id, error = %e, "Failed to deserialize cached CAS result");
                    // Remove corrupted cache entry
                    let _: () = conn.del(&cache_key).await
                        .map_err(|e| SwingBuddyError::Redis(e))?;
                }
            }
        }

        Ok(None)
    }

    /// Cache result in Redis
    async fn cache_result(&self, user_id: i64, result: &CachedCasResult) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        let cache_key = format!("{}cas:check:{}", self.settings.redis.prefix, user_id);
        let serialized = serde_json::to_string(result)
            .map_err(|e| SwingBuddyError::Serialization(e))?;

        let _: () = conn.set_ex(&cache_key, serialized, self.settings.redis.ttl_seconds as u64).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(user_id = user_id, "Cached CAS result");
        Ok(())
    }

    /// Make actual CAS API request
    async fn make_cas_request(&self, user_id: i64) -> Result<CachedCasResult> {
        let url = format!("{}/check?user_id={}", self.settings.cas.api_url, user_id);
        
        debug!(user_id = user_id, url = %url, "Making CAS API request");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SwingBuddyError::Cas(CasError::Timeout)
                } else if e.is_connect() {
                    SwingBuddyError::Cas(CasError::ServiceUnavailable)
                } else {
                    SwingBuddyError::Cas(CasError::RequestFailed(e.to_string()))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(SwingBuddyError::Cas(CasError::RequestFailed(
                format!("HTTP {}: {}", status, error_text)
            )));
        }

        let cas_response: CasResponse = response.json().await
            .map_err(|e| SwingBuddyError::Cas(CasError::InvalidResponse(e.to_string())))?;

        if !cas_response.ok {
            return Err(SwingBuddyError::Cas(CasError::InvalidResponse(
                "CAS API returned ok: false".to_string()
            )));
        }

        let result = match cas_response.result {
            Some(result) => {
                let is_banned = result.offenses > 0;
                
                if is_banned {
                    warn!(
                        user_id = user_id,
                        offenses = result.offenses,
                        messages = ?result.messages,
                        "User is banned according to CAS"
                    );
                } else {
                    debug!(user_id = user_id, "User is clean according to CAS");
                }

                CachedCasResult {
                    is_banned,
                    offenses: result.offenses,
                    messages: result.messages,
                    time_added: result.time_added,
                    checked_at: chrono::Utc::now(),
                }
            }
            None => {
                // No result means user is not in CAS database (clean)
                debug!(user_id = user_id, "User not found in CAS database (clean)");
                CachedCasResult {
                    is_banned: false,
                    offenses: 0,
                    messages: vec![],
                    time_added: None,
                    checked_at: chrono::Utc::now(),
                }
            }
        };

        Ok(result)
    }

    /// Clear cache for specific user
    pub async fn clear_user_cache(&self, user_id: i64) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        let cache_key = format!("{}cas:check:{}", self.settings.redis.prefix, user_id);
        let _: () = conn.del(&cache_key).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        debug!(user_id = user_id, "Cleared CAS cache for user");
        Ok(())
    }

    /// Clear all CAS cache
    pub async fn clear_all_cache(&self) -> Result<u64> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        let pattern = format!("{}cas:check:*", self.settings.redis.prefix);
        let keys: Vec<String> = conn.keys(&pattern).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn.del(&keys).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        info!(deleted_keys = deleted, "Cleared all CAS cache");
        Ok(deleted)
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        let pattern = format!("{}cas:check:*", self.settings.redis.prefix);
        let keys: Vec<String> = conn.keys(&pattern).await
            .map_err(|e| SwingBuddyError::Redis(e))?;

        let total_entries = keys.len() as u64;
        let mut banned_entries = 0u64;
        let mut clean_entries = 0u64;

        // Sample a subset of keys to get statistics (to avoid performance issues)
        let sample_size = std::cmp::min(100, keys.len());
        for key in keys.iter().take(sample_size) {
            if let Ok(Some(data)) = conn.get::<_, Option<String>>(key).await {
                if let Ok(result) = serde_json::from_str::<CachedCasResult>(&data) {
                    if result.is_banned {
                        banned_entries += 1;
                    } else {
                        clean_entries += 1;
                    }
                }
            }
        }

        // Extrapolate from sample if we sampled
        if sample_size < keys.len() {
            let ratio = keys.len() as f64 / sample_size as f64;
            banned_entries = (banned_entries as f64 * ratio) as u64;
            clean_entries = (clean_entries as f64 * ratio) as u64;
        }

        Ok(CacheStats {
            total_entries,
            banned_entries,
            clean_entries,
        })
    }

    /// Check if CAS protection is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.features.cas_protection
    }

    /// Check if auto-ban is enabled
    pub fn is_auto_ban_enabled(&self) -> bool {
        self.settings.cas.auto_ban
    }
}

/// Cache statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: u64,
    pub banned_entries: u64,
    pub clean_entries: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cas_response_deserialization() {
        let json = r#"{"ok": true, "result": {"offenses": 1, "messages": ["spam"], "time_added": "2023-01-01"}}"#;
        let response: CasResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());
        assert_eq!(response.result.unwrap().offenses, 1);
    }

    #[test]
    fn test_cas_response_no_result() {
        let json = r#"{"ok": true, "result": null}"#;
        let response: CasResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok);
        assert!(response.result.is_none());
    }

    #[test]
    fn test_cached_result_serialization() {
        let result = CachedCasResult {
            is_banned: true,
            offenses: 2,
            messages: vec!["spam".to_string(), "flood".to_string()],
            time_added: Some("2023-01-01".to_string()),
            checked_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: CachedCasResult = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(result.is_banned, deserialized.is_banned);
        assert_eq!(result.offenses, deserialized.offenses);
        assert_eq!(result.messages, deserialized.messages);
    }
}