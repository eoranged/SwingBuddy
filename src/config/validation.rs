//! Configuration validation module
//! 
//! This module provides validation functions for application configuration
//! to ensure all required settings are properly configured.

use crate::utils::errors::{SwingBuddyError, Result};
use super::Settings;

/// Validate all configuration settings
pub fn validate_settings(settings: &Settings) -> Result<()> {
    validate_bot_config(&settings.bot)?;
    validate_database_config(&settings.database)?;
    validate_redis_config(&settings.redis)?;
    validate_cas_config(&settings.cas)?;
    validate_i18n_config(&settings.i18n)?;
    validate_logging_config(&settings.logging)?;
    
    if let Some(ref google_config) = settings.google {
        validate_google_config(google_config)?;
    }
    
    Ok(())
}

/// Validate bot configuration
fn validate_bot_config(config: &super::BotConfig) -> Result<()> {
    if config.token.is_empty() {
        return Err(SwingBuddyError::Config(
            "Bot token is required".to_string()
        ));
    }
    
    if config.admin_ids.is_empty() {
        return Err(SwingBuddyError::Config(
            "At least one admin ID must be configured".to_string()
        ));
    }
    
    Ok(())
}

/// Validate database configuration
fn validate_database_config(config: &super::DatabaseConfig) -> Result<()> {
    if config.url.is_empty() {
        return Err(SwingBuddyError::Config(
            "Database URL is required".to_string()
        ));
    }
    
    if config.max_connections == 0 {
        return Err(SwingBuddyError::Config(
            "Max connections must be greater than 0".to_string()
        ));
    }
    
    if config.min_connections > config.max_connections {
        return Err(SwingBuddyError::Config(
            "Min connections cannot be greater than max connections".to_string()
        ));
    }
    
    Ok(())
}

/// Validate Redis configuration
fn validate_redis_config(config: &super::RedisConfig) -> Result<()> {
    if config.url.is_empty() {
        return Err(SwingBuddyError::Config(
            "Redis URL is required".to_string()
        ));
    }
    
    Ok(())
}

/// Validate Google Calendar configuration
fn validate_google_config(config: &super::GoogleConfig) -> Result<()> {
    if config.service_account_path.is_empty() {
        return Err(SwingBuddyError::Config(
            "Google service account path is required".to_string()
        ));
    }
    
    if config.calendar_id.is_empty() {
        return Err(SwingBuddyError::Config(
            "Google calendar ID is required".to_string()
        ));
    }
    
    Ok(())
}

/// Validate CAS configuration
fn validate_cas_config(config: &super::CasConfig) -> Result<()> {
    if config.api_url.is_empty() {
        return Err(SwingBuddyError::Config(
            "CAS API URL is required".to_string()
        ));
    }
    
    if config.timeout_seconds == 0 {
        return Err(SwingBuddyError::Config(
            "CAS timeout must be greater than 0".to_string()
        ));
    }
    
    Ok(())
}

/// Validate internationalization configuration
fn validate_i18n_config(config: &super::I18nConfig) -> Result<()> {
    if config.default_language.is_empty() {
        return Err(SwingBuddyError::Config(
            "Default language is required".to_string()
        ));
    }
    
    if config.supported_languages.is_empty() {
        return Err(SwingBuddyError::Config(
            "At least one supported language is required".to_string()
        ));
    }
    
    if !config.supported_languages.contains(&config.default_language) {
        return Err(SwingBuddyError::Config(
            "Default language must be in supported languages list".to_string()
        ));
    }
    
    Ok(())
}

/// Validate logging configuration
fn validate_logging_config(config: &super::LoggingConfig) -> Result<()> {
    if config.level.is_empty() {
        return Err(SwingBuddyError::Config(
            "Log level is required".to_string()
        ));
    }
    
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&config.level.as_str()) {
        return Err(SwingBuddyError::Config(
            format!("Invalid log level: {}. Valid levels: {:?}", config.level, valid_levels)
        ));
    }
    
    Ok(())
}