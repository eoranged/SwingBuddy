//! Event model

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub event_date: DateTime<Utc>,
    pub location: Option<String>,
    pub max_participants: Option<i32>,
    pub google_calendar_id: Option<String>,
    pub created_by: Option<i64>,
    pub group_id: Option<i64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventParticipant {
    pub id: i64,
    pub event_id: i64,
    pub user_id: i64,
    pub status: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub event_date: DateTime<Utc>,
    pub location: Option<String>,
    pub max_participants: Option<i32>,
    pub created_by: Option<i64>,
    pub group_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub event_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub max_participants: Option<i32>,
    pub google_calendar_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParticipantRequest {
    pub event_id: i64,
    pub user_id: i64,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantStatus {
    Registered,
    Confirmed,
    Cancelled,
    Attended,
}

impl ToString for ParticipantStatus {
    fn to_string(&self) -> String {
        match self {
            ParticipantStatus::Registered => "registered".to_string(),
            ParticipantStatus::Confirmed => "confirmed".to_string(),
            ParticipantStatus::Cancelled => "cancelled".to_string(),
            ParticipantStatus::Attended => "attended".to_string(),
        }
    }
}