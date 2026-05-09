use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use derive_more::{ From, Into};

#[derive(Debug, sqlx::FromRow, From, Into)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, From, Into)]
pub struct Role {
    pub slug: String,
    pub name: String,
    pub permissions: JsonValue,
}
