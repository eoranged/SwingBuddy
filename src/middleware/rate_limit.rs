//! Rate limiting middleware
//! 
//! This module provides rate limiting functionality to prevent abuse
//! and ensure fair usage of the bot's resources.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use teloxide::types::User;
use tracing::{debug, warn, info};
use crate::utils::errors::{SwingBuddyError, Result};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window duration
    pub window_duration: Duration,
    /// Burst allowance (extra requests allowed in short bursts)
    pub burst_allowance: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 10,
            window_duration: Duration::from_secs(60),
            burst_allowance: 5,
        }
    }
}

/// Rate limit entry for tracking user requests
#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<Instant>,
    burst_used: u32,
    last_reset: Instant,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            burst_used: 0,
            last_reset: Instant::now(),
        }
    }

    /// Clean up old requests outside the window
    fn cleanup(&mut self, window_duration: Duration) {
        let cutoff = Instant::now() - window_duration;
        self.requests.retain(|&time| time > cutoff);
        
        // Reset burst if enough time has passed
        if self.last_reset.elapsed() > window_duration {
            self.burst_used = 0;
            self.last_reset = Instant::now();
        }
    }

    /// Check if request is allowed
    fn is_allowed(&mut self, config: &RateLimitConfig) -> bool {
        self.cleanup(config.window_duration);
        
        let current_requests = self.requests.len() as u32;
        
        // Check if within normal limits
        if current_requests < config.max_requests {
            return true;
        }
        
        // Check if burst allowance is available
        if self.burst_used < config.burst_allowance {
            self.burst_used += 1;
            return true;
        }
        
        false
    }

    /// Record a new request
    fn record_request(&mut self) {
        self.requests.push(Instant::now());
    }
}

/// Rate limiting middleware
#[derive(Clone)]
pub struct RateLimitMiddleware {
    config: RateLimitConfig,
    entries: Arc<Mutex<HashMap<i64, RateLimitEntry>>>,
    admin_exempt: bool,
    admin_ids: Vec<i64>,
}

impl RateLimitMiddleware {
    /// Create a new RateLimitMiddleware instance
    pub fn new(config: RateLimitConfig, admin_exempt: bool, admin_ids: Vec<i64>) -> Self {
        Self {
            config,
            entries: Arc::new(Mutex::new(HashMap::new())),
            admin_exempt,
            admin_ids,
        }
    }

    /// Check if user is rate limited
    pub fn check_rate_limit(&self, user: &User) -> Result<()> {
        let user_id = user.id.0 as i64;
        
        // Exempt admins if configured
        if self.admin_exempt && self.admin_ids.contains(&user_id) {
            debug!(user_id = user_id, "Admin user exempt from rate limiting");
            return Ok(());
        }

        let mut entries = self.entries.lock().unwrap();
        let entry = entries.entry(user_id).or_insert_with(RateLimitEntry::new);
        
        if entry.is_allowed(&self.config) {
            entry.record_request();
            debug!(user_id = user_id, "Rate limit check passed");
            Ok(())
        } else {
            warn!(
                user_id = user_id,
                username = user.username.as_deref().unwrap_or("none"),
                "Rate limit exceeded"
            );
            Err(SwingBuddyError::RateLimitExceeded)
        }
    }

    /// Get current rate limit status for user
    pub fn get_rate_limit_status(&self, user_id: i64) -> RateLimitStatus {
        let entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get(&user_id) {
            let mut entry_clone = entry.clone();
            entry_clone.cleanup(self.config.window_duration);
            
            let current_requests = entry_clone.requests.len() as u32;
            let remaining = self.config.max_requests.saturating_sub(current_requests);
            let burst_remaining = self.config.burst_allowance.saturating_sub(entry_clone.burst_used);
            
            RateLimitStatus {
                current_requests,
                max_requests: self.config.max_requests,
                remaining,
                burst_used: entry_clone.burst_used,
                burst_remaining,
                window_duration: self.config.window_duration,
                reset_time: entry_clone.last_reset + self.config.window_duration,
            }
        } else {
            RateLimitStatus {
                current_requests: 0,
                max_requests: self.config.max_requests,
                remaining: self.config.max_requests,
                burst_used: 0,
                burst_remaining: self.config.burst_allowance,
                window_duration: self.config.window_duration,
                reset_time: Instant::now() + self.config.window_duration,
            }
        }
    }

    /// Clear rate limit for specific user (admin function)
    pub fn clear_user_rate_limit(&self, user_id: i64) -> bool {
        let mut entries = self.entries.lock().unwrap();
        let removed = entries.remove(&user_id).is_some();
        
        if removed {
            info!(user_id = user_id, "Rate limit cleared for user");
        }
        
        removed
    }

    /// Clear all rate limits (admin function)
    pub fn clear_all_rate_limits(&self) -> usize {
        let mut entries = self.entries.lock().unwrap();
        let count = entries.len();
        entries.clear();
        
        info!(cleared_count = count, "All rate limits cleared");
        count
    }

