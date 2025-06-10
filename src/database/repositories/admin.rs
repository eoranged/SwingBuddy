//! Admin repository implementation

use sqlx::PgPool;
use chrono::Utc;
use crate::models::admin::{AdminSettings, UserState, CasCheck, CreateAdminSettingRequest, UpdateAdminSettingRequest, CreateUserStateRequest, UpdateUserStateRequest, CreateCasCheckRequest};
use crate::utils::errors::SwingBuddyError;

#[derive(Clone)]
pub struct AdminRepository {
    pool: PgPool,
}

impl AdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Admin Settings methods
    /// Create a new admin setting
    pub async fn create_setting(&self, request: CreateAdminSettingRequest) -> Result<AdminSettings, SwingBuddyError> {
        let setting = sqlx::query_as::<_, AdminSettings>(
            r#"
            INSERT INTO admin_settings (key, value, updated_by, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, key, value, updated_by, updated_at
            "#
        )
        .bind(request.key)
        .bind(request.value)
        .bind(request.updated_by)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(setting)
    }

    /// Get admin setting by key
    pub async fn get_setting(&self, key: &str) -> Result<Option<AdminSettings>, SwingBuddyError> {
        let setting = sqlx::query_as::<_, AdminSettings>(
            "SELECT id, key, value, updated_by, updated_at FROM admin_settings WHERE key = $1"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(setting)
    }

    /// Update admin setting
    pub async fn update_setting(&self, key: &str, request: UpdateAdminSettingRequest) -> Result<AdminSettings, SwingBuddyError> {
        let setting = sqlx::query_as::<_, AdminSettings>(
            r#"
            UPDATE admin_settings
            SET value = $2, updated_by = $3, updated_at = $4
            WHERE key = $1
            RETURNING id, key, value, updated_by, updated_at
            "#
        )
        .bind(key)
        .bind(request.value)
        .bind(request.updated_by)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(setting)
    }

    /// Delete admin setting
    pub async fn delete_setting(&self, key: &str) -> Result<(), SwingBuddyError> {
        sqlx::query("DELETE FROM admin_settings WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List all admin settings
    pub async fn list_settings(&self) -> Result<Vec<AdminSettings>, SwingBuddyError> {
        let settings = sqlx::query_as::<_, AdminSettings>(
            "SELECT id, key, value, updated_by, updated_at FROM admin_settings ORDER BY key ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(settings)
    }

    // User State methods
    /// Create or update user state
    pub async fn upsert_user_state(&self, request: CreateUserStateRequest) -> Result<UserState, SwingBuddyError> {
        let state = sqlx::query_as::<_, UserState>(
            r#"
            INSERT INTO user_states (user_id, scenario, step, data, expires_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id)
            DO UPDATE SET
                scenario = EXCLUDED.scenario,
                step = EXCLUDED.step,
                data = EXCLUDED.data,
                expires_at = EXCLUDED.expires_at,
                updated_at = EXCLUDED.updated_at
            RETURNING user_id, scenario, step, data, expires_at, updated_at
            "#
        )
        .bind(request.user_id)
        .bind(request.scenario)
        .bind(request.step)
        .bind(request.data.unwrap_or_else(|| serde_json::json!({})))
        .bind(request.expires_at)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(state)
    }

    /// Get user state
    pub async fn get_user_state(&self, user_id: i64) -> Result<Option<UserState>, SwingBuddyError> {
        let state = sqlx::query_as::<_, UserState>(
            "SELECT user_id, scenario, step, data, expires_at, updated_at FROM user_states WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(state)
    }

    /// Update user state
    pub async fn update_user_state(&self, user_id: i64, request: UpdateUserStateRequest) -> Result<UserState, SwingBuddyError> {
        let state = sqlx::query_as::<_, UserState>(
            r#"
            UPDATE user_states
            SET scenario = COALESCE($2, scenario),
                step = COALESCE($3, step),
                data = COALESCE($4, data),
                expires_at = COALESCE($5, expires_at),
                updated_at = $6
            WHERE user_id = $1
            RETURNING user_id, scenario, step, data, expires_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(request.scenario)
        .bind(request.step)
        .bind(request.data)
        .bind(request.expires_at)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(state)
    }

    /// Delete user state
    pub async fn delete_user_state(&self, user_id: i64) -> Result<(), SwingBuddyError> {
        sqlx::query("DELETE FROM user_states WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clean expired user states
    pub async fn clean_expired_states(&self) -> Result<i64, SwingBuddyError> {
        let result = sqlx::query(
            "DELETE FROM user_states WHERE expires_at IS NOT NULL AND expires_at < NOW()"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    // CAS Check methods
    /// Create CAS check record
    pub async fn create_cas_check(&self, request: CreateCasCheckRequest) -> Result<CasCheck, SwingBuddyError> {
        let check = sqlx::query_as::<_, CasCheck>(
            r#"
            INSERT INTO cas_checks (user_id, telegram_id, is_banned, ban_reason, checked_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, telegram_id, is_banned, ban_reason, checked_at
            "#
        )
        .bind(request.user_id)
        .bind(request.telegram_id)
        .bind(request.is_banned)
        .bind(request.ban_reason)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(check)
    }

    /// Get latest CAS check for user
    pub async fn get_latest_cas_check(&self, user_id: i64) -> Result<Option<CasCheck>, SwingBuddyError> {
        let check = sqlx::query_as::<_, CasCheck>(
            "SELECT id, user_id, telegram_id, is_banned, ban_reason, checked_at FROM cas_checks WHERE user_id = $1 ORDER BY checked_at DESC LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(check)
    }

    /// Get CAS checks for user
    pub async fn get_user_cas_checks(&self, user_id: i64) -> Result<Vec<CasCheck>, SwingBuddyError> {
        let checks = sqlx::query_as::<_, CasCheck>(
            "SELECT id, user_id, telegram_id, is_banned, ban_reason, checked_at FROM cas_checks WHERE user_id = $1 ORDER BY checked_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(checks)
    }

    /// Get all banned users from CAS checks
    pub async fn get_banned_users_from_cas(&self) -> Result<Vec<CasCheck>, SwingBuddyError> {
        let checks = sqlx::query_as::<_, CasCheck>(
            r#"
            SELECT DISTINCT ON (user_id) id, user_id, telegram_id, is_banned, ban_reason, checked_at
            FROM cas_checks
            WHERE is_banned = true
            ORDER BY user_id, checked_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(checks)
    }

    /// Clean old CAS check records (keep only latest for each user)
    pub async fn clean_old_cas_checks(&self, keep_days: i32) -> Result<i64, SwingBuddyError> {
        let result = sqlx::query(
            r#"
            DELETE FROM cas_checks
            WHERE checked_at < NOW() - INTERVAL '%d days'
            AND id NOT IN (
                SELECT DISTINCT ON (user_id) id
                FROM cas_checks
                ORDER BY user_id, checked_at DESC
            )
            "#
        )
        .bind(keep_days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Get statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value, SwingBuddyError> {
        let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        let group_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM groups")
            .fetch_one(&self.pool)
            .await?;

        let event_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;

        let active_states: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_states WHERE expires_at IS NULL OR expires_at > NOW()")
            .fetch_one(&self.pool)
            .await?;

        let banned_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_banned = true")
            .fetch_one(&self.pool)
            .await?;

        let stats = serde_json::json!({
            "users": {
                "total": user_count.0,
                "banned": banned_users.0
            },
            "groups": {
                "total": group_count.0
            },
            "events": {
                "total": event_count.0
            },
            "states": {
                "active": active_states.0
            }
        });

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_admin_repository_creation() {
        // This would require a test database setup
        // For now, just test that the repository can be created
        let pool = PgPool::connect("postgresql://test").await;
        if let Ok(pool) = pool {
            let repo = AdminRepository::new(pool);
            assert!(!repo.pool.is_closed());
        }
    }
}