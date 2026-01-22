//! Block in file task executor
//!
//! Handles inserting/updating multi-line blocks in files.
//!
//! # Examples
//!
//! ## Insert a configuration block
//!
//! This example inserts a configuration block into a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: blockinfile
//!   description: "Add custom configuration block"
//!   path: /etc/myapp/config.conf
//!   state: present
//!   block: |
//!     # Custom configuration
//!     custom_option = true
//!     custom_value = 42
//!   marker: "# {mark} Custom Config"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "blockinfile",
//!   "description": "Add custom configuration block",
//!   "path": "/etc/myapp/config.conf",
//!   "state": "present",
//!   "block": "# Custom configuration\ncustom_option = true\ncustom_value = 42\n",
//!   "marker": "# {mark} Custom Config"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "blockinfile"
//! description = "Add custom configuration block"
//! path = "/etc/myapp/config.conf"
//! state = "present"
//! block = """
//! # Custom configuration
//! custom_option = true
//! custom_value = 42
//! """
//! marker = "# {mark} Custom Config"
//! ```
//!
//! ## Insert block after specific content
//!
//! This example inserts a block after a specific line in the file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: blockinfile
//!   description: "Add SSL configuration"
//!   path: /etc/httpd/httpd.conf
//!   state: present
//!   block: |
//!     SSLEngine on
//!     SSLCertificateFile /etc/ssl/certs/server.crt
//!     SSLCertificateKeyFile /etc/ssl/private/server.key
//!   insertafter: "^# LoadModule ssl_module"
//!   marker: "# {mark} SSL Config"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "blockinfile",
//!   "description": "Add SSL configuration",
//!   "path": "/etc/httpd/httpd.conf",
//!   "state": "present",
//!   "block": "SSLEngine on\nSSLCertificateFile /etc/ssl/certs/server.crt\nSSLCertificateKeyFile /etc/ssl/private/server.key\n",
//!   "insertafter": "^# LoadModule ssl_module",
//!   "marker": "# {mark} SSL Config"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "blockinfile"
//! description = "Add SSL configuration"
//! path = "/etc/httpd/httpd.conf"
//! state = "present"
//! block = """
//! SSLEngine on
//! SSLCertificateFile /etc/ssl/certs/server.crt
//! SSLCertificateKeyFile /etc/ssl/private/server.key
//! """
//! insertafter = "^# LoadModule ssl_module"
//! marker = "# {mark} SSL Config"
//! ```
//!
//! ## Insert block with backup
//!
//! This example inserts a block and creates a backup of the original file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: blockinfile
//!   description: "Add firewall rules with backup"
//!   path: /etc/iptables/rules.v4
//!   state: present
//!   block: |
//!     -A INPUT -p tcp --dport 80 -j ACCEPT
//!     -A INPUT -p tcp --dport 443 -j ACCEPT
//!   marker: "# {mark} Web Rules"
//!   backup: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "blockinfile",
//!   "description": "Add firewall rules with backup",
//!   "path": "/etc/iptables/rules.v4",
//!   "state": "present",
//!   "block": "-A INPUT -p tcp --dport 80 -j ACCEPT\n-A INPUT -p tcp --dport 443 -j ACCEPT\n",
//!   "marker": "# {mark} Web Rules",
//!   "backup": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "blockinfile"
//! description = "Add firewall rules with backup"
//! path = "/etc/iptables/rules.v4"
//! state = "present"
//! block = """
//! -A INPUT -p tcp --dport 80 -j ACCEPT
//! -A INPUT -p tcp --dport 443 -j ACCEPT
//! """
//! marker = "# {mark} Web Rules"
//! backup = true
//! ```
//!
//! ## Remove a configuration block
//!
//! This example removes a configuration block from a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: blockinfile
//!   description: "Remove old configuration"
//!   path: /etc/myapp/config.conf
//!   state: absent
//!   marker: "# {mark} Old Config"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "blockinfile",
//!   "description": "Remove old configuration",
//!   "path": "/etc/myapp/config.conf",
//!   "state": "absent",
//!   "marker": "# {mark} Old Config"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "blockinfile"
//! description = "Remove old configuration"
//! path = "/etc/myapp/config.conf"
//! state = "absent"
//! marker = "# {mark} Old Config"
//! ```

/// Block in file state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockInFileState {
    /// Ensure block is present
    Present,
    /// Ensure block is absent
    Absent,
}

/// Insert/update multi-line blocks task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockInFileTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Path to the file
    pub path: String,
    /// Block state
    pub state: BlockInFileState,
    /// Block content (multi-line)
    pub block: String,
    /// Marker for block boundaries
    #[serde(default = "default_block_marker")]
    pub marker: String,
    /// Insert after this line (regex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertafter: Option<String>,
    /// Insert before this line (regex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertbefore: Option<String>,
    /// Create file if it doesn't exist
    #[serde(default)]
    pub create: bool,
    /// Backup file before modification
    #[serde(default)]
    pub backup: bool,
}

