# AGENTS.md

## Project Overview

github_issues_rs is a Rust CLI tool for managing GitHub repositories. It uses Diesel ORM for SQLite database access and Clap for command-line parsing.

## Command Structure

The CLI uses nested subcommands organized by domain:

```
github_issues_rs
├── sync          # Sync repositories with GitHub
└── repo          # Repository management
    ├── add       # Add a new repository
    └── list      # List all repositories
```

### Usage Examples

```bash
# Add a repository
cargo run -- repo add --user torvalds --name linux

# List all repositories
cargo run -- repo list

# Sync (placeholder)
cargo run -- sync
```

## Code Structure

- `src/main.rs` - CLI entry point with command definitions and handlers
- `src/models.rs` - Diesel model definitions (Repository, NewRepository)
- `src/schema.rs` - Diesel schema table definitions

## Database

- **Type**: SQLite
- **Path**: `repositories.db` (ignored by git)
- **Table**: `repositories` (id, user, name)

## Development

### Build
```bash
cargo build
```

### Run
```bash
cargo run -- <command>
```

### Lint
```bash
cargo clippy
```

### Test (when added)
```bash
cargo test
```

## Adding New Commands

1. Add a new enum variant to `Commands` or `RepoCommands` in `main.rs`
2. Implement the handler function
3. Add the match arm in `main()` to call the handler
4. Follow the existing error handling pattern using `Result<(), Box<dyn Error>>`

## Adding New Database Tables

1. Add table definition to `src/schema.rs`
2. Create model structs in `src/models.rs` (Queryable, Selectable for reads; Insertable for writes)
3. Use Diesel query builder in handler functions

## Conventions

- Use descriptive command names (e.g., `repo list` not `ls`)
- All errors should be printed to stderr with context
- Database connection is established per command (establish_connection)
- Models use the `#[allow(dead_code)]` attribute for fields that Diesel manages but aren't directly accessed
