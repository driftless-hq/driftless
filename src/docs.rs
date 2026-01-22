//! Documentation generation utilities
//!
//! This module provides utilities for generating documentation from the codebase,
//! including task references, examples, and schema validation.

use crate::doc_extractor::{
    extract_all_facts_docs, extract_all_logs_docs, extract_all_task_docs, TaskDocumentation,
};
use anyhow::Result;

/// Generate documentation for all available configuration operations
pub fn generate_task_documentation() -> Result<String> {
    let mut docs = String::from("# Driftless Configuration Reference\n\n");
    docs.push_str(
        "Comprehensive reference for all available configuration components in Driftless.\n\n",
    );
    docs.push_str("This documentation is auto-generated from the Rust source code.\n\n");

    docs.push_str("## Overview\n\n");
    docs.push_str(
        "Driftless provides three main configuration components that work together to manage systems:\n\n",
    );
    docs.push_str(
        "- **Configuration Operations** (`apply`): Define and enforce desired system state\n",
    );
    docs.push_str(
        "- **Facts Collectors** (`facts`): Gather system metrics and inventory information\n",
    );
    docs.push_str("- **Log Sources/Outputs** (`logs`): Handle log collection and forwarding\n\n");

    // Extract documentation from source code for all task types
    let apply_docs = extract_all_task_docs()?;
    let facts_docs = extract_all_facts_docs()?;
    let logs_docs = extract_all_logs_docs()?;

    // Add detailed task type documentation for each component
    docs.push_str(&generate_apply_section(&apply_docs)?);
    docs.push_str(&generate_facts_section(&facts_docs)?);
    docs.push_str(&generate_logs_section(&logs_docs)?);

    // Add comprehensive examples section
    docs.push_str(&generate_examples_section(&apply_docs)?);

    Ok(docs)
}

/// Categorize a task type into a documentation category
fn categorize_task_type(task_type: &str) -> String {
    crate::apply::TaskRegistry::get_task_category(task_type)
}

/// Generate detailed documentation for all apply task types
fn generate_apply_section(
    task_docs: &std::collections::HashMap<String, TaskDocumentation>,
) -> Result<String> {
    let mut section = String::from("## Configuration Operations (`apply`)\n\n");
    section.push_str(
        "Configuration operations define desired system state and are executed idempotently. ",
    );
    section.push_str(
        "Each operation corresponds to a specific aspect of system configuration management.\n\n",
    );

    // Get all registered task types from the registry
    let registered_task_types = crate::apply::TaskRegistry::get_registered_task_types();

    // Group tasks by category dynamically
    let mut categories = std::collections::HashMap::new();

    for task_type in &registered_task_types {
        let category = categorize_task_type(task_type);
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(task_type.clone());
    }

    // Sort categories and tasks within categories
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

    for (category_name, mut task_types) in sorted_categories {
        task_types.sort();
        section.push_str(&format!("### {}\n\n", category_name));

        for task_type in task_types {
            if let Some(task_doc) = task_docs.get(&task_type) {
                section.push_str(&format!("#### {}\n\n", task_type));
                section.push_str(&format!("**Description**: {}\n\n", task_doc.description));

                if !task_doc.fields.is_empty() {
                    // Collect and sort fields: required first, then alphabetical
                    let mut required_fields = Vec::new();
                    let mut optional_fields = Vec::new();

                    for field in task_doc.fields.values() {
                        if field.required {
                            required_fields.push(field.clone());
                        } else {
                            optional_fields.push(field.clone());
                        }
                    }

                    // Sort each group alphabetically by field name
                    required_fields.sort_by(|a, b| a.name.cmp(&b.name));
                    optional_fields.sort_by(|a, b| a.name.cmp(&b.name));

                    // Display required fields first
                    if !required_fields.is_empty() {
                        section.push_str("**Required Fields**:\n\n");
                        for field in &required_fields {
                            section
                                .push_str(&format!("- `{}` ({}):\n", field.name, field.field_type));
                            // Indent each line of the description
                            for line in field.description.lines() {
                                if !line.trim().is_empty() {
                                    section.push_str(&format!("  {}\n", line));
                                } else {
                                    section.push('\n');
                                }
                            }
                            section.push('\n');
                        }
                    }

                    // Display optional fields second
                    if !optional_fields.is_empty() {
                        section.push_str("**Optional Fields**:\n\n");
                        for field in &optional_fields {
                            section
                                .push_str(&format!("- `{}` ({}):\n", field.name, field.field_type));
                            // Indent each line of the description
                            for line in field.description.lines() {
                                if !line.trim().is_empty() {
                                    section.push_str(&format!("  {}\n", line));
                                } else {
                                    section.push('\n');
                                }
                            }
                            section.push('\n');
                        }
                    }
                }

                // Add examples if available
                if !task_doc.examples.is_empty() {
                    section.push_str("**Examples**:\n\n");
                    for example in &task_doc.examples {
                        section.push_str(&format!("**{}**:\n\n", example.description));
                        section.push_str("**YAML Format**:\n\n");
                        section.push_str("```yaml\n");
                        section.push_str(&example.yaml);
                        section.push_str("\n```\n\n");

                        section.push_str("**JSON Format**:\n\n");
                        section.push_str("```json\n");
                        section.push_str(&example.json);
                        section.push_str("\n```\n\n");

                        section.push_str("**TOML Format**:\n\n");
                        section.push_str("```toml\n");
                        section.push_str(&example.toml);
                        section.push_str("\n```\n\n");
                    }
                }
            }
        }
    }

    Ok(section)
}

