//! Complete user journey integration tests
//!
//! These tests verify end-to-end user scenarios from onboarding to event participation.

use crate::helpers::TestContext;
use crate::integration::{setup_integration_test, teardown_integration_test};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_complete_user_journey_with_location() {
    let ctx = setup_integration_test().await
        .expect("Failed to setup integration test");
    
    let user_id = 300001i64;
    
    // Run complete user journey
    let result = run_complete_user_journey(&ctx, user_id, "en", "Journey User", Some("Moscow")).await;
    
    teardown_integration_test(ctx).await
        .expect("Failed to teardown integration test");
    
    result.expect("Complete user journey should succeed");
}

#[tokio::test]
#[serial]
async fn test_complete_user_journey_without_location() {
    let ctx = setup_integration_test().await
        .expect("Failed to setup integration test");
    
    let user_id = 300002i64;
    
    // Run complete user journey without location
    let result = run_complete_user_journey(&ctx, user_id, "ru", "Пользователь", None).await;
    
    teardown_integration_test(ctx).await
        .expect("Failed to teardown integration test");
    
    result.expect("Complete user journey without location should succeed");
}

async fn run_complete_user_journey(
    ctx: &TestContext,
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
    
    // Verify user profile
    super::ScenarioTestUtils::verify_complete_onboarding(ctx, user_id, language, name, location).await?;
    
    // Verify scenario completion
    super::ScenarioTestUtils::verify_scenario_completed(ctx, user_id).await?;
    
    Ok(())
}