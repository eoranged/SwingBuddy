//! Simple integration test to verify basic test infrastructure
//! 
//! This test file verifies that the basic test infrastructure components
//! work correctly without complex dependencies.

mod helpers;

use helpers::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_simple_infrastructure_setup() {
    // Test that we can create a simple test context
    let ctx = SimpleTestContext::new().expect("Failed to create simple test context");
    
    // Test that temp directory is available
    assert!(ctx.temp_path().is_some());
    let temp_path = ctx.temp_path().unwrap();
    assert!(temp_path.exists());
}

#[tokio::test]
#[serial]
async fn test_telegram_mock_basic() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test API URL generation
    let token = test_bot_token();
    let api_url = mock_server.get_api_url(&token);
    assert!(api_url.contains(&token));
    assert!(!api_url.is_empty());
}

#[tokio::test]
#[serial]
async fn test_mock_response_config() {
    let config = MockResponseConfig::default();
    assert!(config.success);
    assert!(config.delay_ms.is_none());
    assert!(config.custom_response.is_none());
    
    let error_config = MockResponseConfig {
        success: false,
        delay_ms: Some(100),
        custom_response: None,
    };
    assert!(!error_config.success);
    assert_eq!(error_config.delay_ms, Some(100));
}

#[tokio::test]
#[serial]
async fn test_helper_functions() {
    // Test helper functions work
    let token = test_bot_token();
    assert_eq!(token, "12345:test_token");
    
    let chat_id = test_chat_id();
    assert_eq!(chat_id, -1001234567890);
    
    let user_id = test_user_id();
    assert_eq!(user_id, 987654321);
}

#[tokio::test]
#[serial]
async fn test_simple_config_variations() {
    let config1 = SimpleTestConfig {
        mock_telegram: true,
        use_temp_files: true,
    };
    
    let ctx1 = SimpleTestContext::new_with_config(config1).expect("Failed to create context");
    assert!(ctx1.config.mock_telegram);
    assert!(ctx1.temp_dir.is_some());
    
    let config2 = SimpleTestConfig {
        mock_telegram: false,
        use_temp_files: false,
    };
    
    let ctx2 = SimpleTestContext::new_with_config(config2).expect("Failed to create context");
    assert!(!ctx2.config.mock_telegram);
    assert!(ctx2.temp_dir.is_none());
}

#[test]
fn test_sync_helper_functions() {
    // Test that sync functions work
    let ctx = SimpleTestContext::new().expect("Failed to create context");
    assert!(ctx.config.mock_telegram);
    assert!(ctx.config.use_temp_files);
}