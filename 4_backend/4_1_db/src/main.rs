mod cli;
mod db;
mod models;

use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

use cli::{Cli, Commands, RoleCommands, UserCommands};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://user:pass@localhost/dbname)");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations automatically on startup
    sqlx::migrate!("./migrations").run(&pool).await?;

    let cli = Cli::parse();

    match cli.command {
        Commands::User(cmd) => match cmd {
            UserCommands::Create { name, email, role } => {
                db::users::create(&pool, &name, &email, &role).await?;
            }
            UserCommands::Delete { id } => {
                db::users::delete(&pool, id).await?;
            }
            UserCommands::Update { id, name, email } => {
                db::users::update(&pool, id, name.as_deref(), email.as_deref()).await?;
            }
            UserCommands::List => {
                db::users::list(&pool).await?;
            }
            UserCommands::Get { id } => {
                db::users::get(&pool, id).await?;
            }
            UserCommands::AssignRole { id, role } => {
                db::users::assign_role(&pool, id, &role).await?;
            }
            UserCommands::UnassignRole { id, role } => {
                db::users::unassign_role(&pool, id, &role).await?;
            }
        },
        Commands::Role(cmd) => match cmd {
            RoleCommands::Create { slug, name, permissions } => {
                db::roles::create(&pool, &slug, &name, &permissions).await?;
            }
            RoleCommands::Delete { slug } => {
                db::roles::delete(&pool, &slug).await?;
            }
            RoleCommands::Update { slug, name, permissions } => {
                db::roles::update(&pool, &slug, name.as_deref(), permissions.as_deref()).await?;
            }
            RoleCommands::List => {
                db::roles::list(&pool).await?;
            }
            RoleCommands::Get { slug } => {
                db::roles::get(&pool, &slug).await?;
            }
        },
    }

    Ok(())
}