/// Generate comprehensive examples section
fn generate_examples_section(
    _task_docs: &std::collections::HashMap<String, TaskDocumentation>,
) -> Result<String> {
    let mut section = String::from("## Comprehensive Examples\n\n");
    section.push_str("This section provides complete examples showing how to use Driftless for common configuration management tasks.\n\n");

    // Add a complete example configuration
    section.push_str("### Complete Configuration Example\n\n");
    section.push_str("Here's a complete example showing a typical web server setup:\n\n");
    section.push_str("**YAML Format**:\n\n");
    section.push_str("```yaml\n");
    section.push_str("vars:\n");
    section.push_str("  web_user: www-data\n");
    section.push_str("  web_root: /var/www/html\n");
    section.push_str("  nginx_config: /etc/nginx/sites-available/default\n");
    section.push('\n');
    section.push_str("tasks:\n");
    section.push_str("  # Install required packages\n");
    section.push_str("  - type: package\n");
    section.push_str("    name: nginx\n");
    section.push_str("    state: present\n");
    section.push('\n');
    section.push_str("  # Create web directory\n");
    section.push_str("  - type: file\n");
    section.push_str("    path: \"{{ web_root }}\"\n");
    section.push_str("    state: present\n");
    section.push_str("    mode: \"0755\"\n");
    section.push_str("    owner: \"{{ web_user }}\"\n");
    section.push_str("    group: \"{{ web_user }}\"\n");
    section.push('\n');
    section.push_str("  # Configure nginx\n");
    section.push_str("  - type: file\n");
    section.push_str("    path: \"{{ nginx_config }}\"\n");
    section.push_str("    state: present\n");
    section.push_str("    content: |\n");
    section.push_str("      server {\n");
    section.push_str("          listen 80;\n");
    section.push_str("          root {{ web_root }};\n");
    section.push_str("          index index.html index.htm;\n");
    section.push('\n');
    section.push_str("          location / {\n");
    section.push_str("              try_files $uri $uri/ =404;\n");
    section.push_str("          }\n");
    section.push_str("      }\n");
    section.push_str("    mode: \"0644\"\n");
    section.push_str("    owner: root\n");
    section.push_str("    group: root\n");
    section.push('\n');
    section.push_str("  # Create index page\n");
    section.push_str("  - type: file\n");
    section.push_str("    path: \"{{ web_root }}/index.html\"\n");
    section.push_str("    state: present\n");
    section.push_str("    content: |\n");
    section.push_str("      <!DOCTYPE html>\n");
    section.push_str("      <html>\n");
    section.push_str("      <head><title>Welcome to Driftless</title></head>\n");
    section.push_str("      <body><h1>Hello from Driftless!</h1></body>\n");
    section.push_str("      </html>\n");
    section.push_str("    mode: \"0644\"\n");
    section.push_str("    owner: \"{{ web_user }}\"\n");
    section.push_str("    group: \"{{ web_user }}\"\n");
    section.push('\n');
    section.push_str("  # Start and enable nginx service\n");
    section.push_str("  - type: service\n");
    section.push_str("    name: nginx\n");
    section.push_str("    state: started\n");
    section.push_str("    enabled: true\n");
    section.push_str("```\n\n");

    section.push_str("**JSON Format**:\n\n");
    section.push_str("```json\n");
    section.push_str("{\n");
    section.push_str("  \"vars\": {\n");
    section.push_str("    \"web_user\": \"www-data\",\n");
    section.push_str("    \"web_root\": \"/var/www/html\",\n");
    section.push_str("    \"nginx_config\": \"/etc/nginx/sites-available/default\"\n");
    section.push_str("  },\n");
    section.push_str("  \"tasks\": [\n");
    section.push_str("    {\n");
    section.push_str("      \"type\": \"package\",\n");
    section.push_str("      \"name\": \"nginx\",\n");
    section.push_str("      \"state\": \"present\"\n");
    section.push_str("    },\n");
    section.push_str("    {\n");
    section.push_str("      \"type\": \"file\",\n");
    section.push_str("      \"path\": \"{{ web_root }}\",\n");
    section.push_str("      \"state\": \"present\",\n");
    section.push_str("      \"mode\": \"0755\",\n");
    section.push_str("      \"owner\": \"{{ web_user }}\",\n");
    section.push_str("      \"group\": \"{{ web_user }}\"\n");
    section.push_str("    },\n");
    section.push_str("    {\n");
    section.push_str("      \"type\": \"file\",\n");
    section.push_str("      \"path\": \"{{ nginx_config }}\",\n");
    section.push_str("      \"state\": \"present\",\n");
    section.push_str("      \"content\": \"server {\\n    listen 80;\\n    root {{ web_root }};\\n    index index.html index.htm;\\n\\n    location / {\\n        try_files $uri $uri/ =404;\\n    }\\n}\",\n");
    section.push_str("      \"mode\": \"0644\",\n");
    section.push_str("      \"owner\": \"root\",\n");
    section.push_str("      \"group\": \"root\"\n");
    section.push_str("    },\n");
    section.push_str("    {\n");
    section.push_str("      \"type\": \"file\",\n");
    section.push_str("      \"path\": \"{{ web_root }}/index.html\",\n");
    section.push_str("      \"state\": \"present\",\n");
    section.push_str("      \"content\": \"<!DOCTYPE html>\\n<html>\\n<head><title>Welcome to Driftless</title></head>\\n<body><h1>Hello from Driftless!</h1></body>\\n</html>\",\n");
    section.push_str("      \"mode\": \"0644\",\n");
    section.push_str("      \"owner\": \"{{ web_user }}\",\n");
    section.push_str("      \"group\": \"{{ web_user }}\"\n");
    section.push_str("    },\n");
    section.push_str("    {\n");
    section.push_str("      \"type\": \"service\",\n");
    section.push_str("      \"name\": \"nginx\",\n");
    section.push_str("      \"state\": \"started\",\n");
    section.push_str("      \"enabled\": true\n");
    section.push_str("    }\n");
    section.push_str("  ]\n");
    section.push_str("}\n");
    section.push_str("```\n\n");

    section.push_str("**TOML Format**:\n\n");
    section.push_str("```toml\n");
    section.push_str("[vars]\n");
    section.push_str("web_user = \"www-data\"\n");
    section.push_str("web_root = \"/var/www/html\"\n");
    section.push_str("nginx_config = \"/etc/nginx/sites-available/default\"\n");
    section.push('\n');
    section.push_str("[[tasks]]\n");
    section.push_str("type = \"package\"\n");
    section.push_str("name = \"nginx\"\n");
    section.push_str("state = \"present\"\n");
    section.push('\n');
    section.push_str("[[tasks]]\n");
    section.push_str("type = \"file\"\n");
    section.push_str("path = \"{{ web_root }}\"\n");
    section.push_str("state = \"present\"\n");
    section.push_str("mode = \"0755\"\n");
    section.push_str("owner = \"{{ web_user }}\"\n");
    section.push_str("group = \"{{ web_user }}\"\n");
    section.push('\n');
    section.push_str("[[tasks]]\n");
    section.push_str("type = \"file\"\n");
    section.push_str("path = \"{{ nginx_config }}\"\n");
    section.push_str("state = \"present\"\n");
    section.push_str("content = \"\"\"\n");
    section.push_str("server {\n");
    section.push_str("    listen 80;\n");
    section.push_str("    root {{ web_root }};\n");
    section.push_str("    index index.html index.htm;\n");
    section.push('\n');
    section.push_str("    location / {\n");
    section.push_str("    try_files $uri $uri/ =404;\n");
    section.push_str("    }\n");
    section.push_str("}\n");
    section.push_str("\"\"\"\n");
    section.push_str("mode = \"0644\"\n");
    section.push_str("owner = \"root\"\n");
    section.push_str("group = \"root\"\n");
    section.push('\n');
    section.push_str("[[tasks]]\n");
    section.push_str("type = \"file\"\n");
    section.push_str("path = \"{{ web_root }}/index.html\"\n");
    section.push_str("state = \"present\"\n");
    section.push_str("content = \"\"\"\n");
    section.push_str("<!DOCTYPE html>\n");
    section.push_str("<html>\n");
    section.push_str("<head><title>Welcome to Driftless</title></head>\n");
    section.push_str("<body><h1>Hello from Driftless!</h1></body>\n");
    section.push_str("</html>\n");
    section.push_str("\"\"\"\n");
    section.push_str("mode = \"0644\"\n");
    section.push_str("owner = \"{{ web_user }}\"\n");
    section.push_str("group = \"{{ web_user }}\"\n");
    section.push('\n');
    section.push_str("[[tasks]]\n");
    section.push_str("type = \"service\"\n");
    section.push_str("name = \"nginx\"\n");
    section.push_str("state = \"started\"\n");
    section.push_str("enabled = true\n");
    section.push_str("```\n\n");

    Ok(section)
}

