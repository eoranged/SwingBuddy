//! User service implementation
//! 
//! This service handles user registration, profile management, language preferences,
//! location management, and user onboarding flow logic.

use std::collections::HashMap;
use tracing::{info, warn, debug};
use crate::config::settings::Settings;
use crate::database::repositories::UserRepository;
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::utils::errors::{SwingBuddyError, Result};

/// User service for managing user operations
#[derive(Clone)]
pub struct UserService {
    user_repository: UserRepository,
    settings: Settings,
}

impl UserService {
    /// Create a new UserService instance
    pub fn new(user_repository: UserRepository, settings: Settings) -> Self {
        Self {
            user_repository,
            settings,
        }
    }

    /// Register a new user or get existing user
    pub async fn register_or_get_user(&self, telegram_id: i64, username: Option<String>, first_name: Option<String>, last_name: Option<String>) -> Result<User> {
        debug!(telegram_id = telegram_id, "Attempting to register or get user");

        // Check if user already exists
        if let Some(existing_user) = self.user_repository.find_by_telegram_id(telegram_id).await? {
            info!(user_id = existing_user.id, telegram_id = telegram_id, "User already exists, returning existing user");
            return Ok(existing_user);
        }

        // Create new user
        let create_request = CreateUserRequest {
            telegram_id,
            username,
            first_name,
            last_name,
            language_code: Some(self.settings.i18n.default_language.clone()),
            location: None,
        };

        let user = self.user_repository.create(create_request).await?;
        info!(user_id = user.id, telegram_id = telegram_id, "New user registered successfully");
        
        Ok(user)
    }

    /// Get user by Telegram ID
    pub async fn get_user_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>> {
        debug!(telegram_id = telegram_id, "Getting user by Telegram ID");
        self.user_repository.find_by_telegram_id(telegram_id).await
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>> {
        debug!(user_id = user_id, "Getting user by ID");
        self.user_repository.find_by_id(user_id).await
    }

    /// Update user profile
    pub async fn update_user_profile(&self, telegram_id: i64, update_request: UpdateUserRequest) -> Result<User> {
        debug!(telegram_id = telegram_id, "Updating user profile");
        
        // First get the user by telegram_id to get the internal user_id
        let existing_user = self.user_repository.find_by_telegram_id(telegram_id).await?
            .ok_or_else(|| SwingBuddyError::UserNotFound { user_id: telegram_id })?;
        
        let user = self.user_repository.update(existing_user.id, update_request).await?;
        info!(telegram_id = telegram_id, user_id = existing_user.id, "User profile updated successfully");
        
        Ok(user)
    }

    /// Set user language preference
    pub async fn set_language_preference(&self, telegram_id: i64, language_code: String) -> Result<User> {
        debug!(telegram_id = telegram_id, language_code = %language_code, "Setting user language preference");

        // Validate language code
        if !self.settings.i18n.supported_languages.contains(&language_code) {
            warn!(telegram_id = telegram_id, language_code = %language_code, "Unsupported language code");
            return Err(SwingBuddyError::InvalidInput(format!("Unsupported language: {}", language_code)));
        }

        // First get the user by telegram_id to get the internal user_id
        let existing_user = self.user_repository.find_by_telegram_id(telegram_id).await?
            .ok_or_else(|| SwingBuddyError::UserNotFound { user_id: telegram_id })?;

        let update_request = UpdateUserRequest {
            language_code: Some(language_code.clone()),
            ..Default::default()
        };

        let user = self.user_repository.update(existing_user.id, update_request).await?;
        info!(telegram_id = telegram_id, user_id = existing_user.id, language_code = %language_code, "User language preference updated");
        
        Ok(user)
    }

    /// Set user location with city suggestions
    pub async fn set_user_location(&self, telegram_id: i64, location: String) -> Result<User> {
        debug!(telegram_id = telegram_id, location = %location, "Setting user location");

        // Normalize location (basic city suggestions for Moscow and Saint Petersburg)
        let normalized_location = self.normalize_location(&location);

        // First get the user by telegram_id to get the internal user_id
        let existing_user = self.user_repository.find_by_telegram_id(telegram_id).await?
            .ok_or_else(|| SwingBuddyError::UserNotFound { user_id: telegram_id })?;

        let update_request = UpdateUserRequest {
            location: Some(normalized_location.clone()),
            ..Default::default()
        };

        let user = self.user_repository.update(existing_user.id, update_request).await?;
        info!(telegram_id = telegram_id, user_id = existing_user.id, location = %normalized_location, "User location updated");
        
        Ok(user)
    }

    /// Get city suggestions based on input
    pub fn get_city_suggestions(&self, input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();
        let mut suggestions = Vec::new();

        // Predefined cities with variations
        let cities = vec![
            ("Moscow", vec!["moscow", "москва", "msk", "мск"]),
            ("Saint Petersburg", vec!["saint petersburg", "st petersburg", "petersburg", "санкт-петербург", "спб", "питер"]),
        ];

        for (city, variations) in cities {
            for variation in variations {
                if variation.contains(&input_lower) || input_lower.contains(variation) {
                    suggestions.push(city.to_string());
                    break;
                }
            }
        }

        suggestions
    }

    /// Check if user needs onboarding
    pub async fn needs_onboarding(&self, telegram_id: i64) -> Result<bool> {
        debug!(telegram_id = telegram_id, "Checking if user needs onboarding");

        let user = self.user_repository.find_by_telegram_id(telegram_id).await?
            .ok_or(SwingBuddyError::UserNotFound { user_id: telegram_id })?;

        // User needs onboarding if they don't have a location set
        let needs_onboarding = user.location.is_none();
        
        debug!(telegram_id = telegram_id, user_id = user.id, needs_onboarding = needs_onboarding, "Onboarding check completed");
        Ok(needs_onboarding)
    }

    /// Complete user onboarding
    pub async fn complete_onboarding(&self, telegram_id: i64, language_code: Option<String>, location: Option<String>) -> Result<User> {
        info!(telegram_id = telegram_id, "Completing user onboarding");

        // First get the user by telegram_id to get the internal user_id
        let existing_user = self.user_repository.find_by_telegram_id(telegram_id).await?
            .ok_or_else(|| SwingBuddyError::UserNotFound { user_id: telegram_id })?;

        let mut update_request = UpdateUserRequest::default();

        if let Some(lang) = language_code {
            if self.settings.i18n.supported_languages.contains(&lang) {
                update_request.language_code = Some(lang);
            }
        }

        if let Some(loc) = location {
            update_request.location = Some(self.normalize_location(&loc));
        }

        let user = self.user_repository.update(existing_user.id, update_request).await?;
        info!(telegram_id = telegram_id, user_id = existing_user.id, "User onboarding completed successfully");
        
        Ok(user)
    }

    /// Ban or unban user
    pub async fn set_user_ban_status(&self, telegram_id: i64, is_banned: bool, admin_id: i64) -> Result<User> {
        info!(telegram_id = telegram_id, is_banned = is_banned, admin_id = admin_id, "Setting user ban status");

        // First get the user by telegram_id to get the internal user_id
        let existing_user = self.user_repository.find_by_telegram_id(telegram_id).await?
            .ok_or_else(|| SwingBuddyError::UserNotFound { user_id: telegram_id })?;

        let user = self.user_repository.set_ban_status(existing_user.id, is_banned).await?;
        
        if is_banned {
            warn!(telegram_id = telegram_id, user_id = existing_user.id, admin_id = admin_id, "User banned");
        } else {
            info!(telegram_id = telegram_id, user_id = existing_user.id, admin_id = admin_id, "User unbanned");
        }
        
        Ok(user)
    }

    /// Get banned users
    pub async fn get_banned_users(&self) -> Result<Vec<User>> {
        debug!("Getting banned users");
        self.user_repository.get_banned_users().await
    }

    /// Search users by username pattern
    pub async fn search_users_by_username(&self, pattern: &str) -> Result<Vec<User>> {
        debug!(pattern = %pattern, "Searching users by username pattern");
        
        if pattern.len() < 2 {
            return Err(SwingBuddyError::InvalidInput("Search pattern must be at least 2 characters".to_string()));
        }

        self.user_repository.find_by_username_pattern(pattern).await
    }

    /// Get user statistics
    pub async fn get_user_statistics(&self) -> Result<HashMap<String, i64>> {
        debug!("Getting user statistics");

        let total_users = self.user_repository.count().await?;
        let banned_users = self.user_repository.get_banned_users().await?.len() as i64;

        let mut stats = HashMap::new();
        stats.insert("total_users".to_string(), total_users);
        stats.insert("banned_users".to_string(), banned_users);
        stats.insert("active_users".to_string(), total_users - banned_users);

        Ok(stats)
    }

    /// List users with pagination
    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        debug!(limit = limit, offset = offset, "Listing users with pagination");
        
        if limit > 100 {
            return Err(SwingBuddyError::InvalidInput("Limit cannot exceed 100".to_string()));
        }

        self.user_repository.list(limit, offset).await
    }

