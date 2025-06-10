//! Test infrastructure verification - only working components
//! 
//! This test verifies that the basic test infrastructure components work
//! without any complex dependencies.

use serde_json::{json, Value};
use std::collections::HashMap;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use serial_test::serial;
use tempfile::TempDir;

/// Mock Telegram API server for testing
pub struct TelegramMockServer {
    pub server: MockServer,
    pub base_url: String,
}

/// Configuration for mock responses
#[derive(Debug, Clone)]
pub struct MockResponseConfig {
    pub success: bool,
    pub delay_ms: Option<u64>,
    pub custom_response: Option<Value>,
}

impl Default for MockResponseConfig {
    fn default() -> Self {
        Self {
            success: true,
            delay_ms: None,
            custom_response: None,
        }
    }
}

impl TelegramMockServer {
    /// Create a new mock Telegram API server
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let base_url = format!("{}/bot{{token}}", server.uri());
        
        Self { server, base_url }
    }

    /// Get the mock server URL for a given bot token
    pub fn get_api_url(&self, token: &str) -> String {
        self.base_url.replace("{token}", token)
    }

    /// Setup mock for sendMessage endpoint
    pub async fn mock_send_message(&self, config: MockResponseConfig) {
        let response_body = config.custom_response.unwrap_or_else(|| {
            if config.success {
                json!({
                    "ok": true,
                    "result": {
                        "message_id": 123,
                        "from": {
                            "id": 12345,
                            "is_bot": true,
                            "first_name": "TestBot",
                            "username": "test_bot"
                        },
                        "chat": {
                            "id": -1001234567890_i64,
                            "title": "Test Group",
                            "type": "supergroup"
                        },
                        "date": 1640995200,
                        "text": "Test message"
                    }
                })
            } else {
                json!({
                    "ok": false,
                    "error_code": 400,
                    "description": "Bad Request: message text is empty"
                })
            }
        });

        let mut response = ResponseTemplate::new(if config.success { 200 } else { 400 })
            .set_body_json(response_body);

        if let Some(delay) = config.delay_ms {
            response = response.set_delay(std::time::Duration::from_millis(delay));
        }

        Mock::given(method("POST"))
            .and(path("/bot12345:test_token/sendMessage"))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    /// Setup mock for getMe endpoint
    pub async fn mock_get_me(&self, config: MockResponseConfig) {
        let response_body = config.custom_response.unwrap_or_else(|| {
            if config.success {
                json!({
                    "ok": true,
                    "result": {
                        "id": 12345,
                        "is_bot": true,
                        "first_name": "TestBot",
                        "username": "test_bot",
                        "can_join_groups": true,
                        "can_read_all_group_messages": false,
                        "supports_inline_queries": false
                    }
                })
            } else {
                json!({
                    "ok": false,
                    "error_code": 401,
                    "description": "Unauthorized"
                })
            }
        });

        let mut response = ResponseTemplate::new(if config.success { 200 } else { 401 })
            .set_body_json(response_body);

        if let Some(delay) = config.delay_ms {
            response = response.set_delay(std::time::Duration::from_millis(delay));
        }

        Mock::given(method("POST"))
            .and(path("/bot12345:test_token/getMe"))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    /// Setup all common mocks with default success responses
    pub async fn setup_default_mocks(&self) {
        let config = MockResponseConfig::default();
        
        self.mock_send_message(config.clone()).await;
        self.mock_get_me(config).await;
    }

    /// Setup mocks for error scenarios
    pub async fn setup_error_mocks(&self) {
        let config = MockResponseConfig {
            success: false,
            delay_ms: None,
            custom_response: None,
        };
        
        self.mock_send_message(config.clone()).await;
        self.mock_get_me(config).await;
    }

    /// Setup mocks with timeout simulation
    pub async fn setup_timeout_mocks(&self, delay_ms: u64) {
        let config = MockResponseConfig {
            success: true,
            delay_ms: Some(delay_ms),
            custom_response: None,
        };
        
        self.mock_send_message(config.clone()).await;
        self.mock_get_me(config).await;
    }

    /// Reset all mocks
    pub async fn reset(&self) {
        self.server.reset().await;
    }
}

/// Simple test configuration
#[derive(Debug, Clone)]
pub struct SimpleTestConfig {
    pub mock_telegram: bool,
    pub use_temp_files: bool,
}

impl Default for SimpleTestConfig {
    fn default() -> Self {
        Self {
            mock_telegram: true,
            use_temp_files: true,
        }
    }
}

/// Simple test context for basic testing
pub struct SimpleTestContext {
    pub config: SimpleTestConfig,
    pub temp_dir: Option<TempDir>,
}

impl SimpleTestContext {
    /// Create a new simple test context
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = SimpleTestConfig::default();
        let temp_dir = if config.use_temp_files {
            Some(tempfile::tempdir()?)
        } else {
            None
        };

        Ok(Self {
            config,
            temp_dir,
        })
    }

    /// Create with custom config
    pub fn new_with_config(config: SimpleTestConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = if config.use_temp_files {
            Some(tempfile::tempdir()?)
        } else {
            None
        };

        Ok(Self {
            config,
            temp_dir,
        })
    }

    /// Get temp directory path
    pub fn temp_path(&self) -> Option<&std::path::Path> {
        self.temp_dir.as_ref().map(|d| d.path())
    }
}

