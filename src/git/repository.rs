use git2::Repository;
use regex::Regex;
use std::process::{Command, Stdio};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    error::{Result, SemanticReleaseError},
    types::GitCommit,
};

// =============================================================================
// CORE GIT REPOSITORY STRUCTURE
// =============================================================================

pub struct GitRepo {
    repo: Repository,
}

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
}

impl GitRepo {
    #[instrument]
    pub fn new() -> Result<Self> {
        debug!("Initializing git repository");
        let repo = Repository::open(".").map_err(|e| {
            error!(error = %e, "Failed to open git repository");
            SemanticReleaseError::GitError(e)
        })?;

        info!("Git repository initialized successfully");
        Ok(Self { repo })
    }
}

// =============================================================================
// COMMIT HISTORY AND RETRIEVAL
// =============================================================================

impl GitRepo {
    #[instrument(skip(self))]
    pub fn get_commits_since_tag(&self, tag: Option<&str>) -> Result<Vec<GitCommit>> {
        info!(?tag, "Retrieving commits since tag");
        let mut commits = Vec::new();

        let mut revwalk = self.repo.revwalk().map_err(|e| {
            error!(error = %e, "Failed to create revwalk");
            SemanticReleaseError::GitError(e)
        })?;

        revwalk.push_head().map_err(|e| {
            error!(error = %e, "Failed to push HEAD to revwalk");
            SemanticReleaseError::GitError(e)
        })?;

        if let Some(tag) = tag {
            if let Ok(tag_obj) = self.repo.revparse_single(tag) {
                if let Err(e) = revwalk.hide(tag_obj.id()) {
                    warn!(tag = %tag, error = %e, "Failed to hide tag in revwalk");
                }
            } else {
                warn!(tag = %tag, "Tag not found in repository");
            }
        }

        for oid in revwalk {
            let oid = oid.map_err(|e| {
                error!(error = %e, "Failed to get OID from revwalk");
                SemanticReleaseError::GitError(e)
            })?;

            let commit = self.repo.find_commit(oid).map_err(|e| {
                error!(oid = %oid, error = %e, "Failed to find commit");
                SemanticReleaseError::GitError(e)
            })?;

            // Skip merge commits
            if commit.parent_count() > 1 {
                debug!(oid = %oid, "Skipping merge commit");
                continue;
            }

            let git_commit = self.build_git_commit_from_raw(oid, &commit)?;
            commits.push(git_commit);
        }

        info!(
            commit_count = commits.len(),
            "Retrieved commits successfully"
        );
        Ok(commits)
    }

    #[instrument(skip(self, commit))]
    fn build_git_commit_from_raw(
        &self,
        oid: git2::Oid,
        commit: &git2::Commit,
    ) -> Result<GitCommit> {
        let message = commit.message().unwrap_or("");
        let lines: Vec<&str> = message.lines().collect();
        let subject = lines.first().unwrap_or(&"").to_string();
        let body = if lines.len() > 1 {
            lines[1..].join("\n")
        } else {
            String::new()
        };

        let monday_tasks = CommitParser::extract_monday_tasks(&body);
        let jira_tasks = CommitParser::extract_jira_tasks(&body);

        debug!(
            hash = %oid,
            subject = %subject,
            monday_tasks_count = monday_tasks.len(),
            jira_tasks_count = jira_tasks.len(),
            "Built git commit from raw data"
        );

        Ok(GitCommit {
            hash: oid.to_string(),
            description: CommitParser::extract_commit_description(&subject),
            commit_type: CommitParser::extract_commit_type(&subject),
            scope: CommitParser::extract_commit_scope(&subject),
            body: body.clone(),
            breaking_changes: CommitParser::extract_breaking_changes(&body),
            monday_tasks,
            jira_tasks,
        })
    }
}

// =============================================================================
// TAG AND VERSION MANAGEMENT
// =============================================================================

