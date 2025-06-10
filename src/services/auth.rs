//! Authentication service implementation
//! 
//! This service handles admin authentication and authorization, permission checking
//! for bot operations, group admin verification, role-based access control,
//! and integration with admin configuration from TOML.

use std::collections::HashSet;
use teloxide::types::{ChatId, ChatMember, ChatMemberKind, UserId};
use teloxide::{Bot, requests::Requester, prelude::Request};
use tracing::{info, warn, error, debug};
use crate::config::settings::Settings;
use crate::models::User;
use crate::utils::errors::{SwingBuddyError, Result};

/// Permission levels for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Basic user permissions
    User,
    /// Group moderator permissions
    GroupModerator,
    /// Group admin permissions
    GroupAdmin,
    /// Bot admin permissions
    BotAdmin,
    /// Super admin permissions (full access)
    SuperAdmin,
}

/// Authentication context for a user
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub chat_id: Option<ChatId>,
    pub permissions: HashSet<Permission>,
    pub is_bot_admin: bool,
    pub is_group_admin: bool,
    pub is_group_member: bool,
}

/// Authentication service for managing permissions and access control
#[derive(Clone)]
pub struct AuthService {
    bot: Bot,
    settings: Settings,
}

impl AuthService {
    /// Create a new AuthService instance
    pub fn new(bot: Bot, settings: Settings) -> Self {
        Self { bot, settings }
    }

    /// Check if user is a bot admin
    pub fn is_bot_admin(&self, user_id: i64) -> bool {
        self.settings.bot.admin_ids.contains(&user_id)
    }

    /// Check if user is a super admin (first admin in the list)
    pub fn is_super_admin(&self, user_id: i64) -> bool {
        self.settings.bot.admin_ids.first() == Some(&user_id)
    }

    /// Get authentication context for a user
    pub async fn get_auth_context(&self, user_id: i64, chat_id: Option<ChatId>) -> Result<AuthContext> {
        debug!(user_id = user_id, chat_id = ?chat_id, "Getting authentication context");

        let mut permissions = HashSet::new();
        let is_bot_admin = self.is_bot_admin(user_id);
        let is_super_admin = self.is_super_admin(user_id);
        
        // All users have basic user permissions
        permissions.insert(Permission::User);

        // Bot admins get elevated permissions
        if is_bot_admin {
            permissions.insert(Permission::BotAdmin);
            
            if is_super_admin {
                permissions.insert(Permission::SuperAdmin);
            }
        }

        let mut is_group_admin = false;
        let mut is_group_member = false;

        // Check group-specific permissions if chat_id is provided
        if let Some(chat_id) = chat_id {
            match self.get_chat_member_status(chat_id, user_id).await {
                Ok((is_member, is_admin)) => {
                    is_group_member = is_member;
                    is_group_admin = is_admin;
                    
                    if is_member {
                        if is_admin {
                            permissions.insert(Permission::GroupAdmin);
                            permissions.insert(Permission::GroupModerator);
                        }
                    }
                }
                Err(e) => {
                    warn!(user_id = user_id, chat_id = ?chat_id, error = %e, "Failed to get chat member status");
                }
            }
        }

        let context = AuthContext {
            user_id,
            chat_id,
            permissions,
            is_bot_admin,
            is_group_admin,
            is_group_member,
        };

        debug!(user_id = user_id, permissions = ?context.permissions, "Authentication context created");
        Ok(context)
    }

    /// Check if user has specific permission
    pub async fn has_permission(&self, user_id: i64, chat_id: Option<ChatId>, required_permission: Permission) -> Result<bool> {
        let context = self.get_auth_context(user_id, chat_id).await?;
        Ok(context.permissions.contains(&required_permission))
    }

    /// Require specific permission or return error
    pub async fn require_permission(&self, user_id: i64, chat_id: Option<ChatId>, required_permission: Permission) -> Result<AuthContext> {
        let context = self.get_auth_context(user_id, chat_id).await?;
        
        if !context.permissions.contains(&required_permission) {
            return Err(SwingBuddyError::PermissionDenied(
                format!("User {} lacks required permission: {:?}", user_id, required_permission)
            ));
        }

        Ok(context)
    }

