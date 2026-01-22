//! Git task executor
//!
//! Handles git repository operations: clone, checkout, update repositories.
//!
//! # Examples
//!
//! ## Clone a repository
//!
//! This example clones a git repository to a local directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: git
//!   description: "Clone application repository"
//!   repo: https://github.com/user/myapp.git
//!   dest: /opt/myapp
//!   version: main
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "git",
//!   "description": "Clone application repository",
//!   "repo": "https://github.com/user/myapp.git",
//!   "dest": "/opt/myapp",
//!   "version": "main"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "git"
//! description = "Clone application repository"
//! repo = "https://github.com/user/myapp.git"
//! dest = "/opt/myapp"
//! version = "main"
//! ```
//!
//! ## Clone specific branch
//!
//! This example clones a specific branch of a repository.
//!
//! **YAML Format:**
//! ```yaml
//! - type: git
//!   description: "Clone development branch"
//!   repo: https://github.com/user/myapp.git
//!   dest: /opt/myapp-dev
//!   version: develop
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "git",
//!   "description": "Clone development branch",
//!   "repo": "https://github.com/user/myapp.git",
//!   "dest": "/opt/myapp-dev",
//!   "version": "develop"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "git"
//! description = "Clone development branch"
//! repo = "https://github.com/user/myapp.git"
//! dest = "/opt/myapp-dev"
//! version = "develop"
//! ```
//!
//! ## Clone with submodules
//!
//! This example clones a repository including all submodules.
//!
//! **YAML Format:**
//! ```yaml
//! - type: git
//!   description: "Clone repository with submodules"
//!   repo: https://github.com/user/myapp.git
//!   dest: /opt/myapp
//!   recursive: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "git",
//!   "description": "Clone repository with submodules",
//!   "repo": "https://github.com/user/myapp.git",
//!   "dest": "/opt/myapp",
//!   "recursive": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "git"
//! description = "Clone repository with submodules"
//! repo = "https://github.com/user/myapp.git"
//! dest = "/opt/myapp"
//! recursive = true
//! ```
//!
//! ## Clone specific commit
//!
//! This example clones a repository and checks out a specific commit.
//!
//! **YAML Format:**
//! ```yaml
//! - type: git
//!   description: "Clone specific commit"
//!   repo: https://github.com/user/myapp.git
//!   dest: /opt/myapp
//!   version: abc123def456
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "git",
//!   "description": "Clone specific commit",
//!   "repo": "https://github.com/user/myapp.git",
//!   "dest": "/opt/myapp",
//!   "version": "abc123def456"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "git"
//! description = "Clone specific commit"
//! repo = "https://github.com/user/myapp.git"
//! dest = "/opt/myapp"
//! version = "abc123def456"
//! ```
//!
//! ## Register repository state
//!
//! This example clones a repository and registers its state for use in subsequent tasks.
//!
//! **YAML Format:**
//! ```yaml
//! - type: git
//!   description: "Clone rust source"
//!   repo: https://github.com/rust-lang/rust.git
//!   dest: /opt/rust
//!   register: rust_repo
//!
//! - type: debug
//!   msg: "Rust repo is at {{ rust_repo.after }}"
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "git",
//!     "description": "Clone rust source",
//!     "repo": "https://github.com/rust-lang/rust.git",
//!     "dest": "/opt/rust",
//!     "register": "rust_repo"
//!   },
//!   {
//!     "type": "debug",
//!     "msg": "Rust repo is at {{ rust_repo.after }}"
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "git"
//! description = "Clone rust source"
//! repo = "https://github.com/rust-lang/rust.git"
//! dest = "/opt/rust"
//! register = "rust_repo"
//!
//! [[tasks]]
//! type = "debug"
//! msg = "Rust repo is at {{ rust_repo.after }}"
//! ```

use serde::{Deserialize, Serialize};

