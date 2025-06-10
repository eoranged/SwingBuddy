//! Internationalization module
//!
//! This module handles multi-language support for the SwingBuddy bot.
//! It provides translation loading, language detection, message formatting,
//! and pluralization support for multiple languages.

pub mod loader;

// Re-export commonly used i18n components
pub use loader::{I18n, TranslationParams, TranslationStats, LanguageStats};