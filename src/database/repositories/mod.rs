//! Database repositories module
//! 
//! This module contains all repository implementations for data access

pub mod user;
pub mod group;
pub mod event;
pub mod admin;

// Re-export repositories
pub use user::UserRepository;
pub use group::GroupRepository;
pub use event::EventRepository;
pub use admin::AdminRepository;