use clap::{Parser, Subcommand};

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync => {
            println!("hello sync");
        }
    }
}
