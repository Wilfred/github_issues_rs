mod models;
mod schema;

use clap::{Parser, Subcommand, ValueEnum};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::upsert::excluded;
use models::{
    Issue, IssueLabel, IssueReaction, Label, NewIssue, NewLabel, NewRepository, Repository,
};
use serde::Deserialize;
use std::error::Error;

use colored::Colorize;
use pager::Pager;
use termimad::MadSkin;
use terminal_link::Link;

fn get_db_path() -> Result<String, Box<dyn Error>> {
    let data_dir = dirs::data_dir().ok_or("Unable to determine data directory")?;
    let app_dir = data_dir.join("gh-offline");

    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("repositories.db");
    Ok(format!("sqlite://{}", db_path.display()))
}

#[derive(ValueEnum, Clone, Debug)]
enum StateFilter {
    /// Show open issues
    Open,
    /// Show closed issues
    Closed,
    /// Show all issues
    All,
}

impl StateFilter {
    fn as_str(&self) -> &str {
        match self {
            StateFilter::Open => "open",
            StateFilter::Closed => "closed",
            StateFilter::All => "all",
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
enum TypeFilter {
    /// Show issues only
    Issue,
    /// Show pull requests only
    Pr,
    /// Show both issues and pull requests
    All,
}

#[derive(Deserialize)]
struct GitHubLabel {
    name: String,
}

#[derive(Deserialize)]
struct GitHubReactions {
    #[serde(rename = "+1")]
    plus_one: Option<i32>,
    #[serde(rename = "-1")]
    minus_one: Option<i32>,
    laugh: Option<i32>,
    hooray: Option<i32>,
    confused: Option<i32>,
    heart: Option<i32>,
    rocket: Option<i32>,
    eyes: Option<i32>,
}

#[derive(Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Deserialize)]
struct GitHubIssue {
    number: i32,
    title: String,
    body: Option<String>,
    created_at: String,
    state: String,
    pull_request: Option<serde_json::Value>,
    labels: Option<Vec<GitHubLabel>>,
    reactions: Option<GitHubReactions>,
    user: Option<GitHubUser>,
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
    Sync {
        /// Force fetch all issues, ignoring cache
        #[arg(short, long)]
        force: bool,
    },
    /// Repository management
    Repo {
        #[command(subcommand)]
        command: Option<RepoCommands>,
    },
    /// List all issues, or view a specific issue
    Issue {
        /// Optional issue number to view details
        #[arg(value_name = "NUMBER")]
        number: Option<i32>,
        /// Filter by state: all, open, or closed
        #[arg(short, long, default_value = "open")]
        state: StateFilter,
        /// Filter by type: all, issue, or pr
        #[arg(short = 't', long, default_value = "issue")]
        r#type: TypeFilter,
    },
    /// List all pull requests, or view a specific pull request
    Pr {
        /// Optional pull request number to view details
        #[arg(value_name = "NUMBER")]
        number: Option<i32>,
        /// Filter by state: all, open, or closed
        #[arg(short, long, default_value = "open")]
        state: StateFilter,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Add a new repository
    Add {
        /// Repository in format username/projectname
        repo: String,
    },
    /// Remove a repository
    Rm {
        /// Repository in format username/projectname
        repo: String,
    },
}

fn reaction_to_ascii(reaction_type: &str) -> &str {
    match reaction_type {
        "+1" => "[+1]",
        "-1" => "[-1]",
        "laugh" => ":D",
        "hooray" => "^_^",
        "confused" => ":/",
        "heart" => "<3",
        "rocket" => "^^",
        "eyes" => "o_o",
        _ => "?",
    }
}

fn establish_connection() -> Result<SqliteConnection, Box<dyn Error>> {
    let db_path = get_db_path()?;
    let conn = SqliteConnection::establish(&db_path)
        .map_err(|e| format!("Error connecting to {}: {}", db_path, e))?;

    // Create repositories table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY,
            user TEXT NOT NULL,
            name TEXT NOT NULL,
            UNIQUE(user, name)
        )",
    )
    .execute(&mut SqliteConnection::establish(&db_path)?)
    .map_err(|e| format!("Error creating repositories table: {}", e))?;

    // Create issues table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS issues (
            id INTEGER PRIMARY KEY,
            repository_id INTEGER NOT NULL,
            number INTEGER NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL,
            state TEXT NOT NULL,
            is_pull_request BOOLEAN NOT NULL DEFAULT 0,
            author TEXT,
            UNIQUE(repository_id, number)
        )",
    )
    .execute(&mut SqliteConnection::establish(&db_path)?)
    .map_err(|e| format!("Error creating issues table: {}", e))?;