/// Generate JSON schema for configuration validation
pub fn generate_json_schema() -> Result<String> {
    let mut schema = serde_json::Map::new();

    schema.insert(
        "$schema".to_string(),
        serde_json::Value::String("http://json-schema.org/draft-07/schema#".to_string()),
    );
    schema.insert(
        "title".to_string(),
        serde_json::Value::String("Driftless Configuration".to_string()),
    );
    schema.insert(
        "description".to_string(),
        serde_json::Value::String(
            "Configuration schema for Driftless configuration management".to_string(),
        ),
    );
    schema.insert(
        "type".to_string(),
        serde_json::Value::String("object".to_string()),
    );

    let mut properties = serde_json::Map::new();

    // Variables
    let mut vars_schema = serde_json::Map::new();
    vars_schema.insert(
        "type".to_string(),
        serde_json::Value::String("object".to_string()),
    );
    vars_schema.insert(
        "description".to_string(),
        serde_json::Value::String("Variables available to all tasks".to_string()),
    );
    vars_schema.insert(
        "additionalProperties".to_string(),
        serde_json::Value::Bool(true),
    );
    properties.insert("vars".to_string(), serde_json::Value::Object(vars_schema));

    // Tasks
    let mut tasks_schema = serde_json::Map::new();
    tasks_schema.insert(
        "type".to_string(),
        serde_json::Value::String("array".to_string()),
    );
    tasks_schema.insert(
        "description".to_string(),
        serde_json::Value::String("List of configuration operations to execute".to_string()),
    );
    tasks_schema.insert(
        "items".to_string(),
        serde_json::Value::Object(generate_task_schema()),
    );
    properties.insert("tasks".to_string(), serde_json::Value::Object(tasks_schema));

    schema.insert(
        "properties".to_string(),
        serde_json::Value::Object(properties),
    );

    let required = vec![serde_json::Value::String("tasks".to_string())];
    schema.insert("required".to_string(), serde_json::Value::Array(required));

    Ok(serde_json::to_string_pretty(&schema)?)
}