    /// Normalize location input
    fn normalize_location(&self, location: &str) -> String {
        let location_lower = location.trim().to_lowercase();
        
        // Moscow variations
        if location_lower.contains("moscow") || location_lower.contains("москва") || 
           location_lower == "msk" || location_lower == "мск" {
            return "Moscow".to_string();
        }
        
        // Saint Petersburg variations
        if location_lower.contains("saint petersburg") || location_lower.contains("st petersburg") ||
           location_lower.contains("petersburg") || location_lower.contains("санкт-петербург") ||
           location_lower == "спб" || location_lower.contains("питер") {
            return "Saint Petersburg".to_string();
        }
        
        // Return original if no match found, but capitalize first letter
        let mut chars: Vec<char> = location.trim().chars().collect();
        if !chars.is_empty() {
            chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
        }
        chars.into_iter().collect()
    }
}

impl Default for UpdateUserRequest {
    fn default() -> Self {
        Self {
            username: None,
            first_name: None,
            last_name: None,
            language_code: None,
            location: None,
            is_banned: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_normalize_location() {
        let settings = Settings::default();
        let user_repo = UserRepository::new(sqlx::PgPool::connect("postgresql://test").await.unwrap());
        let service = UserService::new(user_repo, settings);

        assert_eq!(service.normalize_location("moscow"), "Moscow");
        assert_eq!(service.normalize_location("МОСКВА"), "Moscow");
        assert_eq!(service.normalize_location("msk"), "Moscow");
        assert_eq!(service.normalize_location("saint petersburg"), "Saint Petersburg");
        assert_eq!(service.normalize_location("спб"), "Saint Petersburg");
        assert_eq!(service.normalize_location("питер"), "Saint Petersburg");
        assert_eq!(service.normalize_location("other city"), "Other city");
    }

    #[tokio::test]
    async fn test_get_city_suggestions() {
        let settings = Settings::default();
        let user_repo = UserRepository::new(sqlx::PgPool::connect("postgresql://test").await.unwrap());
        let service = UserService::new(user_repo, settings);

        let suggestions = service.get_city_suggestions("mos");
        assert!(suggestions.contains(&"Moscow".to_string()));

        let suggestions = service.get_city_suggestions("петер");
        assert!(suggestions.contains(&"Saint Petersburg".to_string()));
    }
}