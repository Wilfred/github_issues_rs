# AGENTS.md

## Project Overview

gh-offline is a Rust CLI tool for browsing cached GitHub issues offline. It syncs issues from repositories and stores them in a local SQLite database. It uses Diesel ORM for database access and Clap for command-line parsing.

## Command Structure

The CLI uses nested subcommands organized by domain:

```
gh-offline
├── sync          # Sync issues from all repositories in database
├── repo          # Repository management (no subcommand = list)
│   ├── add       # Add a new repository
│   └── rm        # Remove a repository
├── issue         # List all issues or view specific issue
└── pr            # List all pull requests or view specific pull request
```

### Usage Examples

```bash
# Add a repository
cargo run -- repo add torvalds/linux

# List all repositories
cargo run -- repo

# Remove a repository
cargo run -- repo rm torvalds/linux

# Sync issues from all repositories
cargo run -- sync

# List all issues (default: open issues only)
cargo run -- issue

# View a specific issue
cargo run -- issue 123

# List all issues with filters
cargo run -- issue --state all --type all

# List all pull requests (default: open only)
cargo run -- pr

# View a specific pull request
cargo run -- pr 123

# List all pull requests with filters
cargo run -- pr --state all
```

## Code Structure

- `src/main.rs` - CLI entry point with command definitions and handlers
- `src/models.rs` - Diesel model definitions for database tables
- `src/schema.rs` - Diesel schema table definitions

## Database

- **Type**: SQLite
- **Path**: `~/.local/share/gh-offline/repositories.db` (XDG_DATA_HOME spec)
- **Tables**: `repositories`, `issues`, `labels`, `issue_labels`, `issue_reactions`

## Development

Never run a build, just use lint or test.

### Run
```bash
cargo run -- <command>
```

### Lint
```bash
cargo clippy
```

### Formatting
```
cargo fmt
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
