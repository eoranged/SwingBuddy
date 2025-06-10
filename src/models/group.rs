//! Group model

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: i64,
    pub telegram_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub language_code: String,
    pub settings: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMember {
    pub id: i64,
    pub group_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub telegram_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub language_code: Option<String>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language_code: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub group_id: i64,
    pub user_id: i64,
    pub role: Option<String>,
}