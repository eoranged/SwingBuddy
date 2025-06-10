//! User repository implementation

use sqlx::PgPool;
use chrono::Utc;
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::utils::errors::SwingBuddyError;

#[derive(Clone)]
#[derive(Debug)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create(&self, request: CreateUserRequest) -> Result<User, SwingBuddyError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (telegram_id, username, first_name, last_name, language_code, location, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at
            "#
        )
        .bind(request.telegram_id)
        .bind(request.username)
        .bind(request.first_name)
        .bind(request.last_name)
        .bind(request.language_code.unwrap_or_else(|| "en".to_string()))
        .bind(request.location)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>, SwingBuddyError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by Telegram ID
    pub async fn find_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>, SwingBuddyError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at FROM users WHERE telegram_id = $1"
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Update user
    pub async fn update(&self, id: i64, request: UpdateUserRequest) -> Result<User, SwingBuddyError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET username = COALESCE($2, username),
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                language_code = COALESCE($5, language_code),
                location = COALESCE($6, location),
                is_banned = COALESCE($7, is_banned),
                updated_at = $8
            WHERE id = $1
            RETURNING id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(request.username)
        .bind(request.first_name)
        .bind(request.last_name)
        .bind(request.language_code)
        .bind(request.location)
        .bind(request.is_banned)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Delete user
    pub async fn delete(&self, id: i64) -> Result<(), SwingBuddyError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List all users with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, SwingBuddyError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    /// Count total users
    pub async fn count(&self) -> Result<i64, SwingBuddyError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    /// Find users by username pattern
    pub async fn find_by_username_pattern(&self, pattern: &str) -> Result<Vec<User>, SwingBuddyError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at FROM users WHERE username ILIKE $1"
        )
        .bind(format!("%{}%", pattern))
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    /// Ban/unban user
    pub async fn set_ban_status(&self, id: i64, is_banned: bool) -> Result<User, SwingBuddyError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET is_banned = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(is_banned)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Get banned users
    pub async fn get_banned_users(&self) -> Result<Vec<User>, SwingBuddyError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at FROM users WHERE is_banned = true ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_repository_creation() {
        // This would require a test database setup
        // For now, just test that the repository can be created
        let pool = PgPool::connect("postgresql://test").await;
        if let Ok(pool) = pool {
            let repo = UserRepository::new(pool);
            assert!(!repo.pool.is_closed());
        }
    }
}