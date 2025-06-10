//! Notification service implementation
//! 
//! This service handles message formatting and sending, multi-language message support,
//! bulk notification handling, message templating system, and integration with teloxide
//! for message sending.

use std::collections::HashMap;
use teloxide::{Bot, types::{ChatId, Message, ParseMode}, requests::Requester, prelude::Request, payloads::SendMessageSetters, sugar::request::RequestLinkPreviewExt};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use crate::config::settings::Settings;
use crate::models::{User, Event, Group};
use crate::utils::errors::{SwingBuddyError, Result};

/// Message template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplate {
    pub key: String,
    pub content: HashMap<String, String>, // language -> content mapping
    pub parse_mode: Option<ParseMode>,
    pub disable_web_page_preview: bool,
}

/// Notification request structure
#[derive(Debug, Clone)]
pub struct NotificationRequest {
    pub chat_id: ChatId,
    pub template_key: String,
    pub language: String,
    pub parameters: HashMap<String, String>,
    pub parse_mode: Option<ParseMode>,
    pub disable_web_page_preview: bool,
}

/// Bulk notification request structure
#[derive(Debug, Clone)]
pub struct BulkNotificationRequest {
    pub chat_ids: Vec<ChatId>,
    pub template_key: String,
    pub language: String,
    pub parameters: HashMap<String, String>,
    pub parse_mode: Option<ParseMode>,
    pub disable_web_page_preview: bool,
}

/// Notification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_sent: u64,
    pub total_failed: u64,
    pub sent_by_language: HashMap<String, u64>,
    pub sent_by_template: HashMap<String, u64>,
}

/// Notification service for message handling
#[derive(Clone)]
pub struct NotificationService {
    bot: Bot,
    settings: Settings,
    templates: HashMap<String, MessageTemplate>,
    stats: NotificationStats,
}

impl NotificationService {
    /// Create a new NotificationService instance
    pub fn new(bot: Bot, settings: Settings) -> Self {
        let templates = Self::load_default_templates();
        let stats = NotificationStats {
            total_sent: 0,
            total_failed: 0,
            sent_by_language: HashMap::new(),
            sent_by_template: HashMap::new(),
        };

        Self {
            bot,
            settings,
            templates,
            stats,
        }
    }

    /// Send a notification using a template
    pub async fn send_notification(&mut self, request: NotificationRequest) -> Result<Message> {
        debug!(chat_id = ?request.chat_id, template_key = %request.template_key, "Sending notification");

        let message_text = self.format_message(&request.template_key, &request.language, &request.parameters)?;
        
        let mut send_request = self.bot.send_message(request.chat_id, message_text);
        
        if let Some(parse_mode) = request.parse_mode {
            send_request = send_request.parse_mode(parse_mode);
        }
        
        if request.disable_web_page_preview {
            send_request = send_request.disable_link_preview(true);
        }

        match send_request.send().await {
            Ok(message) => {
                self.update_stats_success(&request.template_key, &request.language);
                info!(chat_id = ?request.chat_id, template_key = %request.template_key, "Notification sent successfully");
                Ok(message)
            }
            Err(e) => {
                self.update_stats_failure();
                error!(chat_id = ?request.chat_id, template_key = %request.template_key, error = %e, "Failed to send notification");
                Err(SwingBuddyError::Telegram(e))
            }
        }
    }

