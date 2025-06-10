//! Error handling for SwingBuddy
//! 
//! This module defines the main error types used throughout the application
//! and provides a unified error handling strategy.

use thiserror::Error;

/// Main error type for SwingBuddy application
#[derive(Error, Debug)]
pub enum SwingBuddyError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    
    #[error("Telegram API error: {0}")]
    Telegram(#[from] teloxide::RequestError),
    
    #[error("CAS API error: {0}")]
    Cas(#[from] CasError),
    
    #[error("Google Calendar error: {0}")]
    Google(#[from] GoogleError),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: i64 },
    
    #[error("Group not found: {group_id}")]
    GroupNotFound { group_id: i64 },
    
    #[error("Event not found: {event_id}")]
    EventNotFound { event_id: i64 },
    
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// CAS API specific errors
#[derive(Error, Debug)]
pub enum CasError {
    #[error("CAS API request failed: {0}")]
    RequestFailed(String),
    
    #[error("CAS API timeout")]
    Timeout,
    
    #[error("Invalid CAS response: {0}")]
    InvalidResponse(String),
    
    #[error("CAS service unavailable")]
    ServiceUnavailable,
}

/// Google Calendar API specific errors
#[derive(Error, Debug)]
pub enum GoogleError {
    #[error("Google Calendar API error: {0}")]
    ApiError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Calendar not found: {0}")]
    CalendarNotFound(String),
    
    #[error("Event creation failed: {0}")]
    EventCreationFailed(String),
    
    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
}

/// Result type alias for SwingBuddy operations
pub type Result<T> = std::result::Result<T, SwingBuddyError>;

/// Result type alias for CAS operations
pub type CasResult<T> = std::result::Result<T, CasError>;

/// Result type alias for Google Calendar operations
pub type GoogleResult<T> = std::result::Result<T, GoogleError>;

impl SwingBuddyError {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            SwingBuddyError::Database(_) => false,
            SwingBuddyError::Migration(_) => false,
            SwingBuddyError::Telegram(_) => true,
            SwingBuddyError::Cas(_) => true,
            SwingBuddyError::Google(_) => true,
            SwingBuddyError::Config(_) => false,
            SwingBuddyError::PermissionDenied(_) => false,
            SwingBuddyError::UserNotFound { .. } => false,
            SwingBuddyError::GroupNotFound { .. } => false,
            SwingBuddyError::EventNotFound { .. } => false,
            SwingBuddyError::InvalidStateTransition { .. } => false,
            SwingBuddyError::Redis(_) => true,
            SwingBuddyError::Http(_) => true,
            SwingBuddyError::Serialization(_) => false,
            SwingBuddyError::Io(_) => true,
            SwingBuddyError::Authentication(_) => false,
            SwingBuddyError::RateLimitExceeded => true,
            SwingBuddyError::InvalidInput(_) => false,
            SwingBuddyError::ServiceUnavailable(_) => true,
            SwingBuddyError::UrlParse(_) => false,
        }
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SwingBuddyError::Database(_) => ErrorSeverity::Critical,
            SwingBuddyError::Migration(_) => ErrorSeverity::Critical,
            SwingBuddyError::Config(_) => ErrorSeverity::Critical,
            SwingBuddyError::PermissionDenied(_) => ErrorSeverity::Warning,
            SwingBuddyError::Authentication(_) => ErrorSeverity::Warning,
            SwingBuddyError::RateLimitExceeded => ErrorSeverity::Warning,
            SwingBuddyError::InvalidInput(_) => ErrorSeverity::Info,
            _ => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARN"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}