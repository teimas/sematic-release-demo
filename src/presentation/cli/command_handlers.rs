//! CLI Command Handlers
//! 
//! This module provides CLI command implementations using the CQRS application layer.

#[cfg(feature = "new-domains")]
use std::sync::Arc;
#[cfg(feature = "new-domains")]
use clap::{Args, Subcommand};

#[cfg(feature = "new-domains")]
use crate::application::commands::{
    CommandBus, CreateReleaseCommand, SyncTasksCommand, GenerateNotesCommand,
};
#[cfg(feature = "new-domains")]
use crate::application::queries::{QueryBus, GetReleaseStatusQuery, ListTasksQuery};
#[cfg(feature = "new-domains")]
use super::output_formatters::{OutputFormat, format_output};

/// CLI application that uses CQRS commands and queries
#[cfg(feature = "new-domains")]
pub struct CliApplication {
    command_bus: Arc<dyn CommandBus>,
    query_bus: Arc<dyn QueryBus>,
}

#[cfg(feature = "new-domains")]
impl CliApplication {
    pub fn new(
        command_bus: Arc<dyn CommandBus>,
        query_bus: Arc<dyn QueryBus>,
    ) -> Self {
        Self {
            command_bus,
            query_bus,
        }
    }
    
    /// Handle CLI commands
    pub async fn handle_command(&self, cmd: CliCommand) -> Result<(), CliError> {
        match cmd {
            CliCommand::Release(args) => self.handle_release_command(args).await,
            CliCommand::Tasks(args) => self.handle_tasks_command(args).await,
            CliCommand::Notes(args) => self.handle_notes_command(args).await,
            CliCommand::Status(args) => self.handle_status_command(args).await,
        }
    }
    
    /// Handle release-related commands
    async fn handle_release_command(&self, args: ReleaseArgs) -> Result<(), CliError> {
        match args.command {
            ReleaseCommand::Create(create_args) => {
                let mut cmd = CreateReleaseCommand::new(create_args.repository_path.clone());
                
                if let Some(version) = create_args.version {
                    cmd = cmd.with_target_version(version);
                }
                
                if let Some(notes) = create_args.notes {
                    cmd = cmd.with_release_notes(notes);
                }
                
                cmd = cmd.with_dry_run(create_args.dry_run);
                cmd = cmd.with_auto_push(!create_args.no_push);
                
                match self.command_bus.execute(Box::new(cmd)).await {
                    Ok(_) => {
                        println!("Release created successfully!");
                        Ok(())
                    }
                    Err(e) => Err(CliError::CommandFailed(e.to_string())),
                }
            }
        }
    }
    
    /// Handle task-related commands
    async fn handle_tasks_command(&self, args: TasksArgs) -> Result<(), CliError> {
        match args.command {
            TasksCommand::Sync(sync_args) => {
                let cmd = SyncTasksCommand::new(sync_args.systems);
                
                match self.command_bus.execute(Box::new(cmd)).await {
                    Ok(_) => {
                        println!("Tasks synchronized successfully!");
                        Ok(())
                    }
                    Err(e) => Err(CliError::CommandFailed(e.to_string())),
                }
            }
            TasksCommand::List(list_args) => {
                let query = ListTasksQuery {
                    filters: None, // TODO: Convert from list_args
                    pagination: None,
                    sort: None,
                };
                
                match self.query_bus.execute(Box::new(query)).await {
                    Ok(result) => {
                        // Downcast the result to the expected type
                        if let Ok(tasks) = result.downcast::<Vec<crate::domains::tasks::entities::Task>>() {
                            println!("Found {} tasks", tasks.len());
                            for task in tasks.iter() {
                                println!("- {}: {}", task.id.as_str(), task.title);
                            }
                            Ok(())
                        } else {
                            println!("Failed to process task list");
                            Ok(())
                        }
                    }
                    Err(e) => return Err(CliError::CommandFailed(e.to_string())),
                }
            }
        }
    }
    
    /// Handle notes-related commands
    async fn handle_notes_command(&self, args: NotesArgs) -> Result<(), CliError> {
        match args.command {
            NotesCommand::Generate(gen_args) => {
                let cmd = GenerateNotesCommand::new(gen_args.repository_path);
                
                match self.command_bus.execute(Box::new(cmd)).await {
                    Ok(_) => {
                        println!("Release notes generated successfully!");
                        Ok(())
                    }
                    Err(e) => Err(CliError::CommandFailed(e.to_string())),
                }
            }
        }
    }
    
    /// Handle status command
    async fn handle_status_command(&self, args: StatusArgs) -> Result<(), CliError> {
        let query = GetReleaseStatusQuery {
            repository_path: args.repository_path,
        };
        
        match self.query_bus.execute(Box::new(query)).await {
            Ok(result) => {
                // Downcast the result to the expected type
                if let Ok(status) = result.downcast::<String>() {
                    println!("Release Status: {}", status);
                } else {
                    println!("Failed to get release status");
                }
                Ok(())
            }
            Err(e) => Err(CliError::CommandFailed(e.to_string())),
        }
    }
}

/// Main CLI commands
#[cfg(feature = "new-domains")]
#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Release management commands
    Release(ReleaseArgs),
    /// Task management commands
    Tasks(TasksArgs),
    /// Release notes commands
    Notes(NotesArgs),
    /// Status commands
    Status(StatusArgs),
}

/// Release command arguments
#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct ReleaseArgs {
    #[command(subcommand)]
    pub command: ReleaseCommand,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Subcommand)]
pub enum ReleaseCommand {
    /// Create a new release
    Create(CreateReleaseArgs),
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct CreateReleaseArgs {
    /// Repository path
    #[arg(short, long, default_value = ".")]
    pub repository_path: String,
    
    /// Target version for the release
    #[arg(short, long)]
    pub version: Option<String>,
    
    /// Release notes
    #[arg(short, long)]
    pub notes: Option<String>,
    
    /// Perform a dry run without making changes
    #[arg(long)]
    pub dry_run: bool,
    
    /// Don't automatically push changes
    #[arg(long)]
    pub no_push: bool,
}

/// Task command arguments
#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct TasksArgs {
    #[command(subcommand)]
    pub command: TasksCommand,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Subcommand)]
pub enum TasksCommand {
    /// Synchronize tasks with external systems
    Sync(SyncTasksArgs),
    /// List tasks
    List(ListTasksArgs),
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct SyncTasksArgs {
    /// Task systems to sync with
    #[arg(short, long)]
    pub systems: Vec<String>,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct ListTasksArgs {
    /// Output format
    #[arg(short, long, default_value = "table")]
    pub format: OutputFormat,
}

/// Notes command arguments
#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct NotesArgs {
    #[command(subcommand)]
    pub command: NotesCommand,
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Subcommand)]
pub enum NotesCommand {
    /// Generate release notes
    Generate(GenerateNotesArgs),
}

#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct GenerateNotesArgs {
    /// Repository path
    #[arg(short, long, default_value = ".")]
    pub repository_path: String,
}

/// Status command arguments
#[cfg(feature = "new-domains")]
#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Repository path
    #[arg(short, long, default_value = ".")]
    pub repository_path: String,
    
    /// Output format
    #[arg(short, long, default_value = "table")]
    pub format: OutputFormat,
}

/// CLI-specific errors
#[cfg(feature = "new-domains")]
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Command failed: {0}")]
    CommandFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("Output formatting failed: {0}")]
    OutputError(String),
}
