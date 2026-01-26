//! Template rendering task executor
//!
//! Handles rendering template files with variable substitution.
//!
//! # Examples
//!
//! ## Render a template
//!
//! This example renders a template file with variables.
//!
//! **YAML Format:**
//! ```yaml
//! - type: template
//!   description: "Render nginx configuration"
//!   src: /templates/nginx.conf.j2
//!   dest: /etc/nginx/sites-available/default
//!   state: present
//!   vars:
//!     server_name: example.com
//!     port: 80
//!     root_dir: /var/www/html
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "template",
//!   "description": "Render nginx configuration",
//!   "src": "/templates/nginx.conf.j2",
//!   "dest": "/etc/nginx/sites-available/default",
//!   "state": "present",
//!   "vars": {
//!     "server_name": "example.com",
//!     "port": 80,
//!     "root_dir": "/var/www/html"
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "template"
//! description = "Render nginx configuration"
//! src = "/templates/nginx.conf.j2"
//! dest = "/etc/nginx/sites-available/default"
//! state = "present"
//!
//! [tasks.vars]
//! server_name = "example.com"
//! port = 80
//! root_dir = "/var/www/html"
//! ```
//!
//! ## Render template with backup
//!
//! This example renders a template and creates a backup of the destination.
//!
//! **YAML Format:**
//! ```yaml
//! - type: template
//!   description: "Update config with backup"
//!   src: /templates/app.conf.j2
//!   dest: /etc/myapp/config.conf
//!   state: present
//!   backup: true
//!   vars:
//!     database_host: localhost
//!     database_port: 5432
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "template",
//!   "description": "Update config with backup",
//!   "src": "/templates/app.conf.j2",
//!   "dest": "/etc/myapp/config.conf",
//!   "state": "present",
//!   "backup": true,
//!   "vars": {
//!     "database_host": "localhost",
//!     "database_port": 5432
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "template"
//! description = "Update config with backup"
//! src = "/templates/app.conf.j2"
//! dest = "/etc/myapp/config.conf"
//! state = "present"
//! backup = true
//!
//! [tasks.vars]
//! database_host = "localhost"
//! database_port = 5432
//! ```
//!
//! ## Remove rendered template
//!
//! This example removes a file that was rendered from a template.
//!
//! **YAML Format:**
//! ```yaml
//! - type: template
//!   description: "Remove rendered configuration"
//!   src: /templates/old.conf.j2
//!   dest: /etc/oldapp/config.conf
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "template",
//!   "description": "Remove rendered configuration",
//!   "src": "/templates/old.conf.j2",
//!   "dest": "/etc/oldapp/config.conf",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "template"
//! description = "Remove rendered configuration"
//! src = "/templates/old.conf.j2"
//! dest = "/etc/oldapp/config.conf"
//! state = "absent"
//! ```

/// Template state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateState {
    /// Ensure template is rendered
    Present,
    /// Ensure template output is removed
    Absent,
}

/// Template rendering task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source template file
    pub src: String,
    /// Destination file
    pub dest: String,
    /// Template state
    pub state: TemplateState,
    /// Variables for template rendering
    #[serde(default)]
    pub vars: std::collections::HashMap<String, serde_json::Value>,
    /// Backup destination before templating
    #[serde(default)]
    pub backup: bool,
    /// Force template rendering
    #[serde(default)]
    pub force: bool,
    /// Template directory for includes/imports
    ///
    /// Directory containing templates that can be included or imported.
    /// If not specified, includes/imports will not work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_dir: Option<String>,
}

use crate::apply::templating;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// State information for tracking template rendering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TemplateStateInfo {
    /// SHA256 checksum of the source template file
    source_checksum: String,
    /// Size of the source template file
    source_size: u64,
    /// Last modification time of the source template file
    source_modified: SystemTime,
    /// SHA256 checksum of the rendered output
    rendered_checksum: String,
    /// Size of the rendered output
    rendered_size: u64,
    /// Variables used for rendering (serialized as JSON)
    variables_hash: String,
    /// Last modification time when template was rendered
    rendered_at: SystemTime,
}

