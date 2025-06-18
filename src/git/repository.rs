use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::Repository;
use regex::Regex;
use std::process::{Command, Stdio};

use crate::types::{GitCommit, MondayTaskMention};

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
    pub fn new() -> Result<Self> {
        let repo = Repository::open(".")?;
        Ok(Self { repo })
    }
}

// =============================================================================
// COMMIT HISTORY AND RETRIEVAL
// =============================================================================

impl GitRepo {
    pub fn get_commits_since_tag(&self, tag: Option<&str>) -> Result<Vec<GitCommit>> {
        let mut commits = Vec::new();

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        if let Some(tag) = tag {
            if let Ok(tag_obj) = self.repo.revparse_single(tag) {
                revwalk.hide(tag_obj.id())?;
            }
        }

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            // Skip merge commits
            if commit.parent_count() > 1 {
                continue;
            }

            let git_commit = self.build_git_commit_from_raw(oid, &commit)?;
            commits.push(git_commit);
        }

        Ok(commits)
    }

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

        let author = commit.author();
        let author_name = String::from_utf8_lossy(author.name_bytes()).to_string();
        let author_email = String::from_utf8_lossy(author.email_bytes()).to_string();

        let commit_time = author.when();
        let commit_date = DateTime::from_timestamp(commit_time.seconds(), 0).unwrap_or(Utc::now());

        let monday_task_mentions = CommitParser::extract_monday_task_mentions(&body);
        let monday_tasks = CommitParser::extract_monday_tasks(&body);

        let jira_task_mentions = CommitParser::extract_jira_task_mentions(&body);
        let jira_tasks = CommitParser::extract_jira_tasks(&body);

        Ok(GitCommit {
            hash: oid.to_string(),
            body: body.clone(),
            author_name,
            author_email,
            commit_date: commit_date.into(),
            commit_type: CommitParser::extract_commit_type(&subject),
            scope: CommitParser::extract_commit_scope(&subject),
            description: CommitParser::extract_commit_description(&subject),
            breaking_changes: CommitParser::extract_breaking_changes(&body),
            test_details: CommitParser::extract_test_details(&body),
            security: CommitParser::extract_security(&body),
            monday_tasks,
            monday_task_mentions,
            jira_tasks,
            jira_task_mentions,
        })
    }
}

// =============================================================================
// TAG AND VERSION MANAGEMENT
// =============================================================================

impl GitRepo {
    pub fn get_last_tag(&self) -> Result<Option<String>> {
        // Use git command to get the last tag, as git2 doesn't have a simple way
        let output = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if tag.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(tag))
                }
            }
            _ => Ok(None),
        }
    }
}

// =============================================================================
// COMMIT CREATION AND STAGING
// =============================================================================

impl GitRepo {
    pub fn create_commit(&self, message: &str) -> Result<String> {
        // Use git command for committing
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Git commit failed: {}", error))
        }
    }

    pub fn stage_all(&self) -> Result<String> {
        // Use git command for staging all changes (equivalent to git add -A)
        let output = Command::new("git")
            .args(["add", "-A"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Git add failed: {}", error))
        }
    }
}

// =============================================================================
// REPOSITORY STATUS AND INFORMATION
// =============================================================================