/// Default block marker
pub fn default_block_marker() -> String {
    "# {mark}".to_string()
}

use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Execute a blockinfile task
pub async fn execute_blockinfile_task(task: &BlockInFileTask, dry_run: bool) -> Result<()> {
    match task.state {
        BlockInFileState::Present => ensure_block_present(task, dry_run).await,
        BlockInFileState::Absent => ensure_block_absent(task, dry_run).await,
    }
}

/// Ensure block is present in file
async fn ensure_block_present(task: &BlockInFileTask, dry_run: bool) -> Result<()> {
    let path = Path::new(&task.path);

    // Read existing file content
    let content = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("Failed to read file {}", task.path))?
    } else if task.create {
        String::new()
    } else {
        return Err(anyhow::anyhow!(
            "File does not exist and create=false: {}",
            task.path
        ));
    };

    // Generate block markers
    let begin_marker = task.marker.replace("{mark}", "BEGIN");
    let end_marker = task.marker.replace("{mark}", "END");

    let _block_lines: Vec<String> = task.block.lines().map(|s| s.to_string()).collect();
    let full_block = format!("{}\n{}\n{}", begin_marker, task.block, end_marker);

    // Check if block already exists
    let existing_blocks = find_blocks(&content, &begin_marker, &end_marker);
    let block_exists = !existing_blocks.is_empty();

    if block_exists {
        // Replace existing block
        let mut new_content = content.clone();

        for (start_pos, end_pos) in existing_blocks.into_iter().rev() {
            new_content.replace_range(start_pos..end_pos, &full_block);
        }

        if content == new_content {
            println!("Block already present in {}", task.path);
            return Ok(());
        }

        if dry_run {
            println!("Would replace block in file: {}", task.path);
        } else {
            write_with_backup(task, &new_content, path, true).await?;
            println!("Replaced block in {}", task.path);
        }
    } else {
        // Insert new block
        let insert_pos = find_insert_position(&content, task)?;
        let mut new_content = content.clone();
        new_content.insert_str(insert_pos, &full_block);
        new_content.push('\n'); // Ensure newline after block

        if dry_run {
            println!("Would insert block in file: {}", task.path);
        } else {
            write_with_backup(task, &new_content, path, false).await?;
            println!("Inserted block in {}", task.path);
        }
    }

    Ok(())
}

/// Ensure block is absent from file
async fn ensure_block_absent(task: &BlockInFileTask, dry_run: bool) -> Result<()> {
    let path = Path::new(&task.path);

    if !path.exists() {
        println!("File does not exist: {}", task.path);
        return Ok(());
    }

    // Read existing file content
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file {}", task.path))?;

    // Generate block markers
    let begin_marker = task.marker.replace("{mark}", "BEGIN");
    let end_marker = task.marker.replace("{mark}", "END");

    let existing_blocks = find_blocks(&content, &begin_marker, &end_marker);
    if existing_blocks.is_empty() {
        println!("Block not found in {}", task.path);
        return Ok(());
    }

    // Remove all matching blocks
    let mut new_content = content.clone();
    for (start_pos, end_pos) in existing_blocks.into_iter().rev() {
        // Remove the block including surrounding whitespace
        let start_remove = find_line_start(&new_content, start_pos);
        let end_remove = find_line_end(&new_content, end_pos);
        new_content.replace_range(start_remove..end_remove, "");
    }

    if content == new_content {
        println!("Block already absent from {}", task.path);
        return Ok(());
    }

    if dry_run {
        println!("Would remove block from file: {}", task.path);
    } else {
        write_with_backup(task, &new_content, path, true).await?;
        println!("Removed block from {}", task.path);
    }

    Ok(())
}

/// Find all block positions in content
fn find_blocks(content: &str, begin_marker: &str, end_marker: &str) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut search_pos = 0;

    while let Some(begin_pos) = content[search_pos..].find(begin_marker) {
        let begin_pos = search_pos + begin_pos;

        if let Some(end_pos) = content[begin_pos..].find(end_marker) {
            let end_pos = begin_pos + end_pos + end_marker.len();
            blocks.push((begin_pos, end_pos));
            search_pos = end_pos;
        } else {
            break;
        }
    }

    blocks
}

