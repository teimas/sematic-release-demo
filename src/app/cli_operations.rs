use anyhow::Result;

use crate::{app::App, git::GitRepo, services::MondayClient};

impl App {
    // CLI methods for direct command usage
    pub async fn commit_flow(&self) -> Result<()> {
        println!("🚀 TEIMAS Release Committer (TERCO) - Opening Commit Interface...");
        println!("💡 TIP: Use 't' for comprehensive AI analysis");
        println!("📋 Press 'q' to quit, 'Tab' to navigate between fields");

        // Create a new app instance specifically for commit flow
        let mut app = App::new().await?;

        // Set the initial screen to commit instead of main
        app.current_screen = crate::types::AppScreen::Commit;

        // Run the TUI starting on the commit screen
        app.run().await?;

        Ok(())
    }

    pub async fn autocommit_flow(&self) -> Result<()> {
        println!("🚀 TEIMAS Release Committer (TERCO) - Auto-commit Flow");
        println!("🧠 Running comprehensive AI analysis...");

        // Run comprehensive analysis directly without TUI state management
        let analysis_result = self.run_comprehensive_analysis_cli().await?;

        println!("✅ AI analysis completed successfully!");

        // Create a new app instance for the commit editor
        let mut app = App::new().await?;

        // Populate form with AI analysis results
        if let Some(title) = analysis_result.get("title").and_then(|v| v.as_str()) {
            if !title.is_empty() {
                app.commit_form.title = title.to_string();
            }
        }

        if let Some(commit_type) = analysis_result.get("commitType").and_then(|v| v.as_str()) {
            if !commit_type.is_empty() {
                use crate::types::CommitType;
                let commit_type_enum = match commit_type {
                    "feat" => Some(CommitType::Feat),
                    "fix" => Some(CommitType::Fix),
                    "docs" => Some(CommitType::Docs),
                    "style" => Some(CommitType::Style),
                    "refactor" => Some(CommitType::Refactor),
                    "perf" => Some(CommitType::Perf),
                    "test" => Some(CommitType::Test),
                    "chore" => Some(CommitType::Chore),
                    "revert" => Some(CommitType::Revert),
                    _ => None,
                };

                if let Some(ct) = commit_type_enum {
                    app.commit_form.commit_type = Some(ct.clone());
                    // Update UI state to reflect the selected commit type
                    let commit_types = CommitType::all();
                    if let Some(index) = commit_types.iter().position(|t| *t == ct) {
                        app.ui_state.selected_commit_type = index;
                    }
                }
            }
        }

        if let Some(description) = analysis_result.get("description").and_then(|v| v.as_str()) {
            if !description.is_empty() {
                app.commit_form.description = description.to_string();
            }
        }

        if let Some(scope) = analysis_result.get("scope").and_then(|v| v.as_str()) {
            if !scope.is_empty() && scope != "general" {
                app.commit_form.scope = scope.to_string();
            }
        }

        if let Some(security) = analysis_result
            .get("securityAnalysis")
            .and_then(|v| v.as_str())
        {
            if !security.is_empty() {
                app.commit_form.security = security.to_string();
            }
        }

        if let Some(breaking) = analysis_result
            .get("breakingChanges")
            .and_then(|v| v.as_str())
        {
            if !breaking.is_empty() {
                app.commit_form.breaking_change = breaking.to_string();
            }
        }

        if let Some(test_analysis) = analysis_result.get("testAnalysis").and_then(|v| v.as_str()) {
            if !test_analysis.is_empty() {
                app.commit_form.test_details = test_analysis.to_string();
            }
        }

        // Generate commit message preview
        use crate::app::commit_operations::CommitOperations;
        app.preview_commit_message = app.build_commit_message();

        // Set screen to commit preview (like pressing 'c')
        app.current_screen = crate::types::AppScreen::CommitPreview;
        app.ui_state.input_mode = crate::ui::state::InputMode::Editing;
        app.ui_state.current_input = app.preview_commit_message.clone();
        app.ui_state.cursor_position = app.preview_commit_message.len();

        println!("📝 Opening commit editor...");

        // Run the TUI starting on the commit preview screen
        app.run().await?;

        Ok(())
    }

    /// CLI-only comprehensive analysis that doesn't involve TUI state management
    async fn run_comprehensive_analysis_cli(&self) -> Result<serde_json::Value> {
        use crate::git::GitRepo;
        use crate::services::GeminiClient;

        println!("🔍 Analyzing git repository changes...");

        // Get git changes
        let git_repo = GitRepo::new()?;
        let changes = git_repo.get_detailed_changes()?;

        if changes.trim().is_empty() || changes.contains("No hay cambios detectados") {
            return Err(anyhow::anyhow!("No git changes found to analyze"));
        }

        println!("🌐 Connecting to Gemini AI...");

        // Create Gemini client and run analysis
        let gemini_client = GeminiClient::new(&self.config)?;

        println!("🧠 Generating comprehensive commit analysis...");

        let result = gemini_client
            .generate_comprehensive_commit_analysis(&changes)
            .await?;

        Ok(result)
    }

