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
pub use database_helper::*;
pub use test_context::*;
pub use simple_test::*;
pub use test_data::*;

pub type DbUser = crate::models::user::User;

// Re-export commonly used types for convenience
pub mod fixtures;
pub mod integration;