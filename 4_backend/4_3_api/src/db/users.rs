use sqlx::PgPool;
use uuid::Uuid;

use crate::{DbError, UserWithRoles};

/// Internal struct for mapping SQL query results with aggregated roles.
#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    roles: Vec<String>,
}

/// Creates a user with an initial role. User must always have at least one role.
/// Uses a transaction to ensure atomicity.
pub async fn create(
    pool: &PgPool,
    name: &str,
    email: &str,
    role: &str,
) -> Result<UserWithRoles, DbError> {
    let mut tx = pool.begin().await?;
    // Check role existence with FOR SHARE lock (prevents role deletion)
    let role_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM roles WHERE slug = $1 FOR SHARE)")
            .bind(role)
            .fetch_one(&mut *tx)
            .await?;
    if !role_exists {
        return Err(DbError::NotFound(format!("Role '{role}'")));
    }

    let id: Uuid =
        sqlx::query_scalar("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(email)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    DbError::Conflict(format!("Email '{email}' is already taken."))
                } else {
                    DbError::Sqlx(e)
                }
            })?;

    sqlx::query("INSERT INTO users_roles (user_id, role_slug) VALUES ($1, $2)")
        .bind(id)
        .bind(role)
        .execute(&mut *tx)
        .await?;

    let row: Row = sqlx::query_as(
        r"SELECT u.id, u.name, u.email, u.created_at,
                 array_agg(ur.role_slug ORDER BY ur.role_slug) AS roles
          FROM users u
          JOIN users_roles ur ON ur.user_id = u.id
          WHERE u.id = $1
          GROUP BY u.id, u.name, u.email, u.created_at",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(UserWithRoles {
        id: row.id,
        name: row.name,
        email: row.email,
        created_at: row.created_at,
        roles: row.roles,
    })
}

/// Deletes a user. Role associations are deleted automatically (CASCADE).
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), DbError> {
    let rows = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(DbError::NotFound(format!("User '{id}'")));
    }
    Ok(())
}

/// Updates user fields. At least one field must be provided.
/// Uses CTE for atomic update and fetching result with roles.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    email: Option<&str>,
) -> Result<UserWithRoles, DbError> {
    if name.is_none() && email.is_none() {
        return Err(DbError::Validation("Nothing to update.".into()));
    }
    let row: Option<Row> = sqlx::query_as(
        r"WITH updated AS (
              UPDATE users
              SET name = COALESCE($1, name), email = COALESCE($2, email)
              WHERE id = $3
              RETURNING id, name, email, created_at
          )
          SELECT u.id, u.name, u.email, u.created_at,
                 COALESCE(array_agg(ur.role_slug ORDER BY ur.role_slug)
                          FILTER (WHERE ur.role_slug IS NOT NULL), '{}') AS roles
          FROM updated u
          LEFT JOIN users_roles ur ON ur.user_id = u.id
          GROUP BY u.id, u.name, u.email, u.created_at",
    )
    .bind(name)
    .bind(email)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        if is_unique_violation(&e) {
            DbError::Conflict(format!("Email '{}' is already taken.", email.unwrap_or("")))
        } else {
            DbError::Sqlx(e)
        }
    })?;

    let row = row.ok_or_else(|| DbError::NotFound(format!("User '{id}'")))?;
    Ok(UserWithRoles {
        id: row.id,
        name: row.name,
        email: row.email,
        created_at: row.created_at,
        roles: row.roles,
    })
}

/// Returns a list of all users with their roles, sorted by name.
pub async fn list(pool: &PgPool) -> Result<Vec<UserWithRoles>, DbError> {
    let rows: Vec<Row> = sqlx::query_as(
        r"SELECT u.id, u.name, u.email, u.created_at,
                 COALESCE(array_agg(ur.role_slug ORDER BY ur.role_slug)
                          FILTER (WHERE ur.role_slug IS NOT NULL), '{}') AS roles
          FROM users u
          LEFT JOIN users_roles ur ON ur.user_id = u.id
          GROUP BY u.id, u.name, u.email, u.created_at
          ORDER BY u.name",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UserWithRoles {
            id: r.id,
            name: r.name,
            email: r.email,
            created_at: r.created_at,
            roles: r.roles,
        })
        .collect())
}

/// Returns a user with their roles, or None if not found.
pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<UserWithRoles>, DbError> {
    let row: Option<Row> = sqlx::query_as(
        r"SELECT u.id, u.name, u.email, u.created_at,
                 COALESCE(array_agg(ur.role_slug ORDER BY ur.role_slug)
                          FILTER (WHERE ur.role_slug IS NOT NULL), '{}') AS roles
          FROM users u
          LEFT JOIN users_roles ur ON ur.user_id = u.id
          WHERE u.id = $1
          GROUP BY u.id, u.name, u.email, u.created_at",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| UserWithRoles {
        id: r.id,
        name: r.name,
        email: r.email,
        created_at: r.created_at,
        roles: r.roles,
    }))
}

/// Assigns a role to a user. Ignores if role is already assigned.
/// Uses locks to prevent race conditions.
pub async fn assign_role(pool: &PgPool, id: Uuid, role: &str) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    // Atomic check for user and role existence with locks
    let exists: Option<bool> = sqlx::query_scalar(
        r"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 FOR NO KEY UPDATE)
          AND EXISTS(SELECT 1 FROM roles WHERE slug = $2 FOR SHARE)",
    )
    .bind(id)
    .bind(role)
    .fetch_optional(&mut *tx)
    .await?;

    if exists != Some(true) {
        // Determine what exactly is missing (user or role)
        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        if !user_exists {
            return Err(DbError::NotFound(format!("User '{id}'")));
        }
        return Err(DbError::NotFound(format!("Role '{role}'")));
    }

    sqlx::query(
        "INSERT INTO users_roles (user_id, role_slug) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(role)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Unassigns a role from a user. Prevents removing the last role.
/// Uses locks to prevent race conditions.
pub async fn unassign_role(pool: &PgPool, id: Uuid, role: &str) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    // Atomic check: lock user's roles and count them
    let role_count: Option<(bool, i64)> = sqlx::query_as(
        r"SELECT 
              EXISTS(SELECT 1 FROM users_roles WHERE user_id = $1 AND role_slug = $2),
              COUNT(*)
          FROM users_roles
          WHERE user_id = $1
          FOR UPDATE",
    )
    .bind(id)
    .bind(role)
    .fetch_optional(&mut *tx)
    .await?;

    match role_count {
        None => return Err(DbError::NotFound(format!("User '{id}'"))),
        Some((false, _)) => {
            return Err(DbError::NotFound(format!(
                "User '{id}' does not have role '{role}'"
            )));
        }
        Some((true, count)) if count <= 1 => {
            return Err(DbError::BusinessRule(format!(
                "Cannot unassign: user '{id}' must have at least one role."
            )));
        }
        _ => {}
    }

    sqlx::query("DELETE FROM users_roles WHERE user_id = $1 AND role_slug = $2")
        .bind(id)
        .bind(role)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Checks if error is a UNIQUE constraint violation (PostgreSQL code 23505).
fn is_unique_violation(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        // PostgreSQL error code 23505 = unique_violation
        return db_err.code().as_deref() == Some("23505");
    }
    false
}