    // Add author column if it doesn't exist
    let _ = diesel::sql_query("ALTER TABLE issues ADD COLUMN author TEXT")
        .execute(&mut SqliteConnection::establish(&db_path)?);

    // Add last_synced_at column if it doesn't exist
    let _ = diesel::sql_query("ALTER TABLE issues ADD COLUMN last_synced_at TEXT")
        .execute(&mut SqliteConnection::establish(&db_path)?);

    // Create labels table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS labels (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )",
    )
    .execute(&mut SqliteConnection::establish(&db_path)?)
    .map_err(|e| format!("Error creating labels table: {}", e))?;

    // Create issue_labels table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS issue_labels (
            id INTEGER PRIMARY KEY,
            issue_id INTEGER NOT NULL,
            label_id INTEGER NOT NULL,
            UNIQUE(issue_id, label_id),
            FOREIGN KEY(issue_id) REFERENCES issues(id),
            FOREIGN KEY(label_id) REFERENCES labels(id)
        )",
    )
    .execute(&mut SqliteConnection::establish(&db_path)?)
    .map_err(|e| format!("Error creating issue_labels table: {}", e))?;

    // Create issue_reactions table if it doesn't exist
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS issue_reactions (
            id INTEGER PRIMARY KEY,
            issue_id INTEGER NOT NULL,
            reaction_type TEXT NOT NULL,
            count INTEGER NOT NULL,
            UNIQUE(issue_id, reaction_type),
            FOREIGN KEY(issue_id) REFERENCES issues(id)
        )",
    )
    .execute(&mut SqliteConnection::establish(&db_path)?)
    .map_err(|e| format!("Error creating issue_reactions table: {}", e))?;

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

    println!(
        "Repository '{}' added successfully.",
        format!("{}/{}", user, name).cyan()
    );
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

fn remove_repository(user: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection()?;
    
    let deleted = diesel::delete(
        schema::repositories::table
            .filter(schema::repositories::user.eq(user))
            .filter(schema::repositories::name.eq(name))
    )
    .execute(&mut conn)
    .map_err(|e| format!("Error deleting repository: {}", e))?;
    
    if deleted == 0 {
        eprintln!("Repository '{}/{}' not found.", user, name);
    } else {
        println!(
            "Repository '{}' removed successfully.",
            format!("{}/{}", user, name).cyan()
        );
    }
    Ok(())
}

