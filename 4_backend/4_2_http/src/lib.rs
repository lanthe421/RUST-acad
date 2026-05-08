use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

// ── DB layer (re-exported for use by server binary) ───────────────────────────
pub mod db;

// ── Input validation ──────────────────────────────────────────────────────────
pub mod validate {
    use anyhow::{bail, Result};

    /// Non-empty, not-only-whitespace string.
    pub fn non_empty(field: &str, value: &str) -> Result<()> {
        if value.trim().is_empty() {
            bail!("'{field}' must not be empty.");
        }
        Ok(())
    }

    /// Role slug: lowercase letters, digits, hyphens; 1–64 chars.
    pub fn slug(value: &str) -> Result<()> {
        non_empty("slug", value)?;
        if value.len() > 64 {
            bail!("slug must be 64 characters or fewer.");
        }
        let valid = value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !valid || value.starts_with('-') || value.ends_with('-') {
            bail!(
                "slug must contain only lowercase letters, digits, and hyphens, \
                 and must not start or end with a hyphen."
            );
        }
        Ok(())
    }

    /// Basic email sanity check: contains exactly one '@' with non-empty parts.
    pub fn email(value: &str) -> Result<()> {
        non_empty("email", value)?;
        let mut parts = value.splitn(2, '@');
        let local = parts.next().unwrap_or("");
        let domain = parts.next().unwrap_or("");
        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            bail!("'{value}' is not a valid email address.");
        }
        Ok(())
    }
}

// ── Shared data models ────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Role {
    pub slug: String,
    pub name: String,
    pub permissions: JsonValue,
}

// ── Command protocol ──────────────────────────────────────

/// All operations the client can request from the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Command {
    UserCreate {
        name: String,
        email: String,
        role: String,
    },
    UserDelete {
        id: Uuid,
    },
    UserUpdate {
        id: Uuid,
        name: Option<String>,
        email: Option<String>,
    },
    UserList,
    UserGet {
        id: Uuid,
    },
    UserAssignRole {
        id: Uuid,
        role: String,
    },
    UserUnassignRole {
        id: Uuid,
        role: String,
    },
    RoleCreate {
        slug: String,
        name: String,
        permissions: String,
    },
    RoleDelete {
        slug: String,
    },
    RoleUpdate {
        slug: String,
        name: Option<String>,
        permissions: Option<String>,
    },
    RoleList,
    RoleGet {
        slug: String,
    },
}

// ── Response protocol (Requirements 3.2) ─────────────────────────────────────

/// Structured response returned by the server for every command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum CommandResponse {
    #[serde(rename = "ok")]
    Ok { message: String },
    #[serde(rename = "error")]
    Error { message: String },
}

// ── Property-based tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy that generates a random Uuid from two u64 values.
    fn uuid_strategy() -> impl Strategy<Value = Uuid> {
        (any::<u64>(), any::<u64>()).prop_map(|(hi, lo)| {
            let mut bytes = [0u8; 16];
            bytes[..8].copy_from_slice(&hi.to_le_bytes());
            bytes[8..].copy_from_slice(&lo.to_le_bytes());
            Uuid::from_bytes(bytes)
        })
    }

    /// Strategy that generates arbitrary Command variants.
    fn command_strategy() -> impl Strategy<Value = Command> {
        prop_oneof![
            // UserCreate
            (".*", ".*", ".*").prop_map(|(name, email, role)| Command::UserCreate {
                name,
                email,
                role,
            }),
            // UserDelete
            uuid_strategy().prop_map(|id| Command::UserDelete { id }),
            // UserUpdate
            (
                uuid_strategy(),
                proptest::option::of(".*"),
                proptest::option::of(".*")
            )
                .prop_map(|(id, name, email)| Command::UserUpdate { id, name, email }),
            // UserList
            Just(Command::UserList),
            // UserGet
            uuid_strategy().prop_map(|id| Command::UserGet { id }),
            // UserAssignRole
            (uuid_strategy(), ".*").prop_map(|(id, role)| Command::UserAssignRole { id, role }),
            // UserUnassignRole
            (uuid_strategy(), ".*").prop_map(|(id, role)| Command::UserUnassignRole { id, role }),
            // RoleCreate
            (".*", ".*", ".*").prop_map(|(slug, name, permissions)| Command::RoleCreate {
                slug,
                name,
                permissions,
            }),
            // RoleDelete
            ".*".prop_map(|slug| Command::RoleDelete { slug }),
            // RoleUpdate
            (".*", proptest::option::of(".*"), proptest::option::of(".*")).prop_map(
                |(slug, name, permissions)| Command::RoleUpdate {
                    slug,
                    name,
                    permissions,
                }
            ),
            // RoleList
            Just(Command::RoleList),
            // RoleGet
            ".*".prop_map(|slug| Command::RoleGet { slug }),
        ]
    }

    proptest! {
        // Feature: http-client-server, Property 1: Command serialization round-trip
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn prop_command_round_trip(cmd in command_strategy()) {
            // Validates: Requirements 3.4
            let json = serde_json::to_string(&cmd).expect("serialization failed");
            let restored: Command = serde_json::from_str(&json).expect("deserialization failed");
            prop_assert_eq!(cmd, restored);
        }
    }
}