/// Execute a template task
pub async fn execute_template_task(task: &TemplateTask, dry_run: bool) -> Result<()> {
    match task.state {
        TemplateState::Present => ensure_template_rendered(task, dry_run).await,
        TemplateState::Absent => ensure_template_not_rendered(task, dry_run).await,
    }
}

/// Ensure template is rendered to destination
async fn ensure_template_rendered(task: &TemplateTask, dry_run: bool) -> Result<()> {
    let src_path = Path::new(&task.src);
    let dest_path = Path::new(&task.dest);

    // Check if source template exists
    if !src_path.exists() {
        return Err(anyhow::anyhow!(
            "Template source does not exist: {}",
            task.src
        ));
    }

    // Read template content
    let template_content = fs::read_to_string(src_path)
        .with_context(|| format!("Failed to read template {}", task.src))?;

    // Determine template directory
    let template_dir = if let Some(ref dir) = task.template_dir {
        Some(std::path::Path::new(dir))
    } else {
        src_path.parent()
    };

    // Render template with variables
    let rendered_content = render_template(&template_content, &task.vars, template_dir)?;

    // Check if destination needs updating using state tracking
    let needs_update = if dest_path.exists() {
        if task.force {
            true // Force rendering even if destination exists
        } else {
            // Load previous template state
            match load_template_state(&task.dest) {
                Ok(Some(prev_state)) => {
                    // Check if source template has changed
                    let src_metadata = src_path
                        .metadata()
                        .with_context(|| format!("Failed to get metadata for {}", task.src))?;

                    let src_modified = src_metadata.modified().with_context(|| {
                        format!("Failed to get modification time for {}", task.src)
                    })?;

                    // Check if variables have changed
                    let current_vars_hash = calculate_variables_hash(&task.vars)?;

                    // If source modification time is newer, or variables changed, or we can't determine,
                    // check the checksums
                    if src_modified > prev_state.rendered_at
                        || current_vars_hash != prev_state.variables_hash
                    {
                        true
                    } else {
                        // Calculate current source checksum
                        match calculate_file_checksum(src_path) {
                            Ok(current_checksum) => current_checksum != prev_state.source_checksum,
                            Err(_) => true, // If we can't calculate checksum, assume changed
                        }
                    }
                }
                Ok(None) => {
                    // No previous state, check if destination exists (assume it needs update)
                    true
                }
                Err(_) => {
                    // Error loading state, assume update needed
                    true
                }
            }
        }
    } else {
        true // Destination doesn't exist
    };

    if !needs_update {
        println!("Template {} is already rendered at {}", task.src, task.dest);
        return Ok(());
    }

    if dry_run {
        println!("Would render template {} to {}", task.src, task.dest);
        if task.backup && dest_path.exists() {
            println!("  (would backup existing file)");
        }
    } else {
        // Backup destination if requested
        if task.backup && dest_path.exists() {
            let backup_path = format!("{}.backup", task.dest);
            fs::copy(&task.dest, &backup_path)
                .with_context(|| format!("Failed to backup {} to {}", task.dest, backup_path))?;
            println!("Backed up {} to {}", task.dest, backup_path);
        }

        // Ensure destination directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directories for {}", task.dest)
            })?;
        }

        // Write rendered content
        fs::write(dest_path, rendered_content)
            .with_context(|| format!("Failed to write rendered template to {}", task.dest))?;

        println!("Rendered template {} to {}", task.src, task.dest);

        // Save template state for change detection
        let src_metadata = src_path
            .metadata()
            .with_context(|| format!("Failed to get metadata for {}", task.src))?;
        let dest_metadata = dest_path
            .metadata()
            .with_context(|| format!("Failed to get metadata for {}", task.dest))?;

        let src_checksum = calculate_file_checksum(src_path)?;
        let dest_checksum = calculate_file_checksum(dest_path)?;
        let vars_hash = calculate_variables_hash(&task.vars)?;

        let state = TemplateStateInfo {
            source_checksum: src_checksum,
            source_size: src_metadata.len(),
            source_modified: src_metadata
                .modified()
                .with_context(|| format!("Failed to get modification time for {}", task.src))?,
            rendered_checksum: dest_checksum,
            rendered_size: dest_metadata.len(),
            variables_hash: vars_hash,
            rendered_at: SystemTime::now(),
        };

        if let Err(e) = save_template_state(&task.dest, &state) {
            println!("Warning: Failed to save template state: {}", e);
        }
    }

    Ok(())
}