impl GitRepo {
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub fn get_last_tag(&self) -> Result<Option<String>> {
        debug!("Getting last git tag");
        // Use git command to get the last tag, as git2 doesn't have a simple way
        let output = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git describe command");
                SemanticReleaseError::command_error(
                    "git describe --tags --abbrev=0",
                    None,
                    e.to_string(),
                )
            })?;

        match output.status.success() {
            true => {
                let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if tag.is_empty() {
                    info!("No tags found in repository");
                    Ok(None)
                } else {
                    info!(tag = %tag, "Found last tag");
                    Ok(Some(tag))
                }
            }
            false => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                debug!(stderr = %stderr, "No tags found or git describe failed");
                Ok(None)
            }
        }
    }
}

// =============================================================================
// COMMIT CREATION AND STAGING
// =============================================================================

impl GitRepo {
    #[instrument(skip(self))]
    pub fn create_commit(&self, message: &str) -> Result<String> {
        info!(message_length = message.len(), "Creating git commit");

        // Use git command for committing
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git commit command");
                SemanticReleaseError::command_error("git commit", None, e.to_string())
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            info!("Git commit created successfully");
            debug!(output = %stdout, "Git commit output");
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            error!(stderr = %stderr, "Git commit failed");
            Err(SemanticReleaseError::command_error(
                "git commit",
                output.status.code(),
                stderr,
            ))
        }
    }

    #[instrument(skip(self))]
    pub fn stage_all(&self) -> Result<String> {
        info!("Staging all changes");

        // Use git command for staging all changes (equivalent to git add -A)
        let output = Command::new("git")
            .args(["add", "-A"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git add command");
                SemanticReleaseError::command_error("git add -A", None, e.to_string())
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            info!("All changes staged successfully");
            debug!(output = %stdout, "Git add output");
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            error!(stderr = %stderr, "Git staging failed");
            Err(SemanticReleaseError::command_error(
                "git add -A",
                output.status.code(),
                stderr,
            ))
        }
    }
}

// =============================================================================
// REPOSITORY STATUS AND INFORMATION
// =============================================================================

impl GitRepo {
    #[instrument(skip(self))]
    pub fn get_current_branch(&self) -> Result<String> {
        debug!("Getting current branch");

        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git branch command");
                SemanticReleaseError::command_error(
                    "git branch --show-current",
                    None,
                    e.to_string(),
                )
            })?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let result = if branch.is_empty() {
                info!("Repository is in detached HEAD state");
                "HEAD".to_string() // Detached HEAD state
            } else {
                info!(branch = %branch, "Retrieved current branch");
                branch
            };
            Ok(result)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            error!(stderr = %stderr, "Failed to get current branch");
            Err(SemanticReleaseError::command_error(
                "git branch --show-current",
                output.status.code(),
                stderr,
            ))
        }
    }

    #[instrument(skip(self))]
    pub fn get_repository_url(&self) -> Result<Option<String>> {
        debug!("Getting repository URL");

        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git remote command");
                SemanticReleaseError::command_error(
                    "git remote get-url origin",
                    None,
                    e.to_string(),
                )
            })?;

        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if url.is_empty() {
                debug!("No remote origin URL found");
                Ok(None)
            } else {
                // Convert SSH URL to HTTPS if needed
                let url = if url.starts_with("git@github.com:") {
                    // Convert git@github.com:user/repo.git to https://github.com/user/repo.git
                    url.replace("git@github.com:", "https://github.com/")
                } else if url.starts_with("https://") && !url.ends_with(".git") {
                    // Ensure .git suffix for consistency
                    format!("{}.git", url)
                } else {
                    url
                };

                // Add git+ prefix if it's an HTTPS URL for npm package.json format
                let formatted_url = if url.starts_with("https://") {
                    format!("git+{}", url)
                } else {
                    url
                };

                info!(url = %formatted_url, "Retrieved repository URL");
                Ok(Some(formatted_url))
            }
        } else {
            debug!("No remote origin found");
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    pub fn get_status(&self) -> Result<GitStatus> {
        debug!("Getting git repository status");
        let mut status = GitStatus {
            staged: Vec::new(),
            modified: Vec::new(),
            untracked: Vec::new(),
        };

        // Get staged files
        let staged_output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git diff --cached command");
                SemanticReleaseError::command_error(
                    "git diff --cached --name-only",
                    None,
                    e.to_string(),
                )
            })?;

        if staged_output.status.success() {
            status.staged = String::from_utf8_lossy(&staged_output.stdout)
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else {
            warn!("Failed to get staged files");
        }

        // Get modified files
        let modified_output = Command::new("git")
            .args(["diff", "--name-only"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git diff command");
                SemanticReleaseError::command_error("git diff --name-only", None, e.to_string())
            })?;

        if modified_output.status.success() {
            status.modified = String::from_utf8_lossy(&modified_output.stdout)
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else {
            warn!("Failed to get modified files");
        }

        // Get untracked files
        let untracked_output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git ls-files command");
                SemanticReleaseError::command_error(
                    "git ls-files --others --exclude-standard",
                    None,
                    e.to_string(),
                )
            })?;

        if untracked_output.status.success() {
            status.untracked = String::from_utf8_lossy(&untracked_output.stdout)
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else {
            warn!("Failed to get untracked files");
        }

        info!(
            staged_count = status.staged.len(),
            modified_count = status.modified.len(),
            untracked_count = status.untracked.len(),
            "Retrieved git status"
        );

        Ok(status)
    }
}

