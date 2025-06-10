//! Translation loader and i18n management
//! 
//! This module provides the core internationalization functionality including
//! translation loading, caching, language detection, and message formatting.

use std::collections::HashMap;
use std::path::Path;
use serde_json::{Value, Map};
use tokio::fs;
use tracing::{info, warn, error, debug};
use crate::utils::errors::{SwingBuddyError, Result};
use crate::config::I18nConfig;

/// Main internationalization manager
#[derive(Debug, Clone)]
pub struct I18n {
    /// Loaded translations by language code
    translations: HashMap<String, Map<String, Value>>,
    /// Default language code
    default_language: String,
    /// Supported language codes
    supported_languages: Vec<String>,
}

/// Translation parameters for message formatting
pub type TranslationParams = HashMap<String, String>;

impl I18n {
    /// Create a new I18n instance
    pub fn new(config: &I18nConfig) -> Self {
        Self {
            translations: HashMap::new(),
            default_language: config.default_language.clone(),
            supported_languages: config.supported_languages.clone(),
        }
    }

    /// Load all translation files from the translations directory
    pub async fn load_translations(&mut self) -> Result<()> {
        let translations_dir = Path::new("translations");
        
        if !translations_dir.exists() {
            warn!("Translations directory not found, creating it");
            fs::create_dir_all(translations_dir).await?;
        }

        let supported_languages = self.supported_languages.clone();
        for lang_code in &supported_languages {
            let file_path = translations_dir.join(format!("{}.json", lang_code));
            
            if file_path.exists() {
                match self.load_language_file(&file_path, lang_code).await {
                    Ok(_) => info!("Loaded translations for language: {}", lang_code),
                    Err(e) => {
                        error!("Failed to load translations for {}: {}", lang_code, e);
                        if lang_code == &self.default_language {
                            return Err(SwingBuddyError::Config(
                                format!("Failed to load default language translations: {}", e)
                            ));
                        }
                    }
                }
            } else {
                warn!("Translation file not found: {}", file_path.display());
                if lang_code == &self.default_language {
                    return Err(SwingBuddyError::Config(
                        format!("Default language translation file not found: {}", file_path.display())
                    ));
                }
            }
        }

        Ok(())
    }

    /// Load a single language file
    async fn load_language_file(&mut self, file_path: &Path, lang_code: &str) -> Result<()> {
        let content = fs::read_to_string(file_path).await?;
        let translations: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(map) = translations {
            self.translations.insert(lang_code.to_string(), map);
            debug!("Loaded {} translation keys for {}", 
                   self.translations.get(lang_code).unwrap().len(), 
                   lang_code);
        } else {
            return Err(SwingBuddyError::Config(
                format!("Invalid translation file format for {}", lang_code)
            ));
        }

        Ok(())
    }

    /// Get a translated message
    pub fn t(&self, key: &str, lang: &str, params: Option<&TranslationParams>) -> String {
        let effective_lang = self.get_effective_language(lang);
        
        match self.get_translation_value(key, &effective_lang) {
            Some(translation) => {
                let text = self.extract_text_from_value(&translation);
                self.format_message(&text, params)
            }
            None => {
                // Fallback to default language if not found
                if effective_lang != self.default_language {
                    match self.get_translation_value(key, &self.default_language) {
                        Some(translation) => {
                            let text = self.extract_text_from_value(&translation);
                            self.format_message(&text, params)
                        }
                        None => {
                            warn!("Translation key '{}' not found in any language", key);
                            key.to_string()
                        }
                    }
                } else {
                    warn!("Translation key '{}' not found in default language", key);
                    key.to_string()
                }
            }
        }
    }

    /// Get a translated message with pluralization support
    pub fn tp(&self, key: &str, lang: &str, count: i32, params: Option<&TranslationParams>) -> String {
        let effective_lang = self.get_effective_language(lang);
        let plural_key = self.get_plural_key(key, count, &effective_lang);
        
        let mut final_params = params.cloned().unwrap_or_default();
        final_params.insert("count".to_string(), count.to_string());
        
        self.t(&plural_key, &effective_lang, Some(&final_params))
    }

    /// Check if a language is supported
    pub fn is_language_supported(&self, lang: &str) -> bool {
        self.supported_languages.contains(&lang.to_string())
    }

    /// Get the effective language (fallback to default if not supported)
    fn get_effective_language(&self, lang: &str) -> String {
        if self.is_language_supported(lang) && self.translations.contains_key(lang) {
            lang.to_string()
        } else {
            self.default_language.clone()
        }
    }

    /// Get translation value from nested JSON structure
    fn get_translation_value(&self, key: &str, lang: &str) -> Option<Value> {
        let translations = self.translations.get(lang)?;
        
        // Support nested keys like "commands.start.welcome"
        let keys: Vec<&str> = key.split('.').collect();
        let mut current = Value::Object(translations.clone());
        
        for k in keys {
            current = current.get(k)?.clone();
        }
        
        Some(current)
    }

