use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod db;

// ── Domain errors ─────────────────────────────────────────────────────────────

/// Business logic and database errors.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("{0} not found.")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    BusinessRule(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

// ── Custom garde validators ───────────────────────────────────────────────────

/// Validates slug: lowercase letters, digits, hyphens only; cannot start/end with hyphen.
fn validate_slug(value: &String, _ctx: &()) -> garde::Result {
    let s = value.as_str();
    if s.trim().is_empty() {
        return Err(garde::Error::new("must not be empty"));
    }
    if s.len() > 64 {
        return Err(garde::Error::new("must be 64 characters or fewer"));
    }
    let valid = s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid || s.starts_with('-') || s.ends_with('-') {
        return Err(garde::Error::new(
            "must contain only lowercase letters, digits, and hyphens, \
             and must not start or end with a hyphen",
        ));
    }
    Ok(())
}

// ── Data models ───────────────────────────────────────────────────────────────

/// Base user model (without roles).
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// Role model with permissions stored as JSON.
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, ToSchema)]
pub struct Role {
    pub slug: String,
    pub name: String,
    pub permissions: JsonValue,
}

/// User with their assigned roles.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserWithRoles {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub roles: Vec<String>,
}

// ── Request / response bodies ─────────────────────────────────────────────────

/// Request to create a user. Initial role is required.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateUserRequest {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(email)]
    pub email: String,
    /// Initial role (required — user must always have at least one role).
    #[garde(custom(validate_slug))]
    pub role: String,
}

/// Request to update a user. All fields are optional, but at least one must be provided.
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[garde(length(min = 1))]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[garde(email)]
    pub email: Option<String>,
}

/// Request to update a role. All fields are optional, but at least one must be provided.
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateRoleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[garde(length(min = 1))]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub permissions: Option<JsonValue>,
}

/// Request to assign a role to a user.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct AssignRoleRequest {
    #[garde(custom(validate_slug))]
    pub role: String,
}

/// Request to create a role.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateRoleRequest {
    #[garde(custom(validate_slug))]
    pub slug: String,
    #[garde(length(min = 1))]
    pub name: String,
    #[serde(default = "default_permissions")]
    #[garde(skip)]
    pub permissions: JsonValue,
}

fn default_permissions() -> JsonValue {
    JsonValue::Array(vec![])
}

/// Generic error response body.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { error: msg.into() }
    }
}
