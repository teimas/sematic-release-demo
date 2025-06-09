use anyhow::Result;
use dirs::home_dir;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use dialoguer::{Input, Password};

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
            !line.starts_with("MONDAY_API_KEY=") &&
            !line.starts_with("ACCOUNT_SLUG=") &&
            !line.starts_with("MONDAY_BOARD_ID=") &&
            !line.starts_with("MONDAY_URL_TEMPLATE=") &&
            !line.starts_with("GEMINI_TOKEN=")
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
    
    let monday_api_key = if current_config.monday_api_key.is_some() {
        let update: bool = dialoguer::Confirm::new()
            .with_prompt("Monday.com API key is already configured. Update it?")
            .default(false)
            .interact()?;
            
        if update {
            Some(Password::new()
                .with_prompt("Enter your Monday.com API key")
                .interact()?)
        } else {
            current_config.monday_api_key
        }
    } else {
        Some(Password::new()
            .with_prompt("Enter your Monday.com API key")
            .interact()?)
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
    
    let gemini_token = if current_config.gemini_token.is_some() {
        let update: bool = dialoguer::Confirm::new()
            .with_prompt("Google Gemini API token is already configured. Update it?")
            .default(false)
            .interact()?;
            
        if update {
            Some(Password::new()
                .with_prompt("Enter your Google Gemini API token")
                .allow_empty_password(true)
                .interact()?)
        } else {
            current_config.gemini_token
        }
    } else {
        let token = Password::new()
            .with_prompt("Enter your Google Gemini API token (optional)")
            .allow_empty_password(true)
            .interact()?;
        if token.is_empty() { None } else { Some(token) }
    };
    
    let monday_url_template = if !monday_account_slug.is_empty() {
        Some(format!("https://{}.monday.com/boards/{{board_id}}/pulses/{{item_id}}", monday_account_slug))
    } else {
        None
    };
    
    let config = AppConfig {
        monday_api_key,
        monday_account_slug: if monday_account_slug.is_empty() { None } else { Some(monday_account_slug) },
        monday_board_id: if monday_board_id.is_empty() { None } else { Some(monday_board_id) },
        monday_url_template,
        gemini_token,
    };
    
    save_config(&config)?;
    
    println!("✅ Configuration saved successfully!");
    
    // Test Monday.com connection
    if config.monday_api_key.is_some() {
        println!("🔍 Testing Monday.com connection...");
        match test_monday_connection(&config).await {
            Ok(user_info) => println!("✅ Monday.com connection successful! Welcome, {}", user_info),
            Err(e) => println!("⚠️  Monday.com connection test failed: {}", e),
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