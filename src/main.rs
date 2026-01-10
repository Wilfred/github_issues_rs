use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result as SqlResult};

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

fn init_db() -> SqlResult<Connection> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute_batch("DROP TABLE IF EXISTS repositories")?;
    conn.execute(
        "CREATE TABLE repositories (
            id INTEGER PRIMARY KEY,
            user TEXT NOT NULL,
            name TEXT NOT NULL,
            UNIQUE(user, name)
        )",
        [],
    )?;
    Ok(conn)
}

fn insert_repository(user: &str, name: &str) -> SqlResult<()> {
    let conn = init_db()?;
    conn.execute(
        "INSERT INTO repositories (user, name) VALUES (?1, ?2)",
        [user, name],
    )?;
    println!("Repository '{}/{}' added successfully", user, name);
    Ok(())
}

fn list_repositories() -> SqlResult<()> {
    let conn = init_db()?;
    let mut stmt = conn.prepare("SELECT user, name FROM repositories ORDER BY user, name")?;
    let repos = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for repo in repos {
        let (user, name) = repo?;
        println!("{}/{}", user, name);
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
