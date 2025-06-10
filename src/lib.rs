//! SwingBuddy Telegram Bot
//!
//! A comprehensive Telegram bot for swing dancing community management.
//! This library provides modular components for user management, event organization,
//! group administration, and community moderation with multi-language support.

#![allow(non_snake_case)]

pub mod config;
pub mod handlers;
pub mod services;
pub mod models;
pub mod database;
pub mod state;
pub mod i18n;
pub mod utils;
pub mod middleware;

// Re-export commonly used types
pub use config::Settings;
pub use utils::errors::{SwingBuddyError, Result};

// Re-export main components for easy access
pub use database::DatabaseService;
pub use services::ServiceFactory;
pub use state::{ScenarioManager, StateStorage};
pub use i18n::I18n;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Get library information
pub fn info() -> String {
    format!("{} v{}", NAME, VERSION)
}