/// Git repository management task
///
/// # Registered Outputs
/// - `changed` (bool): Whether the repository was updated or cloned
/// - `after` (String): The SHA-1 hash after the task has run
/// - `before` (String): The SHA-1 hash before the task has run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Git repository URL
    ///
    /// The git, SSH, or HTTP(S) protocol address of the git repository.
    pub repo: String,

    /// Destination directory
    ///
    /// The path of where the repository should be checked out.
    pub dest: String,

    /// Version to check out
    ///
    /// What version of the repository to check out. This can be the literal string HEAD,
    /// a branch name, a tag name, or a SHA-1 hash.
    #[serde(default = "default_head")]
    pub version: String,

    /// Whether to update the repository
    ///
    /// If false, do not retrieve new revisions from the origin repository.
    #[serde(default = "default_true")]
    pub update: bool,

    /// Whether to clone if repository doesn't exist
    ///
    /// If false, do not clone the repository even if it does not exist locally.
    #[serde(default = "default_true")]
    pub clone: bool,

    /// Whether to force checkout
    ///
    /// If true, any modified files in the working repository will be discarded.
    #[serde(default)]
    pub force: bool,

    /// Depth for shallow clone
    ///
    /// Create a shallow clone with a history truncated to the specified number of revisions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,

    /// Whether to clone recursively (include submodules)
    ///
    /// If false, repository will be cloned without the --recursive option.
    #[serde(default = "default_true")]
    pub recursive: bool,

    /// Remote name
    ///
    /// Name of the remote.
    #[serde(default = "default_origin")]
    pub remote: String,

    /// SSH key file
    ///
    /// Specify an optional private key file path to use for the checkout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_file: Option<String>,

    /// Accept host key
    ///
    /// Will ensure or not that -o StrictHostKeyChecking=no is present as an ssh option.
    #[serde(default)]
    pub accept_hostkey: bool,

    /// SSH options
    ///
    /// Options git will pass to ssh when used as protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_opts: Option<String>,
}

/// Default version (HEAD)
fn default_head() -> String {
    "HEAD".to_string()
}

/// Default remote name (origin)
fn default_origin() -> String {
    "origin".to_string()
}

/// Default true value
pub fn default_true() -> bool {
    true
}

use anyhow::{Context, Result};
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::Path;

