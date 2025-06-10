//! Admin command handlers

use std::collections::HashMap;
use teloxide::{Bot, types::{Message, InlineKeyboardMarkup, InlineKeyboardButton, ChatId}, prelude::*};
use tracing::{info, debug, warn};
use crate::utils::errors::Result;
use crate::services::ServiceFactory;
use crate::state::{ScenarioManager, StateStorage, ConversationContext};
use crate::i18n::I18n;

/// Handle /admin command - show admin panel
pub async fn handle_admin_panel(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    let user = msg.from().ok_or_else(|| {
        crate::utils::errors::SwingBuddyError::InvalidInput("No user in message".to_string())
    })?;

    let user_id = user.id.0 as i64;
    let chat_id = msg.chat.id;

    debug!(user_id = user_id, chat_id = ?chat_id, "Processing /admin command");

    // Check if user is admin
    if !services.auth_service.can_access_admin_panel(user_id).await? {
        let error_text = i18n.t("commands.admin.access_denied", "en", None);
        bot.send_message(chat_id, error_text).await?;
        return Ok(());
    }

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    // Start admin panel scenario
    let mut context = ConversationContext::new(user_id);
    scenario_manager.start_scenario(&mut context, "admin_panel")?;
    state_storage.save_context(&context).await?;

    // Show admin main menu
    show_admin_main_menu(bot, chat_id, &i18n, &user_lang).await?;

    info!(user_id = user_id, "Admin accessed admin panel");

    Ok(())
}

/// Show admin main menu
async fn show_admin_main_menu(bot: Bot, chat_id: ChatId, i18n: &I18n, language_code: &str) -> Result<()> {
    let title_text = i18n.t("commands.admin.panel_title", language_code, None);
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                i18n.t("commands.admin.user_management", language_code, None),
                "admin:users"
            ),
            InlineKeyboardButton::callback(
                i18n.t("commands.admin.group_management", language_code, None),
                "admin:groups"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("commands.admin.event_management", language_code, None),
                "admin:events"
            ),
            InlineKeyboardButton::callback(
                i18n.t("commands.admin.statistics", language_code, None),
                "admin:stats"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("commands.admin.system_settings", language_code, None),
                "admin:settings"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, title_text)
        .reply_markup(keyboard)
        .await?;
    
    Ok(())
}

/// Handle admin panel callback
pub async fn handle_admin_callback(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    action: String,
    services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, action = %action, "Admin panel action");

    // Verify admin access
    if !services.auth_service.can_access_admin_panel(user_id).await? {
        let error_text = i18n.t("commands.admin.access_denied", "en", None);
        bot.send_message(chat_id, error_text).await?;
        return Ok(());
    }

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    match action.as_str() {
        "users" => show_user_management(bot, chat_id, &services, &i18n, &user_lang).await?,
        "groups" => show_group_management(bot, chat_id, &services, &i18n, &user_lang).await?,
        "events" => show_event_management(bot, chat_id, &services, &i18n, &user_lang).await?,
        "stats" => show_statistics(bot, chat_id, &services, &i18n, &user_lang).await?,
        "settings" => show_system_settings(bot, chat_id, &services, &i18n, &user_lang).await?,
        "back" => show_admin_main_menu(bot, chat_id, &i18n, &user_lang).await?,
        _ => {
            warn!(user_id = user_id, action = %action, "Unknown admin action");
        }
    }

    Ok(())
}

/// Show user management panel
async fn show_user_management(
    bot: Bot,
    chat_id: ChatId,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    let stats = services.user_service.get_user_statistics().await?;
    
    let text = format!(
        "👥 **{}**\n\n📊 Statistics:\n• Total users: {}\n• Active users: {}\n• Banned users: {}",
        i18n.t("commands.admin.user_management", language_code, None),
        stats.get("total_users").unwrap_or(&0),
        stats.get("active_users").unwrap_or(&0),
        stats.get("banned_users").unwrap_or(&0)
    );
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.admin.ban", language_code, None),
                "admin:ban_user"
            ),
            InlineKeyboardButton::callback(
                i18n.t("buttons.admin.unban", language_code, None),
                "admin:unban_user"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", language_code, None),
                "admin:back"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .await?;
    
    Ok(())
}

/// Show group management panel
async fn show_group_management(
    bot: Bot,
    chat_id: ChatId,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    let text = format!(
        "👥 **{}**\n\nGroup management features:\n• View active groups\n• Manage group settings\n• Monitor group activity",
        i18n.t("commands.admin.group_management", language_code, None)
    );
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "📋 List Groups",
                "admin:list_groups"
            ),
            InlineKeyboardButton::callback(
                "⚙️ Group Settings",
                "admin:group_settings"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", language_code, None),
                "admin:back"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .await?;
    
    Ok(())
}

/// Show event management panel
async fn show_event_management(
    bot: Bot,
    chat_id: ChatId,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    let text = format!(
        "🎭 **{}**\n\nEvent management features:\n• Create new events\n• Edit existing events\n• Manage event calendars\n• View event statistics",
        i18n.t("commands.admin.event_management", language_code, None)
    );
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.events.create", language_code, None),
                "admin:create_event"
            ),
            InlineKeyboardButton::callback(
                i18n.t("buttons.events.list", language_code, None),
                "admin:list_events"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "📅 Manage Calendars",
                "admin:manage_calendars"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", language_code, None),
                "admin:back"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .await?;
    
    Ok(())
}

