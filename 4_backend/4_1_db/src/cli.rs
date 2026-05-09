use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "step_4_1", about = "PostgreSQL CRUD CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage users
    #[command(subcommand)]
    User(UserCommands),
    /// Manage roles
    #[command(subcommand)]
    Role(RoleCommands),
}

#[derive(Subcommand)]
pub enum UserCommands {
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
pub enum RoleCommands {
    /// Create a new role
    Create {
        slug: String,
        #[arg(short, long)]
        name: String,
        /// Permissions as JSON array, e.g. '["read","write"]'
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
