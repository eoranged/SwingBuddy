//! Start command handler
//!
//! Handles the /start command and user onboarding flow

use std::collections::HashMap;
use teloxide::{Bot, types::{Message, InlineKeyboardMarkup, InlineKeyboardButton, ChatId}, prelude::*};
use tracing::{info, debug, warn, error};
use crate::utils::errors::Result;
use crate::services::ServiceFactory;
use crate::state::{ScenarioManager, StateStorage, ConversationContext};
use crate::i18n::I18n;
use crate::models::user::CreateUserRequest;

/// Handle /start command - main entry point for user onboarding
pub async fn handle_start(
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

    debug!(user_id = user_id, chat_id = ?chat_id, "Processing /start command");

    // Check if this is a private chat
    if !chat_id.is_user() {
        let text = i18n.t("messages.errors.invalid_command", "en", None);
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }

    // Check if user exists in database
    match services.user_service.get_user_by_telegram_id(user_id).await? {
        Some(existing_user) => {
            // Existing user - show welcome back message
            let user_lang = &existing_user.language_code;
            let mut params = HashMap::new();
            params.insert("name".to_string(),
                existing_user.first_name.clone()
                    .or_else(|| existing_user.username.clone())
                    .unwrap_or_else(|| "there".to_string())
            );
            
            let welcome_text = i18n.t("commands.start.returning_user", user_lang, Some(&params));
            bot.send_message(chat_id, welcome_text).await?;
            
            info!(user_id = user_id, "Existing user started bot");
        }
        None => {
            // New user - start onboarding flow
            info!(user_id = user_id, "New user starting onboarding");
            
            // Create user in database first
            let _create_request = CreateUserRequest {
                telegram_id: user_id,
                username: user.username.clone(),
                first_name: Some(user.first_name.clone()),
                last_name: user.last_name.clone(),
                language_code: Some(i18n.detect_user_language(user.language_code.as_deref())),
                location: None,
            };
            
            let _new_user = services.user_service.register_or_get_user(
                user_id,
                user.username.clone(),
                Some(user.first_name.clone()),
                user.last_name.clone(),
            ).await?;
            
            // Start onboarding scenario
            info!(user_id = user_id, "🔍 START HANDLER: Starting onboarding scenario for new user");
            let mut context = ConversationContext::new(user_id);
            
            match scenario_manager.start_scenario(&mut context, "onboarding") {
                Ok(_) => {
                    info!(user_id = user_id, scenario = ?context.scenario, step = ?context.step,
                           "🔍 START HANDLER: Onboarding scenario started successfully");
                },
                Err(e) => {
                    error!(user_id = user_id, error = %e, "🔍 START HANDLER: Failed to start onboarding scenario");
                    return Err(e);
                }
            }
            
            info!(user_id = user_id, "🔍 START HANDLER: Attempting to save context to storage");
            match state_storage.save_context(&context).await {
                Ok(_) => {
                    info!(user_id = user_id, "🔍 START HANDLER: Context saved successfully after starting onboarding");
                },
                Err(e) => {
                    error!(user_id = user_id, error = %e, "🔍 START HANDLER: Failed to save context after starting onboarding - this could be the issue!");
                    return Err(e);
                }
            }
            
            // Show language selection
            info!(user_id = user_id, "🔍 START HANDLER: Showing language selection to user");
            show_language_selection(bot, chat_id, &i18n).await?;
        }
    }

    Ok(())
}

/// Show language selection keyboard
async fn show_language_selection(bot: Bot, chat_id: ChatId, i18n: &I18n) -> Result<()> {
    info!(chat_id = ?chat_id, "🔍 LANG SELECTION: Creating language selection keyboard");
    
    let welcome_text = i18n.t("commands.start.new_user_greeting", "en", None);
    let choose_lang_text = i18n.t("commands.start.choose_language", "en", None);
    
    info!("🔍 LANG SELECTION: Creating keyboard buttons with callback data");
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                i18n.t("buttons.language.english", "en", None),
                "lang:en"
            ),
            InlineKeyboardButton::callback(
                i18n.t("buttons.language.russian", "ru", None),
                "lang:ru"
            ),
        ]
    ]);
    
    info!("🔍 LANG SELECTION: Buttons created - English: 'lang:en', Russian: 'lang:ru'");
    
    let full_text = format!("{}\n\n{}", welcome_text, choose_lang_text);
    
    info!(chat_id = ?chat_id, "🔍 LANG SELECTION: Sending message with keyboard");
    bot.send_message(chat_id, full_text)
        .reply_markup(keyboard)
        .await?;
    
    info!(chat_id = ?chat_id, "🔍 LANG SELECTION: Language selection message sent successfully");
    Ok(())
}

