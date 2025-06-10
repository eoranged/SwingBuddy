//! Event command handlers

use std::collections::HashMap;
use teloxide::{Bot, types::{Message, InlineKeyboardMarkup, InlineKeyboardButton, ChatId}, prelude::*};
use tracing::{info, debug, warn};
use chrono::{DateTime, Utc};
use crate::utils::errors::Result;
use crate::services::{ServiceFactory, GoogleCalendarService};
use crate::database::repositories::EventRepository;
use crate::i18n::I18n;
use crate::models::event::{Event, CreateEventRequest};

/// Handle /events command - list upcoming events in private chats
pub async fn handle_events_list(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    let user = msg.from().ok_or_else(|| {
        crate::utils::errors::SwingBuddyError::InvalidInput("No user in message".to_string())
    })?;

    let user_id = user.id.0 as i64;
    let chat_id = msg.chat.id;

    debug!(user_id = user_id, chat_id = ?chat_id, "Processing /events command");

    // Only allow in private chats
    if !chat_id.is_user() {
        let text = i18n.t("messages.errors.invalid_command", "en", None);
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    // Show available calendars as inline keyboard buttons
    show_calendar_list(bot, chat_id, &services, &i18n, &user_lang).await?;

    Ok(())
}

/// Show available calendars as inline keyboard buttons
async fn show_calendar_list(
    bot: Bot,
    chat_id: ChatId,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    let title_text = i18n.t("commands.events.list_title", language_code, None);
    
    // Create keyboard with available calendars
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "📅 Swing Dance Events",
                "calendar:swing_events"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "🎭 Workshops & Classes",
                "calendar:workshops"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "🎉 Social Events",
                "calendar:social"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, title_text)
        .reply_markup(keyboard)
        .await?;
    
    Ok(())
}

/// Handle calendar selection callback
pub async fn handle_calendar_callback(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    calendar_type: String,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, calendar_type = %calendar_type, "User selected calendar");
    
    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    // Show calendar description and "Add to Google Calendar" button
    show_calendar_details(bot, chat_id, &calendar_type, &services, &i18n, &user_lang).await?;

    Ok(())
}

/// Show calendar description with "Add to Google Calendar" button
async fn show_calendar_details(
    bot: Bot,
    chat_id: ChatId,
    calendar_type: &str,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    let (title, description) = match calendar_type {
        "swing_events" => (
            "📅 Swing Dance Events",
            "Regular swing dance events, milongas, and dance parties. Perfect for social dancing and meeting other dancers in the community."
        ),
        "workshops" => (
            "🎭 Workshops & Classes",
            "Educational workshops, dance classes, and skill-building sessions. Learn new moves, techniques, and styles from experienced instructors."
        ),
        "social" => (
            "🎉 Social Events",
            "Community gatherings, meetups, and special celebrations. Connect with fellow dancers outside of regular dance events."
        ),
        _ => ("📅 Events", "Dance community events and activities."),
    };

    let message_text = format!("**{}**\n\n{}", title, description);
    
    // Create keyboard with "Add to Google Calendar" button
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::url(
                "📅 Add to Google Calendar",
                reqwest::Url::parse(&services.google_service.generate_calendar_sharing_url()?)?
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", language_code, None),
                "calendar:back"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, message_text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .await?;
    
    Ok(())
}

/// Handle event creation (admin only)
pub async fn handle_create_event(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    let user = msg.from().ok_or_else(|| {
        crate::utils::errors::SwingBuddyError::InvalidInput("No user in message".to_string())
    })?;

    let user_id = user.id.0 as i64;
    let chat_id = msg.chat.id;

    debug!(user_id = user_id, "Processing event creation request");

    // Check if user has permission to create events
    if !services.auth_service.can_manage_events(user_id, Some(chat_id)).await? {
        let error_text = i18n.t("messages.errors.permission_denied", "en", None);
        bot.send_message(chat_id, error_text).await?;
        return Ok(());
    }

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    let create_title = i18n.t("commands.events.create_title", &user_lang, None);
    bot.send_message(chat_id, format!("✨ {}\n\nThis feature will be available in the admin panel.", create_title)).await?;

    Ok(())
}

/// Handle event registration
pub async fn handle_register(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    let user = msg.from().ok_or_else(|| {
        crate::utils::errors::SwingBuddyError::InvalidInput("No user in message".to_string())
    })?;

    let user_id = user.id.0 as i64;
    let chat_id = msg.chat.id;

    debug!(user_id = user_id, "Processing event registration request");

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    let register_text = "📝 Event registration will be available through the calendar interface. Use /events to browse available events.";
    bot.send_message(chat_id, register_text).await?;

    Ok(())
}

/// Handle event registration callback
pub async fn handle_event_register_callback(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    event_id: i64,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, event_id = event_id, "User registering for event");
    
    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    // TODO: Implement actual event registration logic with database
    // For now, show a success message
    let mut params = HashMap::new();
    params.insert("event_name".to_string(), format!("Event #{}", event_id));
    
    let success_text = i18n.t("commands.events.register_success", &user_lang, Some(&params));
    bot.send_message(chat_id, success_text).await?;

    info!(user_id = user_id, event_id = event_id, "User registered for event");

    Ok(())
}

/// Handle event unregistration callback
pub async fn handle_event_unregister_callback(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    event_id: i64,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, event_id = event_id, "User unregistering from event");
    
    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    // TODO: Implement actual event unregistration logic with database
    // For now, show a success message
    let mut params = HashMap::new();
    params.insert("event_name".to_string(), format!("Event #{}", event_id));
    
    let success_text = i18n.t("commands.events.unregister_success", &user_lang, Some(&params));
    bot.send_message(chat_id, success_text).await?;

    info!(user_id = user_id, event_id = event_id, "User unregistered from event");

    Ok(())
}

/// Show event details with registration options
pub async fn show_event_details(
    bot: Bot,
    chat_id: ChatId,
    event: &Event,
    user_id: i64,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    // Format event details
    let mut params = HashMap::new();
    params.insert("title".to_string(), event.title.clone());
    params.insert("location".to_string(), event.location.clone().unwrap_or_else(|| "TBD".to_string()));
    params.insert("date".to_string(), event.event_date.format("%Y-%m-%d %H:%M UTC").to_string());
    params.insert("current".to_string(), "0".to_string()); // TODO: Get actual participant count
    params.insert("max".to_string(), event.max_participants.map(|m| m.to_string()).unwrap_or_else(|| "∞".to_string()));
    params.insert("description".to_string(), event.description.clone().unwrap_or_else(|| "No description available.".to_string()));
    
    let details_text = i18n.t("commands.events.event_details", language_code, Some(&params));
    
    // Create registration keyboard
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.events.register", language_code, None),
                format!("event_register:{}", event.id)
            ),
            InlineKeyboardButton::callback(
                i18n.t("buttons.events.unregister", language_code, None),
                format!("event_unregister:{}", event.id)
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", language_code, None),
                "calendar:back"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, details_text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .await?;
    
    Ok(())
}