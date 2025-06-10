//! Logging configuration and setup
//! 
//! This module provides logging initialization and structured logging utilities
//! for the SwingBuddy application.

use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::config::LoggingConfig;
use crate::utils::errors::Result;

/// Initialize logging based on configuration
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let file_appender = tracing_appender::rolling::daily(&config.file_path, "swingbuddy.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.level))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();
        
    info!("Logging initialized with level: {}", config.level);
    Ok(())
}

/// Log user actions with structured data
pub fn log_user_action(user_id: i64, action: &str, details: Option<&str>) {
    info!(
        user_id = user_id,
        action = action,
        details = details,
        "User action performed"
    );
}

/// Log CAS check results
pub fn log_cas_check(user_id: i64, is_banned: bool, reason: Option<&str>) {
    if is_banned {
        warn!(
            user_id = user_id,
            reason = reason,
            "CAS check: User is banned"
        );
    } else {
        debug!(user_id = user_id, "CAS check: User is clean");
    }
}

/// Log group events
pub fn log_group_event(group_id: i64, event: &str, user_id: Option<i64>, details: Option<&str>) {
    info!(
        group_id = group_id,
        event = event,
        user_id = user_id,
        details = details,
        "Group event occurred"
    );
}

/// Log event management actions
pub fn log_event_action(event_id: i64, action: &str, user_id: i64, details: Option<&str>) {
    info!(
        event_id = event_id,
        action = action,
        user_id = user_id,
        details = details,
        "Event action performed"
    );
}

/// Log admin actions
pub fn log_admin_action(admin_id: i64, action: &str, target: Option<&str>, details: Option<&str>) {
    warn!(
        admin_id = admin_id,
        action = action,
        target = target,
        details = details,
        "Admin action performed"
    );
}

/// Log API errors with context
pub fn log_api_error(api: &str, error: &str, context: Option<&str>) {
    error!(
        api = api,
        error = error,
        context = context,
        "API error occurred"
    );
}

/// Log database operations
pub fn log_database_operation(operation: &str, table: &str, duration_ms: u64, success: bool) {
    if success {
        debug!(
            operation = operation,
            table = table,
            duration_ms = duration_ms,
            "Database operation completed"
        );
    } else {
        error!(
            operation = operation,
            table = table,
            duration_ms = duration_ms,
            "Database operation failed"
        );
    }
}

/// Log performance metrics
pub fn log_performance_metric(metric_name: &str, value: f64, unit: &str) {
    debug!(
        metric = metric_name,
        value = value,
        unit = unit,
        "Performance metric recorded"
    );
}