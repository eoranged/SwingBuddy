//! Integration test scenarios
//!
//! This module contains complete user journey scenarios and complex integration tests
//! that test multiple components working together.

pub mod complete_user_journey_test;
pub mod onboarding_test;

use crate::helpers::TestContext;

/// Common scenario test utilities
pub struct ScenarioTestUtils;

impl ScenarioTestUtils {
    /// Helper to verify a complete user onboarding flow
    pub async fn verify_complete_onboarding(
        ctx: &TestContext,
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
        
        assert_eq!(user.language_code, expected_language);
        assert_eq!(user.first_name.as_deref(), Some(expected_name));
        
        if let Some(expected_loc) = expected_location {
            assert_eq!(user.location.as_deref(), Some(expected_loc));
        } else {
            assert!(user.location.is_none());
        }
        
        Ok(())
    }
    
    /// Helper to verify user state is cleared after scenario completion
    pub async fn verify_scenario_completed(
        ctx: &TestContext,
        user_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app_state = ctx.create_app_state().await?;
        
        let context = app_state.state_storage.load_context(user_id).await?;
        assert!(context.is_none(), "User should have no active scenario context");
        
        Ok(())
    }
}