/// Generate schema for individual tasks
fn generate_task_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut task_schema = serde_json::Map::new();
    task_schema.insert(
        "type".to_string(),
        serde_json::Value::String("object".to_string()),
    );

    let mut properties = serde_json::Map::new();

    // Common fields
    let mut description_schema = serde_json::Map::new();
    description_schema.insert(
        "type".to_string(),
        serde_json::Value::String("string".to_string()),
    );
    description_schema.insert(
        "description".to_string(),
        serde_json::Value::String("Human-readable description of the task's purpose".to_string()),
    );
    properties.insert(
        "description".to_string(),
        serde_json::Value::Object(description_schema),
    );

    // Task type discriminator
    let mut type_schema = serde_json::Map::new();
    type_schema.insert(
        "type".to_string(),
        serde_json::Value::String("string".to_string()),
    );
    type_schema.insert(
        "description".to_string(),
        serde_json::Value::String("The type of configuration operation".to_string()),
    );

    let mut registered_task_types = crate::apply::TaskRegistry::get_registered_task_types();
    registered_task_types.sort();

    let enum_values = registered_task_types
        .into_iter()
        .map(serde_json::Value::String)
        .collect();

    type_schema.insert("enum".to_string(), serde_json::Value::Array(enum_values));
    properties.insert("type".to_string(), serde_json::Value::Object(type_schema));

    task_schema.insert(
        "properties".to_string(),
        serde_json::Value::Object(properties),
    );

    let required = vec![serde_json::Value::String("type".to_string())];
    task_schema.insert("required".to_string(), serde_json::Value::Array(required));

    task_schema
}

