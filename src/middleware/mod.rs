//! Middleware module
//! 
//! This module contains middleware for request processing

pub mod auth;
pub mod logging;
pub mod rate_limit;

// Re-export commonly used middleware
pub use auth::AuthMiddleware;
pub use rate_limit::RateLimitMiddleware;