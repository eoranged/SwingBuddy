//! Group repository implementation

use sqlx::PgPool;
use chrono::Utc;
use crate::models::group::{Group, GroupMember, CreateGroupRequest, UpdateGroupRequest, AddMemberRequest};
use crate::utils::errors::SwingBuddyError;

#[derive(Clone)]
pub struct GroupRepository {
    pool: PgPool,
}

impl GroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new group
    pub async fn create(&self, request: CreateGroupRequest) -> Result<Group, SwingBuddyError> {
        let group = sqlx::query_as::<_, Group>(
            r#"
            INSERT INTO groups (telegram_id, title, description, language_code, settings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, telegram_id, title, description, language_code, settings, is_active, created_at, updated_at
            "#
        )
        .bind(request.telegram_id)
        .bind(request.title)
        .bind(request.description)
        .bind(request.language_code.unwrap_or_else(|| "en".to_string()))
        .bind(request.settings.unwrap_or_else(|| serde_json::json!({})))
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    /// Find group by ID
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Group>, SwingBuddyError> {
        let group = sqlx::query_as::<_, Group>(
            "SELECT id, telegram_id, title, description, language_code, settings, is_active, created_at, updated_at FROM groups WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(group)
    }

    /// Find group by Telegram ID
    pub async fn find_by_telegram_id(&self, telegram_id: i64) -> Result<Option<Group>, SwingBuddyError> {
        let group = sqlx::query_as::<_, Group>(
            "SELECT id, telegram_id, title, description, language_code, settings, is_active, created_at, updated_at FROM groups WHERE telegram_id = $1"
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(group)
    }

    /// Update group
    pub async fn update(&self, id: i64, request: UpdateGroupRequest) -> Result<Group, SwingBuddyError> {
        let group = sqlx::query_as::<_, Group>(
            r#"
            UPDATE groups
            SET title = COALESCE($2, title),
                description = COALESCE($3, description),
                language_code = COALESCE($4, language_code),
                settings = COALESCE($5, settings),
                is_active = COALESCE($6, is_active),
                updated_at = $7
            WHERE id = $1
            RETURNING id, telegram_id, title, description, language_code, settings, is_active, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(request.title)
        .bind(request.description)
        .bind(request.language_code)
        .bind(request.settings)
        .bind(request.is_active)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    /// Delete group
    pub async fn delete(&self, id: i64) -> Result<(), SwingBuddyError> {
        sqlx::query("DELETE FROM groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List all groups with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Group>, SwingBuddyError> {
        let groups = sqlx::query_as::<_, Group>(
            "SELECT id, telegram_id, title, description, language_code, settings, is_active, created_at, updated_at FROM groups ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    /// Count total groups
    pub async fn count(&self) -> Result<i64, SwingBuddyError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM groups")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    /// Add member to group
    pub async fn add_member(&self, request: AddMemberRequest) -> Result<GroupMember, SwingBuddyError> {
        let member = sqlx::query_as::<_, GroupMember>(
            r#"
            INSERT INTO group_members (group_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, group_id, user_id, role, joined_at
            "#
        )
        .bind(request.group_id)
        .bind(request.user_id)
        .bind(request.role.unwrap_or_else(|| "member".to_string()))
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(member)
    }

    /// Remove member from group
    pub async fn remove_member(&self, group_id: i64, user_id: i64) -> Result<(), SwingBuddyError> {
        sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get group members
    pub async fn get_members(&self, group_id: i64) -> Result<Vec<GroupMember>, SwingBuddyError> {
        let members = sqlx::query_as::<_, GroupMember>(
            "SELECT id, group_id, user_id, role, joined_at FROM group_members WHERE group_id = $1 ORDER BY joined_at ASC"
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// Check if user is member of group
    pub async fn is_member(&self, group_id: i64, user_id: i64) -> Result<bool, SwingBuddyError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM group_members WHERE group_id = $1 AND user_id = $2"
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }

    /// Update member role
    pub async fn update_member_role(&self, group_id: i64, user_id: i64, role: &str) -> Result<GroupMember, SwingBuddyError> {
        let member = sqlx::query_as::<_, GroupMember>(
            r#"
            UPDATE group_members
            SET role = $3
            WHERE group_id = $1 AND user_id = $2
            RETURNING id, group_id, user_id, role, joined_at
            "#
        )
        .bind(group_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(member)
    }

    /// Get groups for user
    pub async fn get_user_groups(&self, user_id: i64) -> Result<Vec<Group>, SwingBuddyError> {
        let groups = sqlx::query_as::<_, Group>(
            r#"
            SELECT g.id, g.telegram_id, g.title, g.description, g.language_code, g.settings, g.is_active, g.created_at, g.updated_at
            FROM groups g
            INNER JOIN group_members gm ON g.id = gm.group_id
            WHERE gm.user_id = $1 AND g.is_active = true
            ORDER BY gm.joined_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    /// Get active groups
    pub async fn get_active_groups(&self) -> Result<Vec<Group>, SwingBuddyError> {
        let groups = sqlx::query_as::<_, Group>(
            "SELECT id, telegram_id, title, description, language_code, settings, is_active, created_at, updated_at FROM groups WHERE is_active = true ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_group_repository_creation() {
        // This would require a test database setup
        // For now, just test that the repository can be created
        let pool = PgPool::connect("postgresql://test").await;
        if let Ok(pool) = pool {
            let repo = GroupRepository::new(pool);
            assert!(!repo.pool.is_closed());
        }
    }
}