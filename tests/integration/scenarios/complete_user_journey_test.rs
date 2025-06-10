//! Complete user journey integration tests
//!
//! This module contains end-to-end tests that simulate complete user journeys
//! from onboarding to event participation, testing multi-step workflows
//! and cross-feature interactions.

use serial_test::serial;
use std::collections::HashMap;
use teloxide::types::{Message, CallbackQuery, ChatId};
use teloxide::Bot;
use SwingBuddy::handlers::commands::{start, help, events};
use SwingBuddy::handlers::callbacks::handle_callback_query;
use SwingBuddy::models::user::{User as DbUser};

use crate::helpers::{TestContext, TestConfig, create_simple_test_message, create_simple_test_callback_query};
use crate::fixtures::{TestFixtures, load_test_fixtures};

/// Test complete user journey from onboarding to event browsing
#[tokio::test]
#[serial]
async fn test_complete_user_journey_onboarding_to_events() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    // Load test fixtures
    let _fixtures = load_test_fixtures(ctx.db_pool()).await
        .expect("Failed to load test fixtures");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    let app_state = ctx.create_app_state().await.expect("Failed to create app state");
    
    let user_id = 300001i64;
    let chat_id = user_id;
    
    // === PHASE 1: Complete Onboarding ===
    
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
    
    // Step 2: User selects English language
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
    
    // Step 3: User provides name
    let name_message = create_simple_test_message(user_id, chat_id, "Alice Johnson");
    let result = start::handle_name_input(
        bot.clone(),
        name_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Name input should succeed: {:?}", result);
    
    // Step 4: User selects Moscow as location
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
    
    // Verify onboarding completion
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
        user_id
    )
    .fetch_one(ctx.db_pool())
    .await
    .expect("User should exist in database");
    
    assert_eq!(user.language_code, "en");
    assert_eq!(user.first_name.as_deref(), Some("Alice Johnson"));
    assert_eq!(user.location.as_deref(), Some("Moscow"));
    
    // === PHASE 2: User Explores Bot Features ===
    
    // Step 5: User requests help
    let help_message = create_simple_test_message(user_id, chat_id, "/help");
    let result = help::handle_help(bot.clone(), help_message).await;
    assert!(result.is_ok(), "Help command should succeed: {:?}", result);
    
    // Step 6: User browses events
    let events_message = create_simple_test_message(user_id, chat_id, "/events");
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        app_state.services.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Events command should succeed: {:?}", result);
    
    // Step 7: User selects swing events calendar
    let calendar_callback = create_simple_test_callback_query(user_id, chat_id, "calendar:swing_events");
    let result = handle_callback_query(
        bot.clone(),
        calendar_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Calendar selection should succeed: {:?}", result);
    
    // Step 8: User goes back to calendar list
    let back_callback = create_simple_test_callback_query(user_id, chat_id, "calendar:back");
    let result = handle_callback_query(
        bot.clone(),
        back_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Back navigation should succeed: {:?}", result);
    
    // Step 9: User selects workshops calendar
    let workshop_callback = create_simple_test_callback_query(user_id, chat_id, "calendar:workshops");
    let result = handle_callback_query(
        bot.clone(),
        workshop_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Workshop calendar selection should succeed: {:?}", result);
    
    // Verify all API calls were made successfully
    // The exact number depends on implementation, but should be substantial
    let received_requests = ctx.telegram_mock.server.received_requests().await.unwrap();
    assert!(received_requests.len() >= 8, "Should have made multiple API calls during journey");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test user journey with location skip
#[tokio::test]
#[serial]
async fn test_user_journey_with_location_skip() {
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
    
    let user_id = 300002i64;
    let chat_id = user_id;
    
    // Complete onboarding with location skip
    crate::integration::run_complete_user_journey(&ctx, user_id, "ru", "Борис Петров", None).await
        .expect("Complete user journey should succeed");
    
    // Verify user profile
    crate::integration::verify_user_profile(&ctx, user_id, "ru", "Борис Петров", None).await
        .expect("User profile verification should succeed");
    
    // Verify onboarding completion
    crate::integration::verify_onboarding_completed(&ctx, user_id).await
        .expect("Onboarding completion verification should succeed");
    
    // Continue with post-onboarding activities
    let events_message = create_simple_test_message(user_id, chat_id, "/events");
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        app_state.services.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Events command should succeed after onboarding: {:?}", result);
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test multiple users going through onboarding simultaneously
#[tokio::test]
#[serial]
async fn test_concurrent_user_journeys() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let user_scenarios = vec![
        (300003i64, "en", "User One", Some("Moscow")),
        (300004i64, "ru", "Пользователь Два", Some("Saint Petersburg")),
        (300005i64, "en", "User Three", None), // Skip location
    ];
    
    // Run all user journeys concurrently
    let futures: Vec<_> = user_scenarios.iter().map(|&(user_id, lang, name, location)| {
        crate::integration::run_complete_user_journey(&ctx, user_id, lang, name, location)
    }).collect();
    
    let results = futures::future::join_all(futures).await;
    
    // Verify all journeys succeeded
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "User journey {} should succeed: {:?}", i + 1, result);
    }
    
    // Verify all user profiles
    for &(user_id, lang, name, location) in &user_scenarios {
        crate::integration::verify_user_profile(&ctx, user_id, lang, name, location).await
            .expect("User profile verification should succeed");
        
        crate::integration::verify_onboarding_completed(&ctx, user_id).await
            .expect("Onboarding completion verification should succeed");
    }
    
    // Verify all users exist in database
    let user_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE telegram_id IN ($1, $2, $3)",
        300003i64, 300004i64, 300005i64
    )
    .fetch_one(ctx.db_pool())
    .await
    .expect("Failed to count users");
    
    assert_eq!(user_count, Some(3), "All three users should be created");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test user journey with interruptions and recovery
#[tokio::test]
#[serial]
async fn test_user_journey_with_interruptions() {
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
    
    let user_id = 300006i64;
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
    
    // Select language
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    handle_callback_query(
        bot.clone(),
        lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Language selection should succeed");
    
    // === INTERRUPTION: User tries other commands during onboarding ===
    
    // User tries /help during onboarding
    let help_message = create_simple_test_message(user_id, chat_id, "/help");
    let result = help::handle_help(bot.clone(), help_message).await;
    assert!(result.is_ok(), "Help should work during onboarding: {:?}", result);
    
    // User tries /events during onboarding (should work but show error for private chat requirement)
    let events_message = create_simple_test_message(user_id, chat_id, "/events");
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        app_state.services.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Events should handle onboarding state gracefully: {:?}", result);
    
    // === RECOVERY: User continues onboarding ===
    
    // Verify user is still in onboarding state
    let context = app_state.state_storage.load_context(user_id).await
        .expect("Failed to load context")
        .expect("Context should still exist");
    
    assert_eq!(context.scenario.as_deref(), Some("onboarding"));
    assert_eq!(context.step.as_deref(), Some("name_input"));
    
    // Continue with name input
    let name_message = create_simple_test_message(user_id, chat_id, "Resilient User");
    start::handle_name_input(
        bot.clone(),
        name_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Name input should succeed");
    
    // Complete with location
    let location_callback = create_simple_test_callback_query(user_id, chat_id, "location:Moscow");
    handle_callback_query(
        bot.clone(),
        location_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Location selection should succeed");
    
    // Verify successful completion
    crate::integration::verify_user_profile(&ctx, user_id, "en", "Resilient User", Some("Moscow")).await
        .expect("User profile verification should succeed");
    
    crate::integration::verify_onboarding_completed(&ctx, user_id).await
        .expect("Onboarding completion verification should succeed");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test user journey with invalid inputs and error recovery
#[tokio::test]
#[serial]
async fn test_user_journey_with_invalid_inputs() {
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
    
    let user_id = 300007i64;
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
    
    // Try invalid language first
    let invalid_lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:fr");
    let result = handle_callback_query(
        bot.clone(),
        invalid_lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Invalid language should be handled gracefully: {:?}", result);
    
    // Verify still in language selection
    let context = app_state.state_storage.load_context(user_id).await
        .expect("Failed to load context")
        .expect("Context should exist");
    assert_eq!(context.step.as_deref(), Some("language_selection"));
    
    // Select valid language
    let lang_callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
    handle_callback_query(
        bot.clone(),
        lang_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Valid language selection should succeed");
    
    // Try invalid names
    let invalid_names = vec!["", "A", "123", "@#$%"];
    for invalid_name in invalid_names {
        let name_message = create_simple_test_message(user_id, chat_id, invalid_name);
        let result = start::handle_name_input(
            bot.clone(),
            name_message,
            app_state.services.clone(),
            app_state.scenario_manager.clone(),
            app_state.state_storage.clone(),
            app_state.i18n.clone(),
        ).await;
        
        // Should handle invalid names gracefully
        assert!(result.is_ok(), "Invalid name should be handled gracefully: {}", invalid_name);
        
        // Should remain in name_input step
        let context = app_state.state_storage.load_context(user_id).await
            .expect("Failed to load context")
            .expect("Context should exist");
        assert_eq!(context.step.as_deref(), Some("name_input"));
    }
    
    // Provide valid name
    let name_message = create_simple_test_message(user_id, chat_id, "Valid User");
    start::handle_name_input(
        bot.clone(),
        name_message,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Valid name should succeed");
    
    // Complete with location
    let location_callback = create_simple_test_callback_query(user_id, chat_id, "location:Moscow");
    handle_callback_query(
        bot.clone(),
        location_callback,
        app_state.services.clone(),
        app_state.scenario_manager.clone(),
        app_state.state_storage.clone(),
        app_state.i18n.clone(),
    ).await.expect("Location selection should succeed");
    
    // Verify successful completion despite invalid inputs
    crate::integration::verify_user_profile(&ctx, user_id, "en", "Valid User", Some("Moscow")).await
        .expect("User profile verification should succeed");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test complete admin user journey
#[tokio::test]
#[serial]
async fn test_admin_user_journey() {
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
    
    let admin_user_id = 555666777i64; // Admin ID from test settings
    let chat_id = admin_user_id;
    
    // Complete onboarding for admin user
    crate::integration::run_complete_user_journey(&ctx, admin_user_id, "en", "Admin User", Some("Moscow")).await
        .expect("Admin onboarding should succeed");
    
    // Test admin-specific functionality
    let create_event_message = create_simple_test_message(admin_user_id, chat_id, "/create_event");
    let result = events::handle_create_event(
        bot.clone(),
        create_event_message,
        app_state.services.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Admin should be able to create events: {:?}", result);
    
    // Test regular user functionality still works
    let events_message = create_simple_test_message(admin_user_id, chat_id, "/events");
    let result = events::handle_events_list(
        bot.clone(),
        events_message,
        app_state.services.clone(),
        app_state.i18n.clone(),
    ).await;
    assert!(result.is_ok(), "Admin should be able to list events: {:?}", result);
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}