    /// Extract text from JSON value (handle both strings and objects with pluralization)
    fn extract_text_from_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Object(obj) => {
                // For pluralization objects, default to "other" or first available key
                if let Some(other) = obj.get("other") {
                    self.extract_text_from_value(other)
                } else if let Some((_, first_value)) = obj.iter().next() {
                    self.extract_text_from_value(first_value)
                } else {
                    String::new()
                }
            }
            _ => value.to_string(),
        }
    }

    /// Format message with parameters
    fn format_message(&self, template: &str, params: Option<&TranslationParams>) -> String {
        if let Some(params) = params {
            let mut result = template.to_string();
            for (key, value) in params {
                let placeholder = format!("{{{}}}", key);
                result = result.replace(&placeholder, value);
            }
            result
        } else {
            template.to_string()
        }
    }

    /// Get the appropriate plural key based on count and language rules
    fn get_plural_key(&self, base_key: &str, count: i32, lang: &str) -> String {
        let plural_form = self.get_plural_form(count, lang);
        format!("{}.{}", base_key, plural_form)
    }

    /// Determine plural form based on language-specific rules
    fn get_plural_form(&self, count: i32, lang: &str) -> &'static str {
        match lang {
            "en" => {
                // English: one, other
                if count == 1 { "one" } else { "other" }
            }
            "ru" => {
                // Russian: one, few, many, other
                let abs_count = count.abs();
                let last_digit = abs_count % 10;
                let last_two_digits = abs_count % 100;
                
                if last_digit == 1 && last_two_digits != 11 {
                    "one"
                } else if (2..=4).contains(&last_digit) && !(12..=14).contains(&last_two_digits) {
                    "few"
                } else if last_digit == 0 || (5..=9).contains(&last_digit) || (11..=14).contains(&last_two_digits) {
                    "many"
                } else {
                    "other"
                }
            }
            _ => {
                // Default to English rules
                if count == 1 { "one" } else { "other" }
            }
        }
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> &[String] {
        &self.supported_languages
    }

    /// Get default language
    pub fn default_language(&self) -> &str {
        &self.default_language
    }

    /// Detect user language from Telegram language code
    pub fn detect_user_language(&self, telegram_lang: Option<&str>) -> String {
        if let Some(lang) = telegram_lang {
            // Extract language code from locale (e.g., "en-US" -> "en")
            let lang_code = lang.split('-').next().unwrap_or(lang);
            
            if self.is_language_supported(lang_code) {
                return lang_code.to_string();
            }
        }
        
        self.default_language.clone()
    }

    /// Reload translations (useful for development or dynamic updates)
    pub async fn reload_translations(&mut self) -> Result<()> {
        self.translations.clear();
        self.load_translations().await
    }

    /// Get translation statistics
    pub fn get_stats(&self) -> TranslationStats {
        let mut stats = TranslationStats {
            languages: Vec::new(),
            total_keys: 0,
        };

        for (lang, translations) in &self.translations {
            let key_count = self.count_keys(translations);
            stats.languages.push(LanguageStats {
                code: lang.clone(),
                key_count,
            });
            if lang == &self.default_language {
                stats.total_keys = key_count;
            }
        }

        stats
    }

    /// Recursively count translation keys
    fn count_keys(&self, obj: &Map<String, Value>) -> usize {
        let mut count = 0;
        for value in obj.values() {
            match value {
                Value::Object(nested) => count += self.count_keys(nested),
                _ => count += 1,
            }
        }
        count
    }
}

/// Translation statistics
#[derive(Debug, Clone)]
pub struct TranslationStats {
    pub languages: Vec<LanguageStats>,
    pub total_keys: usize,
}

/// Language-specific statistics
#[derive(Debug, Clone)]
pub struct LanguageStats {
    pub code: String,
    pub key_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::I18nConfig;

    fn create_test_config() -> I18nConfig {
        I18nConfig {
            default_language: "en".to_string(),
            supported_languages: vec!["en".to_string(), "ru".to_string()],
        }
    }

    #[test]
    fn test_plural_form_english() {
        let config = create_test_config();
        let i18n = I18n::new(&config);
        
        assert_eq!(i18n.get_plural_form(0, "en"), "other");
        assert_eq!(i18n.get_plural_form(1, "en"), "one");
        assert_eq!(i18n.get_plural_form(2, "en"), "other");
        assert_eq!(i18n.get_plural_form(5, "en"), "other");
    }

    #[test]
    fn test_plural_form_russian() {
        let config = create_test_config();
        let i18n = I18n::new(&config);
        
        assert_eq!(i18n.get_plural_form(1, "ru"), "one");
        assert_eq!(i18n.get_plural_form(2, "ru"), "few");
        assert_eq!(i18n.get_plural_form(5, "ru"), "many");
        assert_eq!(i18n.get_plural_form(11, "ru"), "many");
        assert_eq!(i18n.get_plural_form(21, "ru"), "one");
    }

    #[test]
    fn test_language_detection() {
        let config = create_test_config();
        let i18n = I18n::new(&config);
        
        assert_eq!(i18n.detect_user_language(Some("en-US")), "en");
        assert_eq!(i18n.detect_user_language(Some("ru")), "ru");
        assert_eq!(i18n.detect_user_language(Some("fr")), "en"); // fallback
        assert_eq!(i18n.detect_user_language(None), "en"); // fallback
    }

    #[test]
    fn test_message_formatting() {
        let config = create_test_config();
        let i18n = I18n::new(&config);
        
        let mut params = HashMap::new();
        params.insert("name".to_string(), "John".to_string());
        params.insert("count".to_string(), "5".to_string());
        
        let result = i18n.format_message("Hello {name}, you have {count} messages", Some(&params));
        assert_eq!(result, "Hello John, you have 5 messages");
    }
}