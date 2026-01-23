//! Documentation generation utilities
//!
//! This module provides utilities for generating documentation from the codebase,
//! including task references, examples, and schema validation.

use crate::doc_extractor::{
    extract_all_facts_docs, extract_all_logs_docs, extract_all_task_docs, TaskDocumentation,
};
use anyhow::Result;

/// Generate documentation for all available facts collectors
pub fn generate_facts_documentation() -> Result<String> {
    let mut docs = String::from("# Driftless Facts Reference\n\n");
    docs.push_str("Comprehensive reference for all available facts collectors in Driftless.\n\n");
    docs.push_str("This documentation is auto-generated from the Rust source code.\n\n");

    docs.push_str("## Overview\n\n");
    docs.push_str("Facts collectors gather system metrics and inventory information. ");
    docs.push_str(
        "Each collector corresponds to a specific type of system information or metric.\n\n",
    );

    // Extract documentation from source code for all facts collector types
    let facts_docs = extract_all_facts_docs()?;

    // Add detailed facts collector documentation
    docs.push_str(&generate_facts_section(&facts_docs)?);

    Ok(docs)
}

/// Generate documentation for all available log processors
pub fn generate_logs_documentation() -> Result<String> {
    let mut docs = String::from("# Driftless Logs Reference\n\n");
    docs.push_str(
        "Comprehensive reference for all available log sources and outputs in Driftless.\n\n",
    );
    docs.push_str("This documentation is auto-generated from the Rust source code.\n\n");

    docs.push_str("## Overview\n\n");
    docs.push_str("Log processors handle log collection and forwarding. ");
    docs.push_str("Each processor corresponds to a specific log source or output destination.\n\n");

    // Extract documentation from source code for all logs processor types
    let logs_docs = extract_all_logs_docs()?;

    // Add detailed logs processor documentation
    docs.push_str(&generate_logs_section(&logs_docs)?);

    Ok(docs)
}

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

    section.push_str("### Task Result Registration and Conditions\n\n");
    section.push_str(
        "All configuration operations support special fields for conditional execution and capturing results:\n\n",
    );
    section.push_str("- **`when`**: An optional expression (usually containing variables) that determines if the task should be executed. If the condition evaluates to `false`, the task is skipped.\n");
    section.push_str("- **`register`**: An optional variable name to capture the result of the task execution. The captured data varies by task type and can be used in subsequent tasks using template expansion (e.g., `{{ my_var.stdout }}`). This field only appears in the documentation for tasks that provide output results.\n\n");

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

                // Display registered outputs if available
                if !task_doc.register_outputs.is_empty() {
                    section.push_str("**Registered Outputs**:\n\n");
                    let mut sorted_outputs: Vec<_> = task_doc.register_outputs.values().collect();
                    sorted_outputs.sort_by(|a, b| a.name.cmp(&b.name));

                    for output in sorted_outputs {
                        section.push_str(&format!(
                            "- `{}` ({}): {}\n",
                            output.name, output.output_type, output.description
                        ));
                    }
                    section.push('\n');
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

/// Generate detailed documentation for all facts collector types
fn generate_facts_section(
    facts_docs: &std::collections::HashMap<String, TaskDocumentation>,
) -> Result<String> {
    let mut section = String::from("## Facts Collectors (`facts`)\n\n");
    section.push_str("Facts collectors gather system metrics and inventory information. ");
    section.push_str(
        "Each collector corresponds to a specific type of system information or metric.\n\n",
    );

    section.push_str("### Collector Configuration\n\n");
    section.push_str(
        "All facts collectors support common configuration fields for controlling collection behavior:\n\n",
    );
    section.push_str("- **`name`**: Collector name (used for metric names)\n");
    section.push_str("- **`enabled`**: Whether this collector is enabled (default: true)\n");
    section.push_str(
        "- **`poll_interval`**: Poll interval in seconds (how often to collect this metric)\n",
    );
    section.push_str("- **`labels`**: Additional labels for this collector\n\n");

    // Get all registered collector types from the registry
    let registered_collector_types = crate::facts::FactsRegistry::get_registered_collector_types();

    // Group collectors by category dynamically
    let mut categories = std::collections::HashMap::new();

    for collector_type in &registered_collector_types {
        let category = crate::facts::FactsRegistry::get_collector_category(collector_type);
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(collector_type.clone());
    }

    // Sort categories and collectors within categories
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

    for (category_name, mut collector_types) in sorted_categories {
        collector_types.sort();
        section.push_str(&format!("### {}\n\n", category_name));

        for collector_type in collector_types {
            if let Some(collector_doc) = facts_docs.get(&collector_type) {
                section.push_str(&format!("#### {}\n\n", collector_type));
                section.push_str(&format!(
                    "**Description**: {}\n\n",
                    collector_doc.description
                ));

                if !collector_doc.fields.is_empty() {
                    // Collect and sort fields: required first, then alphabetical
                    let mut required_fields = Vec::new();
                    let mut optional_fields = Vec::new();

                    for field in collector_doc.fields.values() {
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
                if !collector_doc.examples.is_empty() {
                    section.push_str("**Examples**:\n\n");
                    for example in &collector_doc.examples {
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

/// Generate detailed documentation for all logs processor types
fn generate_logs_section(
    logs_docs: &std::collections::HashMap<String, TaskDocumentation>,
) -> Result<String> {
    let mut section = String::from("## Log Sources/Outputs (`logs`)\n\n");
    section.push_str("Log processors handle log collection and forwarding. ");
    section
        .push_str("Each processor corresponds to a specific log source or output destination.\n\n");

    section.push_str("### Processor Configuration\n\n");
    section.push_str(
        "All log processors support common configuration fields for controlling processing behavior:\n\n",
    );
    section.push_str("- **`enabled`**: Whether this processor is enabled (default: true)\n");
    section.push_str("- **`name`**: Processor name for identification\n\n");

    // Get all registered processor types from the registry
    let registered_processor_types = crate::logs::LogsRegistry::get_registered_processor_types();

    // Group processors by category dynamically
    let mut categories = std::collections::HashMap::new();

    for processor_type in &registered_processor_types {
        let category = crate::logs::LogsRegistry::get_processor_category(processor_type);
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(processor_type.clone());
    }

    // Sort categories and processors within categories
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

    for (category_name, mut processor_types) in sorted_categories {
        processor_types.sort();
        section.push_str(&format!("### {}\n\n", category_name));

        for processor_type in processor_types {
            if let Some(processor_doc) = logs_docs.get(&processor_type) {
                section.push_str(&format!("#### {}\n\n", processor_type));
                section.push_str(&format!(
                    "**Description**: {}\n\n",
                    processor_doc.description
                ));

                if !processor_doc.fields.is_empty() {
                    // Collect and sort fields: required first, then alphabetical
                    let mut required_fields = Vec::new();
                    let mut optional_fields = Vec::new();

                    for field in processor_doc.fields.values() {
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
                if !processor_doc.examples.is_empty() {
                    section.push_str("**Examples**:\n\n");
                    for example in &processor_doc.examples {
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

/// Generate documentation for template filters and functions
pub fn generate_template_documentation() -> Result<String> {
    let mut docs = String::from("# Driftless Template Reference\n\n");
    docs.push_str("Comprehensive reference for all available Jinja2 template filters and functions in Driftless.\n\n");
    docs.push_str("This documentation is auto-generated from the Rust source code.\n\n");

    docs.push_str("## Overview\n\n");
    docs.push_str("Driftless uses Jinja2 templating for dynamic configuration values. ");
    docs.push_str("Templates support both filters (applied with `|` syntax) and functions (called directly).\n\n");

    docs.push_str("### Template Syntax\n\n");
    docs.push_str("```jinja2\n");
    docs.push_str("{{ variable | filter_name(arg1, arg2) }}\n");
    docs.push_str("{{ function_name(arg1, arg2) }}\n");
    docs.push_str("```\n\n");

    // Generate filters section
    docs.push_str(&generate_filters_section()?);
    docs.push_str(&generate_functions_section()?);

    docs.push_str("## Examples\n\n");
    docs.push_str("```yaml\n");
    docs.push_str("# Using filters\n");
    docs.push_str("path: \"/home/{{ username | lower }}\"\n");
    docs.push_str("config: \"{{ app_name | upper }}.conf\"\n");
    docs.push_str("truncated: \"{{ long_text | truncate(50) }}\"\n\n");
    docs.push_str("# Using functions\n");
    docs.push_str("length: \"{{ length(items) }}\"\n");
    docs.push_str("basename: \"{{ basename('/path/to/file.txt') }}\"\n");
    docs.push_str("env_var: \"{{ lookup('env', 'HOME') }}\"\n");
    docs.push_str("```\n\n");

    Ok(docs)
}

/// Generate documentation for template filters
fn generate_filters_section() -> Result<String> {
    let mut section = String::from("## Template Filters\n\n");
    section.push_str("Filters transform values in templates using the `|` syntax.\n\n");

    // Get all registered filters from the registry
    let registered_filters = crate::apply::templating::TemplateRegistry::get_registered_filters();

    // Group filters by category
    let mut categories = std::collections::HashMap::new();

    for filter_name in &registered_filters {
        let category = crate::apply::templating::TemplateRegistry::get_filter_category(filter_name)
            .unwrap_or_else(|| "Uncategorized".to_string());
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(filter_name.clone());
    }

    // Sort categories and filters within categories
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

    for (category_name, mut filter_names) in sorted_categories {
        filter_names.sort();
        section.push_str(&format!("### {}\n\n", category_name));

        for filter_name in filter_names {
            if let Some(description) =
                crate::apply::templating::TemplateRegistry::get_filter_description(&filter_name)
            {
                section.push_str(&format!("#### `{}`\n\n", filter_name));
                section.push_str(&format!("{}\n\n", description));

                // Add arguments if available
                if let Some(arguments) =
                    crate::apply::templating::TemplateRegistry::get_filter_arguments(&filter_name)
                {
                    if !arguments.is_empty() {
                        section.push_str("**Arguments**:\n\n");
                        for (arg_name, arg_desc) in &arguments {
                            if let Some(colon_pos) = arg_desc.find(':') {
                                let type_part = &arg_desc[..colon_pos];
                                let desc_part = &arg_desc[colon_pos + 1..].trim_start();
                                section.push_str(&format!(
                                    "- `{}` ({}): {}\n",
                                    arg_name, type_part, desc_part
                                ));
                            } else {
                                section.push_str(&format!("- `{}`: {}\n", arg_name, arg_desc));
                            }
                        }
                        section.push('\n');
                    }
                }

                // Add usage example
                section.push_str("**Usage**:\n\n");
                if let Some(arguments) =
                    crate::apply::templating::TemplateRegistry::get_filter_arguments(&filter_name)
                {
                    if arguments.is_empty() {
                        section.push_str(&format!(
                            "```jinja2\n{{{{ value | {} }}}}\n```\n\n",
                            filter_name
                        ));
                    } else {
                        // Generate example with actual values for known filters
                        let example_usage = match filter_name.as_str() {
                            "truncate" => "```jinja2\n{{ value | truncate(50) }}\n{{ value | truncate(20, \"...\") }}\n{{ value | truncate(30, true, \"[truncated]\") }}\n```".to_string(),
                            _ => {
                                // For other filters, use parameter names
                                let param_names: Vec<&str> = arguments
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                format!("```jinja2\n{{{{ value | {}({}) }}}}\n```", filter_name, param_names.join(", "))
                            }
                        };
                        section.push_str(&example_usage);
                        section.push('\n');
                    }
                } else {
                    section.push_str(&format!(
                        "```jinja2\n{{{{ value | {} }}}}\n```\n\n",
                        filter_name
                    ));
                }
            }
        }
    }

    Ok(section)
}

/// Generate documentation for template functions
fn generate_functions_section() -> Result<String> {
    let mut section = String::from("## Template Functions\n\n");
    section.push_str("Functions perform operations and return values in templates.\n\n");

    // Get all registered functions from the registry
    let registered_functions =
        crate::apply::templating::TemplateRegistry::get_registered_functions();

    // Group functions by category
    let mut categories = std::collections::HashMap::new();

    for function_name in &registered_functions {
        let category =
            crate::apply::templating::TemplateRegistry::get_function_category(function_name)
                .unwrap_or_else(|| "Uncategorized".to_string());
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(function_name.clone());
    }

    // Sort categories and functions within categories
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

    for (category_name, mut function_names) in sorted_categories {
        function_names.sort();
        section.push_str(&format!("### {}\n\n", category_name));

        for function_name in &function_names {
            if let Some(description) =
                crate::apply::templating::TemplateRegistry::get_function_description(function_name)
            {
                section.push_str(&format!("#### `{}`\n\n", function_name));
                section.push_str(&format!("{}\n\n", description));

                // Add arguments if available
                if let Some(arguments) =
                    crate::apply::templating::TemplateRegistry::get_function_arguments(
                        function_name,
                    )
                {
                    if !arguments.is_empty() {
                        section.push_str("**Arguments**:\n\n");
                        for (name, description) in &arguments {
                            if let Some(colon_pos) = description.find(':') {
                                let type_part = &description[..colon_pos];
                                let desc_part = &description[colon_pos + 1..].trim_start();
                                section.push_str(&format!(
                                    "- `{}` ({}): {}\n",
                                    name, type_part, desc_part
                                ));
                            } else {
                                section.push_str(&format!("- `{}`: {}\n", name, description));
                            }
                        }
                        section.push('\n');
                    }
                }

                // Add usage example
                section.push_str("**Usage**:\n\n");
                if let Some(arguments) =
                    crate::apply::templating::TemplateRegistry::get_function_arguments(
                        function_name,
                    )
                {
                    if arguments.is_empty() {
                        section.push_str(&format!(
                            "```jinja2\n{{{{ {}() }}}}\n```\n\n",
                            function_name
                        ));
                    } else {
                        // Generate example with actual values for known functions
                        let example_usage = match function_name.as_str() {
                            "lookup" => "```jinja2\n{{ lookup('env', 'HOME') }}\n{{ lookup('env', 'USER') }}\n```".to_string(),
                            "basename" => "```jinja2\n{{ basename('/path/to/file.txt') }}\n{{ basename(path_variable) }}\n```".to_string(),
                            "dirname" => "```jinja2\n{{ dirname('/path/to/file.txt') }}\n{{ dirname(path_variable) }}\n```".to_string(),
                            "length" => "```jinja2\n{{ length('hello') }}\n{{ length(items) }}\n{{ length(my_object) }}\n```".to_string(),
                            _ => {
                                // For other functions, use parameter names
                                let param_names: Vec<&str> = arguments
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                format!("```jinja2\n{{{{ {}({}) }}}}\n```", function_name, param_names.join(", "))
                            }
                        };
                        section.push_str(&example_usage);
                        section.push('\n');
                    }
                } else {
                    section.push_str(&format!(
                        "```jinja2\n{{{{ {}() }}}}\n```\n\n",
                        function_name
                    ));
                }
            }
        }
    }

    Ok(section)
}
