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
    // First, run the complete onboarding flow
    crate::integration::run_complete_user_journey(ctx, user_id, language, name, location).await?;
    
    // Then verify the results
    crate::integration::verify_user_profile(ctx, user_id, language, name, location).await?;
    
    // Verify onboarding state is cleared
    crate::integration::verify_onboarding_completed(ctx, user_id).await?;
    
    Ok(())
}