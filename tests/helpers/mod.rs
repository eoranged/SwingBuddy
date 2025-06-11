//! Test helpers module
//!
//! This module provides utilities and helpers for testing the SwingBuddy application.
//! It includes mock servers, database helpers, and test context setup.

pub mod telegram_mock;
pub mod database_helper;
pub mod test_context;
pub mod simple_test;
pub mod test_data;

pub use telegram_mock::*;
pub use test_context::*;
pub use database_helper::TestDatabase;
pub use simple_test::{SimpleTestContext, SimpleTestConfig};
pub use test_data::{create_simple_test_message, create_test_message, create_test_private_chat, create_test_group_chat, create_simple_test_callback_query};

pub type DbUser = SwingBuddy::models::user::User;

// Re-export commonly used types for convenience
pub mod fixtures;
pub mod integration;