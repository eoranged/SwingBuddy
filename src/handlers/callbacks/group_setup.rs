//! Group setup callback handlers
//! 
//! Handles bot configuration when added to groups

use std::collections::HashMap;
use teloxide::{Bot, types::{ChatId, InlineKeyboardMarkup, InlineKeyboardButton, ChatMemberStatus}, prelude::*};
use tracing::{info, debug, warn};
use crate::utils::errors::Result;
use crate::services::ServiceFactory;
use crate::i18n::I18n;

/// Handle group setup callbacks
pub async fn handle_group_setup_callback(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    action: String,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, chat_id = ?chat_id, action = %action, "Processing group setup callback");

    match action.as_str() {
        "check_permissions" => {
            check_bot_permissions(bot, chat_id, &services, &i18n).await?;
        }
        "documentation" => {
            // This should open a URL, handled by inline keyboard URL button
            debug!("Documentation button clicked");
        }
        "language" => {
            show_language_selector(bot, chat_id, &i18n).await?;
        }
        "lang_en" => {
            set_group_language(bot, chat_id, "en".to_string(), &services, &i18n).await?;
        }
        "lang_ru" => {
            set_group_language(bot, chat_id, "ru".to_string(), &services, &i18n).await?;
        }
        "dismiss" => {
            // Delete the setup message
            if let Err(e) = bot.delete_message(chat_id, teloxide::types::MessageId(0)).await {
                warn!(error = %e, "Failed to delete setup message");
            }
        }
        _ => {
            warn!(action = %action, "Unknown group setup action");
        }
    }

    Ok(())
}

/// Handle bot being added to a group
pub async fn handle_bot_added_to_group(
    bot: Bot,
    chat_id: ChatId,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    info!(chat_id = ?chat_id, "Bot added to group");

    // Check bot permissions
    check_bot_permissions(bot, chat_id, &services, &i18n).await?;

    Ok(())
}

/// Check if bot has required permissions
async fn check_bot_permissions(
    bot: Bot,
    chat_id: ChatId,
    services: &ServiceFactory,
    i18n: &I18n,
) -> Result<()> {
    debug!(chat_id = ?chat_id, "Checking bot permissions");

    // Get bot's member status in the chat
    let bot_user = bot.get_me().await?;
    let member = bot.get_chat_member(chat_id, bot_user.id).await?;

    let has_required_permissions = match member.status() {
        ChatMemberStatus::Administrator => {
            // For now, assume administrator has required permissions
            // In a real implementation, you'd check specific permissions
            true
        }
        _ => false,
    };

    if has_required_permissions {
        show_setup_success(bot, chat_id, i18n).await?;
    } else {
        show_permission_request(bot, chat_id, i18n).await?;
    }

    Ok(())
}

/// Show permission request message
async fn show_permission_request(
    bot: Bot,
    chat_id: ChatId,
    i18n: &I18n,
) -> Result<()> {
    let message_text = i18n.t("group.setup.permission_request", "en", None);
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::url(
                i18n.t("buttons.group.documentation", "en", None),
                reqwest::Url::parse("https://github.com/your-repo/swing-buddy/wiki/Bot-Setup")?
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.group.check_again", "en", None),
                "group_setup:check_permissions"
            ),
            InlineKeyboardButton::callback(
                i18n.t("buttons.group.language", "en", None),
                "group_setup:language"
            ),
        ],
    ]);

    bot.send_message(chat_id, message_text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Show setup success message
async fn show_setup_success(
    bot: Bot,
    chat_id: ChatId,
    i18n: &I18n,
) -> Result<()> {
    let message_text = i18n.t("group.setup.success", "en", None);
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.group.got_it", "en", None),
                "group_setup:dismiss"
            ),
        ],
    ]);

    bot.send_message(chat_id, message_text)
        .reply_markup(keyboard)
        .await?;

    info!(chat_id = ?chat_id, "Group setup completed successfully");

    Ok(())
}

/// Show language selector for group
async fn show_language_selector(
    bot: Bot,
    chat_id: ChatId,
    i18n: &I18n,
) -> Result<()> {
    let message_text = i18n.t("group.setup.choose_language", "en", None);
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.language.english", "en", None),
                "group_setup:lang_en"
            ),
            InlineKeyboardButton::callback(
                i18n.t("buttons.language.russian", "ru", None),
                "group_setup:lang_ru"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.navigation.back", "en", None),
                "group_setup:check_permissions"
            ),
        ],
    ]);

    bot.send_message(chat_id, message_text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Set group language preference
async fn set_group_language(
    bot: Bot,
    chat_id: ChatId,
    language_code: String,
    services: &ServiceFactory,
    i18n: &I18n,
) -> Result<()> {
    debug!(chat_id = ?chat_id, language_code = %language_code, "Setting group language");

    // TODO: Store group language preference in database
    // For now, just show confirmation and go back to permission check
    
    let mut params = HashMap::new();
    params.insert("language".to_string(), 
        if language_code == "ru" { "Russian" } else { "English" }.to_string()
    );
    
    let confirmation_text = i18n.t("group.setup.language_set", &language_code, Some(&params));
    bot.send_message(chat_id, confirmation_text).await?;

    // Go back to permission check
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    check_bot_permissions(bot, chat_id, services, i18n).await?;

    Ok(())
}