/// Ensure template output is removed
async fn ensure_template_not_rendered(task: &TemplateTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    if !dest_path.exists() {
        println!("Template output does not exist: {}", task.dest);
        return Ok(());
    }

    // This is a simplified implementation - in practice, we'd need to track
    // which files were created by template rendering vs other files
    if dry_run {
        println!("Would remove template output: {}", task.dest);
    } else {
        fs::remove_file(dest_path)
            .with_context(|| format!("Failed to remove template output {}", task.dest))?;
        println!("Removed template output: {}", task.dest);

        // Clean up the state file
        let state_path = get_template_state_path(&task.dest);
        if Path::new(&state_path).exists() {
            if let Err(e) = fs::remove_file(&state_path) {
                println!(
                    "Warning: Failed to remove template state file {}: {}",
                    state_path, e
                );
            }
        }
    }

    Ok(())
}

/// Get the state file path for a template operation
fn get_template_state_path(dest: &str) -> String {
    format!("{}.driftless-template-state", dest)
}

/// Load template state from file
fn load_template_state(dest: &str) -> Result<Option<TemplateStateInfo>> {
    let state_path = get_template_state_path(dest);
    if !Path::new(&state_path).exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&state_path)
        .with_context(|| format!("Failed to read template state file: {}", state_path))?;

    let state: TemplateStateInfo = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse template state file: {}", state_path))?;

    Ok(Some(state))
}

/// Save template state to file
fn save_template_state(dest: &str, state: &TemplateStateInfo) -> Result<()> {
    let state_path = get_template_state_path(dest);
    let content = serde_json::to_string_pretty(state)
        .with_context(|| format!("Failed to serialize template state for: {}", dest))?;

    fs::write(&state_path, content)
        .with_context(|| format!("Failed to write template state file: {}", state_path))?;

    Ok(())
}