    /// Check if user can manage events in a group
    pub async fn can_manage_events(&self, user_id: i64, chat_id: Option<ChatId>) -> Result<bool> {
        let context = self.get_auth_context(user_id, chat_id).await?;
        
        // Bot admins can always manage events
        if context.is_bot_admin {
            return Ok(true);
        }

        // Group admins can manage events in their groups
        if chat_id.is_some() && context.is_group_admin {
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if user can manage users (ban/unban)
    pub async fn can_manage_users(&self, user_id: i64, chat_id: Option<ChatId>) -> Result<bool> {
        let context = self.get_auth_context(user_id, chat_id).await?;
        
        // Only bot admins can manage users globally
        if context.is_bot_admin {
            return Ok(true);
        }

        // Group admins can manage users in their groups
        if chat_id.is_some() && context.is_group_admin {
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if user can access admin panel
    pub async fn can_access_admin_panel(&self, user_id: i64) -> Result<bool> {
        Ok(self.is_bot_admin(user_id))
    }

    /// Check if user can modify bot settings
    pub async fn can_modify_settings(&self, user_id: i64) -> Result<bool> {
        Ok(self.is_super_admin(user_id))
    }

    /// Verify user identity and get user info
    pub async fn verify_user(&self, user: &User) -> Result<bool> {
        debug!(user_id = user.id, telegram_id = user.telegram_id, "Verifying user identity");

        // Basic verification - check if user is not banned
        if user.is_banned {
            warn!(user_id = user.id, "User is banned");
            return Ok(false);
        }

        // Additional verification logic could be added here
        // For example, checking against external services, rate limiting, etc.

        Ok(true)
    }

    /// Get chat member status
    async fn get_chat_member_status(&self, chat_id: ChatId, user_id: i64) -> Result<(bool, bool)> {
        match self.bot.get_chat_member(chat_id, UserId(user_id as u64)).send().await {
            Ok(chat_member) => {
                let is_member = !matches!(chat_member.kind, ChatMemberKind::Left | ChatMemberKind::Banned(_));
                let is_admin = matches!(
                    chat_member.kind,
                    ChatMemberKind::Owner(_) | ChatMemberKind::Administrator(_)
                );
                
                debug!(
                    user_id = user_id,
                    chat_id = ?chat_id,
                    is_member = is_member,
                    is_admin = is_admin,
                    member_kind = ?chat_member.kind,
                    "Chat member status retrieved"
                );
                
                Ok((is_member, is_admin))
            }
            Err(e) => {
                // If we can't get member status, assume they're not a member
                debug!(user_id = user_id, chat_id = ?chat_id, error = %e, "Could not get chat member status");
                Ok((false, false))
            }
        }
    }

    /// Create authentication middleware for handlers
    pub fn create_auth_middleware(&self) -> AuthMiddleware {
        AuthMiddleware {
            auth_service: self.clone(),
        }
    }

    /// Log authentication event
    pub fn log_auth_event(&self, user_id: i64, action: &str, success: bool, details: Option<&str>) {
        if success {
            info!(
                user_id = user_id,
                action = action,
                details = details,
                "Authentication event: success"
            );
        } else {
            warn!(
                user_id = user_id,
                action = action,
                details = details,
                "Authentication event: failure"
            );
        }
    }

    /// Get permission hierarchy
    pub fn get_permission_hierarchy() -> Vec<Permission> {
        vec![
            Permission::User,
            Permission::GroupModerator,
            Permission::GroupAdmin,
            Permission::BotAdmin,
            Permission::SuperAdmin,
        ]
    }

    /// Check if permission A includes permission B
    pub fn permission_includes(higher: Permission, lower: Permission) -> bool {
        let hierarchy = Self::get_permission_hierarchy();
        let higher_level = hierarchy.iter().position(|&p| p == higher).unwrap_or(0);
        let lower_level = hierarchy.iter().position(|&p| p == lower).unwrap_or(0);
        
        higher_level >= lower_level
    }

    /// Get all admin user IDs
    pub fn get_admin_ids(&self) -> &[i64] {
        &self.settings.bot.admin_ids
    }

    /// Add admin user ID (only super admin can do this)
    pub fn add_admin(&mut self, requester_id: i64, new_admin_id: i64) -> Result<()> {
        if !self.is_super_admin(requester_id) {
            return Err(SwingBuddyError::PermissionDenied(
                "Only super admin can add new admins".to_string()
            ));
        }

        if !self.settings.bot.admin_ids.contains(&new_admin_id) {
            self.settings.bot.admin_ids.push(new_admin_id);
            info!(requester_id = requester_id, new_admin_id = new_admin_id, "New admin added");
        }

        Ok(())
    }

    /// Remove admin user ID (only super admin can do this)
    pub fn remove_admin(&mut self, requester_id: i64, admin_id: i64) -> Result<()> {
        if !self.is_super_admin(requester_id) {
            return Err(SwingBuddyError::PermissionDenied(
                "Only super admin can remove admins".to_string()
            ));
        }

        if admin_id == requester_id {
            return Err(SwingBuddyError::PermissionDenied(
                "Cannot remove yourself as admin".to_string()
            ));
        }

        self.settings.bot.admin_ids.retain(|&id| id != admin_id);
        info!(requester_id = requester_id, removed_admin_id = admin_id, "Admin removed");

        Ok(())
    }
}

/// Authentication middleware for request handling
#[derive(Clone)]
pub struct AuthMiddleware {
    auth_service: AuthService,
}

impl AuthMiddleware {
    /// Check authentication for a request
    pub async fn check_auth(&self, user_id: i64, chat_id: Option<ChatId>, required_permission: Permission) -> Result<AuthContext> {
        self.auth_service.require_permission(user_id, chat_id, required_permission).await
    }

    /// Check if user can perform action
    pub async fn can_perform_action(&self, user_id: i64, chat_id: Option<ChatId>, action: &str) -> Result<bool> {
        let required_permission = match action {
            "create_event" | "edit_event" | "delete_event" => Permission::GroupAdmin,
            "ban_user" | "unban_user" => Permission::BotAdmin,
            "access_admin_panel" => Permission::BotAdmin,
            "modify_settings" => Permission::SuperAdmin,
            _ => Permission::User,
        };

        self.auth_service.has_permission(user_id, chat_id, required_permission).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_hierarchy() {
        assert!(AuthService::permission_includes(Permission::SuperAdmin, Permission::User));
        assert!(AuthService::permission_includes(Permission::BotAdmin, Permission::GroupAdmin));
        assert!(AuthService::permission_includes(Permission::GroupAdmin, Permission::GroupModerator));
        assert!(!AuthService::permission_includes(Permission::User, Permission::BotAdmin));
    }

    #[test]
    fn test_bot_admin_check() {
        let bot = teloxide::Bot::new("test_token");
        let mut settings = Settings::default();
        settings.bot.admin_ids = vec![123456789, 987654321];
        
        let auth_service = AuthService::new(bot, settings);
        
        assert!(auth_service.is_bot_admin(123456789));
        assert!(auth_service.is_bot_admin(987654321));
        assert!(!auth_service.is_bot_admin(111111111));
    }

    #[test]
    fn test_super_admin_check() {
        let bot = teloxide::Bot::new("test_token");
        let mut settings = Settings::default();
        settings.bot.admin_ids = vec![123456789, 987654321];
        
        let auth_service = AuthService::new(bot, settings);
        
        assert!(auth_service.is_super_admin(123456789)); // First admin is super admin
        assert!(!auth_service.is_super_admin(987654321)); // Second admin is not super admin
        assert!(!auth_service.is_super_admin(111111111)); // Non-admin is not super admin
    }

    #[tokio::test]
    async fn test_auth_context_creation() {
        let bot = teloxide::Bot::new("test_token");
        let mut settings = Settings::default();
        settings.bot.admin_ids = vec![123456789];
        
        let auth_service = AuthService::new(bot, settings);
        
        // Test bot admin context
        let context = auth_service.get_auth_context(123456789, None).await.unwrap();
        assert!(context.is_bot_admin);
        assert!(context.permissions.contains(&Permission::BotAdmin));
        assert!(context.permissions.contains(&Permission::SuperAdmin));
        
        // Test regular user context
        let context = auth_service.get_auth_context(111111111, None).await.unwrap();
        assert!(!context.is_bot_admin);
        assert!(context.permissions.contains(&Permission::User));
        assert!(!context.permissions.contains(&Permission::BotAdmin));
    }
}