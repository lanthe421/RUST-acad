use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use dotenvy::dotenv;
use garde::Validate;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;
use step_4_3::{
    AssignRoleRequest, CreateRoleRequest, CreateUserRequest, DbError, ErrorResponse, Role,
    UpdateRoleRequest, UpdateUserRequest, UserWithRoles, db,
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

// ── OpenAPI doc ───────────────────────────────────────────────────────────────

/// Adds API key security scheme to OpenAPI documentation.
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_users, create_user, get_user, update_user, delete_user,
        assign_role_to_user, unassign_role_from_user,
        list_roles, create_role, get_role, update_role, delete_role,
    ),
    components(schemas(
        step_4_3::User,
        step_4_3::Role,
        step_4_3::UserWithRoles,
        step_4_3::CreateUserRequest,
        step_4_3::UpdateUserRequest,
        step_4_3::AssignRoleRequest,
        step_4_3::CreateRoleRequest,
        step_4_3::UpdateRoleRequest,
        step_4_3::ErrorResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "users", description = "User management"),
        (name = "roles", description = "Role management"),
    ),
    info(
        title = "Users & Roles REST API",
        version = "1.0.0",
        description = "RESTful API for managing users and roles. Rework of step 4.2 in thick-client paradigm."
    )
)]
struct ApiDoc;

// ── App builder ───────────────────────────────────────────────────────────────

/// Builds Axum router with all endpoints and Swagger UI.
fn build_app(pool: PgPool) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Users
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route("/users/{id}/roles", post(assign_role_to_user))
        .route("/users/{id}/roles/{role}", delete(unassign_role_from_user))
        // Roles
        .route("/roles", get(list_roles).post(create_role))
        .route(
            "/roles/{slug}",
            get(get_role).put(update_role).delete(delete_role),
        )
        .with_state(pool)
}

// ── Error mapping ─────────────────────────────────────────────────────────────

type ApiError = (StatusCode, Json<ErrorResponse>);
type ApiResult<T> = Result<(StatusCode, Json<T>), ApiError>;

/// Converts domain error to HTTP response with appropriate status code.
fn db_err(e: DbError) -> ApiError {
    let msg = e.to_string();
    let status = match e {
        DbError::NotFound(_) => StatusCode::NOT_FOUND,
        DbError::Conflict(_) => StatusCode::CONFLICT,
        DbError::Validation(_) => StatusCode::BAD_REQUEST,
        DbError::BusinessRule(_) => StatusCode::UNPROCESSABLE_ENTITY,
        DbError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(ErrorResponse::new(msg)))
}

/// Converts validation error to HTTP 400.
fn bad_request(e: anyhow::Error) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new(e.to_string())),
    )
}

// ── User handlers ─────────────────────────────────────────────────────────────

/// List all users with their roles.
#[utoipa::path(
    get, path = "/users",
    responses(
        (status = 200, description = "List of users", body = Vec<UserWithRoles>),
    ),
    tag = "users"
)]
async fn list_users(State(pool): State<PgPool>) -> ApiResult<Vec<UserWithRoles>> {
    let users = db::users::list(&pool).await.map_err(db_err)?;
    Ok((StatusCode::OK, Json(users)))
}

/// Create a new user (must have at least one role).
#[utoipa::path(
    post, path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserWithRoles),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Role not found", body = ErrorResponse),
        (status = 409, description = "Email already taken", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn create_user(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUserRequest>,
) -> ApiResult<UserWithRoles> {
    body.validate()
        .map_err(|e| bad_request(anyhow::anyhow!(e.to_string())))?;

    let with_roles = db::users::create(&pool, &body.name, &body.email, &body.role)
        .await
        .map_err(db_err)?;

    Ok((StatusCode::CREATED, Json(with_roles)))
}

/// Get a single user by ID with their roles.
#[utoipa::path(
    get, path = "/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserWithRoles),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn get_user(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> ApiResult<UserWithRoles> {
    let user = db::users::get(&pool, id)
        .await
        .map_err(db_err)?
        .ok_or_else(|| db_err(DbError::NotFound(format!("User '{id}'"))))?;
    Ok((StatusCode::OK, Json(user)))
}

/// Update user fields (name and/or email).
#[utoipa::path(
    put, path = "/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserWithRoles),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> ApiResult<UserWithRoles> {
    body.validate()
        .map_err(|e| bad_request(anyhow::anyhow!(e.to_string())))?;
    let user = db::users::update(&pool, id, body.name.as_deref(), body.email.as_deref())
        .await
        .map_err(db_err)?;
    Ok((StatusCode::OK, Json(user)))
}

/// Delete a user by ID.
#[utoipa::path(
    delete, path = "/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    db::users::delete(&pool, id).await.map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Assign a role to a user.