fn list_issues(
    issue_number: Option<i32>,
    state_filter: StateFilter,
    type_filter: TypeFilter,
) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection()?;

    // Check if filters are non-default
    let show_type = matches!(type_filter, TypeFilter::Pr | TypeFilter::All);
    let show_state = matches!(state_filter, StateFilter::Closed | StateFilter::All);

    if let Some(number) = issue_number {
        // Display specific issue
        let issue = schema::issues::table
            .filter(schema::issues::number.eq(number))
            .first::<Issue>(&mut conn)
            .map_err(|e| format!("Issue #{} not found: {}", number, e))?;

        // Get repository info
        let repository = schema::repositories::table
            .find(issue.repository_id)
            .first::<Repository>(&mut conn)
            .map_err(|e| format!("Repository not found: {}", e))?;

        // Create hyperlinked title using OSC 8
        let url = format!(
            "https://github.com/{}/{}/issues/{}",
            repository.user, repository.name, issue.number
        );
        let title_display = format!("{}", issue.title.bold());
        let title_link = Link::new(&title_display, &url);

        // Display title and author
        let mut first_line = format!("{}", title_link);

        if let Some(author) = &issue.author {
            let author_url = format!("https://github.com/{}", author);
            let author_link = Link::new(author, &author_url);
            first_line.push_str(&format!(" {}", format!("by {}", author_link).dimmed()));
        }

        // Add state and type badges
        let state_display = if issue.state == "open" {
            issue.state.to_uppercase().green().to_string()
        } else {
            issue.state.to_uppercase().red().to_string()
        };
        first_line.push_str(&format!(" {}", state_display));

        if issue.is_pull_request {
            first_line.push_str(&format!(" {}", "PULL REQUEST".cyan()));
        }

        println!("{}", first_line);

        // Get and display labels immediately after title
        let issue_labels: Vec<(IssueLabel, Label)> = schema::issue_labels::table
            .inner_join(schema::labels::table)
            .filter(schema::issue_labels::issue_id.eq(issue.id))
            .load::<(IssueLabel, Label)>(&mut conn)
            .unwrap_or_default();

        if !issue_labels.is_empty() {
            for (i, (_, label)) in issue_labels.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print!("{}", label.name.cyan());
            }
            println!();
        }

        // Get and display reactions
        let reactions: Vec<IssueReaction> = schema::issue_reactions::table
            .filter(schema::issue_reactions::issue_id.eq(issue.id))
            .order_by(schema::issue_reactions::reaction_type.asc())
            .load::<IssueReaction>(&mut conn)
            .unwrap_or_default();

        if !reactions.is_empty() {
            for (i, reaction) in reactions.iter().enumerate() {
                if i > 0 {
                    print!("\t");
                }
                print!(
                    "{} {}",
                    reaction_to_ascii(&reaction.reaction_type),
                    reaction.count.to_string().cyan()
                );
            }
            println!();
        }

        println!();

        // Render markdown body with termimad
        let skin = MadSkin::default();
        if issue.body.trim().is_empty() {
            println!("{}", "No description provided".dimmed());
        } else {
            skin.print_text(&issue.body);
        }
    } else {
        // Collect issue list output
        let mut output = String::new();

        // List all issues grouped by repository
        let repositories: Vec<Repository> = schema::repositories::table
            .order_by(schema::repositories::user.asc())
            .then_order_by(schema::repositories::name.asc())
            .load::<Repository>(&mut conn)
            .map_err(|e| format!("Error loading repositories: {}", e))?;

        for repo in repositories {
            let mut query = schema::issues::table
                .filter(schema::issues::repository_id.eq(repo.id))
                .order_by(schema::issues::number.desc())
                .into_boxed();

            // Filter by state
            if state_filter.as_str() != "all" {
                query = query.filter(schema::issues::state.eq(state_filter.as_str()));
            }

            // Filter by type
            match type_filter {
                TypeFilter::Issue => {
                    query = query.filter(schema::issues::is_pull_request.eq(false))
                }
                TypeFilter::Pr => query = query.filter(schema::issues::is_pull_request.eq(true)),
                TypeFilter::All => {}
            }

            let repo_issues: Vec<Issue> = query
                .load::<Issue>(&mut conn)
                .map_err(|e| format!("Error loading issues: {}", e))?;

            if !repo_issues.is_empty() {
                output.push('\n');
                output.push_str(&format!("{}/{}\n", repo.user, repo.name));

                // Find the maximum issue number width for alignment
                let max_number_width = repo_issues
                    .iter()
                    .map(|i| i.number.to_string().len())
                    .max()
                    .unwrap_or(1);

                for issue in repo_issues {
                    // Build hyperlink for issue number using OSC 8 with padding
                    let url = format!(
                        "https://github.com/{}/{}/issues/{}",
                        repo.user, repo.name, issue.number
                    );
                    let padded_number =
                        format!("{:>width$}", issue.number, width = max_number_width);
                    let issue_number_display = format!("#{}", padded_number);
                    let issue_number_link = Link::new(&issue_number_display, &url);

                    let mut metadata = String::new();

                    if show_type {
                        let issue_type = if issue.is_pull_request { "PR" } else { "ISSUE" };
                        if !metadata.is_empty() {
                            metadata.push(' ');
                        }
                        metadata.push_str(issue_type);
                    }

                    if show_state {
                        if !metadata.is_empty() {
                            metadata.push(' ');
                        }
                        metadata.push_str(&issue.state.to_uppercase());
                    }

                    let date = issue.created_at.split('T').next().unwrap_or("");
                    if !metadata.is_empty() {
                        metadata.push(' ');
                    }
                    metadata.push_str(date);

                    output.push_str(&format!(
                        "{} {} {}\n",
                        issue_number_link,
                        metadata.dimmed(),
                        issue.title.bold()
                    ));
                }
            }
        }

        // Use pager for output
        Pager::new().setup();
        print!("{}", output);
    }
    Ok(())
}

