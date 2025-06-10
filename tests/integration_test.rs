//! Integration test to verify test infrastructure setup
//! 
//! This test file verifies that all test infrastructure components
//! work correctly together.

mod helpers;

use helpers::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_infrastructure_setup() {
    // Test that we can create a test context
    let ctx = TestContext::new().await.expect("Failed to create test context");
    
    // Test database connectivity
    let user_count = ctx.database.count_records("users").await.expect("Failed to count users");
    assert_eq!(user_count, 0); // Should be empty initially
    
    // Test fixture loading
    ctx.load_fixtures().await.expect("Failed to load fixtures");
    let user_count_after = ctx.database.count_records("users").await.expect("Failed to count users");
    assert!(user_count_after > 0); // Should have test users
    
    // Test mock server
    assert!(!ctx.telegram_api_url().is_empty());
    
    // Test cleanup
    ctx.cleanup().await.expect("Failed to cleanup");
    let user_count_final = ctx.database.count_records("users").await.expect("Failed to count users");
    assert_eq!(user_count_final, 0); // Should be empty after cleanup
}

#[tokio::test]
#[serial]
async fn test_telegram_mock_server() {
    let mock_server = TelegramMockServer::new().await;
    
    // Setup default mocks
    mock_server.setup_default_mocks().await;
    
    // Test API URL generation
    let token = test_bot_token();
    let api_url = mock_server.get_api_url(&token);
    assert!(api_url.contains(&token));
    
    // Test that we can reset mocks
    mock_server.reset().await;
}

#[tokio::test]
#[serial]
async fn test_database_helper() {
    let db = TestDatabase::new().await.expect("Failed to create test database");
    
    // Test fixture loading
    db.load_fixtures().await.expect("Failed to load fixtures");
    
    // Test that we can query test data
    let test_user = db.get_test_user(123456789).await.expect("Failed to get test user");
    assert!(test_user.is_some());
    
    let user = test_user.unwrap();
    assert_eq!(user.telegram_id, 123456789);
    assert_eq!(user.username, Some("testuser1".to_string()));
    
    // Test cleanup
    db.cleanup().await.expect("Failed to cleanup database");
    
    let user_count = db.count_records("users").await.expect("Failed to count users");
    assert_eq!(user_count, 0);
}

#[tokio::test]
#[serial]
async fn test_custom_context_config() {
    let config = TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: false,
        bot_token: Some("custom_test_token".to_string()),
    };
    
    let ctx = TestContext::new_with_config(config).await.expect("Failed to create test context");
    
    assert_eq!(ctx.bot_token, "custom_test_token");
    assert!(ctx.redis_connection.is_none());
    
    // Cleanup
    ctx.cleanup().await.expect("Failed to cleanup");
}

// Test using the helper macros
test_with_context!(test_macro_usage, async |ctx: &TestContext| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Test that the macro works correctly
    let user_count = ctx.database.count_records("users").await?;
    assert!(user_count > 0); // Fixtures should be loaded
    
    Ok(())
});

test_with_custom_context!(
    test_custom_macro_usage,
    (TestConfig {
        use_database: true,
        use_redis: false,
        setup_default_mocks: true,
        bot_token: Some("macro_test_token".to_string()),
    }),
    async |ctx: &TestContext| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        assert_eq!(ctx.bot_token, "macro_test_token");
        Ok(())
    }
);