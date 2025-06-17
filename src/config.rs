use anyhow::Result;
use dialoguer::{Input, Password, Select};
use dirs::home_dir;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::types::AppConfig;

pub fn get_env_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // First priority: .env in current directory (same as original project)
    paths.push(PathBuf::from(".env"));

    // Second priority: .env in home directory for global config
    if let Some(home) = home_dir() {
        paths.push(home.join(".env"));
    }

    paths
}

pub fn load_config() -> Result<AppConfig> {
    let env_paths = get_env_paths();

    // Try to load from each .env file in order of priority
    for env_path in &env_paths {
        if env_path.exists() {
            return load_config_from_env(env_path);
        }
    }

    // If no .env file exists, try to load from environment variables
    load_config_from_env_vars()
}

fn load_config_from_env(env_path: &Path) -> Result<AppConfig> {
    dotenv::from_path(env_path).ok(); // Load .env file into environment
    load_config_from_env_vars()
}

fn load_config_from_env_vars() -> Result<AppConfig> {
    Ok(AppConfig {
        monday_api_key: env::var("MONDAY_API_KEY").ok(),
        monday_account_slug: env::var("ACCOUNT_SLUG").ok(),
        monday_board_id: env::var("MONDAY_BOARD_ID").ok(),
        monday_url_template: env::var("MONDAY_URL_TEMPLATE").ok(),
        jira_url: env::var("JIRA_URL").ok(),
        jira_username: env::var("JIRA_USERNAME").ok(),
        jira_api_token: env::var("JIRA_API_TOKEN").ok(),
        jira_project_key: env::var("JIRA_PROJECT_KEY").ok(),
        gemini_token: env::var("GEMINI_TOKEN").ok(),
    })
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    // Save to .env in current directory (same as original project)
    let env_path = PathBuf::from(".env");
    save_config_to_env(&env_path, config)
}

fn save_config_to_env(env_path: &Path, config: &AppConfig) -> Result<()> {
    let mut env_content = String::new();

    // Load existing .env content if it exists
    if env_path.exists() {
        let existing_content = fs::read_to_string(env_path)?;
        let mut lines: Vec<String> = existing_content.lines().map(|s| s.to_string()).collect();

        // Remove existing keys that we're about to set
        lines.retain(|line| {
            !line.starts_with("MONDAY_API_KEY=")
                && !line.starts_with("ACCOUNT_SLUG=")
                && !line.starts_with("MONDAY_BOARD_ID=")
                && !line.starts_with("MONDAY_URL_TEMPLATE=")
                && !line.starts_with("JIRA_URL=")
                && !line.starts_with("JIRA_USERNAME=")
                && !line.starts_with("JIRA_API_TOKEN=")
                && !line.starts_with("JIRA_PROJECT_KEY=")
                && !line.starts_with("GEMINI_TOKEN=")
        });

        env_content = lines.join("\n");
        if !env_content.is_empty() && !env_content.ends_with('\n') {
            env_content.push('\n');
        }
    }

    // Add our configuration
    if let Some(api_key) = &config.monday_api_key {
        env_content.push_str(&format!("MONDAY_API_KEY={}\n", api_key));
    }

    if let Some(account_slug) = &config.monday_account_slug {
        env_content.push_str(&format!("ACCOUNT_SLUG={}\n", account_slug));
    }

    if let Some(board_id) = &config.monday_board_id {
        env_content.push_str(&format!("MONDAY_BOARD_ID={}\n", board_id));
    }

    if let Some(url_template) = &config.monday_url_template {
        env_content.push_str(&format!("MONDAY_URL_TEMPLATE={}\n", url_template));
    }

    if let Some(jira_url) = &config.jira_url {
        env_content.push_str(&format!("JIRA_URL={}\n", jira_url));
    }

    if let Some(jira_username) = &config.jira_username {
        env_content.push_str(&format!("JIRA_USERNAME={}\n", jira_username));
    }

    if let Some(jira_api_token) = &config.jira_api_token {
        env_content.push_str(&format!("JIRA_API_TOKEN={}\n", jira_api_token));
    }

    if let Some(jira_project_key) = &config.jira_project_key {
        env_content.push_str(&format!("JIRA_PROJECT_KEY={}\n", jira_project_key));
    }

    if let Some(gemini_token) = &config.gemini_token {
        env_content.push_str(&format!("GEMINI_TOKEN={}\n", gemini_token));
    }

    fs::write(env_path, env_content)?;
    Ok(())
}

