//! Data models module
//!
//! This module contains all data structures used throughout the application

pub mod user;
pub mod group;
pub mod event;
pub mod admin;

// Re-export commonly used models
pub use user::{User, CreateUserRequest, UpdateUserRequest};
pub use group::{Group, GroupMember, CreateGroupRequest, UpdateGroupRequest, AddMemberRequest};
pub use event::{Event, EventParticipant, CreateEventRequest, UpdateEventRequest, RegisterParticipantRequest, ParticipantStatus};
pub use admin::{AdminSettings, UserState, CasCheck, CreateAdminSettingRequest, UpdateAdminSettingRequest, CreateUserStateRequest, UpdateUserStateRequest, CreateCasCheckRequest};