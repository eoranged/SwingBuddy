//! Callback query handlers module
//! 
//! This module contains handlers for all inline keyboard button callbacks

pub mod group_setup;

use teloxide::{Bot, types::{CallbackQuery, ChatId}, prelude::*};
use tracing::{info, debug, warn};
use crate::utils::errors::Result;
use crate::services::ServiceFactory;
use crate::state::{ScenarioManager, StateStorage};
use crate::i18n::I18n;
use crate::handlers::commands::{start, events, admin};

/// Main callback query dispatcher
pub async fn handle_callback_query(
    bot: Bot,
    query: CallbackQuery,
    services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    let user = query.from;
    let user_id = user.id.0 as i64;
    let chat_id = query.message.as_ref().map(|m| m.chat().id);
    
    debug!(user_id = user_id, chat_id = ?chat_id, "Processing callback query");

    if let Some(data) = query.data {
        // Answer the callback query first to remove loading state
        if let Err(e) = bot.answer_callback_query(query.id).await {
            warn!(error = %e, "Failed to answer callback query");
        }

        // Parse callback data and route to appropriate handler
        let parts: Vec<&str> = data.split(':').collect();
        if parts.is_empty() {
            warn!(data = %data, "Invalid callback data format");
            return Ok(());
        }

        let action = parts[0];
        let chat_id = chat_id.unwrap_or_else(|| ChatId(user_id));

        match action {
            "lang" => {
                // Language selection callback
                if parts.len() >= 2 {
                    let language_code = parts[1].to_string();
                    start::handle_language_callback(
                        bot,
                        chat_id,
                        user_id,
                        language_code,
                        services,
                        scenario_manager,
                        state_storage,
                        i18n,
                    ).await?;
                }
            }
            "location" => {
                // Location selection callback
                if parts.len() >= 2 {
                    let location = parts[1].to_string();
                    start::handle_location_callback(
                        bot,
                        chat_id,
                        user_id,
                        location,
                        services,
                        scenario_manager,
                        state_storage,
                        i18n,
                    ).await?;
                }
            }
            "calendar" => {
                // Calendar selection callback
                if parts.len() >= 2 {
                    let calendar_type = parts[1].to_string();
                    if calendar_type == "back" {
                        // Show calendar list again
                        if let Some(teloxide::types::MaybeInaccessibleMessage::Regular(message)) = query.message {
                            events::handle_events_list(bot, *message, services, i18n).await?;
                        }
                    } else {
                        events::handle_calendar_callback(
                            bot,
                            chat_id,
                            user_id,
                            calendar_type,
                            services,
                            i18n,
                        ).await?;
                    }
                }
            }
            "event_register" => {
                // Event registration callback
                if parts.len() >= 2 {
                    if let Ok(event_id) = parts[1].parse::<i64>() {
                        events::handle_event_register_callback(
                            bot,
                            chat_id,
                            user_id,
                            event_id,
                            services,
                            i18n,
                        ).await?;
                    }
                }
            }
            "event_unregister" => {
                // Event unregistration callback
                if parts.len() >= 2 {
                    if let Ok(event_id) = parts[1].parse::<i64>() {
                        events::handle_event_unregister_callback(
                            bot,
                            chat_id,
                            user_id,
                            event_id,
                            services,
                            i18n,
                        ).await?;
                    }
                }
            }
            "admin" => {
                // Admin panel callback
                if parts.len() >= 2 {
                    let admin_action = parts[1].to_string();
                    admin::handle_admin_callback(
                        bot,
                        chat_id,
                        user_id,
                        admin_action,
                        services,
                        scenario_manager,
                        state_storage,
                        i18n,
                    ).await?;
                }
            }
            "group_setup" => {
                // Group setup callback
                if parts.len() >= 2 {
                    let setup_action = parts[1].to_string();
                    group_setup::handle_group_setup_callback(
                        bot,
                        chat_id,
                        user_id,
                        setup_action,
                        services,
                        i18n,
                    ).await?;
                }
            }
            _ => {
                warn!(action = %action, "Unknown callback action");
            }
        }
    }

    info!(user_id = user_id, "Callback query processed successfully");
    Ok(())
}