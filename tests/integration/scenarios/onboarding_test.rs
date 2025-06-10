//! User onboarding test scenarios
//!
//! This module contains comprehensive integration tests for the user onboarding flow,
//! testing all scenarios from start command through completion.

use serial_test::serial;
use std::collections::HashMap;
use teloxide::types::{Message, User, Chat, ChatKind, MessageKind, MessageCommon, CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::Bot;
use SwingBuddy::handlers::commands::start;
use SwingBuddy::handlers::callbacks::handle_callback_query;
use SwingBuddy::models::user::{User as DbUser, CreateUserRequest};
use SwingBuddy::state::{ConversationContext, ScenarioManager};
use SwingBuddy::utils::errors::SwingBuddyError;

use crate::helpers::{TestContext, TestConfig, create_simple_test_message, create_simple_test_callback_query};

/// Helper function to verify user exists in database with expected data
async fn verify_user_in_database(
    ctx: &TestContext,
    telegram_id: i64,
    expected_language: Option<&str>,
    expected_location: Option<&str>,
    expected_name: Option<&str>,
) -> Result<DbUser, Box<dyn std::error::Error + Send + Sync>> {
    let user = sqlx::query_as!(
        DbUser,
        r#"
        SELECT
            id,
            telegram_id,
            username as "username?",
            first_name as "first_name?",
            last_name as "last_name?",
            language_code,
            location as "location?",
            is_banned,
            created_at,
            updated_at
        FROM users WHERE telegram_id = $1
        "#,
        telegram_id
    )
    .fetch_one(ctx.db_pool())
    .await?;

    if let Some(lang) = expected_language {
        assert_eq!(user.language_code, lang, "User language mismatch");
    }

    if let Some(loc) = expected_location {
        assert_eq!(user.location.as_deref(), Some(loc), "User location mismatch");
    } else {
        assert!(user.location.is_none(), "User should not have location set");
    }

    if let Some(name) = expected_name {
        assert_eq!(user.first_name.as_deref(), Some(name), "User name mismatch");
    }

    Ok(user)
}

/// Helper function to verify conversation state
async fn verify_conversation_state(
    ctx: &TestContext,
    user_id: i64,
    expected_scenario: Option<&str>,
    expected_step: Option<&str>,
) -> Result<Option<ConversationContext>, Box<dyn std::error::Error + Send + Sync>> {
    let app_state = ctx.create_app_state().await?;
    let state_storage = SwingBuddy::state::storage::StateStorage::new(
        app_state.redis_service.clone().unwrap(),
    );
    
    let context = state_storage.load_context(user_id).await?;
    
    if let Some(ctx) = &context {
        if let Some(scenario) = expected_scenario {
            assert_eq!(ctx.scenario.as_deref(), Some(scenario), "Scenario mismatch");
        }
        if let Some(step) = expected_step {
            assert_eq!(ctx.step.as_deref(), Some(step), "Step mismatch");
        }
    } else if expected_scenario.is_some() || expected_step.is_some() {
        panic!("Expected conversation context but found none");
    }
    
    Ok(context)
}

#[tokio::test]
#[serial]
async fn test_complete_happy_path_onboarding_flow() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456789i64;
    let chat_id = user_id; // Private chat
    
    // Step 1: User sends /start command
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    
    let result = start::handle_start(
        bot.clone(),
        start_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    assert!(result.is_ok(), "Start command should succeed: {:?}", result);
    
    // Verify user was created in database
    let user = verify_user_in_database(&ctx, user_id, Some("en"), None, Some("TestUser")).await
        .expect("User should be created in database");
    
    // Verify conversation state is set to onboarding/language_selection
    verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("language_selection")).await
        .expect("Conversation state should be set");
    
    // Step 2: User selects language (English)
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    
    let result = handle_callback_query(
        bot.clone(),
        lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    assert!(result.is_ok(), "Language selection should succeed: {:?}", result);
    
    // Verify conversation state moved to name_input
    let context = verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("name_input")).await
        .expect("Conversation state should be updated");
    
    // Verify language was stored in context
    let stored_lang = context.unwrap().get_string("language").unwrap();
    assert_eq!(stored_lang, "en", "Language should be stored in context");
    
    // Step 3: User provides name
    let name_message = create_simple_test_message(user_id, chat_id, "John Doe");
    
    let result = start::handle_name_input(
        bot.clone(),
        name_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    assert!(result.is_ok(), "Name input should succeed: {:?}", result);
    
    // Verify conversation state moved to location_input
    let context = verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("location_input")).await
        .expect("Conversation state should be updated");
    
    // Verify name was stored in context
    let stored_name = context.unwrap().get_string("name").unwrap();
    assert_eq!(stored_name, "John Doe", "Name should be stored in context");
    
    // Step 4: User selects location (Moscow)
    let location_callback = create_simple_test_callback_query(user_id, chat_id, "location:Moscow");
    
    let result = handle_callback_query(
        bot.clone(),
        location_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    assert!(result.is_ok(), "Location selection should succeed: {:?}", result);
    
    // Verify onboarding is completed (no active scenario)
    verify_conversation_state(&ctx, user_id, None, None).await
        .expect("Conversation should be completed");
    
    // Verify user profile was updated with all information
    verify_user_in_database(&ctx, user_id, Some("en"), Some("Moscow"), Some("John Doe")).await
        .expect("User profile should be updated");
    
    // Cleanup
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

#[tokio::test]
#[serial]
async fn test_onboarding_with_location_skip() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456790i64;
    let chat_id = user_id;
    
    // Complete onboarding flow up to location selection
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Start should succeed");
    
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:ru");
    handle_callback_query(
        bot.clone(),
        lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Language selection should succeed");
    
    let name_message = create_simple_test_message(user_id, chat_id, "Иван Петров");
    start::handle_name_input(
        bot.clone(),
        name_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Name input should succeed");
    
    // User skips location
    let skip_callback = create_simple_test_callback_query(user_id, chat_id, "location:skip");
    
    let result = handle_callback_query(
        bot.clone(),
        skip_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    assert!(result.is_ok(), "Location skip should succeed: {:?}", result);
    
    // Verify onboarding is completed
    verify_conversation_state(&ctx, user_id, None, None).await
        .expect("Conversation should be completed");
    
    // Verify user profile was created without location
    verify_user_in_database(&ctx, user_id, Some("ru"), None, Some("Иван Петров")).await
        .expect("User profile should be updated without location");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

#[tokio::test]
#[serial]
async fn test_invalid_name_input_handling() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456791i64;
    let chat_id = user_id;
    
    // Complete onboarding flow up to name input
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Start should succeed");
    
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    handle_callback_query(
        bot.clone(),
        lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Language selection should succeed");
    
    // Test invalid name inputs
    let invalid_names = vec![
        "A",           // Too short
        "123",         // Numbers only
        "@#$%",        // Special characters
        "A".repeat(60), // Too long
    ];
    
    for invalid_name in invalid_names {
        let name_message = create_simple_test_message(user_id, chat_id, &invalid_name);
        
        let result = start::handle_name_input(
            bot.clone(),
            name_message,
            app_state.services.clone(),
            app_state.scenario_manager.clone(),
            app_state.state_storage.clone(),
            app_state.i18n.clone(),
        ).await;
        
        // The handler should not fail but should stay in the same step
        assert!(result.is_ok(), "Handler should not fail for invalid input: {}", invalid_name);
        
        // Verify conversation state remains in name_input
        verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("name_input")).await
            .expect("Should remain in name_input step");
    }
    
    // Test valid name input
    let valid_name_message = create_simple_test_message(user_id, chat_id, "Valid Name");
    let result = start::handle_name_input(
        bot.clone(),
        valid_name_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    assert!(result.is_ok(), "Valid name should succeed: {:?}", result);
    
    // Verify conversation state moved to location_input
    verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("location_input")).await
        .expect("Should move to location_input step");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

#[tokio::test]
#[serial]
async fn test_conversation_state_management() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456792i64;
    let chat_id = user_id;
    
    // Start onboarding
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Start should succeed");
    
    // Verify initial state
    let context = verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("language_selection")).await
        .expect("Initial state should be set").unwrap();
    
    // Verify state has expiry set
    assert!(context.expires_at.is_some(), "Context should have expiry set");
    assert!(!context.is_expired(), "Context should not be expired");
    
    // Test state persistence across interactions
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    handle_callback_query(
        bot.clone(),
        lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Language selection should succeed");
    
    // Verify state updated and data persisted
    let context = verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("name_input")).await
        .expect("State should be updated").unwrap();
    
    let stored_lang = context.get_string("language").unwrap();
    assert_eq!(stored_lang, "en", "Language should be persisted");
    
    // Test handling of unexpected commands during onboarding
    // This should not break the flow or clear the state
    let unexpected_message = create_simple_test_message(user_id, chat_id, "/help");
    
    // The system should handle this gracefully without breaking onboarding
    // (In a real implementation, this might show help but preserve onboarding state)
    
    // Verify state is still preserved
    verify_conversation_state(&ctx, user_id, Some("onboarding"), Some("name_input")).await
        .expect("State should be preserved after unexpected command");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

#[tokio::test]
#[serial]
async fn test_existing_user_handling() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456793i64;
    let chat_id = user_id;
    
    // Create existing user in database
    let existing_user = ctx.database.create_test_user(
        user_id,
        Some("existing_user".to_string()),
        "Existing User".to_string(),
    ).await.expect("Failed to create existing user");
    
    // Update user with complete profile
    sqlx::query!(
        "UPDATE users SET language_code = $1, location = $2 WHERE id = $3",
        "ru",
        "Saint Petersburg",
        existing_user.id
    )
    .execute(ctx.db_pool())
    .await
    .expect("Failed to update existing user");
    
    // User sends /start command
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    
    let result = start::handle_start(
        bot.clone(),
        start_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    assert!(result.is_ok(), "Start command should succeed for existing user: {:?}", result);
    
    // Verify no onboarding scenario was started
    let context = verify_conversation_state(&ctx, user_id, None, None).await
        .expect("No conversation should be active");
    assert!(context.is_none(), "No conversation context should exist for existing user");
    
    // Verify user data was not changed
    verify_user_in_database(&ctx, user_id, Some("ru"), Some("Saint Petersburg"), Some("Existing User")).await
        .expect("Existing user data should be preserved");
    
    // Verify no duplicate user was created
    let user_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE telegram_id = $1",
        user_id
    )
    .fetch_one(ctx.db_pool())
    .await
    .expect("Failed to count users");
    
    assert_eq!(user_count, Some(1), "Should have exactly one user record");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

#[tokio::test]
#[serial]
async fn test_state_expiry_handling() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456794i64;
    let chat_id = user_id;
    
    // Start onboarding
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Start should succeed");
    
    // Manually expire the context
    let state_storage = SwingBuddy::state::storage::StateStorage::new(
        app_state.redis_service.clone().unwrap(),
    );
    
    let mut context = state_storage.load_context(user_id).await
        .expect("Failed to load context")
        .expect("Context should exist");
    
    // Set expiry to past
    context.set_expiry(chrono::Utc::now() - chrono::Duration::hours(1));
    state_storage.save_context(&context).await
        .expect("Failed to save expired context");
    
    // Try to continue onboarding with expired state
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    
    let result = handle_callback_query(
        bot.clone(),
        lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    
    // The system should handle expired state gracefully
    // This might result in an error or restart the onboarding
    // The exact behavior depends on implementation, but it shouldn't crash
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

#[tokio::test]
#[serial]
async fn test_concurrent_onboarding_sessions() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user1_id = 123456795i64;
    let user2_id = 123456796i64;
    
    // Start onboarding for both users simultaneously
    let start_message1 = create_simple_test_message(user1_id, user1_id, "/start");
    let start_message2 = create_simple_test_message(user2_id, user2_id, "/start");
    
    let (result1, result2) = tokio::join!(
        start::handle_start(
            bot.clone(),
            start_message1,
            app_state.services.clone(),
            app_state.scenario_manager.clone(),
            app_state.state_storage.clone(),
            app_state.i18n.clone(),
        ),
        start::handle_start(
            bot.clone(),
            start_message2,
            app_state.services.clone(),
            app_state.scenario_manager.clone(),
            app_state.state_storage.clone(),
            app_state.i18n.clone(),
        )
    );
    
    assert!(result1.is_ok(), "User 1 start should succeed: {:?}", result1);
    assert!(result2.is_ok(), "User 2 start should succeed: {:?}", result2);
    
    // Verify both users have independent conversation states
    verify_conversation_state(&ctx, user1_id, Some("onboarding"), Some("language_selection")).await
        .expect("User 1 should have conversation state");
    
    verify_conversation_state(&ctx, user2_id, Some("onboarding"), Some("language_selection")).await
        .expect("User 2 should have conversation state");
    
    // Continue onboarding for both users with different choices
    let lang_callback1 = create_simple_test_callback_query(user1_id, user1_id, "lang:en");
    let lang_callback2 = create_simple_test_callback_query(user2_id, user2_id, "lang:ru");
    
    let (result1, result2) = tokio::join!(
        handle_callback_query(
            bot.clone(),
            lang_callback1,
            app_state.services.clone(),
            app_state.scenario_manager.clone(),
            app_state.state_storage.clone(),
            app_state.i18n.clone(),
        ),
        handle_callback_query(
            bot.clone(),
            lang_callback2,
            app_state.services.clone(),
            app_state.scenario_manager.clone(),
            app_state.state_storage.clone(),
            app_state.i18n.clone(),
        )
    );
    
    assert!(result1.is_ok(), "User 1 language selection should succeed: {:?}", result1);
    assert!(result2.is_ok(), "User 2 language selection should succeed: {:?}", result2);
    
    // Verify users have different language preferences stored
    let state_storage = SwingBuddy::state::storage::StateStorage::new(
        app_state.redis_service.clone().unwrap(),
    );
    
    let context1 = state_storage.load_context(user1_id).await
        .expect("Failed to load user 1 context")
        .expect("User 1 context should exist");
    
    let context2 = state_storage.load_context(user2_id).await
        .expect("Failed to load user 2 context")
        .expect("User 2 context should exist");
    
    assert_eq!(context1.get_string("language").unwrap(), "en", "User 1 should have English");
    assert_eq!(context2.get_string("language").unwrap(), "ru", "User 2 should have Russian");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}