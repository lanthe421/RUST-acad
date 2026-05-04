use sqlx::PgPool;

use crate::{DbError, Role};

/// Creates a new role. Returns error if slug is already taken.
pub async fn create(
    pool: &PgPool,
    slug: &str,
    name: &str,
    permissions: &serde_json::Value,
) -> Result<Role, DbError> {
    let role: Role = sqlx::query_as(
        "INSERT INTO roles (slug, name, permissions) VALUES ($1, $2, $3) RETURNING slug, name, permissions",
    )
    .bind(slug)
    .bind(name)
    .bind(permissions)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if is_unique_violation(&e) {
            DbError::Conflict(format!("Role slug '{slug}' is already taken."))
        } else {
            DbError::Sqlx(e)
        }
    })?;

    Ok(role)
}

/// Deletes a role. Prevents deletion if users would be left without roles.
/// Uses locks to prevent race conditions.
pub async fn delete(pool: &PgPool, slug: &str) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    // Atomic check: lock role and count users who would be left without roles
    let count: Option<i64> = sqlx::query_scalar(
        r"WITH role_check AS (
              SELECT slug FROM roles WHERE slug = $1 FOR UPDATE
          ),
          affected_users AS (
              SELECT u.id
              FROM users u
              WHERE EXISTS (SELECT 1 FROM users_roles WHERE user_id = u.id AND role_slug = $1)
              FOR NO KEY UPDATE
          )
          SELECT COUNT(*)
          FROM users_roles ur1
          WHERE ur1.role_slug = $1
          AND NOT EXISTS (
              SELECT 1 FROM users_roles ur2
              WHERE ur2.user_id = ur1.user_id AND ur2.role_slug != $1
          )
          AND EXISTS (SELECT 1 FROM role_check)",
    )
    .bind(slug)
    .fetch_optional(&mut *tx)
    .await?;

    match count {
        None => return Err(DbError::NotFound(format!("Role '{slug}'"))),
        Some(n) if n > 0 => {
            return Err(DbError::BusinessRule(format!(
                "Cannot delete role '{slug}': {n} user(s) would be left without any role."
            )));
        }
        _ => {}
    }

    sqlx::query("DELETE FROM roles WHERE slug = $1")
        .bind(slug)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Updates role fields. At least one field must be provided.
pub async fn update(
    pool: &PgPool,
    slug: &str,
    name: Option<&str>,
    permissions: Option<&serde_json::Value>,
) -> Result<Role, DbError> {
    if name.is_none() && permissions.is_none() {
        return Err(DbError::Validation("Nothing to update.".into()));
    }

    sqlx::query_as(
        "UPDATE roles SET name = COALESCE($1, name), permissions = COALESCE($2, permissions) \
         WHERE slug = $3 RETURNING slug, name, permissions",
    )
    .bind(name)
    .bind(permissions)
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DbError::NotFound(format!("Role '{slug}'")))
}

/// Returns a list of all roles, sorted by slug.
pub async fn list(pool: &PgPool) -> Result<Vec<Role>, DbError> {
    let roles: Vec<Role> =
        sqlx::query_as("SELECT slug, name, permissions FROM roles ORDER BY slug")
            .fetch_all(pool)
            .await?;
    Ok(roles)
}

/// Returns a role by slug, or None if not found.
pub async fn get(pool: &PgPool, slug: &str) -> Result<Option<Role>, DbError> {
    let role: Option<Role> =
        sqlx::query_as("SELECT slug, name, permissions FROM roles WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(role)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Checks if error is a UNIQUE constraint violation (PostgreSQL code 23505).
fn is_unique_violation(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        return db_err.code().as_deref() == Some("23505");
    }
    false
}