/// Find position to insert new block
fn find_insert_position(content: &str, task: &BlockInFileTask) -> Result<usize> {
    if let Some(insertafter) = &task.insertafter {
        let re = Regex::new(insertafter)
            .with_context(|| format!("Invalid insertafter regexp: {}", insertafter))?;

        for (i, line) in content.lines().enumerate() {
            if re.is_match(line) {
                // Insert after this line
                let line_start = content.lines().take(i + 1).map(|l| l.len() + 1).sum();
                return Ok(line_start);
            }
        }
    } else if let Some(insertbefore) = &task.insertbefore {
        let re = Regex::new(insertbefore)
            .with_context(|| format!("Invalid insertbefore regexp: {}", insertbefore))?;

        for (i, line) in content.lines().enumerate() {
            if re.is_match(line) {
                // Insert before this line
                let line_start: usize = content.lines().take(i).map(|l| l.len() + 1).sum();
                return Ok(line_start);
            }
        }
    }

    // Default to end of file
    Ok(content.len())
}

/// Find start of line containing position
fn find_line_start(content: &str, pos: usize) -> usize {
    content[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0)
}

/// Find end of line containing position
fn find_line_end(content: &str, pos: usize) -> usize {
    content[pos..]
        .find('\n')
        .map(|p| pos + p + 1)
        .unwrap_or(content.len())
}

/// Write content with backup if requested
async fn write_with_backup(
    task: &BlockInFileTask,
    content: &str,
    path: &Path,
    _is_replacement: bool,
) -> Result<()> {
    // Backup file if requested
    if task.backup && path.exists() {
        let backup_path = format!("{}.backup", task.path);
        fs::copy(&task.path, &backup_path)
            .with_context(|| format!("Failed to backup {} to {}", task.path, backup_path))?;
        println!("Backed up {} to {}", task.path, backup_path);
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directories for {}", task.path))?;
    }

    // Write new content
    fs::write(&task.path, content)
        .with_context(|| format!("Failed to write to file {}", task.path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_blockinfile_insert_block_dry_run() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "# Config file\n# End config\n").unwrap();

        let task = BlockInFileTask {
            description: None,
            path: file_path.clone(),
            state: BlockInFileState::Present,
            block: "export PATH=/usr/bin\nexport EDITOR=vim".to_string(),
            marker: "# {mark} ANSIBLE MANAGED BLOCK".to_string(),
            insertafter: Some(r"^# Config file$".to_string()),
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_blockinfile_task(&task, true).await;
        assert!(result.is_ok());

        // Verify file wasn't modified
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("ANSIBLE MANAGED BLOCK"));
    }

    #[tokio::test]
    async fn test_blockinfile_insert_block_real() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "# Config file\n# End config\n").unwrap();

        let task = BlockInFileTask {
            description: None,
            path: file_path.clone(),
            state: BlockInFileState::Present,
            block: "export PATH=/usr/bin\nexport EDITOR=vim".to_string(),
            marker: "# {mark} ANSIBLE MANAGED BLOCK".to_string(),
            insertafter: Some(r"^# Config file$".to_string()),
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_blockinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# BEGIN ANSIBLE MANAGED BLOCK"));
        assert!(content.contains("export PATH=/usr/bin"));
        assert!(content.contains("# END ANSIBLE MANAGED BLOCK"));
    }

    #[tokio::test]
    async fn test_blockinfile_remove_block() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();

        let initial_content = "# Config file
# BEGIN ANSIBLE MANAGED BLOCK
export PATH=/usr/bin
export EDITOR=vim
# END ANSIBLE MANAGED BLOCK
# End config
";
        fs::write(&file_path, initial_content).unwrap();

        let task = BlockInFileTask {
            description: None,
            path: file_path.clone(),
            state: BlockInFileState::Absent,
            block: "dummy".to_string(), // Block content doesn't matter for removal
            marker: "# {mark} ANSIBLE MANAGED BLOCK".to_string(),
            insertafter: None,
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_blockinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("ANSIBLE MANAGED BLOCK"));
        assert!(!content.contains("export PATH"));
    }

    #[test]
    fn test_find_blocks() {
        let content = "# Config
# BEGIN BLOCK1
line1
# END BLOCK1
# Middle
# BEGIN BLOCK2
line2
# END BLOCK2
# End";

        let blocks = find_blocks(content, "# BEGIN BLOCK1", "# END BLOCK1");
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            &content[blocks[0].0..blocks[0].1],
            "# BEGIN BLOCK1\nline1\n# END BLOCK1"
        );
    }

    #[tokio::test]
    async fn test_blockinfile_create_file() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string() + ".new";

        let task = BlockInFileTask {
            description: None,
            path: file_path.clone(),
            state: BlockInFileState::Present,
            block: "new content".to_string(),
            marker: "# {mark}".to_string(),
            insertafter: None,
            insertbefore: None,
            create: true,
            backup: false,
        };

        let result = execute_blockinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# BEGIN"));
        assert!(content.contains("new content"));
        assert!(content.contains("# END"));
    }
}
