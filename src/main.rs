use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

const DB_PATH: &str = "repositories.db";

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
        /// Repository name
        #[arg(short, long)]
        name: String,
        /// Repository URL
        #[arg(short, long)]
        url: String,
    },
    /// List all repositories
    Repos,
}

fn init_db() -> SqlResult<Connection> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            url TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

fn insert_repository(name: &str, url: &str) -> SqlResult<()> {
    let conn = init_db()?;
    conn.execute(
        "INSERT INTO repositories (name, url) VALUES (?1, ?2)",
        [name, url],
    )?;
    println!("Repository '{}' added successfully", name);
    Ok(())
}

fn list_repositories() -> SqlResult<()> {
    let conn = init_db()?;
    let mut stmt = conn.prepare("SELECT name, url FROM repositories ORDER BY name")?;
    let repos = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for repo in repos {
        let (name, url) = repo?;
        println!("{}: {}", name, url);
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync => {
            println!("hello sync");
        }
        Commands::AddRepo { name, url } => {
            if let Err(e) = insert_repository(&name, &url) {
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
