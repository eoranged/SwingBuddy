//! Onboarding scenario integration tests
//!
//! These tests verify the complete user onboarding flow from start to completion.

use crate::helpers::TestContext;
use crate::integration::{setup_integration_test, teardown_integration_test};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_successful_onboarding_with_location() {
    let ctx = setup_integration_test().await
        .expect("Failed to setup integration test");
    
    let user_id = 400001i64;
    
    let result = verify_successful_onboarding(&ctx, user_id, "en", "Test User", Some("Moscow")).await;
    
    teardown_integration_test(ctx).await
        .expect("Failed to teardown integration test");
    
    result.expect("Successful onboarding should work");
}

#[tokio::test]
#[serial]
async fn test_successful_onboarding_without_location() {
    let ctx = setup_integration_test().await
        .expect("Failed to setup integration test");
    
    let user_id = 400002i64;
    
    let result = verify_successful_onboarding(&ctx, user_id, "ru", "Тестовый Пользователь", None).await;
    
    teardown_integration_test(ctx).await
        .expect("Failed to teardown integration test");
    
    result.expect("Successful onboarding without location should work");
}

async fn verify_successful_onboarding(
    ctx: &TestContext,
    user_id: i64,
    language: &str,
    name: &str,
    location: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use SwingBuddy::models::user::User as DbUser;
    
    // Verify user was created with correct data
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
    
    assert_eq!(user.language_code, language);
    assert_eq!(user.first_name.as_deref(), Some(name));
    
    if let Some(expected_location) = location {
        assert_eq!(user.location.as_deref(), Some(expected_location));
    } else {
        assert!(user.location.is_none());
    }
    
    // Verify onboarding state is cleared
    super::ScenarioTestUtils::verify_scenario_completed(ctx, user_id).await?;
    
    Ok(())
}