fn list_pull_requests(
    pr_number: Option<i32>,
    state_filter: StateFilter,
) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection()?;
    
    // Check if filters are non-default
    let show_state = matches!(state_filter, StateFilter::Closed | StateFilter::All);
    
    if let Some(number) = pr_number {
        // Display specific pull request
        let issue = schema::issues::table
            .filter(schema::issues::number.eq(number))
            .filter(schema::issues::is_pull_request.eq(true))
            .first::<Issue>(&mut conn)
            .map_err(|e| format!("Pull request #{} not found: {}", number, e))?;
        
        // Get repository info
        let repository = schema::repositories::table
            .find(issue.repository_id)
            .first::<Repository>(&mut conn)
            .map_err(|e| format!("Repository not found: {}", e))?;
        
        // Create hyperlinked title using OSC 8
        let url = format!("https://github.com/{}/{}/pull/{}", repository.user, repository.name, issue.number);
        let title_display = format!("{}", issue.title.bold());
        let title_link = Link::new(&title_display, &url);
        
        // Display title and author
        let mut first_line = format!("{}", title_link);
        
        if let Some(author) = &issue.author {
            let author_url = format!("https://github.com/{}", author);
            let author_link = Link::new(author, &author_url);
            first_line.push_str(&format!(" {}", format!("by {}", author_link).dimmed()));
        }
        
        // Add state badge
        let state_display = if issue.state == "open" {
            issue.state.to_uppercase().green().to_string()
        } else {
            issue.state.to_uppercase().red().to_string()
        };
        first_line.push_str(&format!(" {}", state_display));
        
        println!("{}", first_line);
        
        // Get and display labels immediately after title
        let issue_labels: Vec<(IssueLabel, Label)> = schema::issue_labels::table
            .inner_join(schema::labels::table)
            .filter(schema::issue_labels::issue_id.eq(issue.id))
            .load::<(IssueLabel, Label)>(&mut conn)
            .unwrap_or_default();
        
        if !issue_labels.is_empty() {
            for (i, (_, label)) in issue_labels.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print!("{}", label.name.cyan());
            }
            println!();
        }
        
        // Get and display reactions
        let reactions: Vec<IssueReaction> = schema::issue_reactions::table
            .filter(schema::issue_reactions::issue_id.eq(issue.id))
            .order_by(schema::issue_reactions::reaction_type.asc())
            .load::<IssueReaction>(&mut conn)
            .unwrap_or_default();
        
        if !reactions.is_empty() {
            for (i, reaction) in reactions.iter().enumerate() {
                if i > 0 {
                    print!("\t");
                }
                print!("{} {}", reaction_to_ascii(&reaction.reaction_type), reaction.count.to_string().cyan());
            }
            println!();
        }
        
        println!();
        
        // Render markdown body with termimad
        let skin = MadSkin::default();
        if issue.body.trim().is_empty() {
            println!("{}", "No description provided".dimmed());
        } else {
            skin.print_text(&issue.body);
        }
    } else {
        // Collect pull request list output
        let mut output = String::new();
        
        // List all pull requests grouped by repository
        let repositories: Vec<Repository> = schema::repositories::table
            .order_by(schema::repositories::user.asc())
            .then_order_by(schema::repositories::name.asc())
            .load::<Repository>(&mut conn)
            .map_err(|e| format!("Error loading repositories: {}", e))?;
        
        for repo in repositories {
            let mut query = schema::issues::table
                .filter(schema::issues::repository_id.eq(repo.id))
                .filter(schema::issues::is_pull_request.eq(true))
                .order_by(schema::issues::number.desc())
                .into_boxed();
            
            // Filter by state
            if state_filter.as_str() != "all" {
                query = query.filter(schema::issues::state.eq(state_filter.as_str()));
            }
            
            let repo_prs: Vec<Issue> = query
                .load::<Issue>(&mut conn)
                .map_err(|e| format!("Error loading pull requests: {}", e))?;
            
            if !repo_prs.is_empty() {
                output.push('\n');
                output.push_str(&format!("{}/{}\n", repo.user, repo.name));
                
                // Find the maximum issue number width for alignment
                let max_number_width = repo_prs
                    .iter()
                    .map(|i| i.number.to_string().len())
                    .max()
                    .unwrap_or(1);
                
                for pr in repo_prs {
                    // Build hyperlink for PR number using OSC 8 with padding
                    let url = format!(
                        "https://github.com/{}/{}/pull/{}",
                        repo.user, repo.name, pr.number
                    );
                    let padded_number =
                        format!("{:>width$}", pr.number, width = max_number_width);
                    let pr_number_display = format!("#{}", padded_number);
                    let pr_number_link = Link::new(&pr_number_display, &url);
                    
                    let mut metadata = String::new();
                    
                    if show_state {
                        metadata.push_str(&pr.state.to_uppercase());
                    }
                    
                    let date = pr.created_at.split('T').next().unwrap_or("");
                    if !metadata.is_empty() {
                        metadata.push(' ');
                    }
                    metadata.push_str(date);
                    
                    output.push_str(&format!(
                        "{} {} {}\n",
                        pr_number_link,
                        metadata.dimmed(),
                        pr.title.bold()
                    ));
                }
            }
        }
        
        // Use pager for output
        Pager::new().setup();
        print!("{}", output);
    }
    Ok(())
}

