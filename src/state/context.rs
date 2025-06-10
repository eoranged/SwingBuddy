//! Conversation context management
//! 
//! This module handles user conversation context, tracking current scenarios,
//! steps, and associated data for each user's interaction with the bot.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use crate::Settings;
use crate::ServiceFactory;
use crate::utils::errors::{SwingBuddyError, Result};
use crate::{
    DatabaseService,
    services::{
        RedisService,
        AuthService,
        NotificationService,
        CasService,
        GoogleCalendarService
    },
    i18n::I18n,
    state::scenarios::ScenarioManager,
    state::storage::StateStorage
};

/// Application-wide context containing services and settings
#[derive(Debug, Clone)]
pub struct AppContext {
    pub settings: Settings,
    pub database: Arc<DatabaseService>,
    pub redis: Option<Arc<RedisService>>,
    pub user_service: Arc<crate::services::user::UserService>,
    pub auth_service: Arc<AuthService>,
    pub notification_service: Arc<NotificationService>,
    pub cas_service: Arc<CasService>,
    pub google_service: Option<Arc<GoogleCalendarService>>,
    pub scenario_manager: Arc<crate::state::scenarios::ScenarioManager>,
    pub state_storage: Arc<crate::state::storage::StateStorage>,
    pub services: Arc<ServiceFactory>,
    pub i18n: Arc<crate::i18n::I18n>,
}

impl AppContext {
    /// Create a new AppContext from services
    pub fn new(
        settings: Settings,
        database: Arc<DatabaseService>,
        redis: Option<Arc<RedisService>>,
        user_service: Arc<crate::services::user::UserService>,
        auth_service: Arc<AuthService>,
        notification_service: Arc<NotificationService>,
        cas_service: Arc<CasService>,
        google_service: Option<Arc<GoogleCalendarService>>,
        scenario_manager: Arc<ScenarioManager>,
        state_storage: Arc<StateStorage>,
        services: Arc<ServiceFactory>,
        i18n: Arc<I18n>,
    ) -> Self {
        Self {
            settings,
            database,
            redis,
            user_service,
            auth_service,
            notification_service,
            cas_service,
            google_service,
            scenario_manager,
            state_storage,
            services,
            i18n,
        }
    }

    /// Create from ServiceFactory and DatabaseService
    pub async fn from_factory(factory: ServiceFactory, database: Arc<DatabaseService>, settings: Settings) -> Result<Self> {
        // Create scenario manager
        let scenario_manager = Arc::new(ScenarioManager::new());
        
        // Create state storage from settings (async)
        let state_storage = Arc::new(StateStorage::new(settings.redis.clone()).await?);
        
        // Create I18n from settings and load translations
        let mut i18n_loader = crate::i18n::I18n::new(&settings.i18n);
        i18n_loader.load_translations().await?;
        let i18n = Arc::new(i18n_loader);
        
        // Create services Arc
        let services = Arc::new(factory.clone());
        
        Ok(Self {
            redis: Some(Arc::new(factory.redis_service.clone())),
            user_service: Arc::new(factory.user_service.clone()),
            auth_service: Arc::new(factory.auth_service.clone()),
            notification_service: Arc::new(factory.notification_service.clone()),
            cas_service: Arc::new(factory.cas_service.clone()),
            google_service: Some(Arc::new(factory.google_service.clone())),
            scenario_manager,
            state_storage,
            services,
            i18n,
            settings,
            database,
        })
    }
}

/// User conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    /// User ID this context belongs to
    pub user_id: i64,
    /// Current scenario the user is in
    pub scenario: Option<String>,
    /// Current step within the scenario
    pub step: Option<String>,
    /// Scenario-specific data
    pub data: HashMap<String, serde_json::Value>,
    /// When this context expires (for cleanup)
    pub expires_at: Option<DateTime<Utc>>,
    /// When this context was last updated
    pub updated_at: DateTime<Utc>,
}