/// Helper function to create a test bot token
pub fn test_bot_token() -> String {
    "12345:test_token".to_string()
}

/// Helper function to create test chat ID
pub fn test_chat_id() -> i64 {
    -1001234567890
}

/// Helper function to create test user ID
pub fn test_user_id() -> i64 {
    987654321
}

// Tests
#[tokio::test]
#[serial]
async fn test_telegram_mock_server_basic() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test that the server was created
    assert!(!mock_server.base_url.is_empty());
    
    // Test API URL generation
    let token = test_bot_token();
    let api_url = mock_server.get_api_url(&token);
    assert!(api_url.contains(&token));
    assert!(!api_url.contains("{token}"));
}

#[tokio::test]
#[serial]
async fn test_mock_response_configuration() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test default configuration
    let default_config = MockResponseConfig::default();
    assert!(default_config.success);
    assert!(default_config.delay_ms.is_none());
    assert!(default_config.custom_response.is_none());
    
    // Test custom configuration
    let custom_config = MockResponseConfig {
        success: false,
        delay_ms: Some(500),
        custom_response: None,
    };
    assert!(!custom_config.success);
    assert_eq!(custom_config.delay_ms, Some(500));
    
    // Setup mocks with different configurations
    mock_server.mock_send_message(default_config).await;
    mock_server.mock_get_me(custom_config).await;
}

#[tokio::test]
#[serial]
async fn test_mock_server_reset() {
    let mock_server = TelegramMockServer::new().await;
    
    // Setup some mocks
    mock_server.setup_default_mocks().await;
    
    // Reset the server
    mock_server.reset().await;
    
    // Server should still be functional after reset
    let api_url = mock_server.get_api_url(&test_bot_token());
    assert!(!api_url.is_empty());
}

#[test]
fn test_helper_functions() {
    // Test that helper functions return expected values
    assert_eq!(test_bot_token(), "12345:test_token");
    assert_eq!(test_chat_id(), -1001234567890);
    assert_eq!(test_user_id(), 987654321);
}

#[test]
fn test_simple_context() {
    let ctx = SimpleTestContext::new().expect("Failed to create simple context");
    assert!(ctx.config.mock_telegram);
    assert!(ctx.config.use_temp_files);
    assert!(ctx.temp_dir.is_some());
    
    let temp_path = ctx.temp_path().expect("Should have temp path");
    assert!(temp_path.exists());
}

#[test]
fn test_simple_context_custom_config() {
    let config = SimpleTestConfig {
        mock_telegram: false,
        use_temp_files: false,
    };
    
    let ctx = SimpleTestContext::new_with_config(config).expect("Failed to create context");
    assert!(!ctx.config.mock_telegram);
    assert!(!ctx.config.use_temp_files);
    assert!(ctx.temp_dir.is_none());
    assert!(ctx.temp_path().is_none());
}

#[tokio::test]
#[serial]
async fn test_mock_endpoints() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test individual endpoint mocking
    let success_config = MockResponseConfig::default();
    mock_server.mock_send_message(success_config.clone()).await;
    mock_server.mock_get_me(success_config).await;
    
    // Test error scenarios
    let error_config = MockResponseConfig {
        success: false,
        delay_ms: None,
        custom_response: None,
    };
    
    mock_server.mock_send_message(error_config).await;
}

#[tokio::test]
#[serial]
async fn test_mock_scenarios() {
    let mock_server = TelegramMockServer::new().await;
    
    // Test different mock scenarios
    mock_server.setup_default_mocks().await;
    mock_server.setup_error_mocks().await;
    mock_server.setup_timeout_mocks(1000).await;
    
    // All should complete without errors
}