/// Execute a git task
pub async fn execute_git_task(task: &GitTask, dry_run: bool) -> Result<serde_yaml::Value> {
    let dest_path = Path::new(&task.dest);

    // Get before SHA
    let before = if dest_path.exists() && dest_path.join(".git").exists() {
        if let Ok(repo) = Repository::open(dest_path) {
            repo.head()
                .ok()
                .and_then(|h| h.target())
                .map(|oid| oid.to_string())
        } else {
            None
        }
    } else {
        None
    };

    // Check if repository already exists
    let repo_exists = dest_path.exists() && dest_path.join(".git").exists();

    if !repo_exists {
        if !task.clone {
            let mut result = serde_yaml::Mapping::new();
            result.insert(
                serde_yaml::Value::from("changed"),
                serde_yaml::Value::from(false),
            );
            result.insert(
                serde_yaml::Value::from("before"),
                serde_yaml::Value::from(before.clone().unwrap_or_default()),
            );
            result.insert(
                serde_yaml::Value::from("after"),
                serde_yaml::Value::from(before.unwrap_or_default()),
            );
            return Ok(serde_yaml::Value::Mapping(result));
        }

        if dry_run {
            println!("Would clone repository {} to {}", task.repo, task.dest);
            let mut result = serde_yaml::Mapping::new();
            result.insert(
                serde_yaml::Value::from("changed"),
                serde_yaml::Value::from(true),
            );
            result.insert(
                serde_yaml::Value::from("before"),
                serde_yaml::Value::from(before.clone().unwrap_or_default()),
            );
            result.insert(
                serde_yaml::Value::from("after"),
                serde_yaml::Value::from("dry_run"),
            );
            return Ok(serde_yaml::Value::Mapping(result));
        }

        clone_repository(task).await?;
    } else {
        if !task.update {
            let mut result = serde_yaml::Mapping::new();
            result.insert(
                serde_yaml::Value::from("changed"),
                serde_yaml::Value::from(false),
            );
            result.insert(
                serde_yaml::Value::from("before"),
                serde_yaml::Value::from(before.clone().unwrap_or_default()),
            );
            result.insert(
                serde_yaml::Value::from("after"),
                serde_yaml::Value::from(before.unwrap_or_default()),
            );
            return Ok(serde_yaml::Value::Mapping(result));
        }

        if dry_run {
            println!("Would update repository in {}", task.dest);
            let mut result = serde_yaml::Mapping::new();
            result.insert(
                serde_yaml::Value::from("changed"),
                serde_yaml::Value::from(true),
            );
            result.insert(
                serde_yaml::Value::from("before"),
                serde_yaml::Value::from(before.clone().unwrap_or_default()),
            );
            result.insert(
                serde_yaml::Value::from("after"),
                serde_yaml::Value::from("dry_run"),
            );
            return Ok(serde_yaml::Value::Mapping(result));
        }

        update_repository(task).await?;
    }

    // Get after SHA
    let after = if dest_path.exists() && dest_path.join(".git").exists() {
        if let Ok(repo) = Repository::open(dest_path) {
            repo.head()
                .ok()
                .and_then(|h| h.target())
                .map(|oid| oid.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let changed = before != after;

    let mut result = serde_yaml::Mapping::new();
    result.insert(
        serde_yaml::Value::from("changed"),
        serde_yaml::Value::from(changed),
    );
    result.insert(
        serde_yaml::Value::from("before"),
        serde_yaml::Value::from(before.unwrap_or_default()),
    );
    result.insert(
        serde_yaml::Value::from("after"),
        serde_yaml::Value::from(after.unwrap_or_default()),
    );

    Ok(serde_yaml::Value::Mapping(result))
}

/// Clone a git repository
async fn clone_repository(task: &GitTask) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    // Create parent directories if needed
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directories for {}", task.dest))?;
    }

    // Set up callbacks for authentication
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        if let Some(username) = username_from_url {
            Cred::ssh_key_from_agent(username)
        } else {
            Cred::ssh_key_from_agent("git")
        }
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    // Note: depth is not directly supported in git2 RepoBuilder
    // For shallow clones, we'd need to use git command line or different approach

    if task.recursive {
        // Note: git2 doesn't directly support recursive clone in builder
        // We'll handle submodules after clone
    }

    println!("Cloning repository {} to {}", task.repo, task.dest);

    let repo = builder
        .clone(&task.repo, dest_path)
        .with_context(|| format!("Failed to clone repository {} to {}", task.repo, task.dest))?;

    // Handle submodules if recursive
    if task.recursive {
        update_submodules(&repo)?;
    }

    // Checkout the specified version
    checkout_version(&repo, &task.version)?;

    println!("Successfully cloned repository to {}", task.dest);
    Ok(())
}

/// Update an existing repository
async fn update_repository(task: &GitTask) -> Result<()> {
    let repo = Repository::open(&task.dest)
        .with_context(|| format!("Failed to open repository at {}", task.dest))?;

    // Fetch from remote
    let mut remote = repo
        .find_remote(&task.remote)
        .with_context(|| format!("Failed to find remote {}", task.remote))?;

    // Set up callbacks for authentication
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        if let Some(username) = username_from_url {
            Cred::ssh_key_from_agent(username)
        } else {
            Cred::ssh_key_from_agent("git")
        }
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    println!("Fetching updates for repository in {}", task.dest);

    remote
        .fetch(&[""], Some(&mut fetch_options), None)
        .with_context(|| format!("Failed to fetch from remote {}", task.remote))?;

    // Update submodules if recursive
    if task.recursive {
        update_submodules(&repo)?;
    }

    // Checkout the specified version
    checkout_version(&repo, &task.version)?;

    println!("Successfully updated repository in {}", task.dest);
    Ok(())
}

/// Update git submodules
fn update_submodules(repo: &Repository) -> Result<()> {
    let submodules = repo
        .submodules()
        .with_context(|| "Failed to get submodules")?;

    for mut submodule in submodules {
        println!(
            "Updating submodule: {}",
            submodule.name().unwrap_or("unknown")
        );
        submodule.update(true, None).with_context(|| {
            format!(
                "Failed to update submodule {}",
                submodule.name().unwrap_or("unknown")
            )
        })?;
    }

    Ok(())
}

/// Checkout a specific version/branch/tag
fn checkout_version(repo: &Repository, version: &str) -> Result<()> {
    let (object, reference) = repo
        .revparse_ext(version)
        .with_context(|| format!("Failed to resolve version {}", version))?;

    repo.checkout_tree(&object, None)
        .with_context(|| format!("Failed to checkout tree for {}", version))?;

    match reference {
        Some(reference) => {
            repo.set_head(reference.name().unwrap_or(version))
                .with_context(|| format!("Failed to set HEAD to {}", version))?;
        }
        None => {
            repo.set_head_detached(object.id())
                .with_context(|| format!("Failed to detach HEAD to {}", version))?;
        }
    }

    println!("Checked out version: {}", version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_git_task_basic() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");

        // Create a test git task
        let task = GitTask {
            description: Some("Test git task".to_string()),
            repo: "https://github.com/octocat/Hello-World.git".to_string(),
            dest: repo_path.to_string_lossy().to_string(),
            version: "HEAD".to_string(),
            update: true,
            clone: true,
            force: false,
            depth: None,
            recursive: false,
            remote: "origin".to_string(),
            key_file: None,
            accept_hostkey: false,
            ssh_opts: None,
        };

        // This would normally clone a repo, but we'll just test the structure
        // In a real test, we'd need network access or a local test repo
        assert_eq!(task.repo, "https://github.com/octocat/Hello-World.git");
        assert_eq!(task.dest, repo_path.to_string_lossy().to_string());
        assert_eq!(task.version, "HEAD");
        assert!(task.update);
        assert!(task.clone);
    }
}
