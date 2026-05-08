use anyhow::{bail, Result};
use crate::User;
use sqlx::PgPool;
use uuid::Uuid;

/// # Errors
///
/// Returns an error if the role does not exist, if the email is already taken, or if the database
/// operation fails.
pub async fn create(pool: &PgPool, name: &str, email: &str, role: &str) -> Result<String> {
    let mut tx = pool.begin().await?;
    let role_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM roles WHERE slug = $1 FOR SHARE)")
            .bind(role)
            .fetch_one(&mut *tx)
            .await?;
    if !role_exists {
        bail!("Role '{role}' does not exist.");
    }

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

    Ok(format!("User created: {} ({})", user.id, user.name))
}

/// # Errors
///
/// Returns an error if the user is not found or if the database operation fails.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<String> {
    let rows = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        bail!("User '{id}' not found.");
    }
    Ok(format!("User '{id}' deleted."))
}

/// # Errors
///
/// Returns an error if neither `name` nor `email` is provided, if the user is not found, or if
/// the database operation fails.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    email: Option<&str>,
) -> Result<String> {
    if name.is_none() && email.is_none() {
        bail!("Nothing to update.");
    }

    // COALESCE keeps the existing value when the argument is NULL — single round-trip, no tx needed.
    let rows = sqlx::query(
        "UPDATE users SET name = COALESCE($1, name), email = COALESCE($2, email) WHERE id = $3",
    )
    .bind(name)
    .bind(email)
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 {
        bail!("User '{id}' not found.");
    }
    Ok(format!("User '{id}' updated."))
}

#[derive(sqlx::FromRow)]
struct UserWithRoles {
    id: Uuid,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    roles: Vec<String>,
}

