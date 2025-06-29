//! Mock Telegram API Server for testing
//! 
//! This module provides a mock HTTP server that simulates the Telegram Bot API
//! for testing purposes. It uses wiremock to create configurable mock responses.

use serde_json::{json, Value};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

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
        let base_url = server.uri();
        
        Self { server, base_url }
    }

    /// Get the mock server URL for a given bot token
    pub fn get_api_url(&self, _token: &str) -> String {
        // Return just the base URL - teloxide will append /bot{token} automatically
        self.base_url.clone()
    }

    /// Setup mock for sendMessage endpoint
    pub async fn mock_send_message(&self, config: MockResponseConfig) {
        self.mock_send_message_with_token(&test_bot_token(), config).await;
    }

    /// Setup mock for sendMessage endpoint with specific token
    pub async fn mock_send_message_with_token(&self, token: &str, config: MockResponseConfig) {
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

        // Use exact token path matching - support both cases
        Mock::given(method("POST"))
            .and(path(format!("/bot{}/sendMessage", token)))
            .respond_with(response.clone())
            .mount(&self.server)
            .await;
            
        Mock::given(method("POST"))
            .and(path(format!("/bot{}/SendMessage", token)))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    /// Setup mock for editMessageText endpoint
    pub async fn mock_edit_message_text(&self, config: MockResponseConfig) {
        self.mock_edit_message_text_with_token(&test_bot_token(), config).await;
    }

    /// Setup mock for editMessageText endpoint with specific token
    pub async fn mock_edit_message_text_with_token(&self, token: &str, config: MockResponseConfig) {
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
                        "edit_date": 1640995260,
                        "text": "Edited test message"
                    }
                })
            } else {
                json!({
                    "ok": false,
                    "error_code": 400,
                    "description": "Bad Request: message not found"
                })
            }
        });

        let mut response = ResponseTemplate::new(if config.success { 200 } else { 400 })
            .set_body_json(response_body);

        if let Some(delay) = config.delay_ms {
            response = response.set_delay(std::time::Duration::from_millis(delay));
        }

        Mock::given(method("POST"))
            .and(path(format!("/bot{}/editMessageText", token)))
            .respond_with(response.clone())
            .mount(&self.server)
            .await;
            
        Mock::given(method("POST"))
            .and(path(format!("/bot{}/EditMessageText", token)))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    /// Setup mock for answerCallbackQuery endpoint
    pub async fn mock_answer_callback_query(&self, config: MockResponseConfig) {
        self.mock_answer_callback_query_with_token(&test_bot_token(), config).await;
    }

    /// Setup mock for answerCallbackQuery endpoint with specific token
    pub async fn mock_answer_callback_query_with_token(&self, token: &str, config: MockResponseConfig) {
        let response_body = config.custom_response.unwrap_or_else(|| {
            if config.success {
                json!({
                    "ok": true,
                    "result": true
                })
            } else {
                json!({
                    "ok": false,
                    "error_code": 400,
                    "description": "Bad Request: query is too old"
                })
            }
        });

        let mut response = ResponseTemplate::new(if config.success { 200 } else { 400 })
            .set_body_json(response_body);

        if let Some(delay) = config.delay_ms {
            response = response.set_delay(std::time::Duration::from_millis(delay));
        }

        Mock::given(method("POST"))
            .and(path(format!("/bot{}/answerCallbackQuery", token)))
            .respond_with(response.clone())
            .mount(&self.server)
            .await;
            
        Mock::given(method("POST"))
            .and(path(format!("/bot{}/AnswerCallbackQuery", token)))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    /// Setup mock for getMe endpoint
    pub async fn mock_get_me(&self, config: MockResponseConfig) {
        self.mock_get_me_with_token(&test_bot_token(), config).await;
    }

    /// Setup mock for getMe endpoint with specific token
    pub async fn mock_get_me_with_token(&self, token: &str, config: MockResponseConfig) {
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
            .and(path(format!("/bot{}/getMe", token)))
            .respond_with(response.clone())
            .mount(&self.server)
            .await;
            
        Mock::given(method("POST"))
            .and(path(format!("/bot{}/GetMe", token)))
            .respond_with(response.clone())
            .mount(&self.server)
            .await;
            
        // Also support GET method for getMe
        Mock::given(method("GET"))
            .and(path(format!("/bot{}/getMe", token)))
            .respond_with(response.clone())
            .mount(&self.server)
            .await;
            
        Mock::given(method("GET"))
            .and(path(format!("/bot{}/GetMe", token)))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    /// Setup all common mocks with default success responses
    pub async fn setup_default_mocks(&self) {
        self.setup_default_mocks_with_token(&test_bot_token()).await;
    }

    /// Setup all common mocks with default success responses for specific token
    pub async fn setup_default_mocks_with_token(&self, token: &str) {
        let config = MockResponseConfig::default();
        
        self.mock_send_message_with_token(token, config.clone()).await;
        self.mock_edit_message_text_with_token(token, config.clone()).await;
        self.mock_answer_callback_query_with_token(token, config.clone()).await;
        self.mock_get_me_with_token(token, config).await;
    }

    /// Setup mocks for error scenarios
    pub async fn setup_error_mocks(&self) {
        self.setup_error_mocks_with_token(&test_bot_token()).await;
    }

    /// Setup mocks for error scenarios with specific token
    pub async fn setup_error_mocks_with_token(&self, token: &str) {
        let config = MockResponseConfig {
            success: false,
            delay_ms: None,
            custom_response: None,
        };
        
        self.mock_send_message_with_token(token, config.clone()).await;
        self.mock_edit_message_text_with_token(token, config.clone()).await;
        self.mock_answer_callback_query_with_token(token, config.clone()).await;
        self.mock_get_me_with_token(token, config).await;
    }

    /// Setup mocks with timeout simulation
    pub async fn setup_timeout_mocks(&self, delay_ms: u64) {
        self.setup_timeout_mocks_with_token(&test_bot_token(), delay_ms).await;
    }

    /// Setup mocks with timeout simulation for specific token
    pub async fn setup_timeout_mocks_with_token(&self, token: &str, delay_ms: u64) {
        let config = MockResponseConfig {
            success: true,
            delay_ms: Some(delay_ms),
            custom_response: None,
        };
        
        self.mock_send_message_with_token(token, config.clone()).await;
        self.mock_edit_message_text_with_token(token, config.clone()).await;
        self.mock_answer_callback_query_with_token(token, config.clone()).await;
        self.mock_get_me_with_token(token, config).await;
    }

    /// Reset all mocks
    pub async fn reset(&self) {
        self.server.reset().await;
    }

    /// Verify that a specific endpoint was called
    pub async fn verify_endpoint_called(&self, endpoint: &str, times: usize) {
        // This would require additional wiremock verification features
        // For now, we'll implement basic verification through request counting
        let received_requests = self.server.received_requests().await.unwrap();
        
        let matching_requests = received_requests
            .iter()
            .filter(|req| {
                let path = req.url.path();
                // Check if the path contains the endpoint (case-insensitive)
                path.to_lowercase().contains(&endpoint.to_lowercase()) ||
                // Also check for exact endpoint match at the end of path
                path.ends_with(&format!("/{}", endpoint)) ||
                path.ends_with(&format!("/{}", endpoint.to_lowercase())) ||
                path.ends_with(&format!("/{}", endpoint.chars().next().unwrap().to_uppercase().collect::<String>() + &endpoint[1..]))
            })
            .count();
        
        assert_eq!(
            matching_requests, times,
            "Expected {} calls to {}, but got {}. Received requests: {:?}",
            times, endpoint, matching_requests,
            received_requests.iter().map(|r| format!("{} {}", r.method, r.url.path())).collect::<Vec<_>>()
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telegram_mock_server_creation() {
        let mock_server = TelegramMockServer::new().await;
        assert!(!mock_server.base_url.is_empty());
        assert!(mock_server.base_url.starts_with("http://"));
    }

    #[tokio::test]
    async fn test_get_api_url() {
        let mock_server = TelegramMockServer::new().await;
        let token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11";
        let api_url = mock_server.get_api_url(token);
        // The API URL should now be just the base URL (teloxide handles the token path)
        assert_eq!(api_url, mock_server.base_url);
        assert!(api_url.starts_with("http://"));
    }

    #[tokio::test]
    async fn test_setup_default_mocks() {
        let mock_server = TelegramMockServer::new().await;
        mock_server.setup_default_mocks().await;
        // If we reach here without panicking, the mocks were set up successfully
    }
}