/// Handle language selection callback
pub async fn handle_language_callback(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    language_code: String,
    services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    info!(user_id = user_id, language_code = %language_code, "🔍 LANG HANDLER: User selected language - starting callback handler");
    
    // Check if language is supported
    info!(user_id = user_id, language_code = %language_code, "🔍 LANG HANDLER: Checking if language is supported");
    if !i18n.is_language_supported(&language_code) {
        warn!(user_id = user_id, language_code = %language_code, "🔍 LANG HANDLER: Unsupported language selected");
        let _error_text = i18n.t("messages.validation.invalid_name", "en", None);
        bot.send_message(chat_id, format!("❌ Unsupported language: {}", language_code)).await?;
        return Ok(());
    }
    
    info!(user_id = user_id, "🔍 LANG HANDLER: Language is supported, loading user context");
    
    // Load user context
    let context_result = state_storage.load_context(user_id).await;
    info!(user_id = user_id, context_loaded = context_result.is_ok(), "🔍 LANG HANDLER: Context load result");
    
    let mut context = match context_result {
        Ok(Some(ctx)) => {
            info!(user_id = user_id, scenario = ?ctx.scenario, step = ?ctx.step, "🔍 LANG HANDLER: Context loaded successfully");
            ctx
        },
        Ok(None) => {
            error!(user_id = user_id, "🔍 LANG HANDLER: No context found for user - this is the likely issue!");
            return Err(crate::utils::errors::SwingBuddyError::InvalidStateTransition {
                from: "no_context".to_string(),
                to: "language_selected".to_string(),
            });
        },
        Err(e) => {
            error!(user_id = user_id, error = %e, "🔍 LANG HANDLER: Failed to load context - this could be the issue!");
            return Err(e);
        }
    };
    
    // Validate we're in the right scenario and step
    let is_correct_state = context.is_at("onboarding", "language_selection");
    info!(user_id = user_id, is_correct_state = is_correct_state,
           current_scenario = ?context.scenario, current_step = ?context.step,
           "🔍 LANG HANDLER: State validation result");
    
    if !is_correct_state {
        error!(user_id = user_id, scenario = ?context.scenario, step = ?context.step,
              "🔍 LANG HANDLER: User not in language selection step - this could be the issue!");
        return Ok(());
    }
    
    info!(user_id = user_id, "🔍 LANG HANDLER: All validations passed, proceeding with language update");
    
    // Update user language preference
    services.user_service.set_language_preference(user_id, language_code.clone()).await?;
    
    // Store language in context
    context.set_data("language", &language_code)?;
    
    // Move to next step
    scenario_manager.next_step(&mut context, "name_input")?;
    state_storage.save_context(&context).await?;
    
    // Show language confirmation and ask for name
    let confirmation_text = i18n.t("commands.start.language_selected", &language_code, None);
    bot.send_message(chat_id, confirmation_text).await?;
    
    // Ask for name with default suggestion
    ask_for_name(bot, chat_id, user_id, &services, &i18n, &language_code).await?;
    
    Ok(())
}

/// Ask user for their name
async fn ask_for_name(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    services: &ServiceFactory,
    i18n: &I18n,
    language_code: &str,
) -> Result<()> {
    let ask_name_text = i18n.t("commands.start.ask_name", language_code, None);
    
    // Get user's Telegram name as suggestion
    if let Some(user) = services.user_service.get_user_by_telegram_id(user_id).await? {
        if let Some(first_name) = &user.first_name {
            let mut params = HashMap::new();
            params.insert("name".to_string(), first_name.clone());
            
            let suggestion_text = format!("{}\n\n💡 Suggestion: {}", ask_name_text, first_name);
            bot.send_message(chat_id, suggestion_text).await?;
        } else {
            bot.send_message(chat_id, ask_name_text).await?;
        }
    } else {
        bot.send_message(chat_id, ask_name_text).await?;
    }
    
    Ok(())
}

/// Handle name input during onboarding
pub async fn handle_name_input(
    bot: Bot,
    msg: Message,
    _services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    let user_id = msg.from.as_ref().unwrap().id.0 as i64;
    let chat_id = msg.chat.id;
    let name = msg.text().unwrap_or("").trim();
    
    debug!(user_id = user_id, name = %name, "User provided name");
    
    // Load context
    let mut context = state_storage.load_context(user_id).await?
        .ok_or_else(|| crate::utils::errors::SwingBuddyError::InvalidStateTransition {
            from: "no_context".to_string(),
            to: "name_provided".to_string(),
        })?;
    
    // Validate we're in the right step
    if !context.is_at("onboarding", "name_input") {
        return Ok(());
    }
    
    let language_code = context.get_string("language").unwrap_or_else(|| "en".to_string());
    
    // Validate name input
    if let Err(_e) = scenario_manager.validate_input(&context, name) {
        let error_text = i18n.t("messages.validation.invalid_name", &language_code, None);
        bot.send_message(chat_id, error_text).await?;
        return Ok(());
    }
    
    // Store name in context
    context.set_data("name", name)?;
    
    // Move to location input
    scenario_manager.next_step(&mut context, "location_input")?;
    state_storage.save_context(&context).await?;
    
    // Ask for location
    ask_for_location(bot, chat_id, &i18n, &language_code).await?;
    
    Ok(())
}