pub async fn run_config() -> Result<()> {
    println!("📚 Semantic Release TUI Configuration");
    println!("=====================================");

    let current_config = load_config().unwrap_or_default();

    // Determine which task system to configure
    let task_system_options = vec!["Monday.com", "JIRA"];

    let current_system = current_config.get_task_system();
    let default_selection = match current_system {
        crate::types::TaskSystem::Monday => 0,
        crate::types::TaskSystem::Jira => 1,
        crate::types::TaskSystem::None => 0, // Default to Monday
    };

    let selection = Select::new()
        .with_prompt("Choose task management system (Monday.com and JIRA are mutually exclusive):")
        .items(&task_system_options)
        .default(default_selection)
        .interact()?;

    let mut config = AppConfig::default();

    match selection {
        0 => {
            // Monday.com configuration
            println!("\n🔵 Configuring Monday.com integration...");

            let monday_api_key = if current_config.monday_api_key.is_some() {
                let update: bool = dialoguer::Confirm::new()
                    .with_prompt("Monday.com API key is already configured. Update it?")
                    .default(false)
                    .interact()?;

                if update {
                    Some(
                        Password::new()
                            .with_prompt("Enter your Monday.com API key")
                            .interact()?,
                    )
                } else {
                    current_config.monday_api_key
                }
            } else {
                Some(
                    Password::new()
                        .with_prompt("Enter your Monday.com API key")
                        .interact()?,
                )
            };

            let monday_account_slug = Input::new()
                .with_prompt("Monday.com account slug (subdomain)")
                .default(current_config.monday_account_slug.unwrap_or_default())
                .interact_text()?;

            let monday_board_id = Input::new()
                .with_prompt("Monday.com board ID (optional)")
                .default(current_config.monday_board_id.unwrap_or_default())
                .allow_empty(true)
                .interact_text()?;

            let monday_url_template = if !monday_account_slug.is_empty() {
                Some(format!(
                    "https://{}.monday.com/boards/{{board_id}}/pulses/{{item_id}}",
                    monday_account_slug
                ))
            } else {
                None
            };

            config.monday_api_key = monday_api_key;
            config.monday_account_slug = if monday_account_slug.is_empty() {
                None
            } else {
                Some(monday_account_slug)
            };
            config.monday_board_id = if monday_board_id.is_empty() {
                None
            } else {
                Some(monday_board_id)
            };
            config.monday_url_template = monday_url_template;
        }
        1 => {
            // JIRA configuration
            println!("\n🟦 Configuring JIRA integration...");

            let jira_url = Input::new()
                .with_prompt("JIRA instance URL (e.g., https://yourcompany.atlassian.net)")
                .default(current_config.jira_url.unwrap_or_default())
                .interact_text()?;

            let jira_username = Input::new()
                .with_prompt("JIRA username/email")
                .default(current_config.jira_username.unwrap_or_default())
                .interact_text()?;

            let jira_api_token = if current_config.jira_api_token.is_some() {
                let update: bool = dialoguer::Confirm::new()
                    .with_prompt("JIRA API token is already configured. Update it?")
                    .default(false)
                    .interact()?;

                if update {
                    Some(
                        Password::new()
                            .with_prompt("Enter your JIRA API token")
                            .interact()?,
                    )
                } else {
                    current_config.jira_api_token
                }
            } else {
                Some(
                    Password::new()
                        .with_prompt("Enter your JIRA API token")
                        .interact()?,
                )
            };

            let jira_project_key = Input::new()
                .with_prompt("JIRA project key (optional, leave empty for global search)")
                .default(current_config.jira_project_key.unwrap_or_default())
                .allow_empty(true)
                .interact_text()?;

            config.jira_url = if jira_url.is_empty() {
                None
            } else {
                Some(jira_url)
            };
            config.jira_username = if jira_username.is_empty() {
                None
            } else {
                Some(jira_username)
            };
            config.jira_api_token = jira_api_token;
            config.jira_project_key = if jira_project_key.is_empty() {
                None
            } else {
                Some(jira_project_key)
            };
        }
        _ => unreachable!(),
    }

    // Configure Gemini AI (common for both)
    let gemini_token = if current_config.gemini_token.is_some() {
        let update: bool = dialoguer::Confirm::new()
            .with_prompt("Google Gemini API token is already configured. Update it?")
            .default(false)
            .interact()?;

        if update {
            Some(
                Password::new()
                    .with_prompt("Enter your Google Gemini API token")
                    .allow_empty_password(true)
                    .interact()?,
            )
        } else {
            current_config.gemini_token
        }
    } else {
        let token = Password::new()
            .with_prompt("Enter your Google Gemini API token (optional)")
            .allow_empty_password(true)
            .interact()?;
        if token.is_empty() {
            None
        } else {
            Some(token)
        }
    };

    config.gemini_token = gemini_token;

    save_config(&config)?;

    println!("✅ Configuration saved successfully!");

    // Test connections based on chosen system
    match config.get_task_system() {
        crate::types::TaskSystem::Monday => {
            if config.monday_api_key.is_some() {
                println!("🔍 Testing Monday.com connection...");
                match test_monday_connection(&config).await {
                    Ok(user_info) => println!(
                        "✅ Monday.com connection successful! Welcome, {}",
                        user_info
                    ),
                    Err(e) => println!("⚠️  Monday.com connection test failed: {}", e),
                }
            }
        }
        crate::types::TaskSystem::Jira => {
            if config.is_jira_configured() {
                println!("🔍 Testing JIRA connection...");
                match test_jira_connection(&config).await {
                    Ok(response) => println!("✅ JIRA connection successful! {}", response),
                    Err(e) => println!("⚠️  JIRA connection test failed: {}", e),
                }
            }
        }
        crate::types::TaskSystem::None => {
            println!("⚠️  No task management system configured");
        }
    }

    Ok(())
}

