use anyhow::{bail, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Role;

pub async fn create(pool: &PgPool, slug: &str, name: &str, permissions: &str) -> Result<()> {
    let perms: serde_json::Value = serde_json::from_str(permissions)
        .map_err(|e| anyhow::anyhow!("permissions must be valid JSON: {e}"))?;

    sqlx::query("INSERT INTO roles (slug, name, permissions) VALUES ($1, $2, $3)")
        .bind(slug)
        .bind(name)
        .bind(perms)
        .execute(pool)
        .await?;

    println!("Role '{slug}' created.");
    Ok(())
}

pub async fn delete(pool: &PgPool, slug: &str) -> Result<()> {
    let mut tx = pool.begin().await?;

    // Find users for whom this is their only role (single query, no N+1)
    let blocked_users: Vec<Uuid> = sqlx::query_scalar(
        r#"SELECT user_id FROM users_roles
        WHERE role_slug = $1
        AND user_id NOT IN (
            SELECT user_id FROM users_roles WHERE role_slug != $1
        )
        FOR UPDATE"#,
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
    println!("Role '{slug}' deleted.");
    Ok(())
}

pub async fn update(
    pool: &PgPool,
    slug: &str,
    name: Option<&str>,
    permissions: Option<&str>,
) -> Result<()> {
    if name.is_none() && permissions.is_none() {
        bail!("Nothing to update.");
    }
    let mut tx = pool.begin().await?;
    if let Some(n) = name {
        sqlx::query("UPDATE roles SET name = $1 WHERE slug = $2")
            .bind(n).bind(slug).execute(&mut *tx).await?;
    }
    if let Some(p) = permissions {
        let perms: serde_json::Value = serde_json::from_str(p)
            .map_err(|e| anyhow::anyhow!("permissions must be valid JSON: {e}"))?;
        sqlx::query("UPDATE roles SET permissions = $1 WHERE slug = $2")
            .bind(perms).bind(slug).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    println!("Role '{slug}' updated.");
    Ok(())
}

pub async fn list(pool: &PgPool) -> Result<()> {
    let roles: Vec<Role> =
        sqlx::query_as("SELECT slug, name, permissions FROM roles ORDER BY slug")
            .fetch_all(pool)
            .await?;
    if roles.is_empty() {
        println!("No roles found.");
        return Ok(());
    }
    for r in &roles {
        println!("[{}] {} - permissions: {}", r.slug, r.name, r.permissions);
    }
    Ok(())
}

pub async fn get(pool: &PgPool, slug: &str) -> Result<()> {
    let role: Option<Role> =
        sqlx::query_as("SELECT slug, name, permissions FROM roles WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    match role {
        Some(r) => println!("[{}] {} - permissions: {}", r.slug, r.name, r.permissions),
        None => bail!("Role '{slug}' not found."),
    }
    Ok(())
}
