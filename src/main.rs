mod models;
mod schema;

use clap::{Parser, Subcommand};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::upsert::excluded;
use models::{NewRepository, Repository, Issue, NewIssue};
use std::error::Error;
use serde::{Deserialize};
use prettytable::{Table, row};
use colored::Colorize;

const DB_PATH: &str = "sqlite://repositories.db";

#[derive(Deserialize)]
struct GitHubIssue {
    number: i32,
    title: String,
    body: Option<String>,
    created_at: String,
    state: String,
}

#[derive(Parser)]
#[command(name = "github_issues_rs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync issues from all repositories in the database
    Sync,
    /// Repository management
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    /// List all issues
    Issue,
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Add a new repository
    Add {
        /// Repository in format username/projectname
        repo: String,
    },
    /// List all repositories
    List,
}

fn establish_connection() -> Result<SqliteConnection, Box<dyn Error>> {
    let conn = SqliteConnection::establish(DB_PATH)
        .map_err(|e| format!("Error connecting to {}: {}", DB_PATH, e))?;
    
    // Create repositories table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY,
            user TEXT NOT NULL,
            name TEXT NOT NULL,
            UNIQUE(user, name)
        )",
    )
    .execute(&mut SqliteConnection::establish(DB_PATH)?)
    .map_err(|e| format!("Error creating repositories table: {}", e))?;
    
    // Create issues table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS issues (
            id INTEGER PRIMARY KEY,
            number INTEGER NOT NULL UNIQUE,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL,
            state TEXT NOT NULL
        )",
    )
    .execute(&mut SqliteConnection::establish(DB_PATH)?)
    .map_err(|e| format!("Error creating issues table: {}", e))?;
    
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
    
    println!("{}", format!("Repository '{}/{}' added successfully.", user, name).green());
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

fn list_issues() -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection()?;
    
    let issues: Vec<Issue> = schema::issues::table
        .order_by(schema::issues::number.desc())
        .load::<Issue>(&mut conn)
        .map_err(|e| format!("Error loading issues: {}", e))?;
    
    let mut table = Table::new();
    table.add_row(row!["#", "Title", "State", "Created"]);
    
    for issue in issues {
        table.add_row(row![issue.number, issue.title, issue.state, issue.created_at]);
    }
    
    table.printstd();
    Ok(())
}

async fn sync_issues_for_repo(user: &str, repo: &str, token: &str) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues?per_page=100",
        user, repo
    );
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "github_issues_rs")
        .send()
        .await?;
    
    let body = response.text().await?;
    let github_issues: Vec<GitHubIssue> = serde_json::from_str(&body)
        .map_err(|e| format!("Error decoding response: {}. Response body: {}", e, body))?;
    
    let mut conn = establish_connection()?;
    let mut count = 0;
    
    for gh_issue in github_issues {
        let new_issue = NewIssue {
            number: gh_issue.number,
            title: gh_issue.title,
            body: gh_issue.body.unwrap_or_default(),
            created_at: gh_issue.created_at,
            state: gh_issue.state,
        };
        
        diesel::insert_into(schema::issues::table)
            .values(&new_issue)
            .on_conflict(schema::issues::number)
            .do_update()
            .set((
                schema::issues::title.eq(excluded(schema::issues::title)),
                schema::issues::body.eq(excluded(schema::issues::body)),
                schema::issues::state.eq(excluded(schema::issues::state)),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Error syncing issue: {}", e))?;
        count += 1;
    }
    
    println!("{}", format!("Successfully synced {} issues from {}/{}.", count, user, repo).green());
    Ok(())
}

#[tokio::main]
async fn sync_all_repos() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN not found in .env file")?;
    
    let mut conn = establish_connection()?;
    
    let repos: Vec<Repository> = schema::repositories::table
        .load::<Repository>(&mut conn)
        .map_err(|e| format!("Error loading repositories: {}", e))?;
    
    if repos.is_empty() {
        println!("{}", "No repositories to sync. Add repositories with: cargo run -- repo add username/projectname.".yellow());
        return Ok(());
    }
    
    for repo in repos {
        if let Err(e) = sync_issues_for_repo(&repo.user, &repo.name, &token).await {
            eprintln!("Error syncing {}/{}: {}", repo.user, repo.name, e);
        }
    }
    
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync => {
            if let Err(e) = sync_all_repos() {
                eprintln!("{}", format!("Error syncing issues: {}", e).red());
            }
        }
        Commands::Repo { command } => match command {
            RepoCommands::Add { repo } => {
                let parts: Vec<&str> = repo.split('/').collect();
                if parts.len() != 2 {
                    eprintln!("{}", "Error: Repository must be in format username/projectname.".red());
                } else {
                    if let Err(e) = insert_repository(parts[0], parts[1]) {
                        eprintln!("{}", format!("Error adding repository: {}", e).red());
                    }
                }
            }
            RepoCommands::List => {
                if let Err(e) = list_repositories() {
                    eprintln!("{}", format!("Error listing repositories: {}", e).red());
                }
            }
        },
        Commands::Issue => {
            if let Err(e) = list_issues() {
                eprintln!("{}", format!("Error listing issues: {}", e).red());
            }
        }
    }
}
