//! Cron task executor
//!
//! Handles cron job management: create, modify, delete scheduled tasks.
//!
//! # Examples
//!
//! ## Create a cron job
//!
//! This example creates a cron job that runs a backup script daily at 2 AM.
//!
//! **YAML Format:**
//! ```yaml
//! - type: cron
//!   description: "Create daily backup cron job"
//!   name: daily-backup
//!   state: present
//!   user: root
//!   minute: "0"
//!   hour: "2"
//!   day: "*"
//!   month: "*"
//!   weekday: "*"
//!   job: "/usr/local/bin/backup.sh"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "cron",
//!   "description": "Create daily backup cron job",
//!   "name": "daily-backup",
//!   "state": "present",
//!   "user": "root",
//!   "minute": "0",
//!   "hour": "2",
//!   "day": "*",
//!   "month": "*",
//!   "weekday": "*",
//!   "job": "/usr/local/bin/backup.sh"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "cron"
//! description = "Create daily backup cron job"
//! name = "daily-backup"
//! state = "present"
//! user = "root"
//! minute = "0"
//! hour = "2"
//! day = "*"
//! month = "*"
//! weekday = "*"
//! job = "/usr/local/bin/backup.sh"
//! ```
//!
//! ## Create cron job with specific schedule
//!
//! This example creates a cron job that runs every Monday at 9 AM.
//!
//! **YAML Format:**
//! ```yaml
//! - type: cron
//!   description: "Weekly maintenance on Mondays"
//!   name: weekly-maintenance
//!   state: present
//!   user: root
//!   minute: "0"
//!   hour: "9"
//!   day: "*"
//!   month: "*"
//!   weekday: "1"
//!   job: "/usr/local/bin/maintenance.sh"
//!   comment: "Weekly system maintenance"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "cron",
//!   "description": "Weekly maintenance on Mondays",
//!   "name": "weekly-maintenance",
//!   "state": "present",
//!   "user": "root",
//!   "minute": "0",
//!   "hour": "9",
//!   "day": "*",
//!   "month": "*",
//!   "weekday": "1",
//!   "job": "/usr/local/bin/maintenance.sh",
//!   "comment": "Weekly system maintenance"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "cron"
//! description = "Weekly maintenance on Mondays"
//! name = "weekly-maintenance"
//! state = "present"
//! user = "root"
//! minute = "0"
//! hour = "9"
//! day = "*"
//! month = "*"
//! weekday = "1"
//! job = "/usr/local/bin/maintenance.sh"
//! comment = "Weekly system maintenance"
//! ```
//!
//! ## Remove a cron job
//!
//! This example removes the daily-backup cron job.
//!
//! **YAML Format:**
//! ```yaml
//! - type: cron
//!   description: "Remove daily backup cron job"
//!   name: daily-backup
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "cron",
//!   "description": "Remove daily backup cron job",
//!   "name": "daily-backup",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "cron"
//! description = "Remove daily backup cron job"
//! name = "daily-backup"
//! state = "absent"
//! ```
//!
//! ## Cron job with complex schedule
//!
//! This example creates a cron job that runs every 15 minutes during business hours.
//!
//! **YAML Format:**
//! ```yaml
//! - type: cron
//!   description: "Monitor service every 15 minutes during business hours"
//!   name: service-monitor
//!   state: present
//!   user: monitor
//!   minute: "*/15"
//!   hour: "9-17"
//!   day: "1-5"
//!   month: "*"
//!   weekday: "*"
//!   job: "/usr/local/bin/check-service.sh"
//!   comment: "Business hours service monitoring"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "cron",
//!   "description": "Monitor service every 15 minutes during business hours",
//!   "name": "service-monitor",
//!   "state": "present",
//!   "user": "monitor",
//!   "minute": "*/15",
//!   "hour": "9-17",
//!   "day": "1-5",
//!   "month": "*",
//!   "weekday": "*",
//!   "job": "/usr/local/bin/check-service.sh",
//!   "comment": "Business hours service monitoring"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "cron"
//! description = "Monitor service every 15 minutes during business hours"
//! name = "service-monitor"
//! state = "present"
//! user = "monitor"
//! minute = "*/15"
//! hour = "9-17"
//! day = "1-5"
//! month = "*"
//! weekday = "*"
//! job = "/usr/local/bin/check-service.sh"
//! comment = "Business hours service monitoring"
//! ```

/// Cron job state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CronState {
    /// Ensure cron job exists
    Present,
    /// Ensure cron job does not exist
    Absent,
}

/// Cron job management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CronTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Unique name for this cron job
    pub name: String,
    /// Cron job state
    pub state: CronState,
    /// User to run the job as
    #[serde(default = "default_cron_user")]
    pub user: String,
    /// Minute (0-59, or * for any)
    #[serde(default = "default_cron_minute")]
    pub minute: String,
    /// Hour (0-23, or * for any)
    #[serde(default = "default_cron_hour")]
    pub hour: String,
    /// Day of month (1-31, or * for any)
    #[serde(default = "default_cron_day")]
    pub day: String,
    /// Month (1-12, or * for any)
    #[serde(default = "default_cron_month")]
    pub month: String,
    /// Day of week (0-7, or * for any)
    #[serde(default = "default_cron_weekday")]
    pub weekday: String,
    /// Command to execute
    pub job: String,
    /// Optional comment/description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Execute a cron task
