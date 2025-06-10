//! SwingBuddy Telegram Bot
//!
//! Main application entry point

use std::sync::Arc;
use teloxide::{prelude::*, types::Update};
use teloxide::dispatching::UpdateHandler;
use teloxide::utils::command::BotCommands as TeloxideBotCommands;
use tracing::{info, warn, error};

use SwingBuddy::{
    config::Settings,
    utils::logging,
    database::{DatabaseService, connection::create_pool},
    services::{ServiceFactory, redis::RedisService},
    state::{ScenarioManager, StateStorage},
    i18n::I18n,
    handlers::{
        commands::{start, events, admin, help},
        callbacks::handle_callback_query,
        messages::{handle_message, handle_new_chat_member},
    },
};

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let settings = Settings::new()?;
    settings.validate()?;
    
    // Initialize logging
    logging::init_logging(&settings.logging)?;
    
    info!("Starting SwingBuddy Telegram Bot...");
    
    // Initialize database connection
    info!("Connecting to database...");
    let db_config = SwingBuddy::database::connection::DatabaseConfig {
        url: settings.database.url.clone(),
        max_connections: settings.database.max_connections,
        min_connections: settings.database.min_connections,
        acquire_timeout: std::time::Duration::from_secs(30),
        idle_timeout: Some(std::time::Duration::from_secs(600)),
        max_lifetime: Some(std::time::Duration::from_secs(1800)),
    };
    let db_pool = create_pool(&db_config).await?;
    
    // Run database migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    
    // Initialize Redis connection
    info!("Connecting to Redis...");
    let redis_service = RedisService::new(settings.clone())?;
    
    // Initialize database service
    let database_service = DatabaseService::new(db_pool);
    
    // Initialize i18n system
    info!("Loading translations...");
    let mut i18n = I18n::new(&settings.i18n);
    i18n.load_translations().await?;
    
    // Initialize state management
    let state_storage = StateStorage::new(settings.redis.clone()).await?;
    let scenario_manager = ScenarioManager::new();
    
    // Initialize bot
    let bot = Bot::new(&settings.bot.token);
    
    // Initialize services
    info!("Initializing services...");
    let redis_client = ::redis::Client::open(settings.redis.url.clone())?;
    let user_repository = database_service.users.clone();
    let services = ServiceFactory::new(
        bot.clone(),
        settings.clone(),
        user_repository,
        redis_client,
    )?;
    
    info!("Setting up bot handlers...");
    
    // Debug: Log service factory creation
    info!("ServiceFactory created successfully");
    
    // Wrap services in Arc for dependency injection
    let services_arc = Arc::new(services);
    let scenario_manager_arc = Arc::new(scenario_manager);
    let state_storage_arc = Arc::new(state_storage);
    let i18n_arc = Arc::new(i18n);
    
    // Create the handler
    let handler = create_handler();
    
    // Create dispatcher with dependencies registered
    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![
            services_arc,
            scenario_manager_arc,
            state_storage_arc,
            i18n_arc
        ])
        .default_handler(|upd| async move {
            warn!("Unhandled update: {:?}", upd);
        })
        .enable_ctrlc_handler()
        .build();
    
    info!("Dispatcher created with dependencies registered in DI system");
    
    info!("SwingBuddy bot is ready!");
    
    // Start the bot
    if let Some(webhook_url) = &settings.bot.webhook_url {
        info!("Webhook URL configured: {}", webhook_url);
        info!("Note: Webhook setup not implemented in this version, falling back to polling");
    }
    
    info!("Starting bot with polling mode...");
    
    // Configure allowed update types to include callback queries
    
    
    
    
    
    dispatcher.dispatch().await;
    
    info!("SwingBuddy bot has been shut down.");
    
    Ok(())
}

/// Create the main update handler
fn create_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use teloxide::dispatching::UpdateFilterExt;
    
    dptree::entry()
    .branch(Update::filter_message()
        .branch(
            // Handle commands
            dptree::entry()
                .filter_command::<BotCommands>()
                .endpoint(handle_commands)
        )
        .branch(
            // Handle new chat members
            dptree::filter(|msg: Message| msg.new_chat_members().is_some())
                .endpoint(handle_new_members)
        )
        .branch(
            // Handle regular messages
            dptree::endpoint(handle_messages)
        )
  
    )
    .branch(// Handle callback queries
            Update::filter_callback_query()
                .endpoint(handle_callbacks)
    )
    .branch(
            // Handle my chat member updates (bot added/removed from groups)
            Update::filter_my_chat_member()
                .endpoint(handle_chat_member_updates)
    )
}

