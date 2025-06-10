//! Utility modules
//! 
//! This module contains common utilities used throughout the application,
//! including error handling, logging setup, and helper functions.

pub mod errors;
pub mod logging;
pub mod helpers;

pub use errors::{SwingBuddyError, Result};