#[utoipa::path(
    post, path = "/users/{id}/roles",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = AssignRoleRequest,
    responses(
        (status = 204, description = "Role assigned"),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "User or role not found", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn assign_role_to_user(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignRoleRequest>,
) -> Result<StatusCode, ApiError> {
    body.validate()
        .map_err(|e| bad_request(anyhow::anyhow!(e.to_string())))?;
    db::users::assign_role(&pool, id, &body.role)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Unassign a role from a user.
#[utoipa::path(
    delete, path = "/users/{id}/roles/{role}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("role" = String, Path, description = "Role slug"),
    ),
    responses(
        (status = 204, description = "Role unassigned"),
        (status = 404, description = "User or role not found", body = ErrorResponse),
        (status = 422, description = "Cannot remove last role", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn unassign_role_from_user(
    State(pool): State<PgPool>,
    Path((id, role)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    db::users::unassign_role(&pool, id, &role)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Role handlers ─────────────────────────────────────────────────────────────

/// List all roles.
#[utoipa::path(
    get, path = "/roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<Role>),
    ),
    tag = "roles"
)]
async fn list_roles(State(pool): State<PgPool>) -> ApiResult<Vec<Role>> {
    let roles = db::roles::list(&pool).await.map_err(db_err)?;
    Ok((StatusCode::OK, Json(roles)))
}

/// Create a new role.
#[utoipa::path(
    post, path = "/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = Role),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Slug already taken", body = ErrorResponse),
    ),
    tag = "roles"
)]
async fn create_role(
    State(pool): State<PgPool>,
    Json(body): Json<CreateRoleRequest>,
) -> ApiResult<Role> {
    body.validate()
        .map_err(|e| bad_request(anyhow::anyhow!(e.to_string())))?;
    let role = db::roles::create(&pool, &body.slug, &body.name, &body.permissions)
        .await
        .map_err(db_err)?;
    Ok((StatusCode::CREATED, Json(role)))
}

/// Get a single role by slug.
#[utoipa::path(
    get, path = "/roles/{slug}",
    params(("slug" = String, Path, description = "Role slug")),
    responses(
        (status = 200, description = "Role found", body = Role),
        (status = 404, description = "Role not found", body = ErrorResponse),
    ),
    tag = "roles"
)]
async fn get_role(State(pool): State<PgPool>, Path(slug): Path<String>) -> ApiResult<Role> {
    let role = db::roles::get(&pool, &slug)
        .await
        .map_err(db_err)?
        .ok_or_else(|| db_err(DbError::NotFound(format!("Role '{slug}'"))))?;
    Ok((StatusCode::OK, Json(role)))
}

/// Update role fields (name and/or permissions).
#[utoipa::path(
    put, path = "/roles/{slug}",
    params(("slug" = String, Path, description = "Role slug")),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = Role),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Role not found", body = ErrorResponse),
    ),
    tag = "roles"
)]
async fn update_role(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
    Json(body): Json<UpdateRoleRequest>,
) -> ApiResult<Role> {
    body.validate()
        .map_err(|e| bad_request(anyhow::anyhow!(e.to_string())))?;
    let role = db::roles::update(
        &pool,
        &slug,
        body.name.as_deref(),
        body.permissions.as_ref(),
    )
    .await
    .map_err(db_err)?;
    Ok((StatusCode::OK, Json(role)))
}

/// Delete a role by slug.
#[utoipa::path(
    delete, path = "/roles/{slug}",
    params(("slug" = String, Path, description = "Role slug")),
    responses(
        (status = 204, description = "Role deleted"),
        (status = 404, description = "Role not found", body = ErrorResponse),
        (status = 422, description = "Users would be left without a role", body = ErrorResponse),
    ),
    tag = "roles"
)]
async fn delete_role(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<StatusCode, ApiError> {
    db::roles::delete(&pool, &slug).await.map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let listener = tokio::net::TcpListener::bind(&server_addr).await?;
    println!("Server listening on http://{server_addr}");
    println!("Swagger UI: http://{server_addr}/swagger-ui");
    println!("OpenAPI JSON: http://{server_addr}/api-docs/openapi.json");

    axum::serve(listener, build_app(pool)).await?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, header};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn post_json(
        pool: PgPool,
        uri: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let app = build_app(pool);
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    async fn get_json(pool: PgPool, uri: &str) -> (StatusCode, serde_json::Value) {
        let app = build_app(pool);
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_and_list_roles(pool: PgPool) {
        let (status, body) = post_json(
            pool.clone(),
            "/roles",
            serde_json::json!({"slug": "admin", "name": "Admin", "permissions": []}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        assert_eq!(body["slug"], "admin");

        let (status, body) = get_json(pool, "/roles").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.as_array()
                .unwrap()
                .iter()
                .any(|r| r["slug"] == "admin")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_user_and_get(pool: PgPool) {
        post_json(
            pool.clone(),
            "/roles",
            serde_json::json!({"slug": "admin", "name": "Admin", "permissions": []}),
        )
        .await;

        let (status, body) = post_json(
            pool.clone(),
            "/users",
            serde_json::json!({"name": "Alice", "email": "alice@example.com", "role": "admin"}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        let id = body["id"].as_str().unwrap().to_string();

        let (status, body) = get_json(pool, &format!("/users/{id}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["name"], "Alice");
        assert!(
            body["roles"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("admin"))
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_user_unknown_role(pool: PgPool) {
        let (status, _) = post_json(
            pool,
            "/users",
            serde_json::json!({"name": "Bob", "email": "bob@example.com", "role": "nonexistent"}),
        )
        .await;
        // NotFound role → 404
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_user_not_found(pool: PgPool) {
        let id = Uuid::new_v4();
        let (status, _) = get_json(pool, &format!("/users/{id}")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_role_invalid_slug(pool: PgPool) {
        let (status, body) = post_json(
            pool,
            "/roles",
            serde_json::json!({"slug": "INVALID SLUG!", "name": "Bad", "permissions": []}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }
}
