//! Minimal test infrastructure verification
//! 
//! This test verifies that the basic test infrastructure components work
//! without complex dependencies.

mod helpers;

use helpers::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_telegram_mock_server_basic() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test that the server was created
    assert!(!mock_server.base_url.is_empty());
    
    // Test API URL generation
    let token = test_bot_token();
    let api_url = mock_server.get_api_url(&token);
    assert!(api_url.contains(&token));
    assert!(!api_url.contains("{token}"));
}

#[tokio::test]
#[serial]
async fn test_mock_response_configuration() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test default configuration
    let default_config = MockResponseConfig::default();
    assert!(default_config.success);
    assert!(default_config.delay_ms.is_none());
    assert!(default_config.custom_response.is_none());
    
    // Test custom configuration
    let custom_config = MockResponseConfig {
        success: false,
        delay_ms: Some(500),
        custom_response: None,
    };
    assert!(!custom_config.success);
    assert_eq!(custom_config.delay_ms, Some(500));
    
    // Setup mocks with different configurations
    mock_server.mock_send_message(default_config).await;
    mock_server.mock_get_me(custom_config).await;
}

#[tokio::test]
#[serial]
async fn test_mock_server_reset() {
    let mock_server = TelegramMockServer::new().await;
    
    // Setup some mocks
    mock_server.setup_default_mocks().await;
    
    // Reset the server
    mock_server.reset().await;
    
    // Server should still be functional after reset
    let api_url = mock_server.get_api_url(&test_bot_token());
    assert!(!api_url.is_empty());
}

#[test]
fn test_helper_functions() {
    // Test that helper functions return expected values
    assert_eq!(test_bot_token(), "12345:test_token");
    assert_eq!(test_chat_id(), -1001234567890);
    assert_eq!(test_user_id(), 987654321);
}

#[test]
fn test_simple_context() {
    let ctx = SimpleTestContext::new().expect("Failed to create simple context");
    assert!(ctx.config.mock_telegram);
    assert!(ctx.config.use_temp_files);
    assert!(ctx.temp_dir.is_some());
    
    let temp_path = ctx.temp_path().expect("Should have temp path");
    assert!(temp_path.exists());
}

#[test]
fn test_simple_context_custom_config() {
    let config = SimpleTestConfig {
        mock_telegram: false,
        use_temp_files: false,
    };
    
    let ctx = SimpleTestContext::new_with_config(config).expect("Failed to create context");
    assert!(!ctx.config.mock_telegram);
    assert!(!ctx.config.use_temp_files);
    assert!(ctx.temp_dir.is_none());
    assert!(ctx.temp_path().is_none());
}

#[tokio::test]
#[serial]
async fn test_mock_endpoints() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test individual endpoint mocking
    let success_config = MockResponseConfig::default();
    mock_server.mock_send_message(success_config.clone()).await;
    mock_server.mock_edit_message_text(success_config.clone()).await;
    mock_server.mock_answer_callback_query(success_config.clone()).await;
    mock_server.mock_get_me(success_config).await;
    
    // Test error scenarios
    let error_config = MockResponseConfig {
        success: false,
        delay_ms: None,
        custom_response: None,
    };
    
    mock_server.mock_send_message(error_config).await;
}

#[tokio::test]
#[serial]
async fn test_mock_scenarios() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test different mock scenarios
    mock_server.setup_default_mocks().await;
    mock_server.setup_error_mocks().await;
    mock_server.setup_timeout_mocks(1000).await;
    
    // All should complete without errors
}