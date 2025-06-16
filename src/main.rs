use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;

mod app;
mod config;
mod services;
mod git;
mod ui;
mod types;

use app::App;
use types::AppScreen;

#[derive(Parser)]
#[command(name = "semantic-release-tui")]
#[command(about = "A TUI for semantic release with Monday.com and AI integration")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Auto-commit: Run comprehensive AI analysis and open commit editor directly
    #[arg(long, global = true)]
    autocommit: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the TUI interface
    Tui,
    /// Configure API keys and settings
    Config,
    /// Create a commit with Monday.com integration
    Commit,
    /// Generate release notes with AI
    ReleaseNotes,
    /// Search Monday.com tasks
    Search { query: Option<String> },
    /// Setup git commit template for consistent commit messages
    SetupTemplate,
    /// Debug mode - show detailed error information
    Debug { 
        #[command(subcommand)]
        debug_command: DebugCommands,
    },
}

#[derive(Subcommand)]
enum DebugCommands {
    /// Test Monday.com connection
    Monday,
    /// Test Gemini connection
    Gemini,
    /// Test Git repository
    Git,
    /// Test commit creation with detailed logs
    Commit,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging based on flags
    if cli.debug {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
        info!("Debug logging enabled");
    } else if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
        info!("Verbose logging enabled");
    }

    // Handle --autocommit flag
    if cli.autocommit {
        let app = App::new().await?;
        app.autocommit_flow().await?;
        return Ok(());
    }

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => {
            let app = App::new().await?;
            app.run().await?;
        }
        Commands::Config => {
            config::run_config().await?;
        }
        Commands::Commit => {
            let app = App::new().await?;
            app.commit_flow().await?;
        }
        Commands::ReleaseNotes => {
            let mut app = App::new().await?;
            app.current_screen = AppScreen::ReleaseNotes;
            app.run().await?;
        }
        Commands::Search { query } => {
            let app = App::new().await?;
            if let Some(query) = query {
                app.search_tasks(&query).await?;
            } else {
                println!("Please provide a search query");
            }
        }
        Commands::SetupTemplate => {
            config::setup_commit_template().await?;
        }
        Commands::Debug { debug_command } => {
            let app = App::new().await?;
            match debug_command {
                DebugCommands::Monday => {
                    app.debug_monday().await?;
                }
                DebugCommands::Gemini => {
                    app.debug_gemini().await?;
                }
                DebugCommands::Git => {
                    app.debug_git().await?;
                }
                DebugCommands::Commit => {
                    app.debug_commit().await?;
                }
            }
        }
    }

    Ok(())
} 