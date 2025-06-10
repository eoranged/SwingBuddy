//! Logging middleware
//! 
//! This module provides logging middleware for tracking bot interactions,
//! performance metrics, and debugging information.

use std::time::Instant;
use teloxide::types::{Message, Update, User};
use tracing::{info, debug, warn, error, Span, instrument};
use serde_json::json;

/// Logging middleware for bot interactions
#[derive(Clone)]
pub struct LoggingMiddleware {
    log_user_interactions: bool,
    log_performance: bool,
    log_errors: bool,
}

impl LoggingMiddleware {
    /// Create a new LoggingMiddleware instance
    pub fn new(log_user_interactions: bool, log_performance: bool, log_errors: bool) -> Self {
        Self {
            log_user_interactions,
            log_performance,
            log_errors,
        }
    }

    /// Log incoming update
    #[instrument(skip(self, update))]
    pub fn log_update(&self, update: &Update) {
        if !self.log_user_interactions {
            return;
        }

        match update.kind {
            teloxide::types::UpdateKind::Message(ref message) => {
                self.log_message(message);
            }
            teloxide::types::UpdateKind::CallbackQuery(ref callback) => {
                let user = &callback.from;
                info!(
                    user_id = user.id.0,
                    callback_data = callback.data.as_deref().unwrap_or("none"),
                    "Callback query received"
                );
            }
            teloxide::types::UpdateKind::InlineQuery(ref query) => {
                info!(
                    user_id = query.from.id.0,
                    query = %query.query,
                    "Inline query received"
                );
            }
            _ => {
                debug!(update_type = ?std::mem::discriminant(&update.kind), "Other update type received");
            }
        }
    }

    /// Log message details
    #[instrument(skip(self, message))]
    pub fn log_message(&self, message: &Message) {
        if !self.log_user_interactions {
            return;
        }

        let user_info = message.from.as_ref().map(|user| {
            json!({
                "id": user.id.0,
                "username": user.username,
                "first_name": user.first_name,
                "is_bot": user.is_bot
            })
        });

        let chat_info = json!({
            "id": message.chat.id.0,
            "type": match message.chat.kind {
                teloxide::types::ChatKind::Public(ref public) => match public.kind {
                    teloxide::types::PublicChatKind::Group => "group",
                    teloxide::types::PublicChatKind::Supergroup(_) => "supergroup",
                    teloxide::types::PublicChatKind::Channel(_) => "channel",
                },
                teloxide::types::ChatKind::Private(_) => "private",
            }
        });

        let message_type = match &message.kind {
            teloxide::types::MessageKind::Common(common) => {
                match &common.media_kind {
                    teloxide::types::MediaKind::Text(text) => {
                        debug!(
                            user = ?user_info,
                            chat = ?chat_info,
                            text = %text.text,
                            "Text message received"
                        );
                        "text"
                    }
                    teloxide::types::MediaKind::Photo(_) => "photo",
                    teloxide::types::MediaKind::Video(_) => "video",
                    teloxide::types::MediaKind::Document(_) => "document",
                    teloxide::types::MediaKind::Audio(_) => "audio",
                    teloxide::types::MediaKind::Voice(_) => "voice",
                    teloxide::types::MediaKind::Sticker(_) => "sticker",
                    _ => "other_media",
                }
            }
            teloxide::types::MessageKind::NewChatMembers(_) => "new_chat_members",
            teloxide::types::MessageKind::LeftChatMember(_) => "left_chat_member",
            teloxide::types::MessageKind::GroupChatCreated(_) => "group_chat_created",
            _ => "other",
        };

        info!(
            user = ?user_info,
            chat = ?chat_info,
            message_type = message_type,
            message_id = message.id.0,
            "Message processed"
        );
    }

    /// Log command execution
    #[instrument(skip(self))]
    pub fn log_command(&self, user: &User, command: &str, args: &[String]) {
        if !self.log_user_interactions {
            return;
        }

        info!(
            user_id = user.id.0,
            username = user.username.as_deref().unwrap_or("none"),
            command = command,
            args = ?args,
            "Command executed"
        );
    }

