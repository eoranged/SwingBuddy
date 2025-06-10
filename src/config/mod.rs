//! Configuration management module
//! 
//! This module handles loading and validation of application configuration
//! from TOML files and environment variables.

pub mod settings;
pub mod validation;

pub use settings::{Settings, I18nConfig, BotConfig, DatabaseConfig, RedisConfig, GoogleConfig, CasConfig, LoggingConfig, FeaturesConfig};