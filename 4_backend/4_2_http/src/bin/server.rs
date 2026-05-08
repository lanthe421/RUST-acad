use anyhow::Result;
use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    routing::post,
};
use dotenvy::dotenv;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;
use step_4_2::{validate, Command, CommandResponse, db};

// ── Router builder ────────────────────────────────────────────────────────────

/// Builds the axum application with the given pool.
/// Extracted so tests can reuse it without starting a real TCP listener.
fn build_app(pool: PgPool) -> Router {
    Router::new()
        .route("/command", post(handle_command))
        .with_state(pool)
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// Dispatches a `Command` to the appropriate DB function and returns a JSON response.
async fn handle_command(
    State(pool): State<PgPool>,
    cmd_result: Result<Json<Command>, JsonRejection>,
) -> Json<CommandResponse> {
    // malformed/invalid JSON returns CommandResponse::Error
    let cmd = match cmd_result {
        Ok(Json(cmd)) => cmd,
        Err(e) => {
            return Json(CommandResponse::Error {
                message: format!("Invalid command: {e}"),
            });
        }
    };

    let result = dispatch(&pool, cmd).await;
    let response = match result {
        Ok(message) => CommandResponse::Ok { message },
        Err(e) => CommandResponse::Error {
            message: e.to_string(),
        },
    };
    Json(response)
}

/// Validates and dispatches the command to the correct DB function.
async fn dispatch(pool: &PgPool, cmd: Command) -> Result<String> {
    match cmd {
        Command::UserCreate { name, email, role } => {
            validate::non_empty("name", &name)?;
            validate::email(&email)?;
            validate::slug(&role)?;
            db::users::create(pool, &name, &email, &role).await
        }
        Command::UserDelete { id } => db::users::delete(pool, id).await,
        Command::UserUpdate { id, name, email } => {
            if let Some(ref n) = name {
                validate::non_empty("name", n)?;
            }
            if let Some(ref e) = email {
                validate::email(e)?;
            }
            db::users::update(pool, id, name.as_deref(), email.as_deref()).await
        }
        Command::UserList => db::users::list(pool).await,
        Command::UserGet { id } => db::users::get(pool, id).await,
        Command::UserAssignRole { id, role } => {
            validate::slug(&role)?;
            db::users::assign_role(pool, id, &role).await
        }
        Command::UserUnassignRole { id, role } => {
            validate::slug(&role)?;
            db::users::unassign_role(pool, id, &role).await
        }
        Command::RoleCreate { slug, name, permissions } => {
            validate::slug(&slug)?;
            validate::non_empty("name", &name)?;
            db::roles::create(pool, &slug, &name, &permissions).await
        }
        Command::RoleDelete { slug } => {
            validate::slug(&slug)?;
            db::roles::delete(pool, &slug).await
        }
        Command::RoleUpdate { slug, name, permissions } => {
            validate::slug(&slug)?;
            if let Some(ref n) = name {
                validate::non_empty("name", n)?;
            }
            db::roles::update(pool, &slug, name.as_deref(), permissions.as_deref()).await
        }
        Command::RoleList => db::roles::list(pool).await,
        Command::RoleGet { slug } => {
            validate::slug(&slug)?;
            db::roles::get(pool, &slug).await
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // read DATABASE_URL from env
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // read SERVER_ADDR from env, default 127.0.0.1:8080
    let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // run migrations on startup
    sqlx::migrate!("./migrations").run(&pool).await?;

    let listener = tokio::net::TcpListener::bind(&server_addr).await?;
    println!("Server listening on {server_addr}");
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
    use proptest::prelude::*;
    use tower::ServiceExt;

    /// Strategy that generates strings that are NOT valid Command JSON.
    fn invalid_command_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("".to_string()),
            Just(r#"{"foo":"bar"}"#.to_string()),
            Just(r#"{"type":"Unknown","payload":{}}"#.to_string()),
            Just(r#"null"#.to_string()),
            Just(r#"42"#.to_string()),
            Just(r#"[]"#.to_string()),
            "[a-zA-Z0-9 ]{1,50}".prop_map(|s| s),
            "[!-~]{1,80}".prop_map(|s| s),
        ]
    }

    /// Send a raw body to POST /command and return the parsed CommandResponse.
    async fn post_raw(pool: PgPool, body: String) -> CommandResponse {
        let app = build_app(pool);
        let req = Request::builder()
            .method(Method::POST)
            .uri("/command")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).expect("server must always return valid JSON")
    }

    // Feature: http-client-server, Property 2: Invalid command returns JSON error
    #[sqlx::test(migrations = "./migrations")]
    async fn prop_invalid_command_returns_json_error(pool: PgPool) {
        let handle = tokio::runtime::Handle::current();
        let pool_clone = pool.clone();

        // Run proptest on a blocking thread so it can call block_on per iteration
        tokio::task::spawn_blocking(move || {
            let config = ProptestConfig::with_cases(100);
            let mut runner = proptest::test_runner::TestRunner::new(config);
            runner
                .run(&invalid_command_strategy(), |body| {
                    let response = handle.block_on(post_raw(pool_clone.clone(), body.clone()));
                    prop_assert!(
                        matches!(&response, CommandResponse::Error { message } if !message.is_empty()),
                        "Expected CommandResponse::Error for body: {body:?}, got: {response:?}"
                    );
                    Ok(())
                })
                .unwrap();
        })
        .await
        .unwrap();
    }
}
