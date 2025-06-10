//! Comprehensive integration tests for SwingBuddy
//!
//! This is the main integration test file that includes all test modules
//! and provides a unified entry point for running the complete test suite.

// Test modules
mod helpers;
mod fixtures;
mod integration;

// Re-export for convenience
pub use helpers::*;
pub use fixtures::*;
pub use integration::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    
    /// Smoke test to ensure all modules compile and are accessible
    #[tokio::test]
    #[serial]
    async fn test_comprehensive_test_suite_smoke_test() {
        // This test ensures that all test modules compile correctly
        // and the test infrastructure is properly set up
        
        let ctx = TestContext::new().await
            .expect("Failed to create test context");
        
        // Verify test context is working
        assert!(!ctx.settings.bot.token.is_empty());
        assert!(!ctx.settings.database.url.is_empty());
        
        // Verify fixtures can be created
        let fixtures = TestFixtures::new();
        assert_eq!(fixtures.users.all_users().len(), 4);
        assert_eq!(fixtures.events.all_events().len(), 4);
        assert_eq!(fixtures.groups.all_groups().len(), 3);
        
        // Verify test helpers work
        let test_message = create_simple_test_message(123, 123, "test");
        assert!(test_message.from.is_some());
        
        let test_callback = create_simple_test_callback_query(123, 123, "test:data");
        assert!(test_callback.data.is_some());
        
        ctx.cleanup().await.expect("Failed to cleanup test context");
    }
    
    /// Test that verifies the complete test infrastructure
    #[tokio::test]
    #[serial]
    async fn test_complete_test_infrastructure() {
        let ctx = setup_integration_test().await
            .expect("Failed to setup integration test");
        
        // Test database functionality
        let user_count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
            .fetch_one(ctx.db_pool())
            .await
            .expect("Failed to query database");
        
        // Should have fixture users loaded
        assert!(user_count.unwrap_or(0) > 0, "Test fixtures should be loaded");
        
        // Test mock server functionality
        let bot = ctx.create_bot().await.expect("Failed to create bot");
        
        // Test that mock server responds
        ctx.verify_telegram_calls("sendMessage", 0).await; // Should start with 0 calls
        
        // Test app state creation
        let app_state = ctx.create_app_state().await.expect("Failed to create app state");
        assert!(app_state.services.user_service.get_user_by_telegram_id(100001).await.is_ok());
        
        teardown_integration_test(ctx).await
            .expect("Failed to teardown integration test");
    }
}