use anyhow::{bail, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

pub async fn create(pool: &PgPool, name: &str, email: &str, role: &str) -> Result<()> {
    let role_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM roles WHERE slug = $1)")
            .bind(role)
            .fetch_one(pool)
            .await?;
    if !role_exists {
        bail!("Role '{role}' does not exist.");
    }

    let mut tx = pool.begin().await?;
    let user: User = sqlx::query_as(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email, created_at",
    )
    .bind(name)
    .bind(email)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO users_roles (user_id, role_slug) VALUES ($1, $2)")
        .bind(user.id)
        .bind(role)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    println!("User created: {} ({})", user.id, user.name);
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let rows = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        bail!("User '{id}' not found.");
    }
    println!("User '{id}' deleted.");
    Ok(())
}

pub async fn update(pool: &PgPool, id: Uuid, name: Option<&str>, email: Option<&str>) -> Result<()> {
    if name.is_none() && email.is_none() {
        bail!("Nothing to update.");
    }
    let mut tx = pool.begin().await?;
    if let Some(n) = name {
        sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
            .bind(n).bind(id).execute(&mut *tx).await?;
    }
    if let Some(e) = email {
        sqlx::query("UPDATE users SET email = $1 WHERE id = $2")
            .bind(e).bind(id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    println!("User '{id}' updated.");
    Ok(())
}

pub async fn list(pool: &PgPool) -> Result<()> {
    let users: Vec<User> =
        sqlx::query_as("SELECT id, name, email, created_at FROM users ORDER BY name")
            .fetch_all(pool)
            .await?;
    if users.is_empty() {
        println!("No users found.");
        return Ok(());
    }
    for u in &users {
        let roles = fetch_user_roles(pool, u.id).await?;
        println!("[{}] {} <{}> (created: {}) - roles: [{}]",
            u.id, u.name, u.email, u.created_at.format("%Y-%m-%d %H:%M"), roles.join(", "));
    }
    Ok(())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<()> {
    let user: Option<User> =
        sqlx::query_as("SELECT id, name, email, created_at FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    match user {
        Some(u) => {
            let roles = fetch_user_roles(pool, u.id).await?;
            println!("[{}] {} <{}> (created: {}) - roles: [{}]",
                u.id, u.name, u.email, u.created_at.format("%Y-%m-%d %H:%M"), roles.join(", "));
        }
        None => bail!("User '{id}' not found."),
    }
    Ok(())
}

pub async fn assign_role(pool: &PgPool, id: Uuid, role: &str) -> Result<()> {
    let user_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
            .bind(id).fetch_one(pool).await?;
    if !user_exists { bail!("User '{id}' not found."); }

    let role_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM roles WHERE slug = $1)")
            .bind(role).fetch_one(pool).await?;
    if !role_exists { bail!("Role '{role}' not found."); }

    sqlx::query("INSERT INTO users_roles (user_id, role_slug) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(id).bind(role).execute(pool).await?;
    println!("Role '{role}' assigned to user '{id}'.");
    Ok(())
}

pub async fn unassign_role(pool: &PgPool, id: Uuid, role: &str) -> Result<()> {
    let mut tx = pool.begin().await?;

    let locked_roles: Vec<String> =
        sqlx::query_scalar("SELECT role_slug FROM users_roles WHERE user_id = $1 FOR UPDATE")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

    if locked_roles.len() <= 1 {
        bail!("Cannot unassign: user '{id}' must have at least one role.");
    }

    let rows = sqlx::query("DELETE FROM users_roles WHERE user_id = $1 AND role_slug = $2")
        .bind(id)
        .bind(role)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if rows == 0 {
        bail!("User '{id}' does not have role '{role}'.");
    }

    tx.commit().await?;
    println!("Role '{role}' unassigned from user '{id}'.");
    Ok(())
}

async fn fetch_user_roles(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
    let roles: Vec<String> =
        sqlx::query_scalar("SELECT role_slug FROM users_roles WHERE user_id = $1 ORDER BY role_slug")
            .bind(user_id)
            .fetch_all(pool)
            .await?;
    Ok(roles)
}