impl ConversationContext {
    /// Create a new conversation context for a user
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            scenario: None,
            step: None,
            data: HashMap::new(),
            expires_at: None,
            updated_at: Utc::now(),
        }
    }

    /// Start a new scenario
    pub fn start_scenario(&mut self, scenario: &str, initial_step: &str) -> Result<()> {
        self.scenario = Some(scenario.to_string());
        self.step = Some(initial_step.to_string());
        self.data.clear();
        self.updated_at = Utc::now();
        self.expires_at = Some(Utc::now() + Duration::hours(24)); // Default 24h expiry
        Ok(())
    }

    /// Move to the next step in the current scenario
    pub fn next_step(&mut self, step: &str) -> Result<()> {
        if self.scenario.is_none() {
            return Err(SwingBuddyError::InvalidStateTransition {
                from: "no_scenario".to_string(),
                to: step.to_string(),
            });
        }
        
        self.step = Some(step.to_string());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Complete the current scenario and clear context
    pub fn complete_scenario(&mut self) {
        self.scenario = None;
        self.step = None;
        self.data.clear();
        self.expires_at = None;
        self.updated_at = Utc::now();
    }

    /// Cancel the current scenario
    pub fn cancel_scenario(&mut self) {
        self.complete_scenario();
    }

    /// Set data for the current context
    pub fn set_data<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        let json_value = serde_json::to_value(value)?;
        self.data.insert(key.to_string(), json_value);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get data from the current context
    pub fn get_data<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        if let Some(value) = self.data.get(key) {
            let result: T = serde_json::from_value(value.clone())?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Get string data (convenience method)
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_data::<String>(key).unwrap_or(None)
    }

    /// Get integer data (convenience method)
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_data::<i64>(key).unwrap_or(None)
    }

    /// Get boolean data (convenience method)
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_data::<bool>(key).unwrap_or(None)
    }

    /// Remove data from context
    pub fn remove_data(&mut self, key: &str) -> Option<serde_json::Value> {
        self.updated_at = Utc::now();
        self.data.remove(key)
    }

    /// Check if context has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Extend the expiry time
    pub fn extend_expiry(&mut self, duration: Duration) {
        let new_expiry = self.expires_at
            .unwrap_or_else(Utc::now)
            .checked_add_signed(duration)
            .unwrap_or_else(|| Utc::now() + Duration::hours(24));
        
        self.expires_at = Some(new_expiry);
        self.updated_at = Utc::now();
    }

    /// Set custom expiry time
    pub fn set_expiry(&mut self, expires_at: DateTime<Utc>) {
        self.expires_at = Some(expires_at);
        self.updated_at = Utc::now();
    }

    /// Clear expiry (context won't expire automatically)
    pub fn clear_expiry(&mut self) {
        self.expires_at = None;
        self.updated_at = Utc::now();
    }

    /// Check if user is in a specific scenario
    pub fn is_in_scenario(&self, scenario: &str) -> bool {
        self.scenario.as_ref().map_or(false, |s| s == scenario)
    }

    /// Check if user is at a specific step
    pub fn is_at_step(&self, step: &str) -> bool {
        self.step.as_ref().map_or(false, |s| s == step)
    }

    /// Check if user is in a specific scenario and step
    pub fn is_at(&self, scenario: &str, step: &str) -> bool {
        self.is_in_scenario(scenario) && self.is_at_step(step)
    }

    /// Get current scenario and step as tuple
    pub fn current_state(&self) -> (Option<&str>, Option<&str>) {
        (
            self.scenario.as_deref(),
            self.step.as_deref(),
        )
    }

    /// Validate context data against expected schema
    pub fn validate_data(&self, required_keys: &[&str]) -> Result<()> {
        for key in required_keys {
            if !self.data.contains_key(*key) {
                return Err(SwingBuddyError::InvalidInput(
                    format!("Missing required context data: {}", key)
                ));
            }
        }
        Ok(())
    }

    /// Get all data keys
    pub fn data_keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// Check if context has any data
    pub fn has_data(&self) -> bool {
        !self.data.is_empty()
    }

    /// Get data count
    pub fn data_count(&self) -> usize {
        self.data.len()
    }

    /// Create a summary of the context for logging
    pub fn summary(&self) -> ContextSummary {
        ContextSummary {
            user_id: self.user_id,
            scenario: self.scenario.clone(),
            step: self.step.clone(),
            data_keys: self.data.keys().cloned().collect(),
            expires_at: self.expires_at,
            updated_at: self.updated_at,
        }
    }
}