async fn sync_issues_for_repo(user: &str, repo: &str, token: &str, force: bool) -> Result<(), Box<dyn Error>> {
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    let client = reqwest::Client::new();
    let mut conn = establish_connection()?;

    // Get repository ID
    let repository: Repository = schema::repositories::table
        .filter(schema::repositories::user.eq(user))
        .filter(schema::repositories::name.eq(repo))
        .first::<Repository>(&mut conn)
        .map_err(|e| format!("Repository {}/{} not found: {}", user, repo, e))?;

    // Load all existing issues for this repository into a HashMap for quick lookup
    let existing_issues: Vec<Issue> = schema::issues::table
        .filter(schema::issues::repository_id.eq(repository.id))
        .load::<Issue>(&mut conn)
        .map_err(|e| format!("Error loading existing issues: {}", e))?;

    let mut issue_cache: HashMap<i32, Option<String>> = HashMap::new();
    for issue in existing_issues {
        issue_cache.insert(issue.number, issue.last_synced_at);
    }

    let mut count = 0;
    let mut skipped = 0;
    let mut page = 1;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues?state=all&per_page=100&page={}",
            user, repo, page
        );

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

        if github_issues.is_empty() {
            break;
        }

        for gh_issue in github_issues {
            // Check if we should skip this issue based on cache
            let should_sync = if !force {
                if let Some(last_synced) = issue_cache.get(&gh_issue.number) {
                    // Issue exists in database
                    if let Some(last_synced_str) = last_synced {
                        // Parse the last_synced_at timestamp
                        if let Ok(last_sync_time) = DateTime::parse_from_rfc3339(last_synced_str) {
                            let now = Utc::now();
                            let duration = now.signed_duration_since(last_sync_time);

                            // Skip if synced less than 10 minutes ago
                            if duration.num_minutes() < 10 {
                                skipped += 1;
                                false
                            } else {
                                true
                            }
                        } else {
                            // If we can't parse the timestamp, sync it
                            true
                        }
                    } else {
                        // last_synced_at is NULL, sync it
                        true
                    }
                } else {
                    // New issue, always sync
                    true
                }
            } else {
                // Force flag is true, sync everything
                true
            };

            if !should_sync {
                continue;
            }

            let current_time = Utc::now().to_rfc3339();
            let new_issue = NewIssue {
                repository_id: repository.id,
                number: gh_issue.number,
                title: gh_issue.title.clone(),
                body: gh_issue.body.clone().unwrap_or_default(),
                created_at: gh_issue.created_at,
                state: gh_issue.state,
                is_pull_request: gh_issue.pull_request.is_some(),
                author: gh_issue.user.map(|u| u.login),
                last_synced_at: Some(current_time.clone()),
            };

            diesel::insert_into(schema::issues::table)
                .values(&new_issue)
                .on_conflict((schema::issues::repository_id, schema::issues::number))
                .do_update()
                .set((
                    schema::issues::title.eq(excluded(schema::issues::title)),
                    schema::issues::body.eq(excluded(schema::issues::body)),
                    schema::issues::state.eq(excluded(schema::issues::state)),
                    schema::issues::last_synced_at.eq(excluded(schema::issues::last_synced_at)),
                ))
                .execute(&mut conn)
                .map_err(|e| format!("Error syncing issue: {}", e))?;

            // Fetch the inserted/updated issue
            let issue_result = schema::issues::table
                .filter(schema::issues::repository_id.eq(repository.id))
                .filter(schema::issues::number.eq(gh_issue.number))
                .first::<Issue>(&mut conn)
                .map_err(|e| format!("Error fetching issue after insert: {}", e))?;

            // Store labels
            if let Some(labels) = gh_issue.labels {
                for label in labels {
                    let _ = diesel::insert_into(schema::labels::table)
                        .values(NewLabel {
                            name: label.name.clone(),
                        })
                        .on_conflict(schema::labels::name)
                        .do_nothing()
                        .execute(&mut conn);

                    let label_obj: Label = schema::labels::table
                        .filter(schema::labels::name.eq(&label.name))
                        .first::<Label>(&mut conn)
                        .ok()
                        .unwrap_or_else(|| Label {
                            id: 0,
                            name: label.name.clone(),
                        });

                    if label_obj.id > 0 {
                        let _ = diesel::insert_into(schema::issue_labels::table)
                            .values(models::NewIssueLabel {
                                issue_id: issue_result.id,
                                label_id: label_obj.id,
                            })
                            .on_conflict((
                                schema::issue_labels::issue_id,
                                schema::issue_labels::label_id,
                            ))
                            .do_nothing()
                            .execute(&mut conn);
                    }
                }
            }

            // Store reactions
            if let Some(reactions) = gh_issue.reactions {
                let reactions_list = vec![
                    ("+1", reactions.plus_one),
                    ("-1", reactions.minus_one),
                    ("laugh", reactions.laugh),
                    ("hooray", reactions.hooray),
                    ("confused", reactions.confused),
                    ("heart", reactions.heart),
                    ("rocket", reactions.rocket),
                    ("eyes", reactions.eyes),
                ];

                for (reaction_type, count) in reactions_list {
                    if let Some(cnt) = count {
                        if cnt > 0 {
                            let _ = diesel::insert_into(schema::issue_reactions::table)
                                .values(models::NewIssueReaction {
                                    issue_id: issue_result.id,
                                    reaction_type: reaction_type.to_string(),
                                    count: cnt,
                                })
                                .on_conflict((
                                    schema::issue_reactions::issue_id,
                                    schema::issue_reactions::reaction_type,
                                ))
                                .do_update()
                                .set(schema::issue_reactions::count.eq(cnt))
                                .execute(&mut conn);
                        }
                    }
                }
            }

            count += 1;
        }

        // Print progress on the same line
        print!(
            "\r{}: {} synced, {} skipped (cached)",
            format!("{}/{}", user, repo).cyan(),
            count,
            skipped
        );
        std::io::Write::flush(&mut std::io::stdout())?;

        page += 1;
    }

    println!(); // Final newline after progress completes
    Ok(())
}

