//! Test context for unified test setup
//! 
//! This module provides a unified test context that initializes all necessary
//! components for testing including mock servers, databases, and services.

use SwingBuddy::config::Settings;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tempfile::TempDir;

use super::{database_helper::TestDatabase, telegram_mock::TelegramMockServer};

/// Unified test context that manages all test components
pub struct TestContext {
    pub database: TestDatabase,
    pub telegram_mock: TelegramMockServer,
    pub redis_connection: Option<ConnectionManager>,
    pub settings: Settings,
    pub temp_dir: TempDir,
    pub bot_token: String,
}

impl TestContext {
    /// Create a new test context with all components initialized
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::new_with_config(TestConfig::default()).await
    }

    /// Create a new test context with custom configuration
    pub async fn new_with_config(config: TestConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize logging once
        let _ = tracing_subscriber::fmt::try_init();

        // Create temporary directory for test files
        let temp_dir = tempfile::tempdir()?;

        // Initialize test database
        let database = if config.use_database {
            TestDatabase::new().await?
        } else {
            TestDatabase::new_with_migrations(false).await?
        };

        // Initialize mock Telegram server
        let telegram_mock = TelegramMockServer::new().await;
        if config.setup_default_mocks {
            telegram_mock.setup_default_mocks().await;
        }

        // Initialize Redis connection (optional)
        let redis_connection = if config.use_redis {
            Some(Self::setup_redis_connection().await?)
        } else {
            None
        };

        // Create test bot token
        let bot_token = config.bot_token.unwrap_or_else(|| "12345:test_token".to_string());

        // Create test settings
        let settings = Self::create_test_settings(&database, &telegram_mock, &bot_token, &temp_dir)?;

        Ok(Self {
            database,
            telegram_mock,
            redis_connection,
            settings,
            temp_dir,
            bot_token,
        })
    }

    /// Setup Redis connection for testing
    async fn setup_redis_connection() -> Result<ConnectionManager, redis::RedisError> {
        let redis_url = std::env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let client = redis::Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;
        
        Ok(connection)
    }

    /// Create test-specific settings
    fn create_test_settings(
        database: &TestDatabase,
        telegram_mock: &TelegramMockServer,
        bot_token: &str,
        temp_dir: &TempDir,
    ) -> Result<Settings, Box<dyn std::error::Error + Send + Sync>> {
        let mut settings = Settings::default();
        
        // Configure bot settings
        settings.bot.token = bot_token.to_string();
        settings.bot.webhook_url = None; // Use polling for tests
        settings.bot.admin_ids = vec![555666777]; // Test admin ID

        // Configure database settings
        settings.database.url = database.database_url.clone();
        settings.database.max_connections = 5;
        settings.database.min_connections = 1;

        // Configure Redis settings
        settings.redis.url = std::env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        settings.redis.prefix = "test_swingbuddy:".to_string();
        settings.redis.ttl_seconds = 300; // Shorter TTL for tests

        // Configure CAS settings (use mock)
        settings.cas.api_url = format!("{}/cas", telegram_mock.server.uri());
        settings.cas.timeout_seconds = 1;
        settings.cas.auto_ban = false; // Disable for tests

        // Configure logging
        settings.logging.level = "debug".to_string();
        settings.logging.file_path = temp_dir.path().join("test.log").to_string_lossy().to_string();

        // Configure features
        settings.features.cas_protection = false; // Disable for tests
        settings.features.google_calendar = false; // Disable for tests
        settings.features.admin_panel = true;

        Ok(settings)
    }

    /// Load test fixtures into the database
    pub async fn load_fixtures(&self) -> Result<(), sqlx::Error> {
        self.database.load_fixtures().await
    }

    /// Clean up all test data
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Clean database
        self.database.cleanup().await?;
        
        // Reset mock server
        self.telegram_mock.reset().await;
        
        // Clear Redis if available
        if let Some(ref conn) = self.redis_connection {
            let mut conn = conn.clone();
            redis::cmd("FLUSHDB").query_async::<_, ()>(&mut conn).await?;
        }

        Ok(())
    }

    /// Get database pool for direct access
    pub fn db_pool(&self) -> &sqlx::PgPool {
        &self.database.pool
    }

    /// Get mock Telegram server URL
    pub fn telegram_api_url(&self) -> String {
        self.telegram_mock.get_api_url(&self.bot_token)
    }

    /// Create a test bot instance with this context
    pub async fn create_bot(&self) -> Result<teloxide::Bot, Box<dyn std::error::Error + Send + Sync>> {
        let bot = teloxide::Bot::new(&self.bot_token)
            .set_api_url(self.telegram_api_url().parse()?);
        
        Ok(bot)
    }

    /// Create application state for testing
    pub async fn create_app_state(&self) -> Result<Arc<SwingBuddy::state::context::AppContext>, Box<dyn std::error::Error + Send + Sync>> {
        let database_service = Arc::new(SwingBuddy::database::service::DatabaseService::new(self.db_pool().clone()));
        
        let redis_service = if let Some(_) = self.redis_connection {
            Some(Arc::new(SwingBuddy::services::redis::RedisService::new(self.settings.clone())?))
        } else {
            None
        };

        // Create user repository
        let user_repository = SwingBuddy::database::repositories::UserRepository::new(self.db_pool().clone());

        let user_service = Arc::new(SwingBuddy::services::user::UserService::new(
            user_repository,
            self.settings.clone(),
        ));

        // Create bot for services that need it
        let bot = self.create_bot().await?;

        let auth_service = Arc::new(SwingBuddy::services::auth::AuthService::new(
            bot.clone(),
            self.settings.clone(),
        ));

        let notification_service = Arc::new(SwingBuddy::services::notification::NotificationService::new(
            bot.clone(),
            self.settings.clone(),
        ));

        // Create Redis client for CAS service
        let redis_client = redis::Client::open(self.settings.redis.url.clone())?;
        let cas_service = Arc::new(SwingBuddy::services::cas::CasService::new(
            redis_client,
            self.settings.clone(),
        )?);

        let google_service = if self.settings.google.is_some() {
            Some(Arc::new(SwingBuddy::services::google::GoogleService::new(
                self.settings.google.as_ref().unwrap().clone(),
            )))
        } else {
            None
        };

        let app_context = Arc::new(SwingBuddy::state::context::AppContext::new(
            self.settings.clone(),
            database_service,
            redis_service,
            user_service,
            auth_service,
            notification_service,
            cas_service,
            google_service,
        ));

        Ok(app_context)
    }

    /// Setup mock responses for specific test scenarios
    pub async fn setup_telegram_mocks(&self, scenario: MockScenario) {
        match scenario {
            MockScenario::Success => {
                self.telegram_mock.setup_default_mocks().await;
            }
            MockScenario::Error => {
                self.telegram_mock.setup_error_mocks().await;
            }
            MockScenario::Timeout => {
                self.telegram_mock.setup_timeout_mocks(5000).await;
            }
        }
    }

    /// Verify Telegram API calls
    pub async fn verify_telegram_calls(&self, endpoint: &str, expected_calls: usize) {
        self.telegram_mock.verify_endpoint_called(endpoint, expected_calls).await;
    }
}