async fn test_monday_connection(config: &AppConfig) -> Result<String> {
    if let Some(api_key) = &config.monday_api_key {
        let client = reqwest::Client::new();
        let query = r#"{"query": "query { me { name email } }"}"#;

        let response = client
            .post("https://api.monday.com/v2")
            .header("Authorization", api_key)
            .header("Content-Type", "application/json")
            .header("API-Version", "2024-10")
            .body(query)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        if let Some(me) = result["data"]["me"].as_object() {
            let name = me["name"].as_str().unwrap_or("Unknown");
            let email = me["email"].as_str().unwrap_or("Unknown");
            Ok(format!("{} ({})", name, email))
        } else {
            Err(anyhow::anyhow!("Invalid response from Monday.com API"))
        }
    } else {
        Err(anyhow::anyhow!("No Monday.com API key configured"))
    }
}

async fn test_jira_connection(config: &AppConfig) -> Result<String> {
    use crate::services::JiraClient;

    let client = JiraClient::new(config)?;
    client.test_connection().await
}

pub async fn setup_commit_template() -> Result<()> {
    println!("🚀 TEIMAS Semantic Release - Git Commit Template Setup");
    println!("======================================================");
    println!();

    // Define the template path
    let template_path = if let Some(home) = home_dir() {
        home.join(".gitmessage")
    } else {
        return Err(anyhow::anyhow!("Could not determine home directory"));
    };

    // Create the commit template content
    let template_content = r#"# Commit Type and Scope
# Format: type(scope): subject
# Types: feat, fix, docs, style, refactor, perf, test, chore, revert
# Scope: Component or area affected (use N/A if not applicable)
type(scope):

# Detailed Description
# Explain what and why vs how
# Use present tense: "change" not "changed" nor "changes"


# Breaking Changes
# List any breaking changes or N/A if none
BREAKING CHANGE:

# Test Details
# Describe testing performed or N/A if none
Test Details:

# Security Considerations
# List security implications or N/A if none
Security:

# Migraciones Lentas
# Describe slow migrations or N/A if none
Migraciones Lentas:

# Partes a Ejecutar
# List deployment steps or N/A if none
Partes a Ejecutar:

# Related Tasks
# List related Monday.com or JIRA tasks or N/A if none
# Monday format: - Task Title (ID: task_id) - Status
# JIRA format: - Task Title (Key: PROJ-123) - Status
RELATED TASKS:


# ────────────────────────────────────────────────────────────────────────────
# COMMIT MESSAGE TEMPLATE GUIDELINES
# ────────────────────────────────────────────────────────────────────────────
#
# Subject Line (First Line):
# • Keep under 50 characters
# • Use imperative mood: "Add feature" not "Added feature"
# • Don't end with a period
# • Be concise but descriptive
#
# Commit Types:
# • feat:     New feature for the user
# • fix:      Bug fix for the user
# • docs:     Documentation changes
# • style:    Code style changes (formatting, etc)
# • refactor: Code changes that neither fix bugs nor add features
# • perf:     Performance improvements
# • test:     Adding or fixing tests
# • chore:    Build process or auxiliary tools changes
# • revert:   Revert to a commit
#
# Body Guidelines:
# • Separate subject from body with a blank line
# • Use the body to explain what and why vs how
# • Each line should be under 72 characters
# • Use present tense: "change" not "changed" nor "changes"
#
# All Fields Required:
# • Use "N/A" for any field that doesn't apply
# • This ensures consistent commit structure across all commits
# • Makes automated parsing and analysis possible
#
# Examples:
# feat(auth): Add JWT authentication system
# fix(api): Resolve null pointer in user service
# docs(readme): Update installation instructions
# test(user): Add unit tests for user validation
#
# Lines starting with # are comments and will be ignored
# ────────────────────────────────────────────────────────────────────────────
"#;

    // Write the template file
    fs::write(&template_path, template_content)?;
    println!("📝 Created commit template at: {}", template_path.display());
    println!();

    // Ask user for setup type
    let setup_options = vec![
        "Global (all repositories on this machine)",
        "Local (current repository only)",
        "Both (global + local)",
    ];

    let selection = Select::new()
        .with_prompt("Choose setup type:")
        .items(&setup_options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Global setup
            setup_git_config_global(&template_path)?;
            println!("✅ Global git commit template configured");
            println!("💡 This will apply to all repositories on this machine");
        }
        1 => {
            // Local setup
            setup_git_config_local(&template_path)?;
        }
        2 => {
            // Both
            setup_git_config_global(&template_path)?;
            match setup_git_config_local(&template_path) {
                Ok(_) => {
                    println!("✅ Both global and local git commit templates configured");
                }
                Err(_) => {
                    println!("✅ Global git commit template configured");
                    println!("⚠️  Local configuration skipped (not in a git repository)");
                }
            }
        }
        _ => unreachable!(),
    }

    println!();
    println!("🎉 Setup complete!");
    println!();
    println!("📋 How to use:");
    println!("• Run 'git commit' (without -m) to open editor with template");
    println!("• Fill in the template fields, replacing placeholders with actual content");
    println!("• Use 'N/A' for fields that don't apply");
    println!("• The TEIMAS Semantic Release TUI will also follow this same structure");
    println!();
    println!("🔧 To disable template:");
    println!("• Global: git config --global --unset commit.template");
    println!("• Local:  git config --unset commit.template");
    println!();
    println!("✨ Happy committing with consistent messages!");

    Ok(())
}

fn setup_git_config_global(template_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args([
            "config",
            "--global",
            "commit.template",
            &template_path.to_string_lossy(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to set global git config: {}",
            stderr
        ));
    }

    Ok(())
}

fn setup_git_config_local(template_path: &Path) -> Result<()> {
    // Check if we're in a git repository
    let git_check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;

    if !git_check.status.success() {
        return Err(anyhow::anyhow!("Not in a git repository. Please run this command from inside a git repository or choose global setup."));
    }

    let output = Command::new("git")
        .args([
            "config",
            "commit.template",
            &template_path.to_string_lossy(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to set local git config: {}",
            stderr
        ));
    }

    println!("✅ Local git commit template configured");
    println!("💡 This will apply only to the current repository");

    Ok(())
}