// =============================================================================
// DIFF AND CHANGE ANALYSIS
// =============================================================================

impl GitRepo {
    #[instrument(skip(self))]
    pub fn get_detailed_changes(&self) -> Result<String> {
        debug!("Getting detailed git changes");
        let mut changes = String::new();

        // Get staged changes with diff
        let staged_output = Command::new("git")
            .args(["diff", "--cached"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git diff --cached command");
                SemanticReleaseError::command_error("git diff --cached", None, e.to_string())
            })?;

        if staged_output.status.success() {
            let staged_diff = String::from_utf8_lossy(&staged_output.stdout);
            if !staged_diff.trim().is_empty() {
                changes.push_str("=== CAMBIOS PREPARADOS (STAGED) ===\n");
                changes.push_str(&staged_diff);
                changes.push_str("\n\n");
                debug!("Added staged changes to detailed diff");
            }
        }

        // Get unstaged changes with diff
        let unstaged_output = Command::new("git").args(["diff"]).output().map_err(|e| {
            error!(error = %e, "Failed to execute git diff command");
            SemanticReleaseError::command_error("git diff", None, e.to_string())
        })?;

        if unstaged_output.status.success() {
            let unstaged_diff = String::from_utf8_lossy(&unstaged_output.stdout);
            if !unstaged_diff.trim().is_empty() {
                changes.push_str("=== CAMBIOS NO PREPARADOS (UNSTAGED) ===\n");
                changes.push_str(&unstaged_diff);
                changes.push_str("\n\n");
                debug!("Added unstaged changes to detailed diff");
            }
        }

        // Get untracked files
        let untracked_output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git ls-files command for untracked files");
                SemanticReleaseError::command_error(
                    "git ls-files --others --exclude-standard",
                    None,
                    e.to_string(),
                )
            })?;

        if untracked_output.status.success() {
            let untracked_files = String::from_utf8_lossy(&untracked_output.stdout);
            if !untracked_files.trim().is_empty() {
                changes.push_str("=== ARCHIVOS NUEVOS (NO RASTREADOS) ===\n");
                for file in untracked_files.lines() {
                    if !file.trim().is_empty() {
                        changes.push_str(&format!("Nuevo archivo: {}\n", file));
                    }
                }
                changes.push('\n');
                debug!("Added untracked files to detailed diff");
            }
        }

        if changes.trim().is_empty() {
            changes = "No hay cambios detectados en el repositorio.".to_string();
            debug!("No changes detected in repository");
        }

        info!(
            changes_length = changes.len(),
            "Retrieved detailed git changes"
        );
        Ok(changes)
    }
}

// =============================================================================
// COMMIT MESSAGE PARSING ENGINE
// =============================================================================