/// Ask user for their location
async fn ask_for_location(bot: Bot, chat_id: ChatId, i18n: &I18n, language_code: &str) -> Result<()> {
    let ask_location_text = i18n.t("commands.start.ask_location", language_code, None);
    
    // Create keyboard with city suggestions
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📍 Moscow", "location:Moscow"),
            InlineKeyboardButton::callback("📍 Saint Petersburg", "location:Saint Petersburg"),
        ],
        vec![
            InlineKeyboardButton::callback("⏭️ Skip", "location:skip"),
        ]
    ]);
    
    bot.send_message(chat_id, ask_location_text)
        .reply_markup(keyboard)
        .await?;
    
    Ok(())
}

/// Handle location input during onboarding
pub async fn handle_location_input(
    bot: Bot,
    msg: Message,
    services: ServiceFactory,
    _scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    let user_id = msg.from.as_ref().unwrap().id.0 as i64;
    let chat_id = msg.chat.id;
    let location = msg.text().unwrap_or("").trim();
    
    debug!(user_id = user_id, location = %location, "User provided location");
    
    // Load context
    let mut context = state_storage.load_context(user_id).await?
        .ok_or_else(|| crate::utils::errors::SwingBuddyError::InvalidStateTransition {
            from: "no_context".to_string(),
            to: "location_provided".to_string(),
        })?;
    
    // Validate we're in the right step
    if !context.is_at("onboarding", "location_input") {
        return Ok(());
    }
    
    let language_code = context.get_string("language").unwrap_or_else(|| "en".to_string());
    
    // Store location in context
    context.set_data("location", location)?;
    
    // Complete onboarding
    complete_onboarding(bot, chat_id, user_id, context, services, i18n, language_code).await?;
    
    Ok(())
}

/// Handle location selection callback
pub async fn handle_location_callback(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    location: String,
    services: ServiceFactory,
    _scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    debug!(user_id = user_id, location = %location, "User selected location");
    
    // Load context
    let mut context = state_storage.load_context(user_id).await?
        .ok_or_else(|| crate::utils::errors::SwingBuddyError::InvalidStateTransition {
            from: "no_context".to_string(),
            to: "location_selected".to_string(),
        })?;
    
    let language_code = context.get_string("language").unwrap_or_else(|| "en".to_string());
    
    // Store location in context (or skip if "skip")
    if location != "skip" {
        context.set_data("location", &location)?;
    }
    
    // Complete onboarding
    complete_onboarding(bot, chat_id, user_id, context, services, i18n, language_code).await?;
    
    Ok(())
}

/// Complete the onboarding process
async fn complete_onboarding(
    bot: Bot,
    chat_id: ChatId,
    user_id: i64,
    mut context: ConversationContext,
    services: ServiceFactory,
    i18n: I18n,
    language_code: String,
) -> Result<()> {
    // Get data from context
    let name = context.get_string("name");
    let location = context.get_string("location");
    
    // Update user profile
    let mut update_request = crate::models::user::UpdateUserRequest::default();
    if let Some(name) = &name {
        update_request.first_name = Some(name.clone());
    }
    if let Some(location) = &location {
        update_request.location = Some(location.clone());
    }
    update_request.language_code = Some(language_code.clone());
    
    services.user_service.update_user_profile(user_id, update_request).await?;
    
    // Complete scenario
    context.complete_scenario();
    services.redis_service.clear_user_state(user_id).await?;
    
    // Show completion message
    let completion_text = i18n.t("commands.start.setup_complete", &language_code, None);
    bot.send_message(chat_id, completion_text).await?;
    
    info!(user_id = user_id, "User onboarding completed successfully");
    
    Ok(())
}

/// Handle /language command - show language selection
pub async fn handle_language_selection(bot: Bot, msg: Message) -> Result<()> {
    let chat_id = msg.chat.id;
    
    // Only allow in private chats
    if !chat_id.is_user() {
        let text = "This command is only available in private chats.";
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }
    
    let text = "🌐 Language Selection\n\nPlease choose your preferred language:";
    bot.send_message(chat_id, text).await?;
    
    Ok(())
}

/// Handle /profile command - show user profile
pub async fn handle_profile(bot: Bot, msg: Message) -> Result<()> {
    let user = msg.from.as_ref().ok_or_else(|| {
        crate::utils::errors::SwingBuddyError::InvalidInput("No user in message".to_string())
    })?;
    
    let chat_id = msg.chat.id;
    
    // Only allow in private chats
    if !chat_id.is_user() {
        let text = "This command is only available in private chats.";
        bot.send_message(chat_id, text).await?;
        return Ok(());
    }
    
    let profile_text = format!(
        "👤 Your Profile\n\n\
        • Telegram ID: {}\n\
        • Username: {}\n\
        • First Name: {}\n\
        • Last Name: {}\n\n\
        Use /language to change your language preference.",
        user.id.0,
        user.username.as_ref().map_or("Not set", |s| s.as_str()),
        &user.first_name,
        user.last_name.as_ref().map_or("Not set", |s| s.as_str())
    );
    
    bot.send_message(chat_id, profile_text).await?;
    
    Ok(())
}