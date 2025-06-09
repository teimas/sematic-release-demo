# Semantic Release TUI

A Terminal User Interface (TUI) version of the semantic release tool built with Rust and Ratatui. This tool replicates the functionality of the original Node.js version with Monday.com integration and AI-powered release notes generation.

## Features

- 🚀 **Interactive TUI**: Beautiful terminal interface built with Ratatui
- 📋 **Commit Management**: Create semantic commits with Monday.com task integration
- 🤖 **AI Release Notes**: Generate professional release notes using Google Gemini
- 🔍 **Task Search**: Search and select Monday.com tasks directly from the TUI
- ⚙️ **Configuration**: Easy setup for API keys and settings
- 🎯 **Git Integration**: Analyze commits and manage repository operations

## Installation

Make sure you have Rust installed, then:

```bash
cargo build --release
```

## Configuration

First, configure your API keys and settings:

```bash
cargo run -- config
```

You'll be prompted to enter:
- Monday.com API Key
- Monday.com Account Slug (subdomain)
- Monday.com Board ID (optional)
- Google Gemini API Token (optional for AI features)

Configuration is stored in `.env` file in the current directory (same as original project)

## Usage

### Interactive TUI Mode

Run the TUI interface:

```bash
cargo run
# or
cargo run -- tui
```

Navigate with:
- `Tab`/`Shift+Tab`: Navigate between tabs
- `Enter`: Select current option
- `q`: Quit application
- `Esc`: Go back/cancel

### Command Line Interface

#### Create Commits

```bash
cargo run -- commit
```

#### Generate Release Notes

```bash
cargo run -- release-notes
```

#### Search Monday.com Tasks

```bash
cargo run -- search "task name"
```

## TUI Screens

### Main Screen
Welcome screen with navigation tabs for different features.

### Commit Screen
Interactive commit creation with:
- Commit type selection (feat, fix, docs, etc.)
- Scope input (can be populated from Monday.com tasks)
- Title and description
- Breaking changes
- Test details
- Security information
- Monday.com task integration

### Task Search
Search Monday.com tasks by name with real-time results.

### Task Selection
Multi-select interface for choosing Monday.com tasks to associate with commits.

### Release Notes
Generate AI-powered release notes by analyzing git commits and Monday.com tasks.

## Commit Message Format

The tool follows the same commit format as the original:

```
type(scope): description

Detailed description if necessary

BREAKING CHANGE: Details if applicable

Test Details: Description of tests
Security: Security information or NA

MONDAY TASKS:
- Task Title (ID: 123456) - https://account.monday.com/boards/board/pulses/task
```

## Release Notes Generation

The tool generates two files in the `release-notes/` directory:

1. `release-notes-YYYY-MM-DD.md` - Structured data with all commit and task information
2. `release-notes-YYYY-MM-DD_GEMINI.md` - AI-generated professional release notes

## Architecture

- **Ratatui**: Terminal UI framework
- **Tokio**: Async runtime for API calls
- **Git2**: Git repository operations
- **Reqwest**: HTTP client for API calls
- **Serde**: JSON serialization/deserialization
- **Crossterm**: Terminal control

## API Integrations

### Monday.com API
- Search tasks by name
- Fetch task details and updates
- Support for board-specific or global search

### Google Gemini API
- Generate professional release notes
- Fallback between Gemini 1.5 Pro and 1.0 Pro
- Structured prompts in Spanish (configurable)

## Error Handling

The application provides user-friendly error messages for:
- Missing configuration
- API connection issues
- Git repository problems
- File system errors

## Development

To contribute to the project:

1. Clone the repository
2. Install Rust and Cargo
3. Run `cargo test` to run tests
4. Run `cargo clippy` for linting
5. Run `cargo fmt` for formatting

## Comparison with Original

This Rust TUI version provides:
- ✅ All core functionality from the Node.js version
- ✅ Interactive terminal interface instead of command prompts
- ✅ Real-time task search and selection
- ✅ Visual commit form with validation
- ✅ Same Monday.com and Gemini API integrations
- ✅ Compatible commit message format
- ✅ Same release notes structure

## License

Same license as the original project. 