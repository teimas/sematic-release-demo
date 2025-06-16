use anyhow::Result;
use log::{debug, info, error};

use crate::{
    app::App,
    git::GitRepo,
};

pub trait CommitOperations {
    fn build_commit_message(&self) -> String;
    async fn create_commit_with_message(&self, message: &str) -> Result<()>;
}

impl CommitOperations for App {
    fn build_commit_message(&self) -> String {
        let mut message = String::new();
        
        // Type and scope
        if let Some(commit_type) = &self.commit_form.commit_type {
            message.push_str(commit_type.as_str());
            
            if !self.commit_form.scope.is_empty() {
                message.push_str(&format!("({})", self.commit_form.scope));
            }
            
            message.push_str(": ");
        }
        
        // Title
        message.push_str(&self.commit_form.title);
        
        // Body
        if !self.commit_form.description.is_empty() {
            message.push_str("\n\n");
            message.push_str(&self.commit_form.description);
        }
        
        // Breaking changes
        if !self.commit_form.breaking_change.is_empty() {
            message.push_str("\n\nBREAKING CHANGE: ");
            message.push_str(&self.commit_form.breaking_change);
        }
        
        // Test details
        if !self.commit_form.test_details.is_empty() {
            message.push_str("\n\nTest Details: ");
            message.push_str(&self.commit_form.test_details);
        }
        
        // Security
        if !self.commit_form.security.is_empty() {
            message.push_str("\n\nSecurity: ");
            message.push_str(&self.commit_form.security);
        }
        
        // Migraciones Lentas
        if !self.commit_form.migraciones_lentas.is_empty() {
            message.push_str("\n\nMigraciones Lentas: ");
            message.push_str(&self.commit_form.migraciones_lentas);
        }
        
        // Partes a Ejecutar
        if !self.commit_form.partes_a_ejecutar.is_empty() {
            message.push_str("\n\nPartes a Ejecutar: ");
            message.push_str(&self.commit_form.partes_a_ejecutar);
        }
        
        // Monday.com tasks
        if !self.commit_form.selected_tasks.is_empty() {
            message.push_str("\n\nMONDAY TASKS:\n");
            for task in &self.commit_form.selected_tasks {
                message.push_str(&format!("- {} (ID: {}) - {}\n", task.title, task.id, task.state));
            }
        }
        
        message
    }

    async fn create_commit_with_message(&self, message: &str) -> Result<()> {
        debug!("Creating commit with custom message");
        
        debug!("Initializing git repository...");
        let git_repo = match GitRepo::new() {
            Ok(repo) => {
                info!("Git repository initialized successfully");
                repo
            }
            Err(e) => {
                error!("Failed to initialize git repository: {}", e);
                return Err(anyhow::anyhow!("Git repository error: {}. Make sure you're in a git repository.", e));
            }
        };
        
        info!("Creating commit with message:\n{}", message);
        
        // Create the commit
        debug!("Creating git commit...");
        match git_repo.create_commit(message) {
            Ok(_) => {
                info!("Commit created successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to create commit: {}", e);
                Err(anyhow::anyhow!("Failed to create commit: {}. Make sure you have staged changes or there are changes to commit.", e))
            }
        }
    }


} 