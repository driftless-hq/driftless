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
}

use crate::apply::templating;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

    // Render template with variables
    let rendered_content = render_template(&template_content, &task.vars)?;

    // Check if destination needs updating
    let needs_update = if dest_path.exists() {
        if task.force {
            true
        } else {
            // Check if rendered content differs from existing file
            match fs::read_to_string(dest_path) {
                Ok(existing_content) => existing_content != rendered_content,
                Err(_) => true, // Can't read existing file, assume update needed
            }
        }
    } else {
        true
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
    }

    Ok(())
}

/// Render template with variable substitution
fn render_template(template: &str, vars: &HashMap<String, serde_json::Value>) -> Result<String> {
    // Convert vars to JinjaValue
    let context = minijinja::Value::from_serialize(vars);

    templating::render_with_context(template, context)
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

        let result = render_template(template, &vars).unwrap();
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

        let result = render_template(template, &vars).unwrap();
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
        let result = render_template(template, &vars).unwrap();
        assert_eq!(result, "Hello world");

        // Test truncate filter
        let template = "{{ text | truncate(8) }}";
        let result = render_template(template, &vars).unwrap();
        assert_eq!(result, "hello...");

        // Test truncate with custom end
        let template = "{{ text | truncate(8, false, '***') }}";
        let result = render_template(template, &vars).unwrap();
        assert_eq!(result, "hello***");

        // Test truncate with killwords=true
        let template = "{{ text | truncate(8, true) }}";
        let result = render_template(template, &vars).unwrap();
        assert_eq!(result, "hello...");
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
        };

        let result = execute_template_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&dest_path).exists());
    }
}
