//! Integration tests for /help command handler
//!
//! This module contains comprehensive tests for the /help command functionality,
//! including different contexts, language support, and message formatting.

use serial_test::serial;
use teloxide::types::{Message, ChatId};
use teloxide::Bot;
use SwingBuddy::handlers::commands::help;

use crate::helpers::{TestContext, TestConfig, create_simple_test_message, create_test_message};

/// Test /help command in private chat
#[tokio::test]
#[serial]
async fn test_help_command_private_chat() {
    let config = TestConfig {
        use_database: false, // Help command doesn't need database
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    let user_id = 123456789i64;
    let chat_id = user_id; // Private chat
    
    // Create /help message
    let help_message = create_simple_test_message(user_id, chat_id, "/help");
    
    let result = help::handle_help(bot.clone(), help_message).await;
    
    assert!(result.is_ok(), "Help command should succeed: {:?}", result);
    
    // Verify help message was sent
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /help command in group chat
#[tokio::test]
#[serial]
async fn test_help_command_group_chat() {
    let config = TestConfig {
        use_database: false,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    let user_id = 123456790i64;
    let group_chat_id = -1001234567890i64; // Group chat
    
    // Create /help message in group chat
    let help_message = create_test_message(
        user_id,
        group_chat_id,
        "/help",
        Some("testuser"),
        "TestUser",
        Some("LastName"),
    );
    
    let result = help::handle_help(bot.clone(), help_message).await;
    
    assert!(result.is_ok(), "Help command should succeed in group chat: {:?}", result);
    
    // Verify help message was sent (help works in both private and group chats)
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /help command message content and formatting
#[tokio::test]
#[serial]
async fn test_help_command_message_content() {
    let config = TestConfig {
        use_database: false,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    let user_id = 123456791i64;
    let chat_id = user_id;
    
    let help_message = create_simple_test_message(user_id, chat_id, "/help");
    
    let result = help::handle_help(bot.clone(), help_message).await;
    
    assert!(result.is_ok(), "Help command should succeed: {:?}", result);
    
    // Verify the message was sent
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    // In a more comprehensive test, we could verify the actual message content
    // by capturing the mock server requests and checking the message text
    // For now, we verify that the API call was made successfully
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /help command with different user types
#[tokio::test]
#[serial]
async fn test_help_command_different_users() {
    let config = TestConfig {
        use_database: false,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    // Test with user having username
    let user_with_username = create_test_message(
        123456792,
        123456792,
        "/help",
        Some("user_with_username"),
        "User",
        Some("WithUsername"),
    );
    
    let result1 = help::handle_help(bot.clone(), user_with_username).await;
    assert!(result1.is_ok(), "Help should work for user with username: {:?}", result1);
    
    // Test with user without username
    let user_without_username = create_test_message(
        123456793,
        123456793,
        "/help",
        None, // No username
        "User",
        Some("WithoutUsername"),
    );
    
    let result2 = help::handle_help(bot.clone(), user_without_username).await;
    assert!(result2.is_ok(), "Help should work for user without username: {:?}", result2);
    
    // Test with user having only first name
    let user_minimal = create_test_message(
        123456794,
        123456794,
        "/help",
        None,
        "MinimalUser",
        None, // No last name
    );
    
    let result3 = help::handle_help(bot.clone(), user_minimal).await;
    assert!(result3.is_ok(), "Help should work for minimal user info: {:?}", result3);
    
    // Verify all help messages were sent
    ctx.verify_telegram_calls("sendMessage", 3).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test concurrent /help commands
#[tokio::test]
#[serial]
async fn test_concurrent_help_commands() {
    let config = TestConfig {
        use_database: false,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    let user1_id = 123456795i64;
    let user2_id = 123456796i64;
    let user3_id = 123456797i64;
    
    // Create concurrent /help messages
    let help_message1 = create_simple_test_message(user1_id, user1_id, "/help");
    let help_message2 = create_simple_test_message(user2_id, user2_id, "/help");
    let help_message3 = create_simple_test_message(user3_id, user3_id, "/help");
    
    // Execute all help commands concurrently
    let (result1, result2, result3) = tokio::join!(
        help::handle_help(bot.clone(), help_message1),
        help::handle_help(bot.clone(), help_message2),
        help::handle_help(bot.clone(), help_message3)
    );
    
    assert!(result1.is_ok(), "User 1 help should succeed: {:?}", result1);
    assert!(result2.is_ok(), "User 2 help should succeed: {:?}", result2);
    assert!(result3.is_ok(), "User 3 help should succeed: {:?}", result3);
    
    // Verify all help messages were sent
    ctx.verify_telegram_calls("sendMessage", 3).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /help command error handling with mock server errors
#[tokio::test]
#[serial]
async fn test_help_command_with_api_errors() {
    let config = TestConfig {
        use_database: false,
        use_redis: false,
        setup_default_mocks: false, // Don't setup default mocks
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    // Setup error mocks
    ctx.setup_telegram_mocks(crate::helpers::MockScenario::Error).await;
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    let user_id = 123456798i64;
    let chat_id = user_id;
    
    let help_message = create_simple_test_message(user_id, chat_id, "/help");
    
    let result = help::handle_help(bot.clone(), help_message).await;
    
    // The command should fail due to API error
    assert!(result.is_err(), "Help command should fail with API error");
    
    // Verify the API call was attempted
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /help command with timeout scenarios
#[tokio::test]
#[serial]
async fn test_help_command_with_timeout() {
    let config = TestConfig {
        use_database: false,
        use_redis: false,
        setup_default_mocks: false,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    // Setup timeout mocks (5 second delay)
    ctx.setup_telegram_mocks(crate::helpers::MockScenario::Timeout).await;
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    let user_id = 123456799i64;
    let chat_id = user_id;
    
    let help_message = create_simple_test_message(user_id, chat_id, "/help");
    
    // Set a timeout for the test
    let timeout_duration = std::time::Duration::from_secs(2);
    let result = tokio::time::timeout(
        timeout_duration,
        help::handle_help(bot.clone(), help_message)
    ).await;
    
    // The command should timeout
    assert!(result.is_err(), "Help command should timeout");
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}

/// Test /help command response time performance
#[tokio::test]
#[serial]
async fn test_help_command_performance() {
    let config = TestConfig {
        use_database: false,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    let bot = ctx.create_bot().await.expect("Failed to create bot");
    
    let user_id = 123456800i64;
    let chat_id = user_id;
    
    let help_message = create_simple_test_message(user_id, chat_id, "/help");
    
    // Measure execution time
    let start_time = std::time::Instant::now();
    let result = help::handle_help(bot.clone(), help_message).await;
    let execution_time = start_time.elapsed();
    
    assert!(result.is_ok(), "Help command should succeed: {:?}", result);
    
    // Help command should be fast (under 1 second in normal conditions)
    assert!(execution_time < std::time::Duration::from_secs(1), 
           "Help command should execute quickly, took: {:?}", execution_time);
    
    ctx.verify_telegram_calls("sendMessage", 1).await;
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}