/// Context summary for logging and debugging
#[derive(Debug, Clone, Serialize)]
pub struct ContextSummary {
    pub user_id: i64,
    pub scenario: Option<String>,
    pub step: Option<String>,
    pub data_keys: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Context validation rules
#[derive(Debug, Clone)]
pub struct ContextValidationRules {
    /// Maximum number of data entries
    pub max_data_entries: usize,
    /// Maximum size of individual data values (in bytes)
    pub max_data_value_size: usize,
    /// Maximum total context size (in bytes)
    pub max_total_size: usize,
    /// Maximum expiry duration
    pub max_expiry_duration: Duration,
}

impl Default for ContextValidationRules {
    fn default() -> Self {
        Self {
            max_data_entries: 50,
            max_data_value_size: 1024 * 10, // 10KB
            max_total_size: 1024 * 100,     // 100KB
            max_expiry_duration: Duration::days(7),
        }
    }
}

impl ConversationContext {
    /// Validate context against rules
    pub fn validate_against_rules(&self, rules: &ContextValidationRules) -> Result<()> {
        // Check data entry count
        if self.data.len() > rules.max_data_entries {
            return Err(SwingBuddyError::InvalidInput(
                format!("Too many data entries: {} > {}", self.data.len(), rules.max_data_entries)
            ));
        }

        // Check individual data value sizes
        for (key, value) in &self.data {
            let value_size = serde_json::to_string(value)?.len();
            if value_size > rules.max_data_value_size {
                return Err(SwingBuddyError::InvalidInput(
                    format!("Data value '{}' too large: {} > {}", key, value_size, rules.max_data_value_size)
                ));
            }
        }

        // Check total context size
        let total_size = serde_json::to_string(self)?.len();
        if total_size > rules.max_total_size {
            return Err(SwingBuddyError::InvalidInput(
                format!("Context too large: {} > {}", total_size, rules.max_total_size)
            ));
        }

        // Check expiry duration
        if let Some(expires_at) = self.expires_at {
            let duration = expires_at - Utc::now();
            if duration > rules.max_expiry_duration {
                return Err(SwingBuddyError::InvalidInput(
                    format!("Expiry duration too long: {} > {}", 
                           duration.num_seconds(), 
                           rules.max_expiry_duration.num_seconds())
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context() {
        let context = ConversationContext::new(123);
        assert_eq!(context.user_id, 123);
        assert!(context.scenario.is_none());
        assert!(context.step.is_none());
        assert!(context.data.is_empty());
        assert!(context.expires_at.is_none());
    }

    #[test]
    fn test_start_scenario() {
        let mut context = ConversationContext::new(123);
        context.start_scenario("onboarding", "language_selection").unwrap();
        
        assert_eq!(context.scenario, Some("onboarding".to_string()));
        assert_eq!(context.step, Some("language_selection".to_string()));
        assert!(context.expires_at.is_some());
    }

    #[test]
    fn test_data_operations() {
        let mut context = ConversationContext::new(123);
        
        // Set data
        context.set_data("name", "John").unwrap();
        context.set_data("age", 25).unwrap();
        context.set_data("active", true).unwrap();
        
        // Get data
        assert_eq!(context.get_string("name"), Some("John".to_string()));
        assert_eq!(context.get_i64("age"), Some(25));
        assert_eq!(context.get_bool("active"), Some(true));
        
        // Non-existent key
        assert_eq!(context.get_string("nonexistent"), None);
        
        // Remove data
        context.remove_data("age");
        assert_eq!(context.get_i64("age"), None);
    }

    #[test]
    fn test_scenario_checks() {
        let mut context = ConversationContext::new(123);
        context.start_scenario("onboarding", "language_selection").unwrap();
        
        assert!(context.is_in_scenario("onboarding"));
        assert!(!context.is_in_scenario("admin"));
        assert!(context.is_at_step("language_selection"));
        assert!(!context.is_at_step("name_input"));
        assert!(context.is_at("onboarding", "language_selection"));
        assert!(!context.is_at("onboarding", "name_input"));
    }

    #[test]
    fn test_expiry() {
        let mut context = ConversationContext::new(123);
        
        // Set expiry in the past
        context.set_expiry(Utc::now() - Duration::hours(1));
        assert!(context.is_expired());
        
        // Set expiry in the future
        context.set_expiry(Utc::now() + Duration::hours(1));
        assert!(!context.is_expired());
        
        // Clear expiry
        context.clear_expiry();
        assert!(!context.is_expired());
    }

    #[test]
    fn test_validation() {
        let context = ConversationContext::new(123);
        let rules = ContextValidationRules::default();
        
        // Empty context should be valid
        assert!(context.validate_against_rules(&rules).is_ok());
    }
}