struct CommitParser;

impl CommitParser {
    fn extract_commit_type(subject: &str) -> Option<String> {
        let re = Regex::new(r"^(feat|fix|docs|style|refactor|perf|test|chore|revert)(\(.+\))?:")
            .unwrap();
        if let Some(captures) = re.captures(subject) {
            captures.get(1).map(|m| m.as_str().to_string())
        } else {
            None
        }
    }

    fn extract_commit_scope(subject: &str) -> Option<String> {
        let re = Regex::new(r"^[a-z]+\(([^)]+)\):").unwrap();
        if let Some(captures) = re.captures(subject) {
            captures.get(1).map(|m| m.as_str().to_string())
        } else {
            None
        }
    }

    fn extract_commit_description(subject: &str) -> String {
        let re = Regex::new(r"^[a-z]+(\(.+\))?: *(.+)").unwrap();
        if let Some(captures) = re.captures(subject) {
            captures.get(2).map_or("", |m| m.as_str()).to_string()
        } else {
            subject.to_string()
        }
    }
}

// =============================================================================
// COMMIT BODY CONTENT EXTRACTION
// =============================================================================

impl CommitParser {
    fn extract_breaking_changes(body: &str) -> Vec<String> {
        let mut changes = Vec::new();
        let lines: Vec<&str> = body.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("BREAKING CHANGE:") || line.starts_with("BREAKING-CHANGE:") {
                let mut change = line
                    .replace("BREAKING CHANGE:", "")
                    .replace("BREAKING-CHANGE:", "")
                    .trim()
                    .to_string();

                // Collect continuation lines
                for next_line in lines.iter().skip(i + 1) {
                    let next_line = next_line.trim();
                    if next_line.is_empty() || next_line.contains(":") {
                        break;
                    }
                    change.push(' ');
                    change.push_str(next_line);
                }

                if !change.is_empty() {
                    changes.push(change);
                }
            }
        }

        changes
    }
}

// =============================================================================
// MONDAY.COM TASK INTEGRATION
// =============================================================================

impl CommitParser {
    fn extract_monday_tasks(body: &str) -> Vec<String> {
        let mut tasks = Vec::new();

        // Look for Monday task references in various formats
        let re = Regex::new(r"(?i)(?:monday|task|item)[:\s]*([0-9]+)").unwrap();

        for line in body.lines() {
            for captures in re.captures_iter(line) {
                if let Some(task_id) = captures.get(1) {
                    tasks.push(task_id.as_str().to_string());
                }
            }
        }

        // Also look for refs format: refs mXXXXXXXXXX
        let refs_re = Regex::new(r"refs\s+m(\d+)").unwrap();
        for line in body.lines() {
            if let Some(captures) = refs_re.captures(line) {
                if let Some(task_id) = captures.get(1) {
                    tasks.push(task_id.as_str().to_string());
                }
            }
        }

        tasks.sort();
        tasks.dedup();
        tasks
    }
}

// =============================================================================
// JIRA TASK INTEGRATION
// =============================================================================

impl CommitParser {
    fn extract_jira_tasks(body: &str) -> Vec<String> {
        let mut tasks = Vec::new();

        // Look for JIRA issue keys (PROJECT-123 format)
        let re = Regex::new(r"(?i)\b([A-Z]{2,10}-\d+)\b").unwrap();

        for line in body.lines() {
            for captures in re.captures_iter(line) {
                if let Some(issue_key) = captures.get(1) {
                    tasks.push(issue_key.as_str().to_uppercase());
                }
            }
        }

        tasks.sort();
        tasks.dedup();
        tasks
    }
}

// =============================================================================
// SEMANTIC VERSIONING UTILITIES
// =============================================================================

use crate::types::{VersionInfo, VersionType};