pub async fn execute_cron_task(task: &CronTask, dry_run: bool) -> Result<()> {
    match task.state {
        CronState::Present => ensure_cron_job_present(task, dry_run).await,
        CronState::Absent => ensure_cron_job_absent(task, dry_run).await,
    }
}

/// Ensure a cron job exists
async fn ensure_cron_job_present(task: &CronTask, dry_run: bool) -> Result<()> {
    let cron_file = get_cron_file_path(&task.user);

    // Read existing crontab
    let existing_crontab = read_crontab(&cron_file)?;
    let job_line = format_cron_job(task);

    // Check if job already exists (by comment or exact match)
    if let Some(comment) = &task.comment {
        let comment_marker = format!("# {}", comment);
        if existing_crontab.contains(&comment_marker) {
            println!("Cron job '{}' already exists", task.name);
            return Ok(());
        }
    } else if existing_crontab.contains(&job_line) {
        println!("Cron job '{}' already exists", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would add cron job: {}", task.name);
        println!(
            "  Schedule: {} {} {} {} {}",
            task.minute, task.hour, task.day, task.month, task.weekday
        );
        println!("  Command: {}", task.job);
        if let Some(comment) = &task.comment {
            println!("  Comment: {}", comment);
        }
    } else {
        add_cron_job(task, &cron_file, &existing_crontab)?;
        println!("Added cron job: {}", task.name);
    }

    Ok(())
}

/// Ensure a cron job does not exist
async fn ensure_cron_job_absent(task: &CronTask, dry_run: bool) -> Result<()> {
    let cron_file = get_cron_file_path(&task.user);

    // Read existing crontab
    let existing_crontab = read_crontab(&cron_file)?;

    // Check if job exists
    let mut job_exists = false;
    let mut lines_to_remove = Vec::new();

    if let Some(comment) = &task.comment {
        let comment_marker = format!("# {}", comment);
        for (i, line) in existing_crontab.iter().enumerate() {
            if line.contains(&comment_marker) {
                job_exists = true;
                // Remove comment and following job line
                lines_to_remove.push(i);
                if i + 1 < existing_crontab.len() && !existing_crontab[i + 1].starts_with('#') {
                    lines_to_remove.push(i + 1);
                }
                break;
            }
        }
    } else {
        let job_line = format_cron_job(task);
        for (i, line) in existing_crontab.iter().enumerate() {
            if line.contains(&job_line) {
                job_exists = true;
                lines_to_remove.push(i);
                break;
            }
        }
    }

    if !job_exists {
        println!("Cron job '{}' does not exist", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove cron job: {}", task.name);
    } else {
        remove_cron_job(&cron_file, &existing_crontab, &lines_to_remove)?;
        println!("Removed cron job: {}", task.name);
    }

    Ok(())
}

/// Get the path to the user's crontab file
fn get_cron_file_path(user: &str) -> String {
    if user == "root" {
        "/var/spool/cron/crontabs/root".to_string()
    } else {
        format!("/var/spool/cron/crontabs/{}", user)
    }
}

/// Read the crontab file
fn read_crontab(cron_file: &str) -> Result<Vec<String>> {
    if !Path::new(cron_file).exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(cron_file)
        .with_context(|| format!("Failed to read crontab file: {}", cron_file))?;

    Ok(content.lines().map(|s| s.to_string()).collect())
}

/// Format a cron job line
fn format_cron_job(task: &CronTask) -> String {
    format!(
        "{} {} {} {} {} {}",
        task.minute, task.hour, task.day, task.month, task.weekday, task.job
    )
}

/// Add a cron job to the crontab
fn add_cron_job(task: &CronTask, cron_file: &str, existing_crontab: &[String]) -> Result<()> {
    let mut new_crontab = existing_crontab.to_vec();

    // Add comment if provided
    if let Some(comment) = &task.comment {
        new_crontab.push(format!("# {}", comment));
    }

    // Add the job
    new_crontab.push(format_cron_job(task));

    // Write back to file
    let content = new_crontab.join("\n") + "\n";
    fs::write(cron_file, content)
        .with_context(|| format!("Failed to write crontab file: {}", cron_file))?;

    Ok(())
}

/// Remove a cron job from the crontab
fn remove_cron_job(
    cron_file: &str,
    existing_crontab: &[String],
    lines_to_remove: &[usize],
) -> Result<()> {
    let mut new_crontab = Vec::new();

    for (i, line) in existing_crontab.iter().enumerate() {
        if !lines_to_remove.contains(&i) {
            new_crontab.push(line.clone());
        }
    }

    // Write back to file
    let content = new_crontab.join("\n");
    if content.is_empty() {
        // Remove empty crontab file
        let _ = fs::remove_file(cron_file);
    } else {
        let content = content + "\n";
        fs::write(cron_file, content)
            .with_context(|| format!("Failed to write crontab file: {}", cron_file))?;
    }

    Ok(())
}

/// Default cron user ("root")
pub fn default_cron_user() -> String {
    "root".to_string()
}
/// Default cron minute ("*")
pub fn default_cron_minute() -> String {
    "*".to_string()
}
/// Default cron hour ("*")
pub fn default_cron_hour() -> String {
    "*".to_string()
}
/// Default cron day ("*")
pub fn default_cron_day() -> String {
    "*".to_string()
}
/// Default cron month ("*")
pub fn default_cron_month() -> String {
    "*".to_string()
}
/// Default cron weekday ("*")
pub fn default_cron_weekday() -> String {
    "*".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cron_job_create_dry_run() {
        let task = CronTask {
            description: None,
            name: "backup".to_string(),
            state: CronState::Present,
            user: "root".to_string(),
            minute: "0".to_string(),
            hour: "2".to_string(),
            day: "*".to_string(),
            month: "*".to_string(),
            weekday: "*".to_string(),
            job: "/usr/local/bin/backup.sh".to_string(),
            comment: Some("Daily backup".to_string()),
        };

        let result = execute_cron_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cron_job_remove_dry_run() {
        let task = CronTask {
            description: None,
            name: "backup".to_string(),
            state: CronState::Absent,
            user: "root".to_string(),
            minute: "*".to_string(),
            hour: "*".to_string(),
            day: "*".to_string(),
            month: "*".to_string(),
            weekday: "*".to_string(),
            job: "/usr/local/bin/backup.sh".to_string(),
            comment: None,
        };

        let result = execute_cron_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_cron_job() {
        let task = CronTask {
            description: None,
            name: "test".to_string(),
            state: CronState::Present,
            user: "root".to_string(),
            minute: "0".to_string(),
            hour: "12".to_string(),
            day: "1".to_string(),
            month: "*".to_string(),
            weekday: "1".to_string(),
            job: "/bin/echo hello".to_string(),
            comment: None,
        };

        let formatted = format_cron_job(&task);
        assert_eq!(formatted, "0 12 1 * 1 /bin/echo hello");
    }

    #[test]
    fn test_get_cron_file_path() {
        assert_eq!(get_cron_file_path("root"), "/var/spool/cron/crontabs/root");
        assert_eq!(get_cron_file_path("user"), "/var/spool/cron/crontabs/user");
    }

    #[tokio::test]
    async fn test_cron_job_empty_name() {
        let task = CronTask {
            description: None,
            name: "".to_string(), // Invalid: empty name
            state: CronState::Present,
            user: "root".to_string(),
            minute: "*".to_string(),
            hour: "*".to_string(),
            day: "*".to_string(),
            month: "*".to_string(),
            weekday: "*".to_string(),
            job: "echo test".to_string(),
            comment: None,
        };

        let result = execute_cron_task(&task, true).await;
        assert!(result.is_ok()); // Empty name doesn't cause execution error, just poor identification
    }

    #[tokio::test]
    async fn test_cron_job_empty_command() {
        let task = CronTask {
            description: None,
            name: "test".to_string(),
            state: CronState::Present,
            user: "root".to_string(),
            minute: "*".to_string(),
            hour: "*".to_string(),
            day: "*".to_string(),
            month: "*".to_string(),
            weekday: "*".to_string(),
            job: "".to_string(), // Invalid: empty command
            comment: None,
        };

        let result = execute_cron_task(&task, true).await;
        assert!(result.is_ok()); // Empty command will still be written to crontab
    }

    #[tokio::test]
    async fn test_cron_job_special_characters() {
        let task = CronTask {
            description: None,
            name: "special_chars".to_string(),
            state: CronState::Present,
            user: "root".to_string(),
            minute: "*/5".to_string(),
            hour: "9-17".to_string(),
            day: "1,15".to_string(),
            month: "1-6,9-12".to_string(),
            weekday: "1-5".to_string(),
            job: "/usr/bin/complex command with spaces && pipes | grep test".to_string(),
            comment: Some("Complex schedule with special characters".to_string()),
        };

        let result = execute_cron_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cron_job_invalid_user() {
        let task = CronTask {
            description: None,
            name: "test".to_string(),
            state: CronState::Present,
            user: "nonexistent_user_12345".to_string(), // User that doesn't exist
            minute: "*".to_string(),
            hour: "*".to_string(),
            day: "*".to_string(),
            month: "*".to_string(),
            weekday: "*".to_string(),
            job: "echo test".to_string(),
            comment: None,
        };

        let result = execute_cron_task(&task, true).await;
        assert!(result.is_ok()); // Invalid user doesn't cause validation error, just wrong file path
    }
}
