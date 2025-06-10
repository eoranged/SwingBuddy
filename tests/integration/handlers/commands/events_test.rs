//! Integration tests for /events command handler
//!
//! This module contains comprehensive tests for the /events command functionality,
//! including event listing, browsing, registration flow, and different user contexts.

use serial_test::serial;
use std::collections::HashMap;
use teloxide::types::{Message, ChatId};
use teloxide::Bot;
use SwingBuddy::handlers::commands::events;
use SwingBuddy::models::user::{User as DbUser};

use crate::helpers::{TestContext, TestConfig, create_simple_test_message, create_test_message};

/// Test /events command in private chat
#[tokio::test]
#[serial]
async fn test_events_command_private_chat() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456789i64;
    let chat_id = user_id; // Private chat
    
    // Create user in database first
    let _user = ctx.database.create_test_user(
        user_id,
        Some("test_user".to_string()),
        "Test User".to_string(),
    ).await.expect("Failed to create test user");
    
    // Create /events message
    let events_message = create_simple_test_message(user_id, chat_id, "/events");
    
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Events command should succeed: {:?}", result);
    
    // Verify calendar list message was sent with inline keyboard
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /events command in group chat (should show error)
#[tokio::test]
#[serial]
async fn test_events_command_group_chat() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456790i64;
    let group_chat_id = -1001234567890i64; // Group chat
    
    // Create /events message in group chat
    let events_message = create_test_message(
        user_id,
        group_chat_id,
        "/events",
        Some("testuser"),
        "TestUser",
        Some("LastName"),
    );
    
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Events command should handle group chat gracefully: {:?}", result);
    
    // Verify error message was sent
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /events command for user with different language preferences
#[tokio::test]
#[serial]
async fn test_events_command_different_languages() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    // Test with English user
    let user_id_en = 123456791i64;
    let user_en = ctx.database.create_test_user(
        user_id_en,
        Some("english_user".to_string()),
        "English User".to_string(),
    ).await.expect("Failed to create English user");
    
    // Update user language to English
    sqlx::query!(
        "UPDATE users SET language_code = $1 WHERE id = $2",
        "en",
        user_en.id
    )
    .execute(ctx.db_pool())
    .await
    .expect("Failed to update user language");
    
    let events_message_en = create_simple_test_message(user_id_en, user_id_en, "/events");
    
    let result_en = events::handle_events_list(
        bot.clone(),
        events_message_en,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result_en.is_ok(), "Events command should succeed for English user: {:?}", result_en);
    
    // Test with Russian user
    let user_id_ru = 123456792i64;
    let user_ru = ctx.database.create_test_user(
        user_id_ru,
        Some("russian_user".to_string()),
        "Русский Пользователь".to_string(),
    ).await.expect("Failed to create Russian user");
    
    // Update user language to Russian
    sqlx::query!(
        "UPDATE users SET language_code = $1 WHERE id = $2",
        "ru",
        user_ru.id
    )
    .execute(ctx.db_pool())
    .await
    .expect("Failed to update user language");
    
    let events_message_ru = create_simple_test_message(user_id_ru, user_id_ru, "/events");
    
    let result_ru = events::handle_events_list(
        bot.clone(),
        events_message_ru,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result_ru.is_ok(), "Events command should succeed for Russian user: {:?}", result_ru);
    
    // Verify both messages were sent
    ctx.verify_telegram_calls("sendMessage", 2).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /events command for non-existent user (should use default language)
#[tokio::test]
#[serial]
async fn test_events_command_non_existent_user() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456793i64;
    let chat_id = user_id;
    
    // Don't create user in database - test with non-existent user
    let events_message = create_simple_test_message(user_id, chat_id, "/events");
    
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Events command should succeed for non-existent user: {:?}", result);
    
    // Should use default language (English) and show calendar list
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test calendar selection callback handling
#[tokio::test]
#[serial]
async fn test_calendar_selection_callbacks() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456794i64;
    let chat_id = ChatId(user_id);
    
    // Create user in database
    let _user = ctx.database.create_test_user(
        user_id,
        Some("test_user".to_string()),
        "Test User".to_string(),
    ).await.expect("Failed to create test user");
    
    // Test different calendar types
    let calendar_types = vec![
        "swing_events",
        "workshops", 
        "social",
    ];
    
    for calendar_type in calendar_types {
        let result = events::handle_calendar_callback(
            bot.clone(),
            chat_id,
            user_id,
            calendar_type.to_string(),
            (*app_state.services).clone(),
            (*app_state.i18n).clone(),
        ).await;
        
        assert!(result.is_ok(), "Calendar callback should succeed for {}: {:?}", calendar_type, result);
    }
    
    // Verify calendar detail messages were sent
    ctx.verify_telegram_calls("sendMessage", 3).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test event registration flow
