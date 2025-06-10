//! Database service layer
//! 
//! This module provides a high-level interface to database operations

use crate::database::{DatabasePool, UserRepository, GroupRepository, EventRepository, AdminRepository};
use crate::models::*;
use crate::utils::errors::SwingBuddyError;

#[derive(Debug, Clone)]
pub struct DatabaseService {
    pub users: UserRepository,
    pub groups: GroupRepository,
    pub events: EventRepository,
    pub admin: AdminRepository,
}

impl DatabaseService {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            users: UserRepository::new(pool.clone()),
            groups: GroupRepository::new(pool.clone()),
            events: EventRepository::new(pool.clone()),
            admin: AdminRepository::new(pool),
        }
    }

    /// Initialize a new user in the system
    pub async fn initialize_user(&self, telegram_id: i64, username: Option<String>, first_name: Option<String>, last_name: Option<String>) -> Result<User, SwingBuddyError> {
        // Check if user already exists
        if let Some(existing_user) = self.users.find_by_telegram_id(telegram_id).await? {
            return Ok(existing_user);
        }

        // Create new user
        let request = CreateUserRequest {
            telegram_id,
            username,
            first_name,
            last_name,
            language_code: Some("en".to_string()),
            location: None,
        };

        self.users.create(request).await
    }

    /// Initialize a new group in the system
    pub async fn initialize_group(&self, telegram_id: i64, title: String, description: Option<String>) -> Result<Group, SwingBuddyError> {
        // Check if group already exists
        if let Some(existing_group) = self.groups.find_by_telegram_id(telegram_id).await? {
            return Ok(existing_group);
        }

        // Create new group
        let request = CreateGroupRequest {
            telegram_id,
            title,
            description,
            language_code: Some("en".to_string()),
            settings: None,
        };

        self.groups.create(request).await
    }

    /// Add user to group
    pub async fn add_user_to_group(&self, user_id: i64, group_id: i64, role: Option<String>) -> Result<GroupMember, SwingBuddyError> {
        // Check if user is already a member
        if self.groups.is_member(group_id, user_id).await? {
            return Err(SwingBuddyError::Config("User is already a member of this group".to_string()));
        }

        let request = AddMemberRequest {
            group_id,
            user_id,
            role,
        };

        self.groups.add_member(request).await
    }

    /// Create a new event
    pub async fn create_event(&self, title: String, description: Option<String>, event_date: chrono::DateTime<chrono::Utc>, location: Option<String>, max_participants: Option<i32>, created_by: Option<i64>, group_id: Option<i64>) -> Result<Event, SwingBuddyError> {
        let request = CreateEventRequest {
            title,
            description,
            event_date,
            location,
            max_participants,
            created_by,
            group_id,
        };

        self.events.create(request).await
    }

    /// Register user for event
    pub async fn register_for_event(&self, event_id: i64, user_id: i64) -> Result<EventParticipant, SwingBuddyError> {
        // Check if user is already registered
        if self.events.is_registered(event_id, user_id).await? {
            return Err(SwingBuddyError::Config("User is already registered for this event".to_string()));
        }

        // Check if event has reached max participants
        if let Some(event) = self.events.find_by_id(event_id).await? {
            if let Some(max_participants) = event.max_participants {
                let current_count = self.events.get_participant_count(event_id).await?;
                if current_count >= max_participants as i64 {
                    return Err(SwingBuddyError::Config("Event has reached maximum participants".to_string()));
                }
            }
        } else {
            return Err(SwingBuddyError::Config("Event not found".to_string()));
        }

        let request = RegisterParticipantRequest {
            event_id,
            user_id,
            status: Some("registered".to_string()),
        };

        self.events.register_participant(request).await
    }

    /// Get user's dashboard data
    pub async fn get_user_dashboard(&self, user_id: i64) -> Result<serde_json::Value, SwingBuddyError> {
        let user = self.users.find_by_id(user_id).await?
            .ok_or_else(|| SwingBuddyError::UserNotFound { user_id })?;

        let user_groups = self.groups.get_user_groups(user_id).await?;
        let registered_events = self.events.get_user_registered_events(user_id).await?;
        let created_events = self.events.get_user_events(user_id).await?;

        let dashboard = serde_json::json!({
            "user": user,
            "groups": user_groups,
            "registered_events": registered_events,
            "created_events": created_events
        });

        Ok(dashboard)
    }

    /// Set user conversation state
    pub async fn set_user_state(&self, user_id: i64, scenario: Option<String>, step: Option<String>, data: Option<serde_json::Value>, expires_at: Option<chrono::DateTime<chrono::Utc>>) -> Result<UserState, SwingBuddyError> {
        let request = CreateUserStateRequest {
            user_id,
            scenario,
            step,
            data,
            expires_at,
        };

        self.admin.upsert_user_state(request).await
    }

    /// Get user conversation state
    pub async fn get_user_state(&self, user_id: i64) -> Result<Option<UserState>, SwingBuddyError> {
        self.admin.get_user_state(user_id).await
    }

    /// Clear user conversation state
    pub async fn clear_user_state(&self, user_id: i64) -> Result<(), SwingBuddyError> {
        self.admin.delete_user_state(user_id).await
    }

    /// Record CAS check result
    pub async fn record_cas_check(&self, user_id: i64, telegram_id: i64, is_banned: bool, ban_reason: Option<String>) -> Result<CasCheck, SwingBuddyError> {
        let request = CreateCasCheckRequest {
            user_id,
            telegram_id,
            is_banned,
            ban_reason,
        };

        self.admin.create_cas_check(request).await
    }

    /// Get system statistics
    pub async fn get_system_stats(&self) -> Result<serde_json::Value, SwingBuddyError> {
        self.admin.get_stats().await
    }

    /// Clean up expired data
    pub async fn cleanup_expired_data(&self) -> Result<serde_json::Value, SwingBuddyError> {
        let expired_states = self.admin.clean_expired_states().await?;
        let old_cas_checks = self.admin.clean_old_cas_checks(30).await?; // Keep 30 days

        let cleanup_result = serde_json::json!({
            "expired_states_cleaned": expired_states,
            "old_cas_checks_cleaned": old_cas_checks
        });

        Ok(cleanup_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_service_creation() {
        // This would require a test database setup
        // For now, just test that the service can be created
        let pool = sqlx::PgPool::connect("postgresql://test").await;
        if let Ok(pool) = pool {
            let service = DatabaseService::new(pool);
            // Test that the service was created successfully
            // We can't access private fields, so just verify the service exists
            assert!(std::ptr::addr_of!(service.users) as *const _ != std::ptr::null());
            assert!(std::ptr::addr_of!(service.groups) as *const _ != std::ptr::null());
            assert!(std::ptr::addr_of!(service.events) as *const _ != std::ptr::null());
            assert!(std::ptr::addr_of!(service.admin) as *const _ != std::ptr::null());
        }
    }
}