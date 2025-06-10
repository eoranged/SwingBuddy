//! Test database helper utilities
//! 
//! This module provides utilities for setting up and managing test databases,
//! including transaction-based test isolation and fixture loading.

use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Once;
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, Image, ImageExt};
use testcontainers_modules::postgres::Postgres as PostgresImage;
use uuid::Uuid;
use SwingBuddy::models::user::User;
use SwingBuddy::models::group::Group;

static INIT: Once = Once::new();

/// Test database helper that manages PostgreSQL test database setup
pub struct TestDatabase {
    pub pool: PgPool,
    pub database_url: String,
    _container: Option<ContainerAsync<PostgresImage>>,
}

impl TestDatabase {
    /// Create a new test database instance
    pub async fn new() -> Result<Self, sqlx::Error> {
        Self::new_with_migrations(true).await
    }

    /// Create a new test database instance with optional migrations
    pub async fn new_with_migrations(run_migrations: bool) -> Result<Self, sqlx::Error> {
        // Initialize logging once
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt::try_init();
        });

        // For CI/CD environments, use environment variable if available
        let database_url = if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
            url
        } else {
            // Use testcontainers for local development
            let postgres_image = PostgresImage::default()
                .with_db_name("test_swingbuddy")
                .with_user("test_user")
                .with_password("test_password");
            
            let container = postgres_image.start().await.expect("Failed to start postgres container");
            let port = container.get_host_port_ipv4(5432).await.expect("Failed to get port");
            
            format!(
                "postgresql://test_user:test_password@localhost:{}/test_swingbuddy",
                port
            )
        };

        let pool = PgPool::connect(&database_url).await?;

        if run_migrations {
            sqlx::migrate!("./migrations").run(&pool).await?;
        }

        Ok(Self {
            pool,
            database_url,
            _container: None,
        })
    }

    /// Create a new test database with a unique name
    pub async fn new_unique() -> Result<Self, sqlx::Error> {
        let unique_id = Uuid::new_v4().to_string().replace('-', "");
        let database_name = format!("test_swingbuddy_{}", &unique_id[..8]);
        
        // Create database with unique name
        let base_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test_user:test_password@localhost:5432".to_string());
        
        let admin_pool = PgPool::connect(&base_url).await?;
        
        sqlx::query(&format!("CREATE DATABASE {}", database_name))
            .execute(&admin_pool)
            .await?;
        
        admin_pool.close().await;

        let database_url = format!("{}/{}", base_url, database_name);
        let pool = PgPool::connect(&database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self {
            pool,
            database_url,
            _container: None,
        })
    }

    /// Begin a new transaction for test isolation
    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
        self.pool.begin().await
    }

    /// Clean all test data from the database
    pub async fn cleanup(&self) -> Result<(), sqlx::Error> {
        // Delete in reverse order of dependencies
        sqlx::query("DELETE FROM event_participants").execute(&self.pool).await?;
        sqlx::query("DELETE FROM events").execute(&self.pool).await?;
        sqlx::query("DELETE FROM group_members").execute(&self.pool).await?;
        sqlx::query("DELETE FROM groups").execute(&self.pool).await?;
        sqlx::query("DELETE FROM users").execute(&self.pool).await?;
        
        Ok(())
    }

    /// Load test fixtures into the database
    pub async fn load_fixtures(&self) -> Result<(), sqlx::Error> {
        self.load_user_fixtures().await?;
        self.load_group_fixtures().await?;
        self.load_event_fixtures().await?;
        Ok(())
    }

    /// Load user test fixtures
    pub async fn load_user_fixtures(&self) -> Result<(), sqlx::Error> {
        // Insert test users
        sqlx::query!(
            r#"
            INSERT INTO users (telegram_id, username, first_name, last_name, language_code, is_banned, created_at, updated_at)
            VALUES
                (123456789, 'testuser1', 'Test', 'User1', 'en', false, NOW(), NOW()),
                (987654321, 'testuser2', 'Test', 'User2', 'ru', false, NOW(), NOW()),
                (555666777, 'testadmin', 'Test', 'Admin', 'en', false, NOW(), NOW())
            ON CONFLICT (telegram_id) DO NOTHING
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load group test fixtures
    pub async fn load_group_fixtures(&self) -> Result<(), sqlx::Error> {
        // Insert test groups
        sqlx::query!(
            r#"
            INSERT INTO groups (telegram_id, title, language_code, is_active, created_at, updated_at)
            VALUES
                (-1001234567890, 'Test Swing Group', 'en', true, NOW(), NOW()),
                (-1009876543210, 'Test Dance Community', 'ru', true, NOW(), NOW())
            ON CONFLICT (telegram_id) DO NOTHING
            "#
        )
        .execute(&self.pool)
        .await?;

        // Insert group admins
        sqlx::query!(
            r#"
            INSERT INTO group_members (group_id, user_id, is_admin)
            SELECT g.id, u.id, true
            FROM groups g, users u
            WHERE g.telegram_id = -1001234567890 AND u.telegram_id = 555666777
            ON CONFLICT (group_id, user_id) DO NOTHING
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load event test fixtures
    pub async fn load_event_fixtures(&self) -> Result<(), sqlx::Error> {
        // Insert test events
        sqlx::query!(
            r#"
            INSERT INTO events (
                group_id, title, description, event_date, location,
                max_participants, is_active, created_by, created_at, updated_at
            )
            SELECT
                g.id, 'Test Swing Dance', 'A test swing dance event',
                NOW() + INTERVAL '7 days', 'Test Venue',
                20, true, u.id, NOW(), NOW()
            FROM groups g, users u
            WHERE g.telegram_id = -1001234567890 AND u.telegram_id = 555666777
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a test user by telegram_id
    pub async fn get_test_user(&self, telegram_id: i64) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                telegram_id,
                username as "username?",
                first_name as "first_name?",
                last_name as "last_name?",
                language_code,
                location as "location?",
                is_banned,
                created_at,
                updated_at
            FROM users WHERE telegram_id = $1
            "#,
            telegram_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Get a test group by telegram_id
    pub async fn get_test_group(&self, telegram_id: i64) -> Result<Option<Group>, sqlx::Error> {
        let group = sqlx::query_as!(
            Group,
            r#"
            SELECT
                id,
                telegram_id,
                title,
                description as "description?",
                language_code,
                settings,
                is_active,
                created_at,
                updated_at
            FROM groups WHERE telegram_id = $1
            "#,
            telegram_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(group)
    }

    /// Create a test user with custom data
    pub async fn create_test_user(
        &self,
        telegram_id: i64,
        username: Option<String>,
        first_name: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (telegram_id, username, first_name, language_code, created_at, updated_at)
            VALUES ($1, $2, $3, 'en', NOW(), NOW())
            RETURNING
                id,
                telegram_id,
                username as "username?",
                first_name as "first_name?",
                last_name as "last_name?",
                language_code,
                location as "location?",
                is_banned,
                created_at,
                updated_at
            "#,
            telegram_id,
            username,
            first_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a test group with custom data
    pub async fn create_test_group(
        &self,
        telegram_id: i64,
        title: String,
    ) -> Result<Group, sqlx::Error> {
        let group = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (telegram_id, title, language_code, created_at, updated_at)
            VALUES ($1, $2, 'en', NOW(), NOW())
            RETURNING
                id,
                telegram_id,
                title,
                description as "description?",
                language_code,
                settings,
                is_active,
                created_at,
                updated_at
            "#,
            telegram_id,
            title
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    /// Execute raw SQL for custom test scenarios
    pub async fn execute_sql(&self, sql: &str) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query(sql).execute(&self.pool).await
    }

    /// Count records in a table
    pub async fn count_records(&self, table: &str) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
            .fetch_one(&self.pool)
            .await?;
        
        Ok(count)
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Cleanup is handled automatically by testcontainers
        // For unique databases, we might want to drop them explicitly
    }
}

/// Helper macro for running tests with database transaction rollback
#[macro_export]
macro_rules! test_with_db {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        #[serial_test::serial]
        async fn $test_name() {
            let db = TestDatabase::new().await.expect("Failed to create test database");
            let mut tx = db.begin_transaction().await.expect("Failed to begin transaction");
            
            // Run the test body with the transaction
            let result = async move {
                $test_body(&mut tx, &db).await
            }.await;
            
            // Always rollback the transaction
            tx.rollback().await.expect("Failed to rollback transaction");
            
            // Propagate any test failures
            if let Err(e) = result {
                panic!("Test failed: {:?}", e);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_database_creation() {
        let db = TestDatabase::new().await.expect("Failed to create test database");
        assert!(!db.database_url.is_empty());
        assert!(db.pool.is_closed() == false);
    }

    #[tokio::test]
    #[serial]
    async fn test_database_cleanup() {
        let db = TestDatabase::new().await.expect("Failed to create test database");
        
        // Load fixtures first
        db.load_fixtures().await.expect("Failed to load fixtures");
        
        // Verify data exists
        let user_count = db.count_records("users").await.expect("Failed to count users");
        assert!(user_count > 0);
        
        // Cleanup
        db.cleanup().await.expect("Failed to cleanup database");
        
        // Verify data is gone
        let user_count_after = db.count_records("users").await.expect("Failed to count users");
        assert_eq!(user_count_after, 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_load_fixtures() {
        let db = TestDatabase::new().await.expect("Failed to create test database");
        
        db.load_fixtures().await.expect("Failed to load fixtures");
        
        // Verify fixtures were loaded
        let user_count = db.count_records("users").await.expect("Failed to count users");
        let group_count = db.count_records("groups").await.expect("Failed to count groups");
        
        assert!(user_count >= 3);
        assert!(group_count >= 2);
        
        // Cleanup
        db.cleanup().await.expect("Failed to cleanup database");
    }
}