//! Integration tests for callback handlers
//!
//! This module contains integration tests for all bot callback handlers.

pub mod language_test;
pub mod location_test;

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    
    /// Test that all callback handler test modules are properly accessible
    #[test]
    fn test_callback_handler_modules_accessible() {
        // This test ensures that all callback handler test modules compile and are accessible
        // It's a basic smoke test for the module structure
    }
}