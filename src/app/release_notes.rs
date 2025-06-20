use crate::error::Result;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{
    app::{background_operations::BackgroundEvent, App},
    error::SemanticReleaseError,
    git::GitRepo,
    types::{AppConfig, AppState, GitCommit},
};
use async_broadcast::Sender;
use tracing::{info, instrument, warn};

#[allow(async_fn_in_trait)]
pub trait ReleaseNotesOperations {
    async fn handle_release_notes_generation(&mut self) -> Result<()>;
    async fn generate_release_notes_with_npm_wrapper(&mut self) -> Result<()>;
}

impl ReleaseNotesOperations for App {
    async fn handle_release_notes_generation(&mut self) -> Result<()> {
        // Check if already processing to avoid multiple concurrent analyses
        if matches!(self.current_state, AppState::Loading) {
            return Ok(());
        }

        // MODERN ASYNC APPROACH: Use BackgroundTaskManager
        self.current_state = AppState::Loading;
        self.message = Some("🚀 Iniciando generación de notas de versión...".to_string());

        // Get commits since last tag only (for release notes)
        let git_repo = GitRepo::new()?;
        let last_tag = git_repo.get_last_tag()?;
        let commits = git_repo.get_commits_since_tag(last_tag.as_deref())?;

        if let Some(tag) = &last_tag {
            info!("Generating release notes for commits since tag: {}", tag);
        } else {
            info!("No previous tag found, generating release notes for all commits");
        }

        // Start async release notes generation
        match self
            .background_task_manager
            .start_release_notes_generation(&self.config, commits)
            .await
        {
            Ok(_operation_id) => {
                info!("Release notes generation started via BackgroundTaskManager");
            }
            Err(e) => {
                self.current_state = AppState::Error(format!("Error iniciando generación: {}", e));
                self.message = Some(format!("❌ {}", e));
            }
        }

        Ok(())
    }

    async fn generate_release_notes_with_npm_wrapper(&mut self) -> Result<()> {
        self.current_state = crate::types::AppState::Loading;

        if let Err(e) = self.generate_release_notes_with_npm().await {
            self.current_state = crate::types::AppState::Error(e.to_string());
        } else {
            self.current_state = crate::types::AppState::Normal;
            self.message = Some("✅ Notas de versión generadas exitosamente".to_string());
            self.current_screen = crate::types::AppScreen::Main;
        }

        Ok(())
    }
}

impl App {
    pub async fn generate_release_notes_with_npm(&mut self) -> Result<()> {
        // Shared state for communication between thread and UI
        let npm_status = Arc::new(Mutex::new(String::from(
            "🚀 Iniciando npm run release-notes...",
        )));
        let npm_finished = Arc::new(Mutex::new(false));
        let npm_success = Arc::new(Mutex::new(true));

        // Clone for the thread
        let status_clone = npm_status.clone();
        let finished_clone = npm_finished.clone();
        let success_clone = npm_success.clone();

        // Update initial status
        self.message = Some("🚀 Iniciando npm run release-notes...".to_string());
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Spawn npm command in background thread
        thread::spawn(move || {
            // Update status to indicate command execution
            if let Ok(mut status) = status_clone.lock() {
                *status = "⚙️ Ejecutando npm run release-notes...".to_string();
            }

            // Execute npm command
            let npm_output = Command::new("npm")
                .args(["run", "release-notes"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            match npm_output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        // Parse output to extract information
                        let mut status_message =
                            "✅ Notas de versión generadas exitosamente".to_string();

                        // Look for generated files in the output
                        if stdout.contains("✅") || stderr.contains("✅") {
                            status_message = stdout
                                .lines()
                                .find(|line| line.contains("✅"))
                                .unwrap_or("✅ Notas de versión generadas exitosamente")
                                .to_string();
                        }

                        if let Ok(mut status) = status_clone.lock() {
                            *status = status_message;
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if let Ok(mut status) = status_clone.lock() {
                            *status = format!("❌ Error en npm: {}", stderr);
                        }
                        if let Ok(mut success) = success_clone.lock() {
                            *success = false;
                        }
                    }
                }
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = format!("❌ Error ejecutando npm: {}", e);
                    }
                    if let Ok(mut success) = success_clone.lock() {
                        *success = false;
                    }
                }
            }

            // Mark as finished
            if let Ok(mut finished) = finished_clone.lock() {
                *finished = true;
            }
        });

        // Wait for npm to finish and update UI
        let mut current_status = String::new();
        loop {
            // Check if npm is finished
            let is_finished = { npm_finished.lock().map(|f| *f).unwrap_or(false) };

            // Update status message if it changed
            if let Ok(status) = npm_status.lock() {
                if *status != current_status {
                    current_status = status.clone();
                    self.message = Some(current_status.clone());
                }
            }

            if is_finished {
                break;
            }

            // Yield control to UI with short sleep
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        // Check if npm succeeded
        let success = npm_success.lock().map(|s| *s).unwrap_or(false);
        if !success {
            return Err(crate::error::SemanticReleaseError::release_error(format!(
                "Release notes operation failed: {}",
                current_status
            )));
        }

        Ok(())
    }
}

