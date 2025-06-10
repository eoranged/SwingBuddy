//! Integration tests for language selection callback handlers
//!
//! This module contains comprehensive tests for language selection callbacks,
//! including language switching functionality and invalid callback handling.

use serial_test::serial;
use teloxide::types::{CallbackQuery, ChatId};
use teloxide::Bot;
use SwingBuddy::handlers::callbacks::handle_callback_query;
use SwingBuddy::handlers::commands::start;
use SwingBuddy::models::user::{User as DbUser};
use SwingBuddy::state::ConversationContext;

use crate::helpers::{TestContext, TestConfig, create_simple_test_callback_query, create_simple_test_message};

/// Test language selection callback during onboarding
#[tokio::test]
#[serial]
async fn test_language_selection_callback_onboarding() {
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
    let chat_id = user_id;
    
    // Start onboarding first
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    let result = start::handle_start(
        bot.clone(),
        start_message,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Start command should succeed: {:?}", result);
    
    // Test English language selection
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    
    let result = handle_callback_query(
        bot.clone(),
        lang_callback,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Language selection callback should succeed: {:?}", result);
    
    // Verify user language was updated in database
    let user = sqlx::query_as!(
        DbUser,
        r#"
        SELECT
            id,
            telegram_id,
            username,
            first_name,
            last_name,
            is_bot,
            language_code,
            is_premium,
            added_to_attachment_menu,
            can_join_groups,
            can_read_all_group_messages,
            supports_inline_queries,
            location,
            is_banned,
            created_at,
            updated_at
        FROM users WHERE telegram_id = $1
        "#,
        user_id
    )
    .fetch_one(ctx.db_pool())
    .await
    .expect("User should exist in database");
    
    assert_eq!(user.language_code, "en");
    
    // Verify conversation state moved to name_input
    let context = app_state.state_storage.load_context(user_id).await
        .expect("Failed to load context")
        .expect("Context should exist");
    
    assert_eq!(context.scenario.as_deref(), Some("onboarding"));
    assert_eq!(context.step.as_deref(), Some("name_input"));
    
    // Verify language was stored in context
    let stored_lang = context.get_string("language").unwrap();
    assert_eq!(stored_lang, "en");
    
    // Verify Telegram API calls (start message + language confirmation + name request)
    ctx.verify_telegram_calls("sendMessage", 3).await;
    ctx.verify_telegram_calls("answerCallbackQuery", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test Russian language selection callback
#[tokio::test]
#[serial]
async fn test_russian_language_selection_callback() {
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
    
    // Start onboarding first
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await.expect("Start should succeed");
    
    // Test Russian language selection
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:ru");
    
    let result = handle_callback_query(
        bot.clone(),
        lang_callback,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    assert!(result.is_ok(), "Russian language selection should succeed: {:?}", result);
    
    // Verify user language was updated to Russian
    let user = sqlx::query_as!(
        DbUser,
        r#"
        SELECT
            id,
            telegram_id,
            username,
            first_name,
            last_name,
            is_bot,
            language_code,
            is_premium,
            added_to_attachment_menu,
            can_join_groups,
            can_read_all_group_messages,
            supports_inline_queries,
            location,
            is_banned,
            created_at,
            updated_at
        FROM users WHERE telegram_id = $1
        "#,
        user_id
    )
    .fetch_one(ctx.db_pool())
    .await
    .expect("User should exist in database");
    
    assert_eq!(user.language_code, "ru");
    
    // Verify language was stored in context
    let context = app_state.state_storage.load_context(user_id).await
        .expect("Failed to load context")
        .expect("Context should exist");
    
    let stored_lang = context.get_string("language").unwrap();
    assert_eq!(stored_lang, "ru");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test invalid language callback handling
#[tokio::test]
#[serial]
async fn test_invalid_language_callback() {
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
    
    // Start onboarding first
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await.expect("Start should succeed");
    
    // Test invalid language codes
    let invalid_languages = vec!["fr", "de", "es", "invalid", ""];
    
    for invalid_lang in invalid_languages {
        let lang_callback = create_simple_test_callback_query(
            user_id, 
            chat_id, 
            &format!("lang:{}", invalid_lang)
        );
        
        let result = handle_callback_query(
            bot.clone(),
            lang_callback,
            (*app_state.services).clone(),
            (*app_state.scenario_manager).clone(),
            (*app_state.state_storage).clone(),
            (*app_state.i18n).clone(),
        ).await;
        
        // Should handle invalid language gracefully
        assert!(result.is_ok(), "Invalid language callback should be handled gracefully: {}", invalid_lang);
        
        // Verify user is still in language_selection step
        let context = app_state.state_storage.load_context(user_id).await
            .expect("Failed to load context")
            .expect("Context should exist");
        
        assert_eq!(context.step.as_deref(), Some("language_selection"), 
                  "Should remain in language_selection step for invalid language: {}", invalid_lang);
    }
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test language callback without active onboarding context
#[tokio::test]
#[serial]
async fn test_language_callback_no_context() {
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
    
    // Create user but don't start onboarding
    let _user = ctx.database.create_test_user(
        user_id,
        Some("test_user".to_string()),
        "Test User".to_string(),
    ).await.expect("Failed to create test user");
    
    // Try language selection without context
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    
    let result = handle_callback_query(
        bot.clone(),
        lang_callback,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    // Should handle gracefully (might return error or ignore)
    // The exact behavior depends on implementation
    // For now, we verify it doesn't crash
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test language callback in wrong onboarding step
#[tokio::test]
#[serial]
async fn test_language_callback_wrong_step() {
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
    
    // Create user and manually set context to wrong step
    let _user = ctx.database.create_test_user(
        user_id,
        Some("test_user".to_string()),
        "Test User".to_string(),
    ).await.expect("Failed to create test user");
    
    // Create context in name_input step instead of language_selection
    let mut context = ConversationContext::new(user_id);
    context.scenario = Some("onboarding".to_string());
    context.step = Some("name_input".to_string());
    
    app_state.state_storage.save_context(&context).await
        .expect("Failed to save context");
    
    // Try language selection in wrong step
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    
    let result = handle_callback_query(
        bot.clone(),
        lang_callback,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await;
    
    // Should handle gracefully and not change the step
    assert!(result.is_ok(), "Language callback in wrong step should be handled gracefully");
    
    // Verify step didn't change
    let context_after = app_state.state_storage.load_context(user_id).await
        .expect("Failed to load context")
        .expect("Context should exist");
    
    assert_eq!(context_after.step.as_deref(), Some("name_input"), 
              "Step should remain unchanged");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test concurrent language selection callbacks
#[tokio::test]
#[serial]
async fn test_concurrent_language_callbacks() {
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
    
    let user1_id = 123456794i64;
    let user2_id = 123456795i64;
    let user3_id = 123456796i64;
    
    // Start onboarding for all users
    for &user_id in &[user1_id, user2_id, user3_id] {
        let start_message = create_simple_test_message(user_id, user_id, "/start");
        start::handle_start(
            bot.clone(),
            start_message,
            (*app_state.services).clone(),
            (*app_state.scenario_manager).clone(),
            (*app_state.state_storage).clone(),
            (*app_state.i18n).clone(),
        ).await.expect("Start should succeed");
    }
    
    // Create concurrent language selection callbacks
    let lang_callback1 = create_simple_test_callback_query(user1_id, user1_id, "lang:en");
    let lang_callback2 = create_simple_test_callback_query(user2_id, user2_id, "lang:ru");
    let lang_callback3 = create_simple_test_callback_query(user3_id, user3_id, "lang:en");
    
    // Execute all callbacks concurrently
    let (result1, result2, result3) = tokio::join!(
        handle_callback_query(
            bot.clone(),
            lang_callback1,
            (*app_state.services).clone(),
            (*app_state.scenario_manager).clone(),
            (*app_state.state_storage).clone(),
            (*app_state.i18n).clone(),
        ),
        handle_callback_query(
            bot.clone(),
            lang_callback2,
            (*app_state.services).clone(),
            (*app_state.scenario_manager).clone(),
            (*app_state.state_storage).clone(),
            (*app_state.i18n).clone(),
        ),
        handle_callback_query(
            bot.clone(),
            lang_callback3,
            (*app_state.services).clone(),
            (*app_state.scenario_manager).clone(),
            (*app_state.state_storage).clone(),
            (*app_state.i18n).clone(),
        )
    );
    
    assert!(result1.is_ok(), "User 1 language callback should succeed: {:?}", result1);
    assert!(result2.is_ok(), "User 2 language callback should succeed: {:?}", result2);
    assert!(result3.is_ok(), "User 3 language callback should succeed: {:?}", result3);
    
    // Verify all users have correct languages
    let user1 = sqlx::query_as!(
        DbUser,
        r#"
        SELECT
            id,
            telegram_id,
            username,
            first_name,
            last_name,
            is_bot,
            language_code,
            is_premium,
            added_to_attachment_menu,
            can_join_groups,
            can_read_all_group_messages,
            supports_inline_queries,
            location,
            is_banned,
            created_at,
            updated_at
        FROM users WHERE telegram_id = $1
        "#,
        user1_id
    )
        .fetch_one(ctx.db_pool()).await.expect("User 1 should exist");
    let user2 = sqlx::query_as!(
        DbUser,
        r#"
        SELECT
            id,
            telegram_id,
            username,
            first_name,
            last_name,
            is_bot,
            language_code,
            is_premium,
            added_to_attachment_menu,
            can_join_groups,
            can_read_all_group_messages,
            supports_inline_queries,
            location,
            is_banned,
            created_at,
            updated_at
        FROM users WHERE telegram_id = $1
        "#,
        user2_id
    )
        .fetch_one(ctx.db_pool()).await.expect("User 2 should exist");
    let user3 = sqlx::query_as!(
        DbUser,
        r#"
        SELECT
            id,
            telegram_id,
            username,
            first_name,
            last_name,
            is_bot,
            language_code,
            is_premium,
            added_to_attachment_menu,
            can_join_groups,
            can_read_all_group_messages,
            supports_inline_queries,
            location,
            is_banned,
            created_at,
            updated_at
        FROM users WHERE telegram_id = $1
        "#,
        user3_id
    )
        .fetch_one(ctx.db_pool()).await.expect("User 3 should exist");
    
    assert_eq!(user1.language_code, "en");
    assert_eq!(user2.language_code, "ru");
    assert_eq!(user3.language_code, "en");
    
    // Verify all users moved to name_input step
    for &user_id in &[user1_id, user2_id, user3_id] {
        let context = app_state.state_storage.load_context(user_id).await
            .expect("Failed to load context")
            .expect("Context should exist");
        
        assert_eq!(context.step.as_deref(), Some("name_input"), 
                  "User {} should be in name_input step", user_id);
    }
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test malformed language callback data
#[tokio::test]
#[serial]
async fn test_malformed_language_callback_data() {
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
    
    let user_id = 123456797i64;
    let chat_id = user_id;
    
    // Start onboarding first
    let start_message = create_simple_test_message(user_id, chat_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await.expect("Start should succeed");
    
    // Test malformed callback data
    let malformed_callbacks = vec![
        "lang",           // Missing language code
        "lang:",          // Empty language code
        "lang:en:extra",  // Extra parts
        "language:en",    // Wrong prefix
        "",               // Empty callback data
    ];
    
    for malformed_data in malformed_callbacks {
        let callback = create_simple_test_callback_query(user_id, chat_id, malformed_data);
        
        let result = handle_callback_query(
            bot.clone(),
            callback,
            (*app_state.services).clone(),
            (*app_state.scenario_manager).clone(),
            (*app_state.state_storage).clone(),
            (*app_state.i18n).clone(),
        ).await;
        
        // Should handle malformed data gracefully
        assert!(result.is_ok(), "Malformed callback should be handled gracefully: {}", malformed_data);
    }
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}