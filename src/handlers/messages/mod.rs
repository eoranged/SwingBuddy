//! Message handlers module
//! 
//! Handles incoming text messages, new member events, and CAS API checking

use teloxide::{Bot, types::Message, prelude::*};
use tracing::{info, debug, warn, error};
use crate::utils::errors::Result;
use crate::services::ServiceFactory;
use crate::state::{ScenarioManager, StateStorage};
use crate::i18n::I18n;
use crate::handlers::commands::start;

/// Handle incoming text messages
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    let user = msg.from.as_ref().ok_or_else(|| {
        crate::utils::errors::SwingBuddyError::InvalidInput("No user in message".to_string())
    })?;

    let user_id = user.id.0 as i64;
    let chat_id = msg.chat.id;

    debug!(user_id = user_id, chat_id = ?chat_id, "Processing message");

    // Check for CAS ban in groups
    if !chat_id.is_user() {
        if let Err(e) = check_and_handle_cas_ban(&bot, &msg, &services).await {
            error!(error = %e, user_id = user_id, "Failed to check CAS ban");
        }
    }

    // Handle state-based conversations in private chats
    if chat_id.is_user() {
        if let Some(context) = state_storage.load_context(user_id).await? {
            return handle_conversation_message(
                bot, msg, context, services, scenario_manager, state_storage, i18n
            ).await;
        }
    }

    // Handle regular messages (no active conversation)
    handle_regular_message(bot, msg, services, i18n).await
}

/// Handle new chat member events
pub async fn handle_new_chat_member(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
) -> Result<()> {
    if let Some(new_members) = msg.new_chat_members() {
        for member in new_members {
            let user_id = member.id.0 as i64;
            debug!(user_id = user_id, chat_id = ?msg.chat.id, "New member joined chat");

            // Check CAS ban for new member
            match services.cas_service.check_user(user_id).await {
                Ok(result) => {
                    if result.is_banned {
                        info!(user_id = user_id, "Banning user due to CAS listing");
                        
                        // Ban the user
                        if let Err(e) = bot.ban_chat_member(msg.chat.id, member.id).await {
                            error!(error = %e, user_id = user_id, "Failed to ban user");
                        }
                        
                        // Delete the join message
                        if let Err(e) = bot.delete_message(msg.chat.id, msg.id).await {
                            warn!(error = %e, "Failed to delete join message");
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, user_id = user_id, "Failed to check CAS ban for new member");
                }
            }
        }
    }

    Ok(())
}

/// Check and handle CAS ban for message author
async fn check_and_handle_cas_ban(
    bot: &Bot,
    msg: &Message,
    services: &ServiceFactory,
) -> Result<()> {
    let user = msg.from.as_ref().unwrap();
    let user_id = user.id.0 as i64;

    match services.cas_service.check_user(user_id).await {
        Ok(result) => {
            if result.is_banned {
                info!(user_id = user_id, "Banning user due to CAS listing");
                
                // Ban the user
                if let Err(e) = bot.ban_chat_member(msg.chat.id, user.id).await {
                    error!(error = %e, user_id = user_id, "Failed to ban user");
                }
                
                // Delete the message
                if let Err(e) = bot.delete_message(msg.chat.id, msg.id).await {
                    warn!(error = %e, "Failed to delete message from banned user");
                }
            }
        }
        Err(e) => {
            error!(error = %e, user_id = user_id, "Failed to check CAS ban");
        }
    }

    Ok(())
}

/// Handle conversation-based messages (when user is in a scenario)
async fn handle_conversation_message(
    bot: Bot,
    msg: Message,
    context: crate::state::ConversationContext,
    services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    let scenario = context.scenario.as_deref().unwrap_or("");
    let step = context.step.as_deref().unwrap_or("");

    debug!(scenario = scenario, step = step, "Handling conversation message");

    match (scenario, step) {
        ("onboarding", "name_input") => {
            start::handle_name_input(bot, msg, services, scenario_manager, state_storage, i18n).await
        }
        ("onboarding", "location_input") => {
            start::handle_location_input(bot, msg, services, scenario_manager, state_storage, i18n).await
        }
        _ => {
            // Unknown scenario/step - clear context and handle as regular message
            warn!(scenario = scenario, step = step, "Unknown conversation state");
            state_storage.delete_context(msg.from.as_ref().unwrap().id.0 as i64).await?;
            handle_regular_message(bot, msg, services, i18n).await
        }
    }
}

/// Handle regular messages (no active conversation)
async fn handle_regular_message(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
    i18n: I18n,
) -> Result<()> {
    let user_id = msg.from.as_ref().unwrap().id.0 as i64;
    let chat_id = msg.chat.id;

    // In private chats, suggest using commands
    if chat_id.is_user() {
        let user_lang = if let Some(user_data) = services.user_service.get_user_by_telegram_id(user_id).await? {
            user_data.language_code
        } else {
            "en".to_string()
        };

        let help_text = i18n.t("messages.help.use_commands", &user_lang, None);
        bot.send_message(chat_id, help_text).await?;
    }

    Ok(())
}