#[tokio::test]
#[serial]
async fn test_event_registration_flow() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456795i64;
    let chat_id = user_id;
    
    // Create user in database
    let _user = ctx.database.create_test_user(
        user_id,
        Some("test_user".to_string()),
        "Test User".to_string(),
    ).await.expect("Failed to create test user");
    
    // Test /register command
    let register_message = create_simple_test_message(user_id, chat_id, "/register");
    
    let result = events::handle_register(
        bot.clone(),
        register_message,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Register command should succeed: {:?}", result);
    
    // Test event registration callback
    let event_id = 123i64;
    let chat_id_obj = ChatId(user_id);
    
    let result = events::handle_event_register_callback(
        bot.clone(),
        chat_id_obj,
        user_id,
        event_id,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Event registration callback should succeed: {:?}", result);
    
    // Test event unregistration callback
    let result = events::handle_event_unregister_callback(
        bot.clone(),
        chat_id_obj,
        user_id,
        event_id,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Event unregistration callback should succeed: {:?}", result);
    
    // Verify all messages were sent
    ctx.verify_telegram_calls("sendMessage", 3).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test event creation (admin functionality)
#[tokio::test]
#[serial]
async fn test_event_creation_admin() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let admin_user_id = 555666777i64; // This matches the admin ID in test settings
    let chat_id = admin_user_id;
    
    // Create admin user in database
    let _admin_user = ctx.database.create_test_user(
        admin_user_id,
        Some("admin_user".to_string()),
        "Admin User".to_string(),
    ).await.expect("Failed to create admin user");
    
    // Test /create_event command
    let create_event_message = create_simple_test_message(admin_user_id, chat_id, "/create_event");
    
    let result = events::handle_create_event(
        bot.clone(),
        create_event_message,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Create event command should succeed for admin: {:?}", result);
    
    // Verify admin panel message was sent
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test event creation for non-admin user (should show permission error)
#[tokio::test]
#[serial]
async fn test_event_creation_non_admin() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let regular_user_id = 123456796i64; // Not an admin
    let chat_id = regular_user_id;
    
    // Create regular user in database
    let _user = ctx.database.create_test_user(
        regular_user_id,
        Some("regular_user".to_string()),
        "Regular User".to_string(),
    ).await.expect("Failed to create regular user");
    
    // Test /create_event command
    let create_event_message = create_simple_test_message(regular_user_id, chat_id, "/create_event");
    
    let result = events::handle_create_event(
        bot.clone(),
        create_event_message,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Create event command should handle non-admin gracefully: {:?}", result);
    
    // Verify permission denied message was sent
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test concurrent events commands
#[tokio::test]
#[serial]
async fn test_concurrent_events_commands() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user1_id = 123456797i64;
    let user2_id = 123456798i64;
    let user3_id = 123456799i64;
    
    // Create users in database
    for &user_id in &[user1_id, user2_id, user3_id] {
        ctx.database.create_test_user(
            user_id,
            Some(format!("user_{}", user_id)),
            format!("User {}", user_id),
        ).await.expect("Failed to create test user");
    }
    
    // Create concurrent /events messages
    let events_message1 = create_simple_test_message(user1_id, user1_id, "/events");
    let events_message2 = create_simple_test_message(user2_id, user2_id, "/events");
    let events_message3 = create_simple_test_message(user3_id, user3_id, "/events");
    
    // Execute all events commands concurrently
    let (result1, result2, result3) = tokio::join!(
        events::handle_events_list(
            bot.clone(),
            events_message1,
            (*app_state.services).clone(),
            (*app_state.i18n).clone(),
        ),
        events::handle_events_list(
            bot.clone(),
            events_message2,
            (*app_state.services).clone(),
            (*app_state.i18n).clone(),
        ),
        events::handle_events_list(
            bot.clone(),
            events_message3,
            (*app_state.services).clone(),
            (*app_state.i18n).clone(),
        )
    );
    
    assert!(result1.is_ok(), "User 1 events should succeed: {:?}", result1);
    assert!(result2.is_ok(), "User 2 events should succeed: {:?}", result2);
    assert!(result3.is_ok(), "User 3 events should succeed: {:?}", result3);
    
    // Verify all calendar list messages were sent
    ctx.verify_telegram_calls("sendMessage", 3).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test events command error handling
#[tokio::test]
#[serial]
async fn test_events_command_error_handling() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: false, // Don't setup default mocks
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    // Setup error mocks
    ctx.setup_telegram_mocks(crate::helpers::MockScenario::Error).await;
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 123456800i64;
    let chat_id = user_id;
    
    // Create user in database
    let _user = ctx.database.create_test_user(
        user_id,
        Some("test_user".to_string()),
        "Test User".to_string(),
    ).await.expect("Failed to create test user");
    
    let events_message = create_simple_test_message(user_id, chat_id, "/events");
    
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        (*app_state.services).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    // The command should fail due to API error
    assert!(result.is_err(), "Events command should fail with API error");
    
    // Verify the API call was attempted
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}