/// Configuration for test context setup
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub use_database: bool,
    pub use_redis: bool,
    pub setup_default_mocks: bool,
    pub bot_token: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            use_database: true,
            use_redis: false,
            setup_default_mocks: true,
            bot_token: None,
        }
    }
}

/// Mock scenarios for testing different conditions
#[derive(Debug, Clone)]
pub enum MockScenario {
    Success,
    Error,
    Timeout,
}

/// Helper macro for running tests with full test context
#[macro_export]
macro_rules! test_with_context {
    ($test_name:ident, async $test_body:expr) => {
        #[tokio::test]
        #[serial_test::serial]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let ctx = TestContext::new().await?;
            
            // Load fixtures
            ctx.load_fixtures().await?;
            
            // Run the test body
            let test_fn = $test_body;
            let fut = async move { test_fn(&ctx).await? };
            fut.await
            
            // Cleanup
            ctx.cleanup().await?;
            
            result
        }
    };
}

/// Helper macro for running tests with custom test context configuration
#[macro_export]
macro_rules! test_with_custom_context {
    ($test_name:ident, $config:expr, async $test_body:expr) => {
        #[tokio::test]
        #[serial_test::serial]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let config = $config;
            let ctx = TestContext::new_with_config(config.clone()).await?;
            
            // Load fixtures if database is enabled
            if config.use_database {
                ctx.load_fixtures().await?;
            }
            
            // Run the test body
            let test_fn = $test_body;
            let fut = async move { test_fn(&ctx).await? };
            fut.await
            
            // Cleanup
            ctx.cleanup().await?;
            
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_context_creation() {
        let ctx = TestContext::new().await.expect("Failed to create test context");
        
        assert!(!ctx.settings.bot.token.is_empty());
        assert!(!ctx.settings.database.url.is_empty());
        assert!(ctx.temp_dir.path().exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_context_with_custom_config() {
        let config = TestConfig {
            use_database: true,
            use_redis: false,
            setup_default_mocks: false,
            bot_token: Some("custom_token".to_string()),
        };
        
        let ctx = TestContext::new_with_config(config).await.expect("Failed to create test context");
        
        assert_eq!(ctx.bot_token, "custom_token");
        assert!(ctx.redis_connection.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_context_cleanup() {
        let ctx = TestContext::new().await.expect("Failed to create test context");
        
        // Load some test data
        ctx.load_fixtures().await.expect("Failed to load fixtures");
        
        // Verify data exists
        let user_count = ctx.database.count_records("users").await.expect("Failed to count users");
        assert!(user_count > 0);
        
        // Cleanup
        ctx.cleanup().await.expect("Failed to cleanup");
        
        // Verify data is gone
        let user_count_after = ctx.database.count_records("users").await.expect("Failed to count users");
        assert_eq!(user_count_after, 0);
    }
}