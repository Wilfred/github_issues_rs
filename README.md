# gh-offline

A Rust CLI tool for browsing cached GitHub issues and pull requests offline.

`gh-offline` syncs issues and pull requests from your favorite GitHub repositories and stores them in a local SQLite database. View and browse issues offline with a beautiful terminal interface - perfect for when you're traveling without internet or want faster access to your issues.

## Features

- **Offline Access**: Sync issues and PRs once, browse them anytime without internet
- **Multi-Repository Support**: Track issues from multiple GitHub repositories
- **Rich Terminal Output**: Colored output with clickable links and formatted markdown
- **Powerful Filtering**: Filter by state (open/closed/all) and type (issues/PRs/all)
- **Fast & Lightweight**: Built in Rust with SQLite for quick local queries
- **Comprehensive Data**: Stores labels, reactions, and full issue content

## Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **GitHub Token**: Required for API access (see [Configuration](#configuration))

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/Wilfred/github_issues_rs.git
cd github_issues_rs

# Build and install
cargo install --path .
```

This will install the `gh-offline` binary to `~/.cargo/bin/`, which should be in your PATH.

### Development Build

To run without installing:

```bash
cargo run -- <command>
```

## Configuration

Create a GitHub personal access token:

1. Go to [GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Give it a descriptive name (e.g., "gh-offline")
4. Select the `public_repo` scope (or `repo` for private repositories)
5. Click "Generate token" and copy it

Create a `.env` file in the project directory or set the environment variable:

```bash
# Option 1: Create .env file
echo "GITHUB_TOKEN=your_token_here" > .env

# Option 2: Export environment variable
export GITHUB_TOKEN=your_token_here
```

## Usage

### Quick Start

```bash
# Add a repository to track
gh-offline repo add torvalds/linux

# Sync issues from all tracked repositories
gh-offline sync

# List all open issues
gh-offline issue

# View a specific issue
gh-offline issue 123
```

### Repository Management

```bash
# List all tracked repositories
gh-offline repo

# Add a repository
gh-offline repo add owner/repo
gh-offline repo add rust-lang/rust

# Remove a repository
gh-offline repo rm owner/repo
```

### Syncing Issues

```bash
# Sync all repositories
gh-offline sync
```

This fetches all issues, pull requests, labels, and reactions from your tracked repositories and stores them locally.

### Browsing Issues

```bash
# List all open issues (default)
gh-offline issue

# View a specific issue
gh-offline issue 123

# List all issues (open and closed)
gh-offline issue --state all

# List everything (issues and pull requests)
gh-offline issue --state all --type all
```

### Browsing Pull Requests

```bash
# List all open pull requests (default)
gh-offline pr

# View a specific pull request
gh-offline pr 456

# List all pull requests (open and closed)
gh-offline pr --state all
```

## Commands Reference

```
gh-offline
├── sync          # Sync issues from all tracked repositories
├── repo          # List all repositories (no subcommand = list)
│   ├── add       # Add a repository (usage: repo add owner/name)
│   └── rm        # Remove a repository (usage: repo rm owner/name)
├── issue         # List issues or view specific issue
│                 # Options: --state [open|closed|all], --type [issue|pr|all]
└── pr            # List pull requests or view specific PR
                  # Options: --state [open|closed|all]
```

## Data Storage

Issues are stored in a SQLite database at:
- Linux/macOS: `~/.local/share/gh-offline/repositories.db`
- Follows XDG Base Directory specification