#[instrument(skip(config, _commits, event_tx))]
async fn analyze_commits_with_ai(
    config: &AppConfig,
    _commits: &[GitCommit],
    event_tx: &Sender<BackgroundEvent>,
) -> crate::error::Result<String> {
    if let Some(_token) = &config.gemini_token {
        // Update progress
        if let Err(e) = event_tx
            .broadcast(BackgroundEvent::ReleaseNotesProgress(
                "Running AI analysis on commits...".to_string(),
            ))
            .await
        {
            warn!("Failed to broadcast AI progress: {}", e);
        }

        // TODO: Implement proper Gemini service integration
        // For now, return a placeholder
        warn!("AI analysis not yet implemented in async version");
        Ok("AI analysis will be implemented in a future update.".to_string())
    } else {
        Err(SemanticReleaseError::config_error(
            "Gemini token not configured",
        ))
    }
}

fn add_commit_section(release_notes: &mut String, title: &str, commits: &[&GitCommit]) {
    if !commits.is_empty() {
        release_notes.push_str(&format!("## {}\n\n", title));
        for commit in commits {
            let scope_str = if let Some(scope) = &commit.scope {
                if !scope.is_empty() {
                    format!("**{}**: ", scope)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            release_notes.push_str(&format!(
                "- {}{} ([{}])\n",
                scope_str,
                commit.description,
                &commit.hash[..8]
            ));

            // Add task references if available
            if !commit.monday_tasks.is_empty() || !commit.jira_tasks.is_empty() {
                let mut task_refs = Vec::new();
                task_refs.extend(commit.monday_tasks.iter().map(|t| format!("Monday: {}", t)));
                task_refs.extend(commit.jira_tasks.iter().map(|t| format!("JIRA: {}", t)));

                if !task_refs.is_empty() {
                    release_notes.push_str(&format!("  - Related: {}\n", task_refs.join(", ")));
                }
            }

            if !commit.body.trim().is_empty() && commit.body.len() > 50 {
                // Add commit body if it's substantial
                release_notes.push_str(&format!("  - {}\n", commit.body.trim()));
            }
        }
        release_notes.push('\n');
    }
}

#[instrument(skip(release_notes, commits, config))]
async fn add_task_management_section(
    release_notes: &mut String,
    commits: &[GitCommit],
    config: &AppConfig,
) {
    let mut monday_tasks = std::collections::HashSet::new();
    let mut jira_tasks = std::collections::HashSet::new();

    // Collect unique task references
    for commit in commits {
        for task in &commit.monday_tasks {
            monday_tasks.insert(task.clone());
        }
        for task in &commit.jira_tasks {
            jira_tasks.insert(task.clone());
        }
    }

    if !monday_tasks.is_empty() || !jira_tasks.is_empty() {
        release_notes.push_str("## 📋 Related Tasks\n\n");

        if !monday_tasks.is_empty() && config.is_monday_configured() {
            release_notes.push_str("### Monday.com Tasks\n");
            for task_id in &monday_tasks {
                // TODO: Implement async Monday service integration
                release_notes.push_str(&format!("- {}\n", task_id));
            }
            release_notes.push('\n');
        }

        if !jira_tasks.is_empty() && config.is_jira_configured() {
            release_notes.push_str("### JIRA Issues\n");
            for task_key in &jira_tasks {
                // TODO: Implement async JIRA service integration
                release_notes.push_str(&format!("- {}\n", task_key));
            }
            release_notes.push('\n');
        }
    }
}

#[instrument(skip(event_tx, config, commits))]
pub async fn generate_release_notes_task(
    event_tx: Sender<BackgroundEvent>,
    operation_id: String,
    config: AppConfig,
    commits: Vec<GitCommit>,
) -> crate::error::Result<()> {
    info!("Starting release notes generation task");

    // Broadcast progress: preparation phase
    if let Err(e) = event_tx
        .broadcast(BackgroundEvent::ReleaseNotesProgress(
            "Preparing commit data for analysis...".to_string(),
        ))
        .await
    {
        warn!("Failed to broadcast progress: {}", e);
    }

    let mut release_notes = String::new();
    release_notes.push_str("# 🚀 Release Notes\n\n");

    if commits.is_empty() {
        let message = "No commits found for release notes generation.";
        if let Err(e) = event_tx
            .broadcast(BackgroundEvent::ReleaseNotesCompleted(
                serde_json::json!({"message": message, "status": "completed"}),
            ))
            .await
        {
            warn!("Failed to broadcast completion: {}", e);
        }
        return Ok(());
    }

    // Broadcast progress: categorization phase
    if let Err(e) = event_tx
        .broadcast(BackgroundEvent::ReleaseNotesProgress(
            "Categorizing commits by type...".to_string(),
        ))
        .await
    {
        warn!("Failed to broadcast progress: {}", e);
    }

    // Group commits by type with better organization
    let mut features = Vec::new();
    let mut fixes = Vec::new();
    let mut docs = Vec::new();
    let mut style = Vec::new();
    let mut refactor = Vec::new();
    let mut performance = Vec::new();
    let mut tests = Vec::new();
    let mut chores = Vec::new();
    let mut reverts = Vec::new();
    let mut breaking_changes = Vec::new();

    for commit in &commits {
        if !commit.breaking_changes.is_empty() {
            breaking_changes.extend(commit.breaking_changes.iter().cloned());
        }

        match commit.commit_type.as_deref() {
            Some("feat") => features.push(commit),
            Some("fix") => fixes.push(commit),
            Some("docs") => docs.push(commit),
            Some("style") => style.push(commit),
            Some("refactor") => refactor.push(commit),
            Some("perf") => performance.push(commit),
            Some("test") => tests.push(commit),
            Some("chore") => chores.push(commit),
            Some("revert") => reverts.push(commit),
            _ => chores.push(commit), // Default fallback
        }
    }

    // Breaking Changes Section (highest priority)
    if !breaking_changes.is_empty() {
        release_notes.push_str("## ⚠️  BREAKING CHANGES\n\n");
        for change in &breaking_changes {
            release_notes.push_str(&format!("- {}\n", change));
        }
        release_notes.push('\n');
    }

    // Broadcast progress: AI enhancement phase
    if let Err(e) = event_tx
        .broadcast(BackgroundEvent::ReleaseNotesProgress(
            "Enhancing release notes with AI analysis...".to_string(),
        ))
        .await
    {
        warn!("Failed to broadcast progress: {}", e);
    }

    // Enhanced sections with AI analysis if available
    if config.gemini_token.is_some() {
        match analyze_commits_with_ai(&config, &commits, &event_tx).await {
            Ok(ai_analysis) => {
                release_notes.push_str("## 🤖 AI Summary\n\n");
                release_notes.push_str(&ai_analysis);
                release_notes.push_str("\n\n");
            }
            Err(e) => {
                warn!("AI analysis failed: {}", e);
                // Continue with standard generation
            }
        }
    }

    // Standard sections
    add_commit_section(&mut release_notes, "✨ New Features", &features);
    add_commit_section(&mut release_notes, "🐛 Bug Fixes", &fixes);
    add_commit_section(
        &mut release_notes,
        "⚡ Performance Improvements",
        &performance,
    );
    add_commit_section(&mut release_notes, "♻️  Code Refactoring", &refactor);
    add_commit_section(&mut release_notes, "📚 Documentation", &docs);
    add_commit_section(&mut release_notes, "🧪 Tests", &tests);
    add_commit_section(&mut release_notes, "💎 Style Changes", &style);
    add_commit_section(&mut release_notes, "🔧 Chores", &chores);
    add_commit_section(&mut release_notes, "⏪ Reverts", &reverts);

    // Broadcast progress: task management integration
    if let Err(e) = event_tx
        .broadcast(BackgroundEvent::ReleaseNotesProgress(
            "Integrating task management data...".to_string(),
        ))
        .await
    {
        warn!("Failed to broadcast progress: {}", e);
    }

    // Add task management integration
    add_task_management_section(&mut release_notes, &commits, &config).await;

    // Broadcast progress: saving files
    if let Err(e) = event_tx
        .broadcast(BackgroundEvent::ReleaseNotesProgress(
            "Creating release-notes directory and saving files...".to_string(),
        ))
        .await
    {
        warn!("Failed to broadcast progress: {}", e);
    }

    // Create output directory
    if let Err(e) = std::fs::create_dir_all("release-notes") {
        warn!("Could not create release-notes directory: {}", e);
        if let Err(e) = event_tx
            .broadcast(BackgroundEvent::ReleaseNotesError(format!(
                "Failed to create release-notes directory: {}",
                e
            )))
            .await
        {
            warn!("Failed to broadcast error: {}", e);
        }
        return Err(SemanticReleaseError::config_error(format!(
            "Could not create release-notes directory: {}",
            e
        )));
    }

    // Generate filenames
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let script_filename = format!(
        "release-notes/release-notes-{}_SCRIPT_WITH_ENTER_KEY.md",
        date_str
    );
    let gemini_filename = format!("release-notes/release-notes-{}_GEMINI.md", date_str);

    // Save the basic release notes file
    if let Err(e) = std::fs::write(&script_filename, &release_notes) {
        warn!(
            "Failed to write release notes file {}: {}",
            script_filename, e
        );
        if let Err(e) = event_tx
            .broadcast(BackgroundEvent::ReleaseNotesError(format!(
                "Failed to save release notes file: {}",
                e
            )))
            .await
        {
            warn!("Failed to broadcast error: {}", e);
        }
        return Err(SemanticReleaseError::config_error(format!(
            "Failed to write release notes file: {}",
            e
        )));
    }

    info!("Successfully saved release notes to: {}", script_filename);

    // Try to process with Gemini if configured
    if config.gemini_token.is_some() {
        if let Err(e) = event_tx
            .broadcast(BackgroundEvent::ReleaseNotesProgress(
                "Processing release notes with Gemini AI...".to_string(),
            ))
            .await
        {
            warn!("Failed to broadcast progress: {}", e);
        }

        // Read the template file
        let template_path = std::path::Path::new("scripts/plantilla.md");
        let template_content = match std::fs::read_to_string(template_path) {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read template file scripts/plantilla.md: {}", e);
                if let Err(e) = event_tx
                    .broadcast(BackgroundEvent::ReleaseNotesError(format!(
                        "Failed to read template file: {}",
                        e
                    )))
                    .await
                {
                    warn!("Failed to broadcast error: {}", e);
                }
                // Continue without Gemini processing if template can't be read
                return Ok(());
            }
        };

        match crate::services::GeminiClient::new(&config) {
            Ok(gemini_client) => {
                // Combine release notes and template for Gemini processing
                let combined_input = format!(
                    "RELEASE NOTES TO PROCESS:\n{}\n\nTEMPLATE TO FOLLOW:\n{}",
                    release_notes, template_content
                );

                match gemini_client
                    .process_release_notes_document(&combined_input)
                    .await
                {
                    Ok(gemini_response) => {
                        // Save the Gemini-processed version
                        if let Err(e) = std::fs::write(&gemini_filename, &gemini_response) {
                            warn!("Failed to write Gemini file {}: {}", gemini_filename, e);
                            // Don't fail the entire operation, just log the warning
                        } else {
                            info!(
                                "Successfully saved Gemini-processed release notes to: {}",
                                gemini_filename
                            );
                        }
                    }
                    Err(e) => {
                        warn!("Gemini processing failed: {}", e);
                        // Continue without Gemini processing
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create Gemini client: {}", e);
                // Continue without Gemini processing
            }
        }
    }

    // Final broadcast: completion with file paths
    let completion_message = if std::path::Path::new(&gemini_filename).exists() {
        format!(
            "Release notes generated successfully!\n\n📄 Basic release notes: {}\n🤖 AI-enhanced release notes: {}",
            script_filename, gemini_filename
        )
    } else {
        format!(
            "Release notes generated successfully!\n\n📄 Basic release notes: {}\n💡 Install Gemini API key for AI-enhanced notes",
            script_filename
        )
    };

    if let Err(e) = event_tx.broadcast(BackgroundEvent::ReleaseNotesCompleted(
        serde_json::json!({
            "notes": release_notes,
            "script_file": script_filename,
            "gemini_file": if std::path::Path::new(&gemini_filename).exists() { Some(gemini_filename) } else { None },
            "status": "completed",
            "message": completion_message
        })
    )).await {
        warn!("Failed to broadcast completion: {}", e);
    }

    info!("Release notes generation completed successfully");
    Ok(())
}
