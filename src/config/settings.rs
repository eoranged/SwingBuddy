//! Application settings management
//! 
//! This module defines the configuration structure and provides methods
//! for loading settings from TOML files and environment variables.

use serde::{Deserialize, Serialize};

/// Main application configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub bot: BotConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub google: Option<GoogleConfig>,
    pub cas: CasConfig,
    pub i18n: I18nConfig,
    pub logging: LoggingConfig,
    pub features: FeaturesConfig,
}

/// Telegram bot configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotConfig {
    pub token: String,
    pub webhook_url: Option<String>,
    pub admin_ids: Vec<i64>,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub prefix: String,
    pub ttl_seconds: u64,
}

/// Google Calendar configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleConfig {
    pub service_account_path: String,
    pub calendar_id: String,
}

/// CAS API configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CasConfig {
    pub api_url: String,
    pub timeout_seconds: u64,
    pub auto_ban: bool,
}

/// Internationalization configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct I18nConfig {
    pub default_language: String,
    pub supported_languages: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size: String,
    pub max_files: u32,
}

/// Feature flags configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeaturesConfig {
    pub cas_protection: bool,
    pub google_calendar: bool,
    pub admin_panel: bool,
}

impl Settings {
    /// Load settings from configuration file and environment variables
    pub fn new() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("SWINGBUDDY"))
            .build()?;

        settings.try_deserialize()
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), crate::utils::errors::SwingBuddyError> {
        super::validation::validate_settings(self)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bot: BotConfig {
                token: String::new(),
                webhook_url: None,
                admin_ids: vec![],
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/swingbuddy".to_string(),
                max_connections: 10,
                min_connections: 1,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                prefix: "swingbuddy:".to_string(),
                ttl_seconds: 3600,
            },
            google: None,
            cas: CasConfig {
                api_url: "https://api.cas.chat".to_string(),
                timeout_seconds: 5,
                auto_ban: true,
            },
            i18n: I18nConfig {
                default_language: "en".to_string(),
                supported_languages: vec!["en".to_string(), "ru".to_string()],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "/var/log/swingbuddy.log".to_string(),
                max_file_size: "10MB".to_string(),
                max_files: 5,
            },
            features: FeaturesConfig {
                cas_protection: true,
                google_calendar: false,
                admin_panel: true,
            },
        }
    }
}