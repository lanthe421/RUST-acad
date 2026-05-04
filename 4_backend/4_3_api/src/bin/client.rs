/// Thick CLI client for the REST API.
/// Parses commands itself and makes precise HTTP requests to corresponding endpoints.
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
use std::{env, process};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "client", about = "REST API client for the step_4_3 server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage users
    #[command(subcommand)]
    User(UserCommands),
    /// Manage roles
    #[command(subcommand)]
    Role(RoleCommands),
}

#[derive(Subcommand)]
enum UserCommands {
    /// Create a new user (must have at least one role)
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        email: String,
        /// Initial role slug
        #[arg(short, long)]
        role: String,
    },
    /// Delete a user by ID
    Delete { id: Uuid },
    /// Update user fields
    Update {
        id: Uuid,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        email: Option<String>,
    },
    /// List all users with their roles
    List,
    /// Get a single user by ID with roles
    Get { id: Uuid },
    /// Assign a role to a user
    AssignRole { id: Uuid, role: String },
    /// Unassign a role from a user
    UnassignRole { id: Uuid, role: String },
}

#[derive(Subcommand)]
enum RoleCommands {
    /// Create a new role
    Create {
        slug: String,
        #[arg(short, long)]
        name: String,
        /// Permissions as comma-separated list (e.g., "read,write,edit") or empty for no permissions
        #[arg(short, long, default_value = "")]
        permissions: String,
    },
    /// Delete a role by slug
    Delete { slug: String },
    /// Update role fields
    Update {
        slug: String,
        #[arg(short, long)]
        name: Option<String>,
        /// Permissions as comma-separated list (e.g., "read,write,edit")
        #[arg(short, long)]
        permissions: Option<String>,
    },
    /// List all roles
    List,
    /// Get a single role by slug
    Get { slug: String },
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

/// Performs a GET request.
async fn get(client: &Client, url: &str) -> reqwest::Result<reqwest::Response> {
    client.get(url).send().await
}

/// Performs a POST request with JSON body.
async fn post(client: &Client, url: &str, body: Value) -> reqwest::Result<reqwest::Response> {
    client.post(url).json(&body).send().await
}

/// Performs a DELETE request.
async fn delete(client: &Client, url: &str) -> reqwest::Result<reqwest::Response> {
    client.delete(url).send().await
}

type BoxError = anyhow::Error;

/// Prints response body as pretty JSON. Returns error on HTTP error.
async fn print_response(resp: reqwest::Response) -> Result<(), BoxError> {
    let status = resp.status();
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    if status.is_success() {
        println!("{}", serde_json::to_string_pretty(&body).unwrap());
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Error {status}: {}",
            serde_json::to_string_pretty(&body).unwrap()
        ))
    }
}

/// Prints response status. Returns error on HTTP error.
async fn print_status(resp: reqwest::Response) -> Result<(), BoxError> {
    let status = resp.status();
    if status.is_success() {
        println!("OK ({status})");
        Ok(())
    } else {
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        Err(anyhow::anyhow!(
            "Error {status}: {}",
            serde_json::to_string_pretty(&body).unwrap()
        ))
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv().ok();

    let server_url = env::var("SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let base = server_url.trim_end_matches('/');

    let cli = Cli::parse();
    let client = Client::new();

    let result: anyhow::Result<()> = async {
        match cli.command {
            Commands::User(cmd) => match cmd {
                UserCommands::Create { name, email, role } => {
                    let resp = post(
                        &client,
                        &format!("{base}/users"),
                        serde_json::json!({"name": name, "email": email, "role": role}),
                    )
                    .await?;
                    print_response(resp).await?;
                }
                UserCommands::Delete { id } => {
                    let resp = delete(&client, &format!("{base}/users/{id}")).await?;
                    print_status(resp).await?;
                }
                UserCommands::Update { id, name, email } => {
                    let body = step_4_3::UpdateUserRequest { name, email };
                    let resp = client
                        .put(&format!("{base}/users/{id}"))
                        .json(&body)
                        .send()
                        .await?;
                    print_response(resp).await?;
                }
                UserCommands::List => {
                    let resp = get(&client, &format!("{base}/users")).await?;
                    print_response(resp).await?;
                }
                UserCommands::Get { id } => {
                    let resp = get(&client, &format!("{base}/users/{id}")).await?;
                    print_response(resp).await?;
                }
                UserCommands::AssignRole { id, role } => {
                    let resp = post(
                        &client,
                        &format!("{base}/users/{id}/roles"),
                        serde_json::json!({"role": role}),
                    )
                    .await?;
                    print_status(resp).await?;
                }
                UserCommands::UnassignRole { id, role } => {
                    let resp = delete(&client, &format!("{base}/users/{id}/roles/{role}")).await?;
                    print_status(resp).await?;
                }
            },
            Commands::Role(cmd) => match cmd {
                RoleCommands::Create {
                    slug,
                    name,
                    permissions,
                } => {
                    let perms: Vec<String> = if permissions.is_empty() {
                        Vec::new()
                    } else {
                        permissions
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect()
                    };

                    let resp = post(
                        &client,
                        &format!("{base}/roles"),
                        serde_json::json!({"slug": slug, "name": name, "permissions": perms}),
                    )
                    .await?;
                    print_response(resp).await?;
                }
                RoleCommands::Delete { slug } => {
                    let resp = delete(&client, &format!("{base}/roles/{slug}")).await?;
                    print_status(resp).await?;
                }
                RoleCommands::Update {
                    slug,
                    name,
                    permissions,
                } => {
                    let perms_value = permissions.as_ref().map(|p| {
                        let perms: Vec<String> = if p.is_empty() {
                            Vec::new()
                        } else {
                            p.split(',').map(|s| s.trim().to_string()).collect()
                        };
                        Value::Array(perms.into_iter().map(Value::String).collect())
                    });

                    let mut body = serde_json::Map::new();
                    if let Some(n) = name {
                        body.insert("name".to_string(), Value::String(n));
                    }
                    if let Some(p) = perms_value {
                        body.insert("permissions".to_string(), p);
                    }

                    let resp = client
                        .put(&format!("{base}/roles/{slug}"))
                        .json(&Value::Object(body))
                        .send()
                        .await?;
                    print_response(resp).await?;
                }
                RoleCommands::List => {
                    let resp = get(&client, &format!("{base}/roles")).await?;
                    print_response(resp).await?;
                }
                RoleCommands::Get { slug } => {
                    let resp = get(&client, &format!("{base}/roles/{slug}")).await?;
                    print_response(resp).await?;
                }
            },
        }
        Ok(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("Connection error: {e}");
        process::exit(1);
    }
}
