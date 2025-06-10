//! Authentication middleware
//! 
//! This module provides authentication and authorization middleware
//! for protecting bot commands and features.

use std::collections::HashSet;
use teloxide::types::{Message, User};
use tracing::{debug, warn};
use crate::config::settings::Settings;
use crate::utils::errors::{SwingBuddyError, Result};

/// Authentication middleware
#[derive(Clone)]
pub struct AuthMiddleware {
    admin_ids: HashSet<i64>,
    settings: Settings,
}

impl AuthMiddleware {
    /// Create a new AuthMiddleware instance
    pub fn new(settings: Settings) -> Self {
        let admin_ids: HashSet<i64> = settings.bot.admin_ids.iter().cloned().collect();
        
        Self {
            admin_ids,
            settings,
        }
    }

    /// Check if user is an admin
    pub fn is_admin(&self, user_id: i64) -> bool {
        self.admin_ids.contains(&user_id)
    }

    /// Check if user is authorized for admin commands
    pub fn check_admin_auth(&self, user: &User) -> Result<()> {
        let user_id = user.id.0 as i64;
        
        if self.is_admin(user_id) {
            debug!(user_id = user_id, "Admin authentication successful");
            Ok(())
        } else {
            warn!(user_id = user_id, "Unauthorized admin access attempt");
            Err(SwingBuddyError::PermissionDenied(
                "Admin privileges required".to_string()
            ))
        }
    }

    /// Check if user can access group features
    pub fn check_group_auth(&self, message: &Message) -> Result<()> {
        match &message.chat.kind {
            teloxide::types::ChatKind::Public(_) => {
                // Public groups are allowed
                Ok(())
            }
            teloxide::types::ChatKind::Private(_) => {
                // Private chats are allowed for individual users
                Ok(())
            }
        }
    }

    /// Check rate limiting for user
    pub fn check_rate_limit(&self, user_id: i64) -> Result<()> {
        // TODO: Implement rate limiting logic with Redis
        // For now, just return Ok
        debug!(user_id = user_id, "Rate limit check passed");
        Ok(())
    }

    /// Validate user permissions for specific action
    pub fn validate_permissions(&self, user: &User, action: &str) -> Result<()> {
        let user_id = user.id.0 as i64;
        
        match action {
            "admin" => self.check_admin_auth(user),
            "create_event" => {
                // For now, allow all users to create events
                // TODO: Add user_event_creation feature flag to settings
                if self.is_admin(user_id) {
                    Ok(())
                } else {
                    Err(SwingBuddyError::PermissionDenied(
                        "Event creation not allowed for regular users".to_string()
                    ))
                }
            }
            "manage_group" => self.check_admin_auth(user),
            _ => {
                // Default: allow basic actions for all users
                Ok(())
            }
        }
    }

    /// Add admin user
    pub fn add_admin(&mut self, user_id: i64) {
        self.admin_ids.insert(user_id);
    }

    /// Remove admin user
    pub fn remove_admin(&mut self, user_id: i64) -> bool {
        self.admin_ids.remove(&user_id)
    }

    /// Get list of admin IDs
    pub fn get_admin_ids(&self) -> Vec<i64> {
        self.admin_ids.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use teloxide::types::{UserId, User as TgUser};

    fn create_test_user(id: u64) -> TgUser {
        TgUser {
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
    fn test_admin_check() {
        let mut settings = Settings::default();
        settings.bot.admin_ids = vec![123, 456];
        
        let auth = AuthMiddleware::new(settings);
        
        assert!(auth.is_admin(123));
        assert!(auth.is_admin(456));
        assert!(!auth.is_admin(789));
    }

    #[test]
    fn test_admin_auth() {
        let mut settings = Settings::default();
        settings.bot.admin_ids = vec![123];
        
        let auth = AuthMiddleware::new(settings);
        
        let admin_user = create_test_user(123);
        let regular_user = create_test_user(456);
        
        assert!(auth.check_admin_auth(&admin_user).is_ok());
        assert!(auth.check_admin_auth(&regular_user).is_err());
    }

    #[test]
    fn test_permission_validation() {
        let mut settings = Settings::default();
        settings.bot.admin_ids = vec![123];
        
        let auth = AuthMiddleware::new(settings);
        
        let admin_user = create_test_user(123);
        let regular_user = create_test_user(456);
        
        // Admin can do everything
        assert!(auth.validate_permissions(&admin_user, "admin").is_ok());
        assert!(auth.validate_permissions(&admin_user, "create_event").is_ok());
        assert!(auth.validate_permissions(&admin_user, "manage_group").is_ok());
        
        // Regular user has limited permissions
        assert!(auth.validate_permissions(&regular_user, "admin").is_err());
        assert!(auth.validate_permissions(&regular_user, "create_event").is_err());
        assert!(auth.validate_permissions(&regular_user, "manage_group").is_err());
        
        // Basic actions are allowed for everyone
        assert!(auth.validate_permissions(&regular_user, "view_events").is_ok());
    }
}