/// # Errors
///
/// Returns an error if the database query fails.
pub async fn list(pool: &PgPool) -> Result<String> {
    // Single query with array_agg — no N+1
    let rows: Vec<UserWithRoles> = sqlx::query_as(
        r"
        SELECT u.id, u.name, u.email, u.created_at,
               COALESCE(array_agg(ur.role_slug ORDER BY ur.role_slug)
                        FILTER (WHERE ur.role_slug IS NOT NULL), '{}') AS roles
        FROM users u
        LEFT JOIN users_roles ur ON ur.user_id = u.id
        GROUP BY u.id, u.name, u.email, u.created_at
        ORDER BY u.name
        ",
    )
    .fetch_all(pool)
    .await?;
    if rows.is_empty() {
        return Ok("No users found.".to_string());
    }
    Ok(rows
        .iter()
        .map(|r| {
            format!(
                "[{}] {} <{}> (created: {}) - roles: [{}]",
                r.id,
                r.name,
                r.email,
                r.created_at.format("%Y-%m-%d %H:%M"),
                r.roles.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// # Errors
///
/// Returns an error if the user is not found or if the database query fails.
pub async fn get(pool: &PgPool, id: Uuid) -> Result<String> {
    let row: Option<UserWithRoles> = sqlx::query_as(
        r"
        SELECT u.id, u.name, u.email, u.created_at,
               COALESCE(array_agg(ur.role_slug ORDER BY ur.role_slug)
                        FILTER (WHERE ur.role_slug IS NOT NULL), '{}') AS roles
        FROM users u
        LEFT JOIN users_roles ur ON ur.user_id = u.id
        WHERE u.id = $1
        GROUP BY u.id, u.name, u.email, u.created_at
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(format!(
            "[{}] {} <{}> (created: {}) - roles: [{}]",
            r.id,
            r.name,
            r.email,
            r.created_at.format("%Y-%m-%d %H:%M"),
            r.roles.join(", ")
        )),
        None => bail!("User '{id}' not found."),
    }
}

/// # Errors
///
/// Returns an error if the role or user is not found, or if the database operation fails.
pub async fn assign_role(pool: &PgPool, id: Uuid, role: &str) -> Result<String> {
    let mut tx = pool.begin().await?;

    // Lock the role row so a concurrent roles::delete cannot proceed until we commit.
    // If the role is being deleted concurrently, one of the two transactions will block
    // and then see a consistent state after the other commits.
    let role_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM roles WHERE slug = $1 FOR UPDATE)")
            .bind(role)
            .fetch_one(&mut *tx)
            .await?;
    if !role_exists {
        bail!("Role '{role}' not found.");
    }

    let user_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if !user_exists {
        bail!("User '{id}' not found.");
    }

    sqlx::query(
        "INSERT INTO users_roles (user_id, role_slug) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(role)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(format!("Role '{role}' assigned to user '{id}'."))
}

/// # Errors
///
/// Returns an error if the user does not have the role, if it is the user's last role, or if the
/// database operation fails.
pub async fn unassign_role(pool: &PgPool, id: Uuid, role: &str) -> Result<String> {
    let mut tx = pool.begin().await?;

    // Lock all role assignments for this user atomically
    let locked_roles: Vec<String> =
        sqlx::query_scalar("SELECT role_slug FROM users_roles WHERE user_id = $1 FOR UPDATE")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

    // 1. Check the role actually belongs to the user (correct error message first)
    if !locked_roles.iter().any(|r| r == role) {
        bail!("User '{id}' does not have role '{role}'.");
    }

    // 2. Check it's not the last role
    if locked_roles.len() <= 1 {
        bail!("Cannot unassign: user '{id}' must have at least one role.");
    }

    sqlx::query("DELETE FROM users_roles WHERE user_id = $1 AND role_slug = $2")
        .bind(id)
        .bind(role)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(format!("Role '{role}' unassigned from user '{id}'."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_and_get_user(pool: PgPool) {
        // Need a role first
        sqlx::query(
            "INSERT INTO roles (slug, name, permissions) VALUES ('admin', 'Admin', '{}'::jsonb)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let msg = create(&pool, "Alice", "alice@example.com", "admin")
            .await
            .unwrap();
        assert!(msg.contains("User created:"));
        assert!(msg.contains("Alice"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_user_unknown_role(pool: PgPool) {
        let err = create(&pool, "Bob", "bob@example.com", "nonexistent").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("does not exist"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_user(pool: PgPool) {
        sqlx::query(
            "INSERT INTO roles (slug, name, permissions) VALUES ('admin', 'Admin', '{}'::jsonb)",
        )
        .execute(&pool)
        .await
        .unwrap();
        create(&pool, "Alice", "alice@example.com", "admin")
            .await
            .unwrap();

        // Get the user id
        let id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = 'alice@example.com'")
            .fetch_one(&pool)
            .await
            .unwrap();

        let msg = delete(&pool, id).await.unwrap();
        assert!(msg.contains("deleted"));

        let err = delete(&pool, id).await;
        assert!(err.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_user(pool: PgPool) {
        sqlx::query(
            "INSERT INTO roles (slug, name, permissions) VALUES ('admin', 'Admin', '{}'::jsonb)",
        )
        .execute(&pool)
        .await
        .unwrap();
        create(&pool, "Alice", "alice@example.com", "admin")
            .await
            .unwrap();
        let id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = 'alice@example.com'")
            .fetch_one(&pool)
            .await
            .unwrap();

        let msg = update(&pool, id, Some("Alicia"), None).await.unwrap();
        assert!(msg.contains("updated"));

        let err = update(&pool, id, None, None).await;
        assert!(err.is_err());

        // Non-existent user should return an error
        let missing = Uuid::new_v4();
        let err = update(&pool, missing, Some("Ghost"), None).await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("not found"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_users(pool: PgPool) {
        let msg = list(&pool).await.unwrap();
        assert_eq!(msg, "No users found.");

        sqlx::query(
            "INSERT INTO roles (slug, name, permissions) VALUES ('admin', 'Admin', '{}'::jsonb)",
        )
        .execute(&pool)
        .await
        .unwrap();
        create(&pool, "Alice", "alice@example.com", "admin")
            .await
            .unwrap();

        let msg = list(&pool).await.unwrap();
        assert!(msg.contains("Alice"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_user_not_found(pool: PgPool) {
        let id = Uuid::new_v4();
        let err = get(&pool, id).await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("not found"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_assign_and_unassign_role(pool: PgPool) {
        sqlx::query(
            "INSERT INTO roles (slug, name, permissions) VALUES ('admin', 'Admin', '{}'::jsonb)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO roles (slug, name, permissions) VALUES ('editor', 'Editor', '{}'::jsonb)",
        )
        .execute(&pool)
        .await
        .unwrap();
        create(&pool, "Alice", "alice@example.com", "admin")
            .await
            .unwrap();
        let id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = 'alice@example.com'")
            .fetch_one(&pool)
            .await
            .unwrap();

        let msg = assign_role(&pool, id, "editor").await.unwrap();
        assert!(msg.contains("assigned"));

        // Can unassign one since two roles exist
        let msg = unassign_role(&pool, id, "editor").await.unwrap();
        assert!(msg.contains("unassigned"));

        // Cannot unassign last role
        let err = unassign_role(&pool, id, "admin").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("at least one role"));

        // Unassigning a role the user doesn't have should say "does not have role", not "at least one role"
        let err = unassign_role(&pool, id, "nonexistent").await;
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("does not have role"), "got: {msg}");
    }
}