#[derive(TeloxideBotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "SwingBuddy Bot Commands")]
enum BotCommands {
    #[command(description = "Start the bot and user onboarding")]
    Start,
    #[command(description = "Show help information")]
    Help,
    #[command(description = "Browse dance events and calendars")]
    Events,
    #[command(description = "Admin panel (admin only)")]
    Admin,
    #[command(description = "Show bot statistics (admin only)")]
    Stats,
}

/// Handle bot commands
async fn handle_commands(
    bot: Bot,
    msg: Message,
    cmd: BotCommands,
    services: Arc<ServiceFactory>,
    scenario_manager: Arc<ScenarioManager>,
    state_storage: Arc<StateStorage>,
    i18n: Arc<I18n>,
) -> HandlerResult {
    let services = (*services).clone();
    let scenario_manager = (*scenario_manager).clone();
    let state_storage = (*state_storage).clone();
    let i18n = (*i18n).clone();
    
    let result = match cmd {
        BotCommands::Start => {
            start::handle_start(bot, msg, services, scenario_manager, state_storage, i18n).await
        }
        BotCommands::Help => {
            help::handle_help(bot, msg).await
        }
        BotCommands::Events => {
            events::handle_events_list(bot, msg, services, i18n).await
        }
        BotCommands::Admin => {
            admin::handle_admin_panel(bot, msg, services, scenario_manager, state_storage, i18n).await
        }
        BotCommands::Stats => {
            admin::handle_stats(bot, msg, services, i18n).await
        }
    };
    
    if let Err(e) = result {
        error!(error = %e, "Error handling command");
        return Err(e.into());
    }
    
    Ok(())
}

/// Handle regular messages
async fn handle_messages(
    bot: Bot,
    msg: Message,
    services: Arc<ServiceFactory>,
    scenario_manager: Arc<ScenarioManager>,
    state_storage: Arc<StateStorage>,
    i18n: Arc<I18n>,
) -> HandlerResult {
    let services = (*services).clone();
    let scenario_manager = (*scenario_manager).clone();
    let state_storage = (*state_storage).clone();
    let i18n = (*i18n).clone();
    
    if let Err(e) = handle_message(bot, msg, services, scenario_manager, state_storage, i18n).await {
        error!(error = %e, "Error handling message");
        return Err(e.into());
    }
    
    Ok(())
}

/// Handle new chat members
async fn handle_new_members(
    bot: Bot,
    msg: Message,
    services: Arc<ServiceFactory>,
) -> HandlerResult {
    let services = (*services).clone();
    
    if let Err(e) = handle_new_chat_member(bot, msg, services).await {
        error!(error = %e, "Error handling new chat member");
        return Err(e.into());
    }
    
    Ok(())
}

/// Handle callback queries
async fn handle_callbacks(
    bot: Bot,
    query: teloxide::types::CallbackQuery,
    services: Arc<ServiceFactory>,
    scenario_manager: Arc<ScenarioManager>,
    state_storage: Arc<StateStorage>,
    i18n: Arc<I18n>,
) -> HandlerResult {
    let user_id = query.from.id.0 as i64;
    info!(user_id = user_id, callback_data = ?query.data, "🔍 MAIN DISPATCHER: Callback query received in main handler");
    
    let services = (*services).clone();
    let scenario_manager = (*scenario_manager).clone();
    let state_storage = (*state_storage).clone();
    let i18n = (*i18n).clone();
    
    info!(user_id = user_id, "🔍 MAIN DISPATCHER: Dispatching to callback handler");
    if let Err(e) = handle_callback_query(bot, query, services, scenario_manager, state_storage, i18n).await {
        error!(user_id = user_id, error = %e, "🔍 MAIN DISPATCHER: Error handling callback query");
        return Err(e.into());
    }
    
    info!(user_id = user_id, "🔍 MAIN DISPATCHER: Callback query handled successfully");
    Ok(())
}

/// Handle chat member updates (bot added/removed from groups)
async fn handle_chat_member_updates(
    bot: Bot,
    update: teloxide::types::ChatMemberUpdated,
    services: Arc<ServiceFactory>,
    i18n: Arc<I18n>,
) -> HandlerResult {
    use SwingBuddy::handlers::callbacks::group_setup;
    
    let services = (*services).clone();
    let i18n = (*i18n).clone();
    
    // Check if this is the bot being added to a group
    let bot_user = bot.get_me().await?;
    if update.new_chat_member.user.id == bot_user.id {
        if let Err(e) = group_setup::handle_bot_added_to_group(
            bot,
            update.chat.id,
            services,
            i18n,
        ).await {
            error!(error = %e, "Error handling bot added to group");
            return Err(e.into());
        }
    }
    
    Ok(())
}
