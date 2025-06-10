//! Database module
//!
//! This module handles database connections and operations

pub mod connection;
pub mod repositories;
pub mod service;

// Re-export commonly used database components
pub use connection::{DatabasePool, DatabaseConfig, create_pool, run_migrations, health_check};
pub use repositories::{UserRepository, GroupRepository, EventRepository, AdminRepository};
pub use service::DatabaseService;