/// Generate detailed documentation for all facts collectors
fn generate_facts_section(
    _facts_docs: &std::collections::HashMap<String, TaskDocumentation>,
) -> Result<String> {
    let mut section = String::from("## Facts Collectors (`facts`)\n\n");
    section.push_str("Facts collectors gather system metrics and inventory information. ");
    section.push_str("These collectors run at specified intervals to provide monitoring data.\n\n");

    section.push_str("**Note**: Facts collectors are not yet implemented. ");
    section.push_str(
        "This section will be populated when facts collection functionality is added.\n\n",
    );

    // Placeholder for future facts collector documentation
    section.push_str("### Planned Collectors\n\n");
    section.push_str("- **system**: System information (OS, kernel, hardware)\n");
    section.push_str("- **cpu**: CPU usage and performance metrics\n");
    section.push_str("- **memory**: Memory usage statistics\n");
    section.push_str("- **disk**: Disk usage and I/O metrics\n");
    section.push_str("- **network**: Network interface statistics\n");
    section.push_str("- **process**: Process information and metrics\n");
    section.push_str("- **command**: Custom command output collection\n\n");

    Ok(section)
}

/// Generate detailed documentation for all logs sources and outputs
fn generate_logs_section(
    _logs_docs: &std::collections::HashMap<String, TaskDocumentation>,
) -> Result<String> {
    let mut section = String::from("## Log Sources/Outputs (`logs`)\n\n");
    section.push_str("Log sources and outputs handle log collection and forwarding. ");
    section
        .push_str("Sources tail log files while outputs forward logs to various destinations.\n\n");

    section.push_str("**Note**: Log collection is not yet implemented. ");
    section
        .push_str("This section will be populated when log collection functionality is added.\n\n");

    // Placeholder for future logs documentation
    section.push_str("### Planned Components\n\n");
    section.push_str("#### Sources\n");
    section.push_str("- **file**: Tail local log files\n");
    section.push_str("- **journald**: Systemd journal collection\n");
    section.push_str("- **syslog**: Syslog message collection\n\n");

    section.push_str("#### Outputs\n");
    section.push_str("- **file**: Write to local files\n");
    section.push_str("- **http**: Forward via HTTP/HTTPS\n");
    section.push_str("- **s3**: Store in Amazon S3\n");
    section.push_str("- **elasticsearch**: Send to Elasticsearch\n");
    section.push_str("- **syslog**: Forward to syslog\n\n");

    Ok(section)
}
