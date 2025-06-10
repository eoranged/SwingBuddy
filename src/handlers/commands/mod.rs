//! Command handlers module
//! 
//! This module contains handlers for all bot commands like /start, /help, etc.

pub mod start;
pub mod help;
pub mod events;
pub mod admin;

use teloxide::{Bot, types::Message, utils::command::BotCommands};
use crate::utils::errors::Result;
use crate::services::ServiceFactory;
use crate::state::{ScenarioManager, StateStorage};
use crate::i18n::I18n;

/// All available bot commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "SwingBuddy commands:")]
pub enum Command {
    #[command(description = "Start the bot and show welcome message")]
    Start,
    #[command(description = "Show help information")]
    Help,
    #[command(description = "List upcoming events")]
    Events,
    #[command(description = "Create a new event")]
    CreateEvent,
    #[command(description = "Register for an event")]
    Register,
    #[command(description = "Admin panel (admin only)")]
    Admin,
    #[command(description = "Set language preference")]
    Language,
    #[command(description = "Show user profile")]
    Profile,
    #[command(description = "Show bot statistics (admin only)")]
    Stats,
}

/// Main command dispatcher
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    services: ServiceFactory,
    scenario_manager: ScenarioManager,
    state_storage: StateStorage,
    i18n: I18n,
) -> Result<()> {
    match cmd {
        Command::Start => start::handle_start(bot, msg, services, scenario_manager, state_storage, i18n).await,
        Command::Help => help::handle_help(bot, msg).await,
        Command::Events => events::handle_events_list(bot, msg, services, i18n).await,
        Command::CreateEvent => events::handle_create_event(bot, msg, services, i18n).await,
        Command::Register => events::handle_register(bot, msg, services, i18n).await,
        Command::Admin => admin::handle_admin_panel(bot, msg, services, scenario_manager, state_storage, i18n).await,
        Command::Language => start::handle_language_selection(bot, msg).await,
        Command::Profile => start::handle_profile(bot, msg).await,
        Command::Stats => admin::handle_stats(bot, msg, services, i18n).await,
    }
}