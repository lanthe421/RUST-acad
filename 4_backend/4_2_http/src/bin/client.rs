use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use std::process;
use step_4_2::{Command, CommandResponse};
use uuid::Uuid;

// ── CLI definitions (Requirements 2.1) ───────────────────────────────────────

#[derive(Parser)]
#[command(name = "client", about = "HTTP client for the step_4_2 server")]
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
        /// Initial role slug to assign
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
        /// Permissions as JSON array
        #[arg(short, long, default_value = "[]")]
        permissions: String,
    },
    /// Delete a role by slug
    Delete { slug: String },
    /// Update role fields
    Update {
        slug: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        permissions: Option<String>,
    },
    /// List all roles
    List,
    /// Get a single role by slug
    Get { slug: String },
}

// ── Conversion: CLI args → Command ────────────────────────

fn to_command(cli: Cli) -> Command {
    match cli.command {
        Commands::User(cmd) => match cmd {
            UserCommands::Create { name, email, role } => Command::UserCreate { name, email, role },
            UserCommands::Delete { id } => Command::UserDelete { id },
            UserCommands::Update { id, name, email } => Command::UserUpdate { id, name, email },
            UserCommands::List => Command::UserList,
            UserCommands::Get { id } => Command::UserGet { id },
            UserCommands::AssignRole { id, role } => Command::UserAssignRole { id, role },
            UserCommands::UnassignRole { id, role } => Command::UserUnassignRole { id, role },
        },
        Commands::Role(cmd) => match cmd {
            RoleCommands::Create {
                slug,
                name,
                permissions,
            } => Command::RoleCreate {
                slug,
                name,
                permissions,
            },
            RoleCommands::Delete { slug } => Command::RoleDelete { slug },
            RoleCommands::Update {
                slug,
                name,
                permissions,
            } => Command::RoleUpdate {
                slug,
                name,
                permissions,
            },
            RoleCommands::List => Command::RoleList,
            RoleCommands::Get { slug } => Command::RoleGet { slug },
        },
    }
}

// ── HTTP send ────────────────────────────────────────

async fn send_command(client: &reqwest::Client, server_url: &str, cmd: &Command) -> Result<CommandResponse, reqwest::Error> {
    let url = format!("{}/command", server_url.trim_end_matches('/'));
    let response = client.post(&url).json(cmd).send().await?;
    let cr = response.json::<CommandResponse>().await?;
    Ok(cr)
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv().ok();

    // read SERVER_URL from env, default http://127.0.0.1:8080
    let server_url = env::var("SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let cli = Cli::parse();
    let cmd = to_command(cli);
    let client = reqwest::Client::new();

    // send command to server
    match send_command(&client, &server_url, &cmd).await {
        Ok(CommandResponse::Ok { message }) => {
            // print success to stdout
            println!("{message}");
        }
        Ok(CommandResponse::Error { message }) => {
            // print error to stderr + exit 1
            eprintln!("Error: {message}");
            process::exit(1);
        }
        Err(e) => {
            // connection error → stderr + exit 1
            eprintln!("Connection error: {e}");
            process::exit(1);
        }
    }
}

// ── Unit tests for CLI parsing ───────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn parse(args: &[&str]) -> Command {
        let cli = Cli::try_parse_from(args).expect("parse failed");
        to_command(cli)
    }

    // ── User subcommands ──────────────────────────────────────────────────────

    #[test]
    fn user_create() {
        let cmd = parse(&[
            "client", "user", "create", "-n", "Alice", "-e", "a@b.com", "-r", "admin",
        ]);
        assert_eq!(
            cmd,
            Command::UserCreate {
                name: "Alice".into(),
                email: "a@b.com".into(),
                role: "admin".into(),
            }
        );
    }

    #[test]
    fn user_delete() {
        let id = Uuid::new_v4();
        let cmd = parse(&["client", "user", "delete", &id.to_string()]);
        assert_eq!(cmd, Command::UserDelete { id });
    }

    #[test]
    fn user_update_both_fields() {
        let id = Uuid::new_v4();
        let cmd = parse(&[
            "client",
            "user",
            "update",
            &id.to_string(),
            "-n",
            "Bob",
            "-e",
            "bob@example.com",
        ]);
        assert_eq!(
            cmd,
            Command::UserUpdate {
                id,
                name: Some("Bob".into()),
                email: Some("bob@example.com".into()),
            }
        );
    }

    #[test]
    fn user_update_name_only() {
        let id = Uuid::new_v4();
        let cmd = parse(&["client", "user", "update", &id.to_string(), "-n", "Carol"]);
        assert_eq!(
            cmd,
            Command::UserUpdate {
                id,
                name: Some("Carol".into()),
                email: None
            }
        );
    }

    #[test]
    fn user_list() {
        let cmd = parse(&["client", "user", "list"]);
        assert_eq!(cmd, Command::UserList);
    }

    #[test]
    fn user_get() {
        let id = Uuid::new_v4();
        let cmd = parse(&["client", "user", "get", &id.to_string()]);
        assert_eq!(cmd, Command::UserGet { id });
    }

    #[test]
    fn user_assign_role() {
        let id = Uuid::new_v4();
        let cmd = parse(&["client", "user", "assign-role", &id.to_string(), "editor"]);
        assert_eq!(
            cmd,
            Command::UserAssignRole {
                id,
                role: "editor".into()
            }
        );
    }

    #[test]
    fn user_unassign_role() {
        let id = Uuid::new_v4();
        let cmd = parse(&["client", "user", "unassign-role", &id.to_string(), "editor"]);
        assert_eq!(
            cmd,
            Command::UserUnassignRole {
                id,
                role: "editor".into()
            }
        );
    }

    // ── Role subcommands ──────────────────────────────────────────────────────

    #[test]
    fn role_create_with_permissions() {
        let cmd = parse(&[
            "client",
            "role",
            "create",
            "moderator",
            "-n",
            "Moderator",
            "-p",
            r#"["read","write"]"#,
        ]);
        assert_eq!(
            cmd,
            Command::RoleCreate {
                slug: "moderator".into(),
                name: "Moderator".into(),
                permissions: r#"["read","write"]"#.into(),
            }
        );
    }

    #[test]
    fn role_create_default_permissions() {
        let cmd = parse(&["client", "role", "create", "viewer", "-n", "Viewer"]);
        assert_eq!(
            cmd,
            Command::RoleCreate {
                slug: "viewer".into(),
                name: "Viewer".into(),
                permissions: "[]".into(),
            }
        );
    }

    #[test]
    fn role_delete() {
        let cmd = parse(&["client", "role", "delete", "viewer"]);
        assert_eq!(
            cmd,
            Command::RoleDelete {
                slug: "viewer".into()
            }
        );
    }

    #[test]
    fn role_update() {
        let cmd = parse(&["client", "role", "update", "viewer", "-n", "Read-only"]);
        assert_eq!(
            cmd,
            Command::RoleUpdate {
                slug: "viewer".into(),
                name: Some("Read-only".into()),
                permissions: None,
            }
        );
    }

    #[test]
    fn role_list() {
        let cmd = parse(&["client", "role", "list"]);
        assert_eq!(cmd, Command::RoleList);
    }

    #[test]
    fn role_get() {
        let cmd = parse(&["client", "role", "get", "admin"]);
        assert_eq!(
            cmd,
            Command::RoleGet {
                slug: "admin".into()
            }
        );
    }
}