/// Show system statistics
async fn show_statistics(
    bot: Bot,
    chat_id: ChatId,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    // Get various statistics
    let user_stats = services.user_service.get_user_statistics().await?;
    let health_status = services.health_check().await;
    
    let text = format!(
        "📊 **{}**\n\n👥 Users:\n• Total: {}\n• Active: {}\n• Banned: {}\n\n� System:\n• Redis: {}\n• Google Calendar: {}\n• CAS Protection: {}",
        i18n.t("commands.admin.statistics", language_code, None),
        user_stats.get("total_users").unwrap_or(&0),
        user_stats.get("active_users").unwrap_or(&0),
        user_stats.get("banned_users").unwrap_or(&0),
        if health_status.redis_healthy { "✅" } else { "❌" },
        if health_status.google_enabled { "✅" } else { "❌" },
        if health_status.cas_enabled { "✅" } else { "❌" }
    );
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "🔄 Refresh",
                "admin:stats"
            ),
            InlineKeyboardButton::callback(
                i18n.t("buttons.admin.backup", language_code, None),
                "admin:backup"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", language_code, None),
                "admin:back"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .await?;
    
    Ok(())
}

/// Show system settings panel
async fn show_system_settings(
    bot: Bot,
    chat_id: ChatId,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    let text = format!(
        "⚙️ **{}**\n\nSystem configuration:\n• Feature toggles\n• API settings\n• Cache management\n• Backup & restore",
        i18n.t("commands.admin.system_settings", language_code, None)
    );
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "🔧 Features",
                "admin:features"
            ),
            InlineKeyboardButton::callback(
                "🗄️ Cache",
                "admin:cache"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", language_code, None),
                "admin:back"
            ),
        ],
    ]);
    
    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .await?;
    
    Ok(())
}

/// Handle /stats command - show bot statistics
pub async fn handle_stats(
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

    debug!(user_id = user_id, "Processing /stats command");

    // Check if user is admin
    if !services.auth_service.can_access_admin_panel(user_id).await? {
        let error_text = i18n.t("commands.admin.access_denied", "en", None);
        bot.send_message(chat_id, error_text).await?;
        return Ok(());
    }

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    // Show statistics directly
    show_statistics(bot, chat_id, &services, &i18n, &user_lang).await?;

    Ok(())
}

/// Handle calendar management
pub async fn handle_calendar_management(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    action: String,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, action = %action, "Calendar management action");

    // Verify admin access
    if !services.auth_service.can_access_admin_panel(user_id).await? {
        return Ok(());
    }

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    match action.as_str() {
        "add" => {
            let text = "➕ **Add New Calendar**\n\nTo add a new calendar, please provide:\n• Calendar name\n• Description\n• Google Calendar ID (optional)";
            bot.send_message(chat_id, text)
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        "edit" => {
            let text = "✏️ **Edit Calendar**\n\nSelect a calendar to edit from the list below:";
            // TODO: Show list of existing calendars
            bot.send_message(chat_id, text)
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        _ => {
            warn!(user_id = user_id, action = %action, "Unknown calendar management action");
        }
    }

    Ok(())
}

/// Handle user ban/unban operations
pub async fn handle_user_moderation(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    action: String,
    target_user_id: Option<i64>,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, action = %action, target_user_id = ?target_user_id, "User moderation action");

    // Verify admin access
    if !services.auth_service.can_manage_users(user_id, Some(chat_id)).await? {
        let error_text = i18n.t("commands.admin.access_denied", "en", None);
        bot.send_message(chat_id, error_text).await?;
        return Ok(());
    }

    // Get user language
    let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
        user_data.language_code
    } else {
        "en".to_string()
    };

    match action.as_str() {
        "ban" => {
            if let Some(target_id) = target_user_id {
                services.user_service.set_user_ban_status(target_id, true, user_id).await?;
                let mut params = HashMap::new();
                params.insert("user_name".to_string(), format!("User #{}", target_id));
                let success_text = i18n.t("commands.admin.ban_user_success", &user_lang, Some(&params));
                bot.send_message(chat_id, success_text).await?;
                info!(admin_id = user_id, target_user_id = target_id, "User banned by admin");
            } else {
                bot.send_message(chat_id, "Please provide user ID to ban.").await?;
            }
        }
        "unban" => {
            if let Some(target_id) = target_user_id {
                services.user_service.set_user_ban_status(target_id, false, user_id).await?;
                let mut params = HashMap::new();
                params.insert("user_name".to_string(), format!("User #{}", target_id));
                let success_text = i18n.t("commands.admin.unban_user_success", &user_lang, Some(&params));
                bot.send_message(chat_id, success_text).await?;
                info!(admin_id = user_id, target_user_id = target_id, "User unbanned by admin");
            } else {
                bot.send_message(chat_id, "Please provide user ID to unban.").await?;
            }
        }
        _ => {
            warn!(user_id = user_id, action = %action, "Unknown user moderation action");
        }
    }

    Ok(())
}