    /// Get rate limit statistics
    pub fn get_statistics(&self) -> RateLimitStatistics {
        let entries = self.entries.lock().unwrap();
        let total_users = entries.len();
        let mut active_users = 0;
        let mut total_requests = 0;
        let mut users_at_limit = 0;

        for entry in entries.values() {
            let mut entry_clone = entry.clone();
            entry_clone.cleanup(self.config.window_duration);
            
            if !entry_clone.requests.is_empty() {
                active_users += 1;
                total_requests += entry_clone.requests.len();
                
                if entry_clone.requests.len() >= self.config.max_requests as usize {
                    users_at_limit += 1;
                }
            }
        }

        RateLimitStatistics {
            total_users,
            active_users,
            total_requests,
            users_at_limit,
            config: self.config.clone(),
        }
    }

    /// Cleanup old entries (should be called periodically)
    pub fn cleanup_old_entries(&self) {
        let mut entries = self.entries.lock().unwrap();
        let cutoff = Instant::now() - self.config.window_duration * 2; // Keep entries for 2x window duration
        
        entries.retain(|_, entry| {
            entry.requests.iter().any(|&time| time > cutoff)
        });
        
        debug!(remaining_entries = entries.len(), "Cleaned up old rate limit entries");
    }

    /// Update configuration
    pub fn update_config(&mut self, config: RateLimitConfig) {
        self.config = config;
        info!("Rate limit configuration updated");
    }
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self::new(RateLimitConfig::default(), true, vec![])
    }
}

/// Rate limit status for a user
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub current_requests: u32,
    pub max_requests: u32,
    pub remaining: u32,
    pub burst_used: u32,
    pub burst_remaining: u32,
    pub window_duration: Duration,
    pub reset_time: Instant,
}

/// Rate limit statistics
#[derive(Debug, Clone)]
pub struct RateLimitStatistics {
    pub total_users: usize,
    pub active_users: usize,
    pub total_requests: usize,
    pub users_at_limit: usize,
    pub config: RateLimitConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use teloxide::types::UserId;

    fn create_test_user(id: u64) -> User {
        User {
            id: UserId(id),
            is_bot: false,
            first_name: "Test".to_string(),
            last_name: None,
            username: None,
            language_code: None,
            is_premium: false,
            added_to_attachment_menu: false,
        }
    }

    #[test]
    fn test_rate_limit_basic() {
        let config = RateLimitConfig {
            max_requests: 3,
            window_duration: Duration::from_secs(60),
            burst_allowance: 1,
        };
        
        let middleware = RateLimitMiddleware::new(config, false, vec![]);
        let user = create_test_user(123);
        
        // First 3 requests should pass
        assert!(middleware.check_rate_limit(&user).is_ok());
        assert!(middleware.check_rate_limit(&user).is_ok());
        assert!(middleware.check_rate_limit(&user).is_ok());
        
        // 4th request should use burst allowance
        assert!(middleware.check_rate_limit(&user).is_ok());
        
        // 5th request should fail
        assert!(middleware.check_rate_limit(&user).is_err());
    }

    #[test]
    fn test_admin_exemption() {
        let config = RateLimitConfig {
            max_requests: 1,
            window_duration: Duration::from_secs(60),
            burst_allowance: 0,
        };
        
        let middleware = RateLimitMiddleware::new(config, true, vec![123]);
        let admin_user = create_test_user(123);
        let regular_user = create_test_user(456);
        
        // Admin should not be rate limited
        assert!(middleware.check_rate_limit(&admin_user).is_ok());
        assert!(middleware.check_rate_limit(&admin_user).is_ok());
        assert!(middleware.check_rate_limit(&admin_user).is_ok());
        
        // Regular user should be rate limited
        assert!(middleware.check_rate_limit(&regular_user).is_ok());
        assert!(middleware.check_rate_limit(&regular_user).is_err());
    }

    #[test]
    fn test_rate_limit_status() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_secs(60),
            burst_allowance: 2,
        };
        
        let middleware = RateLimitMiddleware::new(config, false, vec![]);
        let user = create_test_user(123);
        
        // Initial status
        let status = middleware.get_rate_limit_status(123);
        assert_eq!(status.current_requests, 0);
        assert_eq!(status.remaining, 5);
        
        // After some requests
        middleware.check_rate_limit(&user).unwrap();
        middleware.check_rate_limit(&user).unwrap();
        
        let status = middleware.get_rate_limit_status(123);
        assert_eq!(status.current_requests, 2);
        assert_eq!(status.remaining, 3);
    }

    #[test]
    fn test_cleanup() {
        let middleware = RateLimitMiddleware::default();
        let user = create_test_user(123);
        
        // Make some requests
        middleware.check_rate_limit(&user).unwrap();
        middleware.check_rate_limit(&user).unwrap();
        
        // Check statistics
        let stats = middleware.get_statistics();
        assert_eq!(stats.active_users, 1);
        assert_eq!(stats.total_requests, 2);
        
        // Cleanup should not remove recent entries
        middleware.cleanup_old_entries();
        let stats = middleware.get_statistics();
        assert_eq!(stats.total_users, 1);
    }
}