    pub async fn search_tasks(&self, query: &str) -> Result<()> {
        println!("🔍 Searching Monday.com tasks for: {}", query);

        let client = MondayClient::new(&self.config)?;
        let tasks = client.search_tasks(query).await?;

        println!("📋 Found {} tasks:", tasks.len());
        for task in tasks {
            println!(
                "  • {} [{}] (ID: {})",
                task.title,
                task.state.to_uppercase(),
                task.id
            );
            println!("    State: {}", task.state);
            if let Some(board_name) = task.board_name {
                println!("    Board: {}", board_name);
            }
            println!();
        }

        Ok(())
    }

    // Debug methods for troubleshooting
    pub async fn debug_monday(&self) -> Result<()> {
        println!("🔍 Debug: Testing Monday.com connection...");

        if self.config.monday_api_key.is_none() {
            println!("❌ No Monday.com API key configured");
            return Ok(());
        }

        println!("✅ Monday.com API key: Configured");
        println!(
            "✅ Account slug: {}",
            self.config
                .monday_account_slug
                .as_deref()
                .unwrap_or("Not set")
        );

        let client = MondayClient::new(&self.config)?;
        match client.test_connection().await {
            Ok(response) => {
                println!("✅ Monday.com connection: SUCCESS");
                println!("📋 Response: {}", response);
            }
            Err(e) => {
                println!("❌ Monday.com connection: FAILED");
                println!("🔍 Error details: {}", e);
            }
        }

        Ok(())
    }

    pub async fn debug_gemini(&self) -> Result<()> {
        println!("🤖 Debug: Testing Gemini AI connection...");

        if self.config.gemini_token.is_none() {
            println!("❌ No Gemini API token configured");
            return Ok(());
        }

        println!("✅ Gemini API token: Configured");

        match crate::services::test_gemini_connection(&self.config).await {
            Ok(response) => {
                println!("✅ Gemini connection: SUCCESS");
                println!("🤖 Response: {}", response);
            }
            Err(e) => {
                println!("❌ Gemini connection: FAILED");
                println!("🔍 Error details: {}", e);
            }
        }

        Ok(())
    }

    pub async fn debug_git(&self) -> Result<()> {
        println!("📂 Debug: Testing Git repository...");

        match GitRepo::new() {
            Ok(repo) => {
                println!("✅ Git repository: Found");

                match repo.get_current_branch() {
                    Ok(branch) => println!("📍 Current branch: {}", branch),
                    Err(e) => println!("❌ Could not get current branch: {}", e),
                }

                match repo.get_status() {
                    Ok(status) => {
                        println!("📊 Repository status:");
                        println!("  • Modified files: {}", status.modified.len());
                        println!("  • Staged files: {}", status.staged.len());
                        println!("  • Untracked files: {}", status.untracked.len());

                        if !status.staged.is_empty() {
                            println!("📝 Staged files:");
                            for file in &status.staged {
                                println!("    + {}", file);
                            }
                        }

                        if !status.modified.is_empty() {
                            println!("📝 Modified files:");
                            for file in &status.modified {
                                println!("    ~ {}", file);
                            }
                        }
                    }
                    Err(e) => println!("❌ Could not get repository status: {}", e),
                }
            }
            Err(e) => {
                println!("❌ Git repository: NOT FOUND");
                println!("🔍 Error: {}", e);
                println!("💡 Make sure you're in a git repository directory");
            }
        }

        Ok(())
    }

    pub async fn debug_commit(&self) -> Result<()> {
        use crate::app::commit_operations::CommitOperations;

        println!("💾 Debug: Testing commit creation...");

        // Check git repository
        println!("\n1. Checking Git repository...");
        let git_repo = match GitRepo::new() {
            Ok(repo) => {
                println!("✅ Git repository found");
                repo
            }
            Err(e) => {
                println!("❌ Git repository error: {}", e);
                return Ok(());
            }
        };

        // Check repository status
        println!("\n2. Checking repository status...");
        match git_repo.get_status() {
            Ok(status) => {
                if status.staged.is_empty() && status.modified.is_empty() {
                    println!("⚠️  No changes to commit");
                    println!("💡 Try making some changes and staging them with 'git add .'");
                    return Ok(());
                }

                if status.staged.is_empty() {
                    println!("⚠️  No staged changes found");
                    println!("💡 Stage your changes with 'git add .' first");

                    println!("\n📝 Available modified files:");
                    for file in &status.modified {
                        println!("    ~ {}", file);
                    }
                    return Ok(());
                }

                println!("✅ Found {} staged changes", status.staged.len());
                for file in &status.staged {
                    println!("    + {}", file);
                }
            }
            Err(e) => {
                println!("❌ Could not check repository status: {}", e);
                return Ok(());
            }
        }

        // Test commit message building
        println!("\n3. Testing commit message...");
        if self.commit_form.commit_type.is_none() {
            println!("⚠️  No commit type selected in current form");
        } else {
            println!("✅ Commit type: {:?}", self.commit_form.commit_type);
        }

        if self.commit_form.title.trim().is_empty() {
            println!("⚠️  No commit title in current form");
        } else {
            println!("✅ Commit title: {}", self.commit_form.title);
        }

        let commit_message = self.build_commit_message();
        println!("\n📝 Generated commit message:");
        println!("---");
        println!("{}", commit_message);
        println!("---");

        println!("\n✅ Debug complete! Use this information to troubleshoot commit issues.");

        Ok(())
    }
}