/// Calculate SHA256 checksum of a file
fn calculate_file_checksum(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut hasher = Sha256::new();
    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open file for checksum: {}", path.display()))?;

    let mut buffer = [0; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read file for checksum: {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Calculate hash of variables for state tracking
fn calculate_variables_hash(vars: &HashMap<String, serde_json::Value>) -> Result<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    let sorted_vars: std::collections::BTreeMap<_, _> = vars.iter().collect();

    for (key, value) in sorted_vars {
        hasher.update(key.as_bytes());
        hasher.update(b"=");
        hasher.update(serde_json::to_string(value)?.as_bytes());
        hasher.update(b";");
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Render template with variable substitution
fn render_template(
    template: &str,
    vars: &HashMap<String, serde_json::Value>,
    template_dir: Option<&std::path::Path>,
) -> Result<String> {
    // Convert vars to JinjaValue
    let context = minijinja::Value::from_serialize(vars);

    templating::render_template_with_loader(template, "main", template_dir, context)
        .with_context(|| "Failed to render template".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_template_rendering_dry_run() {
        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        let template_content = "Hello {{name}}! Today is {{day}}.";
        fs::write(&src_path, template_content).unwrap();

        let dest_path = src_path.clone() + ".rendered";

        let mut vars = HashMap::new();
        vars.insert(
            "name".to_string(),
            serde_json::Value::String("World".to_string()),
        );
        vars.insert(
            "day".to_string(),
            serde_json::Value::String("Monday".to_string()),
        );

        let task = TemplateTask {
            description: None,
            src: src_path.clone(),
            dest: dest_path.clone(),
            state: TemplateState::Present,
            vars,
            backup: false,
            force: false,
            template_dir: None,
        };

        let result = execute_template_task(&task, true).await;
        assert!(result.is_ok());
        assert!(!Path::new(&dest_path).exists()); // File shouldn't exist in dry run
    }

    #[tokio::test]
    async fn test_template_rendering_real() {
        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        let template_content = "Hello {{name}}! Count: {{count}}";
        fs::write(&src_path, template_content).unwrap();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_str().unwrap().to_string();
        drop(dest_file); // Remove temp file

        let mut vars = HashMap::new();
        vars.insert(
            "name".to_string(),
            serde_json::Value::String("Alice".to_string()),
        );
        vars.insert("count".to_string(), serde_json::json!(42));

        let task = TemplateTask {
            description: None,
            src: src_path.clone(),
            dest: dest_path.clone(),
            state: TemplateState::Present,
            vars,
            backup: false,
            force: false,
            template_dir: None,
        };

        let result = execute_template_task(&task, false).await;
        assert!(result.is_ok());
        assert!(Path::new(&dest_path).exists());

        let rendered = fs::read_to_string(&dest_path).unwrap();
        assert_eq!(rendered, "Hello Alice! Count: 42");
    }

    #[test]
    fn test_template_variable_substitution() {
        let template = "User: {{username}}, Age: {{age}}, Active: {{active}}";
        let mut vars = HashMap::new();
        vars.insert(
            "username".to_string(),
            serde_json::Value::String("john".to_string()),
        );
        vars.insert("age".to_string(), serde_json::json!(25));
        vars.insert("active".to_string(), serde_json::json!(true));

        let result = render_template(template, &vars, None).unwrap();
        assert_eq!(result, "User: john, Age: 25, Active: true");
    }

    #[test]
    fn test_template_default_values() {
        let template = "DB: {{db_host | default('localhost')}}, Port: {{db_port | default(5432)}}";
        let mut vars = HashMap::new();
        vars.insert(
            "db_host".to_string(),
            serde_json::Value::String("prod-db".to_string()),
        );
        // db_port not provided, should use default

        let result = render_template(template, &vars, None).unwrap();
        assert_eq!(result, "DB: prod-db, Port: 5432");
    }

    #[test]
    fn test_template_capitalize_and_truncate_filters() {
        let mut vars = HashMap::new();
        vars.insert(
            "text".to_string(),
            serde_json::Value::String("hello world".to_string()),
        );

        // Test capitalize filter
        let template = "{{ text | capitalize }}";
        let result = render_template(template, &vars, None).unwrap();
        assert_eq!(result, "Hello world");

        // Test truncate filter
        let template = "{{ text | truncate(8) }}";
        let result = render_template(template, &vars, None).unwrap();
        assert_eq!(result, "hello...");

        // Test truncate with custom end
        let template = "{{ text | truncate(8, false, '***') }}";
        let result = render_template(template, &vars, None).unwrap();
        assert_eq!(result, "hello***");

        // Test truncate with killwords=true
        let template = "{{ text | truncate(8, true) }}";
        let result = render_template(template, &vars, None).unwrap();
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_template_include() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path();

        // Create an included template
        let included_path = template_dir.join("included.j2");
        std::fs::write(&included_path, "Included content: {{ var }}").unwrap();

        // Create main template that includes the other
        let main_template = "Main template\n{% include 'included.j2' %}\nEnd";

        let mut vars = HashMap::new();
        vars.insert("var".to_string(), serde_json::json!("test_value"));

        let result = render_template(main_template, &vars, Some(template_dir)).unwrap();
        assert_eq!(result, "Main template\nIncluded content: test_value\nEnd");
    }

    #[test]
    fn test_template_import() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path();

        // Create a template with macros
        let macros_path = template_dir.join("macros.j2");
        std::fs::write(
            &macros_path,
            "{% macro greet(name) %}Hello {{ name }}!{% endmacro %}",
        )
        .unwrap();

        // Create main template that imports and uses the macro
        let main_template = "{% import 'macros.j2' as macros %}{{ macros.greet('World') }}";

        let vars = HashMap::new();

        let result = render_template(main_template, &vars, Some(template_dir)).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[tokio::test]
    async fn test_template_remove_output() {
        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_str().unwrap().to_string();
        fs::write(&dest_path, "some content").unwrap();

        let task = TemplateTask {
            description: None,
            src: "/nonexistent/template".to_string(), // Source doesn't matter for removal
            dest: dest_path.clone(),
            state: TemplateState::Absent,
            vars: HashMap::new(),
            backup: false,
            force: false,
            template_dir: None,
        };

        let result = execute_template_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&dest_path).exists());
    }
}
