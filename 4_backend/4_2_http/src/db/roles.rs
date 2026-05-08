use anyhow::{Result, bail};
use sqlx::PgPool;
use uuid::Uuid;

use crate::Role;

/// # Errors
///
/// Returns an error if `permissions` is not valid JSON or if the database insert fails.
pub async fn create(pool: &PgPool, slug: &str, name: &str, permissions: &str) -> Result<String> {
    let perms: serde_json::Value = serde_json::from_str(permissions)
        .map_err(|e| anyhow::anyhow!("permissions must be valid JSON: {e}"))?;

    sqlx::query("INSERT INTO roles (slug, name, permissions) VALUES ($1, $2, $3)")
        .bind(slug)
        .bind(name)
        .bind(perms)
        .execute(pool)
        .await?;

    Ok(format!("Role '{slug}' created."))
}

/// # Errors
///
/// Returns an error if the role is not found, if any user would be left without a role, or if the
/// database operation fails.
pub async fn delete(pool: &PgPool, slug: &str) -> Result<String> {
    let mut tx = pool.begin().await?;

    // Find users for whom this is their only role (single query, no N+1)
    let blocked_users: Vec<Uuid> = sqlx::query_scalar(
        r"SELECT user_id FROM users_roles
        WHERE role_slug = $1
        AND user_id NOT IN (
            SELECT user_id FROM users_roles WHERE role_slug != $1
        )
        FOR UPDATE",
    )
    .bind(slug)
    .fetch_all(&mut *tx)
    .await?;

    if !blocked_users.is_empty() {
        bail!(
            "Cannot delete role '{slug}': {} user(s) would be left without any role.",
            blocked_users.len()
        );
    }

    let rows = sqlx::query("DELETE FROM roles WHERE slug = $1")
        .bind(slug)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if rows == 0 {
        bail!("Role '{slug}' not found.");
    }

    tx.commit().await?;
    Ok(format!("Role '{slug}' deleted."))
}

/// # Errors
///
/// Returns an error if neither `name` nor `permissions` is provided, if the role is not found,
/// if `permissions` is not valid JSON, or if the database operation fails.
pub async fn update(
    pool: &PgPool,
    slug: &str,
    name: Option<&str>,
    permissions: Option<&str>,
) -> Result<String> {
    if name.is_none() && permissions.is_none() {
        bail!("Nothing to update.");
    }

    // Parse permissions JSON before touching the DB — fail fast on bad input.
    let perms: Option<serde_json::Value> = permissions
        .map(|p| {
            serde_json::from_str(p)
                .map_err(|e| anyhow::anyhow!("permissions must be valid JSON: {e}"))
        })
        .transpose()?;

    // COALESCE keeps the existing value when the argument is NULL — single round-trip, no tx needed.
    let rows = sqlx::query(
        "UPDATE roles SET name = COALESCE($1, name), permissions = COALESCE($2, permissions) WHERE slug = $3",
    )
    .bind(name)
    .bind(perms)
    .bind(slug)
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 {
        bail!("Role '{slug}' not found.");
    }
    Ok(format!("Role '{slug}' updated."))
}

/// # Errors
///
/// Returns an error if the database query fails.
pub async fn list(pool: &PgPool) -> Result<String> {
    let roles: Vec<Role> =
        sqlx::query_as("SELECT slug, name, permissions FROM roles ORDER BY slug")
            .fetch_all(pool)
            .await?;
    if roles.is_empty() {
        return Ok("No roles found.".to_string());
    }
    let lines: Vec<String> = roles
        .iter()
        .map(|r| format!("[{}] {} - permissions: {}", r.slug, r.name, r.permissions))
        .collect();
    Ok(lines.join("\n"))
}

/// # Errors
///
/// Returns an error if the role is not found or if the database query fails.
pub async fn get(pool: &PgPool, slug: &str) -> Result<String> {
    let role: Option<Role> =
        sqlx::query_as("SELECT slug, name, permissions FROM roles WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    match role {
        Some(r) => Ok(format!(
            "[{}] {} - permissions: {}",
            r.slug, r.name, r.permissions
        )),
        None => bail!("Role '{slug}' not found."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_and_get_role(pool: PgPool) {
        let msg = create(&pool, "admin", "Admin", r#"{"read":true}"#)
            .await
            .unwrap();
        assert!(msg.contains("created"));

        let msg = get(&pool, "admin").await.unwrap();
        assert!(msg.contains("admin"));
        assert!(msg.contains("Admin"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_role_invalid_permissions(pool: PgPool) {
        let err = create(&pool, "bad", "Bad", "not-json").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("valid JSON"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_role(pool: PgPool) {
        create(&pool, "editor", "Editor", "{}").await.unwrap();
        let msg = delete(&pool, "editor").await.unwrap();
        assert!(msg.contains("deleted"));

        let err = delete(&pool, "editor").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("not found"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_role_blocked_by_user(pool: PgPool) {
        create(&pool, "admin", "Admin", "{}").await.unwrap();
        // Insert a user with only this role
        let user_id: Uuid = sqlx::query_scalar(
            "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com') RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO users_roles (user_id, role_slug) VALUES ($1, 'admin')")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

        let err = delete(&pool, "admin").await;
        assert!(err.is_err());
        assert!(
            err.unwrap_err()
                .to_string()
                .contains("would be left without any role")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_role(pool: PgPool) {
        create(&pool, "editor", "Editor", "{}").await.unwrap();
        let msg = update(&pool, "editor", Some("Senior Editor"), None)
            .await
            .unwrap();
        assert!(msg.contains("updated"));

        let err = update(&pool, "editor", None, None).await;
        assert!(err.is_err());

        // Non-existent user should return an error
        let missing = "";
        let err = update(&pool, missing, Some("Ghost"), None).await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("not found"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_roles(pool: PgPool) {
        let msg = list(&pool).await.unwrap();
        assert_eq!(msg, "No roles found.");

        create(&pool, "admin", "Admin", "{}").await.unwrap();
        let msg = list(&pool).await.unwrap();
        assert!(msg.contains("admin"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_role_not_found(pool: PgPool) {
        let err = get(&pool, "nonexistent").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("not found"));
    }
}
