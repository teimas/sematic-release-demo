use crate::error::Result;
use tracing::{debug, error, info, instrument};

use crate::{app::App, git::repository::GitRepo};

#[allow(async_fn_in_trait)]
pub trait CommitOperations {
    fn build_commit_message(&self) -> String;
    async fn create_commit_with_message(&self, message: &str) -> Result<()>;
}

impl CommitOperations for App {
    #[instrument(skip(self))]
    fn build_commit_message(&self) -> String {
        debug!("Building commit message from form data");

        let mut message = String::new();

        // Type and scope
        if let Some(commit_type) = &self.commit_form.commit_type {
            message.push_str(commit_type.as_str());

            if !self.commit_form.scope.is_empty() {
                message.push_str(&format!("({})", self.commit_form.scope));
            } else {
                message.push_str("(N/A)");
            }

            message.push_str(": ");
        }

        // Title
        if !self.commit_form.title.is_empty() {
            message.push_str(&self.commit_form.title);
        } else {
            message.push_str("N/A");
        }

        // Body/Description
        message.push_str("\n\n");
        if !self.commit_form.description.is_empty() {
            message.push_str(&self.commit_form.description);
        } else {
            message.push_str("N/A");
        }

        // Breaking changes - only include if there are actual breaking changes
        if !self.commit_form.breaking_change.is_empty() {
            message.push_str("\n\nBREAKING CHANGE: ");
            message.push_str(&self.commit_form.breaking_change);
        }

        // Test details
        message.push_str("\n\nTest Details: ");
        if !self.commit_form.test_details.is_empty() {
            message.push_str(&self.commit_form.test_details);
        } else {
            message.push_str("N/A");
        }

        // Security
        message.push_str("\n\nSecurity: ");
        if !self.commit_form.security.is_empty() {
            message.push_str(&self.commit_form.security);
        } else {
            message.push_str("N/A");
        }

        // Migraciones Lentas
        message.push_str("\n\nMigraciones Lentas: ");
        if !self.commit_form.migraciones_lentas.is_empty() {
            message.push_str(&self.commit_form.migraciones_lentas);
        } else {
            message.push_str("N/A");
        }

        // Partes a Ejecutar
        message.push_str("\n\nPartes a Ejecutar: ");
        if !self.commit_form.partes_a_ejecutar.is_empty() {
            message.push_str(&self.commit_form.partes_a_ejecutar);
        } else {
            message.push_str("N/A");
        }

        // Task system section - dynamic based on configuration
        match self.config.get_task_system() {
            crate::types::TaskSystem::Monday => {
                message.push_str("\n\nMONDAY TASKS: ");
                if !self.commit_form.selected_monday_tasks.is_empty() {
                    message.push('\n');
                    for task in &self.commit_form.selected_monday_tasks {
                        message.push_str(&format!(
                            "- {} (ID: {}) - {}\n",
                            task.title, task.id, task.state
                        ));
                    }
                } else {
                    message.push_str("N/A");
                }
            }
            crate::types::TaskSystem::Jira => {
                message.push_str("\n\nJIRA TASKS: ");
                if !self.commit_form.selected_jira_tasks.is_empty() {
                    message.push('\n');
                    for task in &self.commit_form.selected_jira_tasks {
                        message.push_str(&format!(
                            "- {} (Key: {}) - {}\n",
                            task.summary, task.key, task.status
                        ));
                    }
                } else {
                    message.push_str("N/A");
                }
            }
            crate::types::TaskSystem::None => {
                message.push_str("\n\nRELATED TASKS: N/A");
            }
        }

        debug!(
            message_len = message.len(),
            "Commit message built successfully"
        );
        message
    }

    #[instrument(skip(self), fields(message_len = message.len()))]
    async fn create_commit_with_message(&self, message: &str) -> Result<()> {
        info!("Creating commit with custom message");
        debug!("Initializing git repository...");

        let git_repo = GitRepo::new().map_err(|e| {
            error!(error = %e, "Failed to initialize git repository");
            crate::error::SemanticReleaseError::git_error(e)
        })?;

        info!("Git repository initialized successfully");

        info!("Creating commit with message (length: {})", message.len());
        debug!("Commit message content:\n{}", message);

        // Create the commit
        debug!("Creating git commit...");
        git_repo.create_commit(message).map_err(|e| {
            error!(error = %e, "Failed to create commit");
            crate::error::SemanticReleaseError::git_error(e)
        })?;

        info!("Commit created successfully");
        Ok(())
    }
}