#[tokio::main]
async fn sync_all_repos(force: bool) -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let token = std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN not found in .env file")?;

    let mut conn = establish_connection()?;

    let repos: Vec<Repository> = schema::repositories::table
        .load::<Repository>(&mut conn)
        .map_err(|e| format!("Error loading repositories: {}", e))?;

    if repos.is_empty() {
        println!(
            "No repositories to sync. Add repositories with: {}.",
            "cargo run -- repo add username/projectname".yellow()
        );
        return Ok(());
    }

    for repo in repos {
        if let Err(e) = sync_issues_for_repo(&repo.user, &repo.name, &token, force).await {
            eprintln!("Error syncing {}/{}: {}", repo.user, repo.name, e);
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync { force } => {
            if let Err(e) = sync_all_repos(force) {
                eprintln!("{}: {}", "Error".red(), e);
            }
        }
        Commands::Repo { command } => match command {
            Some(RepoCommands::Add { repo }) => {
                let parts: Vec<&str> = repo.split('/').collect();
                if parts.len() != 2 {
                    eprintln!(
                        "{}: Repository must be in format {}.",
                        "Error".red(),
                        "username/projectname".yellow()
                    );
                } else if let Err(e) = insert_repository(parts[0], parts[1]) {
                    eprintln!("{}: {}", "Error".red(), e);
                }
            }
            Some(RepoCommands::Rm { repo }) => {
                let parts: Vec<&str> = repo.split('/').collect();
                if parts.len() != 2 {
                    eprintln!(
                        "{}: Repository must be in format {}.",
                        "Error".red(),
                        "username/projectname".yellow()
                    );
                } else if let Err(e) = remove_repository(parts[0], parts[1]) {
                    eprintln!("{}: {}", "Error".red(), e);
                }
            }
            None => {
                if let Err(e) = list_repositories() {
                    eprintln!("{}: {}", "Error".red(), e);
                }
            }
        },
        Commands::Issue {
            number,
            state,
            r#type,
        } => {
            if let Err(e) = list_issues(number, state, r#type) {
                eprintln!("{}: {}", "Error".red(), e);
            }
        }
        Commands::Pr { number, state } => {
            if let Err(e) = list_pull_requests(number, state) {
                eprintln!("{}: {}", "Error".red(), e);
            }
        }
    }
}