/// Enhanced function to get comprehensive version information
#[instrument]
pub fn get_version_info() -> Result<VersionInfo> {
    info!("Getting comprehensive version information");

    // 1. Get current version from last tag
    let current_version = get_current_version().ok();

    // 2. Execute semantic-release dry run
    let (next_version, version_type, dry_run_output) = execute_semantic_release_dry_run()?;

    // 3. Get commit count since last tag
    let commit_count = get_commit_count_since_last_tag().unwrap_or(0);

    // 4. Check if there are unreleased changes
    let has_unreleased_changes = commit_count > 0;

    info!(
        current_version = ?current_version,
        next_version = %next_version,
        version_type = ?version_type,
        commit_count = commit_count,
        has_unreleased_changes = has_unreleased_changes,
        "Version information retrieved"
    );

    Ok(VersionInfo {
        next_version,
        current_version,
        version_type,
        commit_count,
        has_unreleased_changes,
        dry_run_output,
    })
}

#[instrument]
fn execute_semantic_release_dry_run() -> Result<(String, VersionType, String)> {
    debug!("Executing semantic-release dry run");

    let output = Command::new("npx")
        .args(["semantic-release", "--dry-run"])
        .output()
        .map_err(|e| {
            error!(error = %e, "Failed to execute semantic-release command");
            SemanticReleaseError::command_error(
                "npx semantic-release --dry-run",
                None,
                e.to_string(),
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let full_output = format!("{}\n{}", stdout, stderr);

    // Extract version
    let version_regex = Regex::new(r"The next release version is (\d+\.\d+\.\d+)").unwrap();
    let next_version = if let Some(captures) = version_regex.captures(&full_output) {
        captures.get(1).unwrap().as_str().to_string()
    } else if full_output.contains("no release") || full_output.contains("No release published") {
        "No release needed".to_string()
    } else {
        "Unable to determine".to_string()
    };

    // Determine version type
    let version_type = determine_version_type(&full_output);

    debug!(
        next_version = %next_version,
        version_type = ?version_type,
        "Semantic release dry run completed"
    );

    Ok((next_version, version_type, full_output))
}

fn determine_version_type(output: &str) -> VersionType {
    if output.contains("BREAKING CHANGE") || output.contains("major") {
        VersionType::Major
    } else if output.contains("feat") || output.contains("minor") {
        VersionType::Minor
    } else if output.contains("fix") || output.contains("patch") {
        VersionType::Patch
    } else {
        VersionType::None
    }
}

#[instrument]
fn get_current_version() -> Result<String> {
    debug!("Getting current version from git tags");

    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .map_err(|e| {
            error!(error = %e, "Failed to execute git describe command");
            SemanticReleaseError::command_error(
                "git describe --tags --abbrev=0",
                None,
                e.to_string(),
            )
        })?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!(version = %version, "Retrieved current version");
        Ok(version)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!(stderr = %stderr, "No current version found");
        Err(SemanticReleaseError::command_error(
            "git describe --tags --abbrev=0",
            output.status.code(),
            stderr.to_string(),
        ))
    }
}

#[instrument]
fn get_commit_count_since_last_tag() -> Result<usize> {
    debug!("Getting commit count since last tag");

    let output = Command::new("git")
        .args([
            "rev-list",
            "--count",
            "HEAD",
            "^$(git describe --tags --abbrev=0 2>/dev/null || echo '')",
        ])
        .output()
        .map_err(|e| {
            error!(error = %e, "Failed to execute git rev-list command");
            SemanticReleaseError::command_error("git rev-list --count", None, e.to_string())
        })?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let count_str = output_str.trim();
        let count = count_str.parse::<usize>().unwrap_or(0);
        debug!(count = count, "Retrieved commit count since last tag");
        Ok(count)
    } else {
        // Fallback: just count all commits
        let output = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .map_err(|e| {
                error!(error = %e, "Failed to execute git rev-list fallback command");
                SemanticReleaseError::command_error(
                    "git rev-list --count HEAD",
                    None,
                    e.to_string(),
                )
            })?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let count_str = output_str.trim();
            let count = count_str.parse::<usize>().unwrap_or(0);
            debug!(count = count, "Retrieved total commit count as fallback");
            Ok(count)
        } else {
            warn!("Failed to get commit count, returning 0");
            Ok(0)
        }
    }
}