impl GitRepo {
    pub fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if branch.is_empty() {
                Ok("HEAD".to_string()) // Detached HEAD state
            } else {
                Ok(branch)
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to get current branch: {}", error))
        }
    }

    pub fn get_status(&self) -> Result<GitStatus> {
        let mut status = GitStatus {
            staged: Vec::new(),
            modified: Vec::new(),
            untracked: Vec::new(),
        };

        // Get staged files
        let staged_output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .output()?;

        if staged_output.status.success() {
            status.staged = String::from_utf8_lossy(&staged_output.stdout)
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Get modified files
        let modified_output = Command::new("git").args(["diff", "--name-only"]).output()?;

        if modified_output.status.success() {
            status.modified = String::from_utf8_lossy(&modified_output.stdout)
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Get untracked files
        let untracked_output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .output()?;

        if untracked_output.status.success() {
            status.untracked = String::from_utf8_lossy(&untracked_output.stdout)
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        Ok(status)
    }
}

// =============================================================================
// DIFF AND CHANGE ANALYSIS
// =============================================================================

impl GitRepo {
    pub fn get_detailed_changes(&self) -> Result<String> {
        let mut changes = String::new();

        // Get staged changes with diff
        let staged_output = Command::new("git").args(["diff", "--cached"]).output()?;

        if staged_output.status.success() {
            let staged_diff = String::from_utf8_lossy(&staged_output.stdout);
            if !staged_diff.trim().is_empty() {
                changes.push_str("=== CAMBIOS PREPARADOS (STAGED) ===\n");
                changes.push_str(&staged_diff);
                changes.push_str("\n\n");
            }
        }

        // Get unstaged changes with diff
        let unstaged_output = Command::new("git").args(["diff"]).output()?;

        if unstaged_output.status.success() {
            let unstaged_diff = String::from_utf8_lossy(&unstaged_output.stdout);
            if !unstaged_diff.trim().is_empty() {
                changes.push_str("=== CAMBIOS NO PREPARADOS (UNSTAGED) ===\n");
                changes.push_str(&unstaged_diff);
                changes.push_str("\n\n");
            }
        }

        // Get untracked files
        let untracked_output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .output()?;

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
            }
        }

        if changes.trim().is_empty() {
            changes = "No hay cambios detectados en el repositorio.".to_string();
        }

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

    fn extract_test_details(body: &str) -> Vec<String> {
        let mut tests = Vec::new();
        let re = Regex::new(r"(?i)^test\s*(\d*)\s*:\s*(.+)$").unwrap();

        for line in body.lines() {
            if let Some(captures) = re.captures(line.trim()) {
                if let Some(test_desc) = captures.get(2) {
                    tests.push(test_desc.as_str().to_string());
                }
            }
        }

        tests
    }

    fn extract_security(body: &str) -> Option<String> {
        let re = Regex::new(r"(?i)^security:\s*(.+)$").unwrap();

        for line in body.lines() {
            if let Some(captures) = re.captures(line.trim()) {
                if let Some(security) = captures.get(1) {
                    return Some(security.as_str().to_string());
                }
            }
        }

        None
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

    fn extract_monday_task_mentions(body: &str) -> Vec<MondayTaskMention> {
        let mut mentions = Vec::new();

        // Look for "MONDAY TASKS:" section in the body
        if let Some(monday_section_start) = body.find("MONDAY TASKS:") {
            let monday_section = &body[monday_section_start..];

            // Find the end of the monday tasks section (next double newline or end of string)
            let monday_text = if let Some(end) = monday_section.find("\n\n") {
                &monday_section[..end]
            } else {
                monday_section
            };

            // Extract task lines
            for line in monday_text.lines().skip(1) {
                // Skip the "MONDAY TASKS:" line
                let clean_line = line.trim().trim_start_matches('-').trim();

                // Look for pattern: "Title (ID: 123456789, URL: url)"
                if let Some(id_start) = clean_line.find("(ID: ") {
                    if let Some(id_end) = clean_line[id_start + 5..].find(',') {
                        let id = &clean_line[id_start + 5..id_start + 5 + id_end];

                        // Extract title (everything before the (ID: part)
                        let title = clean_line[..id_start].trim();

                        mentions.push(MondayTaskMention {
                            id: id.to_string(),
                            title: title.to_string(),
                        });
                    }
                }
            }
        }

        mentions
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

    fn extract_jira_task_mentions(body: &str) -> Vec<crate::types::JiraTaskMention> {
        let mut mentions = Vec::new();

        // Look for "JIRA TASKS:" section in the body
        if let Some(jira_section_start) = body.find("JIRA TASKS:") {
            let jira_section = &body[jira_section_start..];

            // Find the end of the jira tasks section (next double newline or end of string)
            let jira_text = if let Some(end) = jira_section.find("\n\n") {
                &jira_section[..end]
            } else {
                jira_section
            };

            // Extract task lines
            for line in jira_text.lines().skip(1) {
                // Skip the "JIRA TASKS:" line
                let clean_line = line.trim().trim_start_matches('-').trim();

                // Look for pattern: "Title (KEY: PROJECT-123)"
                if let Some(key_start) = clean_line.find("(KEY: ") {
                    if let Some(key_end) = clean_line[key_start + 6..].find(')') {
                        let key = &clean_line[key_start + 6..key_start + 6 + key_end];

                        // Extract title (everything before the (KEY: part)
                        let title = clean_line[..key_start].trim();

                        mentions.push(crate::types::JiraTaskMention {
                            key: key.to_string(),
                            summary: title.to_string(),
                        });
                    }
                }
            }
        }

        mentions
    }
}

// =============================================================================
// SEMANTIC VERSIONING UTILITIES
// =============================================================================

use crate::types::{VersionInfo, VersionType};

/// Enhanced function to get comprehensive version information
pub fn get_version_info() -> Result<VersionInfo> {
    // 1. Get current version from last tag
    let current_version = get_current_version().ok();

    // 2. Execute semantic-release dry run
    let (next_version, version_type, dry_run_output) = execute_semantic_release_dry_run()?;

    // 3. Get commit count since last tag
    let commit_count = get_commit_count_since_last_tag().unwrap_or(0);

    // 4. Check if there are unreleased changes
    let has_unreleased_changes = commit_count > 0;

    Ok(VersionInfo {
        next_version,
        current_version,
        version_type,
        commit_count,
        has_unreleased_changes,
        dry_run_output,
    })
}

fn execute_semantic_release_dry_run() -> Result<(String, VersionType, String)> {
    let output = Command::new("npx")
        .args(["semantic-release", "--dry-run"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute semantic-release: {}", e))?;

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

    Ok((next_version, version_type, full_output))
}

fn determine_version_type(output: &str) -> VersionType {
    if output.contains("BREAKING CHANGE") || output.contains("major") {
        VersionType::Major
    } else if output.contains("feat") || output.contains("minor") {
        VersionType::Minor
    } else if output.contains("fix") || output.contains("patch") {
        VersionType::Patch
    } else if output.contains("no release") || output.contains("No release published") {
        VersionType::None
    } else {
        VersionType::Unknown
    }
}

fn get_current_version() -> Result<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        Err(anyhow::anyhow!("No tags found"))
    }
}

fn get_commit_count_since_last_tag() -> Result<usize> {
    // Get last tag
    let last_tag_output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output();

    let range = match last_tag_output {
        Ok(output) if output.status.success() => {
            let last_tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            format!("{}..HEAD", last_tag)
        }
        _ => "HEAD".to_string(), // No tags found, count all commits
    };

    let output = Command::new("git")
        .args(["rev-list", "--count", &range])
        .output()?;

    if output.status.success() {
        let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(count_str.parse().unwrap_or(0))
    } else {
        Ok(0)
    }
}

/// Simple function for backward compatibility
pub fn get_next_version() -> Result<String> {
    let version_info = get_version_info()?;
    Ok(version_info.next_version)
}
