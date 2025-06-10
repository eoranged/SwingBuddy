//! Services module
//!
//! This module contains business logic services

pub mod auth;
pub mod cas;
pub mod google;
pub mod notification;
pub mod redis;
pub mod user;

// Re-export commonly used services
pub use auth::{AuthService, AuthContext, Permission, AuthMiddleware};
pub use cas::{CasService, CachedCasResult, CacheStats as CasCacheStats};
pub use google::{GoogleCalendarService, GoogleCalendarEvent, CalendarStats};
pub use notification::{NotificationService, MessageTemplate, NotificationRequest, BulkNotificationRequest, NotificationStats};
pub use redis::{RedisService, CacheEntry, CacheStats as RedisCacheStats};
pub use user::UserService;

use crate::config::settings::Settings;
use crate::database::repositories::UserRepository;
use crate::utils::errors::Result;
use teloxide::Bot;

/// Service factory for creating and managing all services
#[derive(Clone)]
pub struct ServiceFactory {
    pub user_service: UserService,
    pub auth_service: AuthService,
    pub cas_service: CasService,
    pub google_service: GoogleCalendarService,
    pub notification_service: NotificationService,
    pub redis_service: RedisService,
}

impl ServiceFactory {
    /// Create a new ServiceFactory with all services initialized
    pub fn new(
        bot: Bot,
        settings: Settings,
        user_repository: UserRepository,
        redis_client: ::redis::Client,
    ) -> Result<Self> {
        let user_service = UserService::new(user_repository, settings.clone());
        let auth_service = AuthService::new(bot.clone(), settings.clone());
        let cas_service = CasService::new(redis_client.clone(), settings.clone())?;
        let google_service = GoogleCalendarService::new(settings.clone())?;
        let notification_service = NotificationService::new(bot, settings.clone());
        let redis_service = RedisService::new(settings)?;

        Ok(Self {
            user_service,
            auth_service,
            cas_service,
            google_service,
            notification_service,
            redis_service,
        })
    }

    /// Get authentication middleware
    pub fn auth_middleware(&self) -> AuthMiddleware {
        self.auth_service.create_auth_middleware()
    }

    /// Health check for all services
    pub async fn health_check(&self) -> ServiceHealthStatus {
        let redis_healthy = self.redis_service.health_check().await.unwrap_or(false);
        let google_enabled = self.google_service.is_enabled();
        let cas_enabled = self.cas_service.is_enabled();

        ServiceHealthStatus {
            redis_healthy,
            google_enabled,
            cas_enabled,
            notification_service_ready: true, // Always ready if constructed
            user_service_ready: true, // Always ready if constructed
            auth_service_ready: true, // Always ready if constructed
        }
    }
}

/// Health status for all services
#[derive(Debug, Clone)]
pub struct ServiceHealthStatus {
    pub redis_healthy: bool,
    pub google_enabled: bool,
    pub cas_enabled: bool,
    pub notification_service_ready: bool,
    pub user_service_ready: bool,
    pub auth_service_ready: bool,
}

impl ServiceHealthStatus {
    /// Check if all critical services are healthy
    pub fn is_healthy(&self) -> bool {
        self.user_service_ready && self.auth_service_ready && self.notification_service_ready
    }

    /// Get list of unhealthy services
    pub fn get_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.redis_healthy {
            issues.push("Redis connection failed".to_string());
        }
        if !self.user_service_ready {
            issues.push("User service not ready".to_string());
        }
        if !self.auth_service_ready {
            issues.push("Auth service not ready".to_string());
        }
        if !self.notification_service_ready {
            issues.push("Notification service not ready".to_string());
        }

        issues
    }
}