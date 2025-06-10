//! Admin model

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminSettings {
    pub id: i64,
    pub key: String,
    pub value: serde_json::Value,
    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserState {
    pub user_id: i64,
    pub scenario: Option<String>,
    pub step: Option<String>,
    pub data: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CasCheck {
    pub id: i64,
    pub user_id: i64,
    pub telegram_id: i64,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdminSettingRequest {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdminSettingRequest {
    pub value: serde_json::Value,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserStateRequest {
    pub user_id: i64,
    pub scenario: Option<String>,
    pub step: Option<String>,
    pub data: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserStateRequest {
    pub scenario: Option<String>,
    pub step: Option<String>,
    pub data: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCasCheckRequest {
    pub user_id: i64,
    pub telegram_id: i64,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
}