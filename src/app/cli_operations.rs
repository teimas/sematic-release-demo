use anyhow::Result;
use std::path::Path;
use std::fs;
use chrono::Utc;

use crate::{
    app::App,
    git::{GitRepo, get_next_version},
    services::MondayClient,
    services::GeminiClient,
};

impl App {
    // CLI methods for direct command usage
    pub async fn commit_flow(&self) -> Result<()> {
        println!("🚀 Semantic Release TUI - Commit Flow");
        println!("This would open the TUI commit interface");
        Ok(())
    }

    pub async fn generate_release_notes(&self) -> Result<()> {
        println!("📝 Generating release notes...");
        
        // Get version
        let version = get_next_version().unwrap_or_else(|_| "next".to_string());
        println!("📦 Version: {}", version);
        
        // Get git repository and commits
        let git_repo = GitRepo::new()?;
        let last_tag = git_repo.get_last_tag()?;
        let commits = git_repo.get_commits_since_tag(last_tag.as_deref())?;
        
        println!("📋 Found {} commits since last tag", commits.len());
        
        // Extract Monday task IDs from commits
        let mut task_ids = Vec::new();
        for commit in &commits {
            // Add task IDs from monday_tasks field
            task_ids.extend(commit.monday_tasks.clone());
            
            // Add task IDs from monday_task_mentions
            for mention in &commit.monday_task_mentions {
                task_ids.push(mention.id.clone());
            }
            
            // Also check the scope for task IDs (pipe-separated)
            if let Some(scope) = &commit.scope {
                let scope_ids: Vec<String> = scope.split('|')
                    .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
                    .map(|id| id.to_string())
                    .collect();
                task_ids.extend(scope_ids);
            }
        }
        task_ids.sort();
        task_ids.dedup();
        
        // Get Monday task details
        let monday_tasks = if !task_ids.is_empty() && self.config.monday_api_key.is_some() {
            println!("🔍 Fetching Monday.com task details...");
            let client = MondayClient::new(&self.config)?;
            client.get_task_details(&task_ids).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        
        println!("📋 Found {} related Monday.com tasks", monday_tasks.len());
        
        // Extract responsible person from most recent commit author
        let responsible_person = if !commits.is_empty() {
            commits[0].author_name.clone()
        } else {
            "".to_string()
        };
        
        // Create release notes directory
        let release_notes_dir = Path::new("release-notes");
        if !release_notes_dir.exists() {
            fs::create_dir_all(release_notes_dir)?;
        }
        
        // Generate structured document
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let script_file = release_notes_dir.join(format!("release-notes-{}_SCRIPT.md", date));
        let ai_file = release_notes_dir.join(format!("release-notes-{}_GEMINI.md", date));
        
        use crate::app::release_notes::*;
        let script_content = self.generate_raw_release_notes(&version, &commits, &monday_tasks, &responsible_person);
        fs::write(&script_file, &script_content)?;
        println!("✅ Script release notes saved to: {}", script_file.display());
        
        // Generate AI release notes if Gemini is configured
        if self.config.gemini_token.is_some() {
            println!("🤖 Generating AI-powered release notes...");
            let gemini_client = GeminiClient::new(&self.config)?;
            match gemini_client.generate_release_notes(&version, &commits, &monday_tasks).await {
                Ok(ai_content) => {
                    fs::write(&ai_file, &ai_content)?;
                    println!("✅ AI release notes saved to: {}", ai_file.display());
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate AI release notes: {}", e);
                }
            }
        } else {
            println!("⚠️  Google Gemini not configured. Skipping AI generation.");
        }
        
        Ok(())
    }

    pub async fn search_tasks(&self, query: &str) -> Result<()> {
        println!("🔍 Searching Monday.com tasks for: {}", query);
        
        let client = MondayClient::new(&self.config)?;
        let tasks = client.search_tasks(query).await?;
        
        println!("📋 Found {} tasks:", tasks.len());
        for task in tasks {
            println!("  • {} [{}] (ID: {})", task.title, task.state.to_uppercase(), task.id);
            println!("    URL: {}", task.url);
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
        println!("✅ Account slug: {}", self.config.monday_account_slug.as_deref().unwrap_or("Not set"));
        
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