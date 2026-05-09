use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use derive_more::{ Display, From, Into};

#[derive(Clone, Debug, PartialEq, From, Into, sqlx::Type, Display, Copy)]
#[sqlx(transparent)]
pub struct UserId(Uuid);


#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, From, Into, Display, sqlx::Type)]
#[sqlx(transparent)]
pub struct RoleSlug(String);




#[derive(Debug, sqlx::FromRow)]
pub struct Role {
    pub slug: RoleSlug,
    pub name: String,
    pub permissions: JsonValue,
}
