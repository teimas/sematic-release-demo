use clap::{Parser, Subcommand};
use semantic_release_tui::observability::log_user_message;
use tracing::{error, info};

mod app;
mod config;
mod error;
mod git;
mod observability;
mod services;
mod types;
mod ui;
mod utils;

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

    /// Enable development mode with hierarchical logging
    #[arg(long, global = true)]
    dev: bool,
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
    /// Get detailed version information using semantic-release
    VersionInfo,
    /// Debug mode - show detailed error information
    Debug {
        #[command(subcommand)]
        debug_command: DebugCommands,
    },
}

#[derive(Subcommand, Debug)]
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
async fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    // Note: Observability has been simplified - logging is handled via tracing defaults

    // Log to file only, not console
    info!(
        version = env!("CARGO_PKG_VERSION"),
        autocommit = cli.autocommit,
        debug = cli.debug,
        verbose = cli.verbose,
        dev = cli.dev,
        "🚀 Starting Semantic Release TUI"
    );

    // Handle --autocommit flag
    if cli.autocommit {
        // File logging only
        info!("🤖 Running autocommit flow");
        let app = App::new().await
            .map_err(|e| miette::miette!("Failed to initialize app for autocommit: {}", e))?;
        app.autocommit_flow().await
            .map_err(|e| miette::miette!("Autocommit flow failed: {}", e))?;
        return Ok(());
    }

    let result = match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => {
            // File logging only
            info!("🖥️ Starting TUI interface");
            let app = App::new().await
                .map_err(|e| miette::miette!("Failed to initialize app for TUI: {}", e))?;
            app.run().await
        }
        Commands::Config => {
            // File logging only
            info!("⚙️ Running configuration");
            config::run_config().await
        }
        Commands::Commit => {
            // File logging only
            info!("📝 Running commit flow");
            let app = App::new().await
                .map_err(|e| miette::miette!("Failed to initialize app for commit: {}", e))?;
            app.commit_flow().await
        }
        Commands::ReleaseNotes => {
            // File logging only
            info!("📝 Running release notes generation");
            let mut app = App::new().await
                .map_err(|e| miette::miette!("Failed to initialize app for release notes: {}", e))?;
            app.current_screen = AppScreen::ReleaseNotes;
            app.run().await
        }
        Commands::Search { query } => {
            // File logging only
            info!(?query, "🔍 Running task search");
            let app = App::new().await
                .map_err(|e| miette::miette!("Failed to initialize app for search: {}", e))?;
            if let Some(query) = query {
                app.search_tasks(&query).await
            } else {
                log_user_message("Please provide a search query");
                Ok(())
            }
        }
        Commands::SetupTemplate => {
            // File logging only
            info!("🔧 Setting up commit template");
            config::setup_commit_template().await
        }
        Commands::VersionInfo => {
            // File logging only
            info!("📦 Analyzing version information");
            log_user_message("🔍 Analyzing version information...");
            match git::repository::get_version_info() {
                Ok(version_info) => {
                    log_user_message("\n📦 VERSION INFORMATION");
                    log_user_message(&"=".repeat(50));

                    if let Some(current) = &version_info.current_version {
                        log_user_message(&format!("🏷️  Current version: {}", current));
                    } else {
                        log_user_message("🏷️  Current version: No previous versions");
                    }

                    log_user_message(&format!("🚀 Next version: {}", version_info.next_version));
                    log_user_message(&format!("📊 Release type: {}", version_info.version_type));
                    log_user_message(&format!(
                        "📈 Commits since last version: {}",
                        version_info.commit_count
                    ));

                    if version_info.has_unreleased_changes {
                        log_user_message("✅ Has changes to release");
                    } else {
                        log_user_message("⚠️  No changes to release");
                    }

                    log_user_message("\n🔍 DETAILED ANALYSIS");
                    log_user_message(&"=".repeat(50));
                    log_user_message(&version_info.dry_run_output);
                    Ok(())
                }
                Err(e) => {
                    error!(error = %e, "Failed to analyze version information");
                    log_user_message(&format!("❌ Error analyzing version: {}", e));
                    std::process::exit(1);
                }
            }
        }
        Commands::Debug { debug_command } => {
            // File logging only
            info!(?debug_command, "🐛 Running debug command");
            let app = App::new().await
                .map_err(|e| miette::miette!("Failed to initialize app for debug: {}", e))?;
            match debug_command {
                DebugCommands::Monday => {
                    app.debug_monday().await
                }
                DebugCommands::Gemini => {
                    app.debug_gemini().await
                }
                DebugCommands::Git => {
                    app.debug_git().await
                }
                DebugCommands::Commit => {
                    app.debug_commit().await
                }
            }
        }
    };

    match result {
        Ok(_) => {
            // File logging only
            info!("✅ Application completed successfully");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Application failed");
            // Only show error to console if it's critical
            log_user_message(&format!("❌ Application failed: {}", e));
            Err(miette::miette!("Application failed: {}", e))
        }
    }
}
