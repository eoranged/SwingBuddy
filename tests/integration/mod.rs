//! Integration tests module
//!
//! This module contains all integration tests for the SwingBuddy Telegram bot,
//! organized by functionality and test scenarios.

pub mod handlers;
pub mod scenarios;

use std::sync::Once;
use tracing_subscriber;

static INIT: Once = Once::new();

/// Initialize logging for tests (called once)
pub fn init_test_logging() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_test_writer()
            .try_init();
    });
}

/// Common setup function for integration tests
pub async fn setup_integration_test() -> Result<crate::helpers::TestContext, Box<dyn std::error::Error + Send + Sync>> {
    init_test_logging();
    
    let config = crate::helpers::TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = crate::helpers::TestContext::new_with_config(config).await?;
    ctx.load_fixtures().await?;
    
    Ok(ctx)
}

/// Common teardown function for integration tests
pub async fn teardown_integration_test(ctx: crate::helpers::TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ctx.cleanup().await
}

/// Helper macro for running integration tests with automatic setup/teardown
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, async $test_body:expr) => {
        #[tokio::test]
        #[serial_test::serial]
        async fn $test_name() {
            let ctx = crate::integration::setup_integration_test().await
                .expect("Failed to setup integration test");
            
            let test_fn = $test_body;
            let result = test_fn(&ctx).await;
            
            crate::integration::teardown_integration_test(ctx).await
                .expect("Failed to teardown integration test");
            
            if let Err(e) = result {
                panic!("Integration test failed: {:?}", e);
            }
        }
    };
}

/// Helper function to create a complete user journey test scenario
pub async fn run_complete_user_journey(
    ctx: &crate::helpers::TestContext,
    user_id: i64,
    language: &str,
    name: &str,
    location: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use SwingBuddy::handlers::commands::start;
    use SwingBuddy::handlers::callbacks::handle_callback_query;
    use crate::helpers::{create_simple_test_message, create_simple_test_callback_query};
    
    let bot = ctx.create_bot().await?;
    let app_state = ctx.create_app_state().await?;
    
    // Step 1: Start onboarding
    let start_message = create_simple_test_message(user_id, user_id, "/start");
    start::handle_start(
        bot.clone(),
        start_message,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await?;
    
    // Step 2: Select language
    let lang_callback = create_simple_test_callback_query(user_id, user_id, &format!("lang:{}", language));
    handle_callback_query(
        bot.clone(),
        lang_callback,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await?;
    
    // Step 3: Provide name
    let name_message = create_simple_test_message(user_id, user_id, name);
    start::handle_name_input(
        bot.clone(),
        name_message,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await?;
    
    // Step 4: Select location or skip
    let location_data = if let Some(loc) = location {
        format!("location:{}", loc)
    } else {
        "location:skip".to_string()
    };
    
    let location_callback = create_simple_test_callback_query(user_id, user_id, &location_data);
    handle_callback_query(
        bot.clone(),
        location_callback,
        (*app_state.services).clone(),
        (*app_state.scenario_manager).clone(),
        (*app_state.state_storage).clone(),
        (*app_state.i18n).clone(),
    ).await?;
    
    Ok(())
}

/// Helper function to verify user profile after onboarding
pub async fn verify_user_profile(
    ctx: &crate::helpers::TestContext,
    user_id: i64,
    expected_language: &str,
    expected_name: &str,
    expected_location: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use SwingBuddy::models::user::User as DbUser;
    
    let user = sqlx::query_as!(
        DbUser,
        r#"
        SELECT
            id,
            telegram_id,
            username as "username?",
            first_name as "first_name?",
            last_name as "last_name?",
            language_code as "language_code!",
            location as "location?",
            is_banned as "is_banned!",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM users WHERE telegram_id = $1
        "#,
        user_id
    )
    .fetch_one(ctx.db_pool())
    .await?;
    
    assert_eq!(user.language_code, expected_language, "Language mismatch");
    assert_eq!(user.first_name.as_deref(), Some(expected_name), "Name mismatch");
    
    if let Some(expected_loc) = expected_location {
        assert_eq!(user.location.as_deref(), Some(expected_loc), "Location mismatch");
    } else {
        assert!(user.location.is_none(), "Location should be None");
    }
    
    Ok(())
}

/// Helper function to verify onboarding completion
pub async fn verify_onboarding_completed(
    ctx: &crate::helpers::TestContext,
    user_id: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app_state = ctx.create_app_state().await?;
    
    let context = app_state.state_storage.load_context(user_id).await?;
    assert!(context.is_none(), "Onboarding should be completed (no active context)");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    
    #[tokio::test]
    #[serial]
    async fn test_integration_test_setup_teardown() {
        let ctx = setup_integration_test().await
            .expect("Setup should succeed");
        
        // Verify test context is properly initialized
        assert!(!ctx.settings.bot.token.is_empty());
        assert!(!ctx.settings.database.url.is_empty());
        
        teardown_integration_test(ctx).await
            .expect("Teardown should succeed");
    }
    
    #[tokio::test]
    #[serial]
    async fn test_complete_user_journey_helper() {
        let ctx = setup_integration_test().await
            .expect("Setup should succeed");
        
        let user_id = 999888777i64;
        
        // Test complete journey with location
        run_complete_user_journey(&ctx, user_id, "en", "Test User", Some("Moscow")).await
            .expect("Complete user journey should succeed");
        
        // Verify the results
        verify_user_profile(&ctx, user_id, "en", "Test User", Some("Moscow")).await
            .expect("User profile verification should succeed");
        
        verify_onboarding_completed(&ctx, user_id).await
            .expect("Onboarding completion verification should succeed");
        
        teardown_integration_test(ctx).await
            .expect("Teardown should succeed");
    }
    
    #[tokio::test]
    #[serial]
    async fn test_complete_user_journey_with_skip() {
        let ctx = setup_integration_test().await
            .expect("Setup should succeed");
        
        let user_id = 999888778i64;
        
        // Test complete journey with location skip
        run_complete_user_journey(&ctx, user_id, "ru", "Тестовый Пользователь", None).await
            .expect("Complete user journey with skip should succeed");
        
        // Verify the results
        verify_user_profile(&ctx, user_id, "ru", "Тестовый Пользователь", None).await
            .expect("User profile verification should succeed");
        
        verify_onboarding_completed(&ctx, user_id).await
            .expect("Onboarding completion verification should succeed");
        
        teardown_integration_test(ctx).await
            .expect("Teardown should succeed");
    }
}