//! Database connection management

use sqlx::{Pool, Postgres};
use std::time::Duration;
use crate::utils::errors::SwingBuddyError;

pub type DatabasePool = Pool<Postgres>;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/swingbuddy".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
        }
    }
}

/// Create a new database connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<DatabasePool, SwingBuddyError> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(&config.url)
        .await?;

    // Test the connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await?;

    tracing::info!("Database connection pool created successfully");
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<(), SwingBuddyError> {
    tracing::info!("Running database migrations...");
    
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    
    tracing::info!("Database migrations completed successfully");
    Ok(())
}

/// Check database health
pub async fn health_check(pool: &DatabasePool) -> Result<(), SwingBuddyError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert!(config.url.contains("postgresql://"));
    }
}