    /// Send bulk notifications
    pub async fn send_bulk_notifications(&mut self, request: BulkNotificationRequest) -> Result<Vec<Result<Message>>> {
        info!(count = request.chat_ids.len(), template_key = %request.template_key, "Sending bulk notifications");

        let message_text = self.format_message(&request.template_key, &request.language, &request.parameters)?;
        let mut results = Vec::new();

        for chat_id in request.chat_ids {
            let mut send_request = self.bot.send_message(chat_id, message_text.clone());
            
            if let Some(parse_mode) = request.parse_mode {
                send_request = send_request.parse_mode(parse_mode);
            }
            
            if request.disable_web_page_preview {
                send_request = send_request.disable_link_preview(true);
            }

            match send_request.send().await {
                Ok(message) => {
                    self.update_stats_success(&request.template_key, &request.language);
                    debug!(chat_id = ?chat_id, "Bulk notification sent successfully");
                    results.push(Ok(message));
                }
                Err(e) => {
                    self.update_stats_failure();
                    warn!(chat_id = ?chat_id, error = %e, "Failed to send bulk notification");
                    results.push(Err(SwingBuddyError::Telegram(e)));
                }
            }

            // Small delay between messages to avoid rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        info!(total = results.len(), successful = results.iter().filter(|r| r.is_ok()).count(), "Bulk notifications completed");
        Ok(results)
    }

    /// Send welcome message to new user
    pub async fn send_welcome_message(&mut self, user: &User) -> Result<Message> {
        let chat_id = ChatId(user.telegram_id);
        let mut parameters = HashMap::new();
        
        if let Some(first_name) = &user.first_name {
            parameters.insert("first_name".to_string(), first_name.clone());
        } else {
            parameters.insert("first_name".to_string(), "Friend".to_string());
        }

        let request = NotificationRequest {
            chat_id,
            template_key: "welcome".to_string(),
            language: user.language_code.clone(),
            parameters,
            parse_mode: Some(ParseMode::Html),
            disable_web_page_preview: false,
        };

        self.send_notification(request).await
    }

    /// Send event notification
    pub async fn send_event_notification(&mut self, users: &[User], event: &Event, notification_type: &str) -> Result<Vec<Result<Message>>> {
        let chat_ids: Vec<ChatId> = users.iter().map(|u| ChatId(u.telegram_id)).collect();
        let mut parameters = HashMap::new();
        
        parameters.insert("event_title".to_string(), event.title.clone());
        parameters.insert("event_date".to_string(), event.event_date.format("%Y-%m-%d %H:%M UTC").to_string());
        
        if let Some(location) = &event.location {
            parameters.insert("event_location".to_string(), location.clone());
        }
        
        if let Some(description) = &event.description {
            parameters.insert("event_description".to_string(), description.clone());
        }

        // Use the first user's language as default, or fallback to default language
        let language = users.first()
            .map(|u| u.language_code.clone())
            .unwrap_or_else(|| self.settings.i18n.default_language.clone());

        let template_key = format!("event_{}", notification_type);
        
        let request = BulkNotificationRequest {
            chat_ids,
            template_key,
            language,
            parameters,
            parse_mode: Some(ParseMode::Html),
            disable_web_page_preview: false,
        };

        self.send_bulk_notifications(request).await
    }

    /// Send group notification
    pub async fn send_group_notification(&mut self, chat_id: ChatId, group: &Group, notification_type: &str, parameters: HashMap<String, String>) -> Result<Message> {
        let mut params = parameters;
        params.insert("group_title".to_string(), group.title.clone());
        
        if let Some(description) = &group.description {
            params.insert("group_description".to_string(), description.clone());
        }

        let template_key = format!("group_{}", notification_type);
        
        let request = NotificationRequest {
            chat_id,
            template_key,
            language: group.language_code.clone(),
            parameters: params,
            parse_mode: Some(ParseMode::Html),
            disable_web_page_preview: false,
        };

        self.send_notification(request).await
    }

    /// Send admin notification
    pub async fn send_admin_notification(&mut self, message: &str) -> Result<Vec<Result<Message>>> {
        let admin_chat_ids: Vec<ChatId> = self.settings.bot.admin_ids
            .iter()
            .map(|&id| ChatId(id))
            .collect();

        if admin_chat_ids.is_empty() {
            warn!("No admin IDs configured for admin notifications");
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        
        for chat_id in admin_chat_ids {
            match self.bot.send_message(chat_id, message).send().await {
                Ok(msg) => {
                    debug!(chat_id = ?chat_id, "Admin notification sent successfully");
                    results.push(Ok(msg));
                }
                Err(e) => {
                    warn!(chat_id = ?chat_id, error = %e, "Failed to send admin notification");
                    results.push(Err(SwingBuddyError::Telegram(e)));
                }
            }
        }

        Ok(results)
    }

    /// Format message using template and parameters
    fn format_message(&self, template_key: &str, language: &str, parameters: &HashMap<String, String>) -> Result<String> {
        let template = self.templates.get(template_key)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Template not found: {}", template_key)))?;

        let content = template.content.get(language)
            .or_else(|| template.content.get(&self.settings.i18n.default_language))
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Template content not found for language: {}", language)))?;

        let mut formatted = content.clone();
        
        // Replace parameters in the template
        for (key, value) in parameters {
            let placeholder = format!("{{{}}}", key);
            formatted = formatted.replace(&placeholder, value);
        }

        Ok(formatted)
    }

    /// Update success statistics
    fn update_stats_success(&mut self, template_key: &str, language: &str) {
        self.stats.total_sent += 1;
        *self.stats.sent_by_language.entry(language.to_string()).or_insert(0) += 1;
        *self.stats.sent_by_template.entry(template_key.to_string()).or_insert(0) += 1;
    }

    /// Update failure statistics
    fn update_stats_failure(&mut self) {
        self.stats.total_failed += 1;
    }

    /// Get notification statistics
    pub fn get_stats(&self) -> &NotificationStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = NotificationStats {
            total_sent: 0,
            total_failed: 0,
            sent_by_language: HashMap::new(),
            sent_by_template: HashMap::new(),
        };
    }

    /// Add or update a message template
    pub fn add_template(&mut self, template: MessageTemplate) {
        self.templates.insert(template.key.clone(), template);
    }

    /// Remove a message template
    pub fn remove_template(&mut self, template_key: &str) -> Option<MessageTemplate> {
        self.templates.remove(template_key)
    }

    /// Get available template keys
    pub fn get_template_keys(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Load default message templates
    fn load_default_templates() -> HashMap<String, MessageTemplate> {
        let mut templates = HashMap::new();

        // Welcome message template
        let mut welcome_content = HashMap::new();
        welcome_content.insert("en".to_string(), 
            "🎉 Welcome to SwingBuddy, {first_name}!\n\nI'm here to help you discover and join swing dancing events. Let's get started!".to_string());
        welcome_content.insert("ru".to_string(), 
            "🎉 Добро пожаловать в SwingBuddy, {first_name}!\n\nЯ здесь, чтобы помочь вам найти и присоединиться к мероприятиям по свинг-танцам. Давайте начнем!".to_string());

        templates.insert("welcome".to_string(), MessageTemplate {
            key: "welcome".to_string(),
            content: welcome_content,
            parse_mode: Some(ParseMode::Html),
            disable_web_page_preview: false,
        });

        // Event created template
        let mut event_created_content = HashMap::new();
        event_created_content.insert("en".to_string(), 
            "📅 <b>New Event Created!</b>\n\n<b>{event_title}</b>\n📍 {event_location}\n🕒 {event_date}\n\n{event_description}".to_string());
        event_created_content.insert("ru".to_string(), 
            "📅 <b>Создано новое мероприятие!</b>\n\n<b>{event_title}</b>\n📍 {event_location}\n🕒 {event_date}\n\n{event_description}".to_string());

        templates.insert("event_created".to_string(), MessageTemplate {
            key: "event_created".to_string(),
            content: event_created_content,
            parse_mode: Some(ParseMode::Html),
            disable_web_page_preview: false,
        });

        // Event reminder template
        let mut event_reminder_content = HashMap::new();
        event_reminder_content.insert("en".to_string(), 
            "⏰ <b>Event Reminder</b>\n\n<b>{event_title}</b>\n📍 {event_location}\n🕒 {event_date}\n\nDon't forget about this event!".to_string());
        event_reminder_content.insert("ru".to_string(), 
            "⏰ <b>Напоминание о мероприятии</b>\n\n<b>{event_title}</b>\n📍 {event_location}\n🕒 {event_date}\n\nНе забудьте об этом мероприятии!".to_string());

        templates.insert("event_reminder".to_string(), MessageTemplate {
            key: "event_reminder".to_string(),
            content: event_reminder_content,
            parse_mode: Some(ParseMode::Html),
            disable_web_page_preview: false,
        });

        // Group welcome template
        let mut group_welcome_content = HashMap::new();
        group_welcome_content.insert("en".to_string(), 
            "👋 Welcome to <b>{group_title}</b>!\n\n{group_description}\n\nI'm SwingBuddy, your assistant for swing dancing events in this group.".to_string());
        group_welcome_content.insert("ru".to_string(), 
            "👋 Добро пожаловать в <b>{group_title}</b>!\n\n{group_description}\n\nЯ SwingBuddy, ваш помощник по мероприятиям свинг-танцев в этой группе.".to_string());

        templates.insert("group_welcome".to_string(), MessageTemplate {
            key: "group_welcome".to_string(),
            content: group_welcome_content,
            parse_mode: Some(ParseMode::Html),
            disable_web_page_preview: false,
        });

        templates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use teloxide::Bot;

    #[test]
    fn test_format_message() {
        let bot = Bot::new("test_token");
        let settings = Settings::default();
        let service = NotificationService::new(bot, settings);

        let mut parameters = HashMap::new();
        parameters.insert("first_name".to_string(), "John".to_string());

        let result = service.format_message("welcome", "en", &parameters).unwrap();
        assert!(result.contains("John"));
        assert!(result.contains("Welcome to SwingBuddy"));
    }

    #[test]
    fn test_template_management() {
        let bot = Bot::new("test_token");
        let settings = Settings::default();
        let mut service = NotificationService::new(bot, settings);

        let mut content = HashMap::new();
        content.insert("en".to_string(), "Test message".to_string());

        let template = MessageTemplate {
            key: "test".to_string(),
            content,
            parse_mode: None,
            disable_web_page_preview: false,
        };

        service.add_template(template);
        assert!(service.get_template_keys().contains(&"test".to_string()));

        let removed = service.remove_template("test");
        assert!(removed.is_some());
        assert!(!service.get_template_keys().contains(&"test".to_string()));
    }

    #[test]
    fn test_stats_update() {
        let bot = Bot::new("test_token");
        let settings = Settings::default();
        let mut service = NotificationService::new(bot, settings);

        service.update_stats_success("welcome", "en");
        service.update_stats_success("welcome", "ru");
        service.update_stats_failure();

        let stats = service.get_stats();
        assert_eq!(stats.total_sent, 2);
        assert_eq!(stats.total_failed, 1);
        assert_eq!(stats.sent_by_language.get("en"), Some(&1));
        assert_eq!(stats.sent_by_language.get("ru"), Some(&1));
        assert_eq!(stats.sent_by_template.get("welcome"), Some(&2));
    }
}