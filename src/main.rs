mod models;
mod schema;

use clap::{Parser, Subcommand};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use models::{NewRepository, Repository};
use std::error::Error;

const DB_PATH: &str = "sqlite://repositories.db";

#[derive(Parser)]
#[command(name = "github_issues_rs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync command
    Sync,
    /// Add a new repository
    #[command(name = "add-repo")]
    AddRepo {
        /// GitHub user or organization
        #[arg(short, long)]
        user: String,
        /// Repository name
        #[arg(short, long)]
        name: String,
    },
    /// List all repositories
    Repos,
}

fn establish_connection() -> Result<SqliteConnection, Box<dyn Error>> {
    let conn = SqliteConnection::establish(DB_PATH)
        .map_err(|e| format!("Error connecting to {}: {}", DB_PATH, e))?;
    
    // Create table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY,
            user TEXT NOT NULL,
            name TEXT NOT NULL,
            UNIQUE(user, name)
        )",
    )
    .execute(&mut SqliteConnection::establish(DB_PATH)?)
    .map_err(|e| format!("Error creating table: {}", e))?;
    
    Ok(conn)
}

fn insert_repository(user: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection()?;
    let new_repo = NewRepository {
        user: user.to_string(),
        name: name.to_string(),
    };
    
    diesel::insert_into(schema::repositories::table)
        .values(&new_repo)
        .execute(&mut conn)
        .map_err(|e| format!("Error inserting repository: {}", e))?;
    
    println!("Repository '{}/{}' added successfully", user, name);
    Ok(())
}

fn list_repositories() -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection()?;
    
    let repos: Vec<Repository> = schema::repositories::table
        .order_by(schema::repositories::user.asc())
        .then_order_by(schema::repositories::name.asc())
        .load::<Repository>(&mut conn)
        .map_err(|e| format!("Error loading repositories: {}", e))?;
    
    for repo in repos {
        println!("{}/{}", repo.user, repo.name);
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync => {
            println!("hello sync");
        }
        Commands::AddRepo { user, name } => {
            if let Err(e) = insert_repository(&user, &name) {
                eprintln!("Error adding repository: {}", e);
            }
        }
        Commands::Repos => {
            if let Err(e) = list_repositories() {
                eprintln!("Error listing repositories: {}", e);
            }
        }
    }
}
