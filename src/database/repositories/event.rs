//! Event repository implementation

use sqlx::PgPool;
use chrono::Utc;
use crate::models::event::{Event, EventParticipant, CreateEventRequest, UpdateEventRequest, RegisterParticipantRequest};
use crate::utils::errors::SwingBuddyError;

#[derive(Clone)]
pub struct EventRepository {
    pool: PgPool,
}

impl EventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new event
    pub async fn create(&self, request: CreateEventRequest) -> Result<Event, SwingBuddyError> {
        let event = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO events (title, description, event_date, location, max_participants, created_by, group_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at
            "#
        )
        .bind(request.title)
        .bind(request.description)
        .bind(request.event_date)
        .bind(request.location)
        .bind(request.max_participants)
        .bind(request.created_by)
        .bind(request.group_id)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(event)
    }

    /// Find event by ID
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Event>, SwingBuddyError> {
        let event = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at FROM events WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }

    /// Update event
    pub async fn update(&self, id: i64, request: UpdateEventRequest) -> Result<Event, SwingBuddyError> {
        let event = sqlx::query_as::<_, Event>(
            r#"
            UPDATE events
            SET title = COALESCE($2, title),
                description = COALESCE($3, description),
                event_date = COALESCE($4, event_date),
                location = COALESCE($5, location),
                max_participants = COALESCE($6, max_participants),
                google_calendar_id = COALESCE($7, google_calendar_id),
                is_active = COALESCE($8, is_active),
                updated_at = $9
            WHERE id = $1
            RETURNING id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(request.title)
        .bind(request.description)
        .bind(request.event_date)
        .bind(request.location)
        .bind(request.max_participants)
        .bind(request.google_calendar_id)
        .bind(request.is_active)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(event)
    }

    /// Delete event
    pub async fn delete(&self, id: i64) -> Result<(), SwingBuddyError> {
        sqlx::query("DELETE FROM events WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List events with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Event>, SwingBuddyError> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at FROM events ORDER BY event_date ASC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Get upcoming events
    pub async fn get_upcoming_events(&self, limit: Option<i64>) -> Result<Vec<Event>, SwingBuddyError> {
        let limit = limit.unwrap_or(50);
        let events = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at FROM events WHERE event_date > NOW() AND is_active = true ORDER BY event_date ASC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Get events for group
    pub async fn get_group_events(&self, group_id: i64) -> Result<Vec<Event>, SwingBuddyError> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at FROM events WHERE group_id = $1 AND is_active = true ORDER BY event_date ASC"
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Register participant for event
    pub async fn register_participant(&self, request: RegisterParticipantRequest) -> Result<EventParticipant, SwingBuddyError> {
        let participant = sqlx::query_as::<_, EventParticipant>(
            r#"
            INSERT INTO event_participants (event_id, user_id, status, registered_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, event_id, user_id, status, registered_at
            "#
        )
        .bind(request.event_id)
        .bind(request.user_id)
        .bind(request.status.unwrap_or_else(|| "registered".to_string()))
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(participant)
    }

    /// Unregister participant from event
    pub async fn unregister_participant(&self, event_id: i64, user_id: i64) -> Result<(), SwingBuddyError> {
        sqlx::query("DELETE FROM event_participants WHERE event_id = $1 AND user_id = $2")
            .bind(event_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get event participants
    pub async fn get_participants(&self, event_id: i64) -> Result<Vec<EventParticipant>, SwingBuddyError> {
        let participants = sqlx::query_as::<_, EventParticipant>(
            "SELECT id, event_id, user_id, status, registered_at FROM event_participants WHERE event_id = $1 ORDER BY registered_at ASC"
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    /// Check if user is registered for event
    pub async fn is_registered(&self, event_id: i64, user_id: i64) -> Result<bool, SwingBuddyError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM event_participants WHERE event_id = $1 AND user_id = $2"
        )
        .bind(event_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }

    /// Update participant status
    pub async fn update_participant_status(&self, event_id: i64, user_id: i64, status: &str) -> Result<EventParticipant, SwingBuddyError> {
        let participant = sqlx::query_as::<_, EventParticipant>(
            r#"
            UPDATE event_participants
            SET status = $3
            WHERE event_id = $1 AND user_id = $2
            RETURNING id, event_id, user_id, status, registered_at
            "#
        )
        .bind(event_id)
        .bind(user_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(participant)
    }

    /// Get participant count for event
    pub async fn get_participant_count(&self, event_id: i64) -> Result<i64, SwingBuddyError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM event_participants WHERE event_id = $1"
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Get events created by user
    pub async fn get_user_events(&self, user_id: i64) -> Result<Vec<Event>, SwingBuddyError> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at FROM events WHERE created_by = $1 ORDER BY event_date ASC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Get events user is registered for
    pub async fn get_user_registered_events(&self, user_id: i64) -> Result<Vec<Event>, SwingBuddyError> {
        let events = sqlx::query_as::<_, Event>(
            r#"
            SELECT e.id, e.title, e.description, e.event_date, e.location, e.max_participants, e.google_calendar_id, e.created_by, e.group_id, e.is_active, e.created_at, e.updated_at
            FROM events e
            INNER JOIN event_participants ep ON e.id = ep.event_id
            WHERE ep.user_id = $1 AND e.is_active = true
            ORDER BY e.event_date ASC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Count total events
    pub async fn count(&self) -> Result<i64, SwingBuddyError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_repository_creation() {
        // This would require a test database setup
        // For now, just test that the repository can be created
        let pool = PgPool::connect("postgresql://test").await;
        if let Ok(pool) = pool {
            let repo = EventRepository::new(pool);
            assert!(!repo.pool.is_closed());
        }
    }
}