    /// Log performance metrics
    #[instrument(skip(self))]
    pub fn log_performance(&self, operation: &str, duration: std::time::Duration, success: bool) {
        if !self.log_performance {
            return;
        }

        let duration_ms = duration.as_millis();
        
        if success {
            info!(
                operation = operation,
                duration_ms = duration_ms,
                "Operation completed successfully"
            );
        } else {
            warn!(
                operation = operation,
                duration_ms = duration_ms,
                "Operation failed"
            );
        }

        // Log slow operations
        if duration_ms > 1000 {
            warn!(
                operation = operation,
                duration_ms = duration_ms,
                "Slow operation detected"
            );
        }
    }

    /// Log error with context
    #[instrument(skip(self, error))]
    pub fn log_error(&self, error: &dyn std::error::Error, context: &str, user_id: Option<i64>) {
        if !self.log_errors {
            return;
        }

        error!(
            error = %error,
            context = context,
            user_id = user_id,
            "Error occurred"
        );
    }

    /// Create a performance tracking span
    pub fn create_performance_span(&self, operation: &str) -> Option<PerformanceTracker> {
        if self.log_performance {
            Some(PerformanceTracker::new(operation.to_string()))
        } else {
            None
        }
    }

    /// Log database operation
    #[instrument(skip(self))]
    pub fn log_database_operation(&self, operation: &str, table: &str, duration: std::time::Duration, rows_affected: Option<u64>) {
        if !self.log_performance {
            return;
        }

        info!(
            operation = operation,
            table = table,
            duration_ms = duration.as_millis(),
            rows_affected = rows_affected,
            "Database operation completed"
        );
    }

    /// Log API call
    #[instrument(skip(self))]
    pub fn log_api_call(&self, service: &str, endpoint: &str, duration: std::time::Duration, status_code: Option<u16>) {
        if !self.log_performance {
            return;
        }

        info!(
            service = service,
            endpoint = endpoint,
            duration_ms = duration.as_millis(),
            status_code = status_code,
            "API call completed"
        );
    }

    /// Log user action
    #[instrument(skip(self))]
    pub fn log_user_action(&self, user_id: i64, action: &str, details: Option<&str>) {
        if !self.log_user_interactions {
            return;
        }

        info!(
            user_id = user_id,
            action = action,
            details = details,
            "User action logged"
        );
    }

    /// Log security event
    #[instrument(skip(self))]
    pub fn log_security_event(&self, event_type: &str, user_id: Option<i64>, details: &str) {
        warn!(
            event_type = event_type,
            user_id = user_id,
            details = details,
            "Security event detected"
        );
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new(true, true, true)
    }
}

/// Performance tracker for measuring operation duration
pub struct PerformanceTracker {
    operation: String,
    start_time: Instant,
    _span: Span,
}

impl PerformanceTracker {
    fn new(operation: String) -> Self {
        let span = tracing::info_span!("performance", operation = %operation);
        
        Self {
            operation,
            start_time: Instant::now(),
            _span: span,
        }
    }

    /// Complete the performance tracking and log the result
    pub fn complete(self, success: bool) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_millis();
        
        if success {
            info!(
                operation = %self.operation,
                duration_ms = duration_ms,
                "Operation completed successfully"
            );
        } else {
            warn!(
                operation = %self.operation,
                duration_ms = duration_ms,
                "Operation failed"
            );
        }
    }
}

impl Drop for PerformanceTracker {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        debug!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            "Performance tracker dropped"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_middleware_creation() {
        let middleware = LoggingMiddleware::new(true, true, true);
        assert!(middleware.log_user_interactions);
        assert!(middleware.log_performance);
        assert!(middleware.log_errors);
    }

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::new("test_operation".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        tracker.complete(true);
    }

    #[test]
    fn test_default_logging_middleware() {
        let middleware = LoggingMiddleware::default();
        assert!(middleware.log_user_interactions);
        assert!(middleware.log_performance);
        assert!(middleware.log_errors);
    }
}