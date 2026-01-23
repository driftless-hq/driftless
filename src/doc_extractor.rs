//! Documentation extraction utilities
//!
//! This module provides functionality to extract documentation and examples
//! from Rust source files at build time.

use anyhow::Result;
use std::collections::HashMap;
use std::fs;

/// Extracted documentation for a task type
#[derive(Debug, Clone)]
pub struct TaskDocumentation {
    /// Human-readable description
    pub description: String,
    /// Field documentation
    pub fields: HashMap<String, FieldDocumentation>,
    /// Examples from the implementation file
    pub examples: Vec<TaskExample>,
    /// Registered output documentation
    pub register_outputs: HashMap<String, RegisterOutputDocumentation>,
}

/// Documentation for a task field
#[derive(Debug, Clone)]
pub struct FieldDocumentation {
    /// Field name
    pub name: String,
    /// Field type description
    pub field_type: String,
    /// Whether the field is required
    pub required: bool,
    /// Human-readable description
    pub description: String,
}

/// Documentation for a registered output
#[derive(Debug, Clone)]
pub struct RegisterOutputDocumentation {
    /// Output name
    pub name: String,
    /// Output type description
    pub output_type: String,
    /// Human-readable description
    pub description: String,
}

/// Example for a task
#[derive(Debug, Clone)]
pub struct TaskExample {
    /// Description of what the example demonstrates
    pub description: String,
    /// YAML format example
    pub yaml: String,
    /// JSON format example
    pub json: String,
    /// TOML format example
    pub toml: String,
}

/// Extract documentation for all tasks
pub fn extract_all_task_docs() -> Result<HashMap<String, TaskDocumentation>> {
    let mut docs = HashMap::new();

    // Get all registered task types from the registry
    let registered_task_types = crate::apply::TaskRegistry::get_registered_task_types();

    // Extract from each task file
    for task_type in registered_task_types {
        let filename = crate::apply::TaskAction::task_filename(&task_type);
        let file_path = format!("src/apply/{}.rs", filename);
        if let Ok(content) = fs::read_to_string(&file_path) {
            extract_task_struct_docs(&content, &mut docs, &task_type)?;
        }
    }

    // Extract examples from implementation files
    extract_examples_from_files(&mut docs)?;

    // Add common fields to all tasks
    add_common_task_fields(&mut docs);

    Ok(docs)
}

/// Add common fields (register, when) to all task documentation
fn add_common_task_fields(docs: &mut HashMap<String, TaskDocumentation>) {
    for task_doc in docs.values_mut() {
        // Always add 'when' field documentation
        task_doc.fields.insert(
            "when".to_string(),
            FieldDocumentation {
                name: "when".to_string(),
                field_type: "Option<String>".to_string(),
                required: false,
                description: "Optional condition to determine if the task should run".to_string(),
            },
        );

        // Only add 'register' field documentation if the task has outputs to register
        if !task_doc.register_outputs.is_empty() {
            task_doc.fields.insert(
                "register".to_string(),
                FieldDocumentation {
                    name: "register".to_string(),
                    field_type: "Option<String>".to_string(),
                    required: false,
                    description: "Optional variable name to register the task result in"
                        .to_string(),
                },
            );
        }
    }
}

/// Extract documentation for all facts collectors
pub fn extract_all_facts_docs() -> Result<HashMap<String, TaskDocumentation>> {
    let mut docs = HashMap::new();

    // Get all registered collector types from the registry
    let registered_collector_types = crate::facts::FactsRegistry::get_registered_collector_types();

    // Extract from facts/mod.rs
    let file_path = "src/facts/mod.rs";
    if let Ok(content) = fs::read_to_string(file_path) {
        extract_facts_struct_docs(&content, &mut docs, &registered_collector_types)?;
    }

    // Extract examples from implementation files
    extract_facts_examples_from_files(&mut docs)?;

    // Add common fields to all facts collectors
    add_common_facts_fields(&mut docs);

    Ok(docs)
}

/// Extract documentation for all logs sources and outputs
pub fn extract_all_logs_docs() -> Result<HashMap<String, TaskDocumentation>> {
    let mut docs = HashMap::new();

    // Get all registered processor types from the registry
    let registered_processor_types = crate::logs::LogsRegistry::get_registered_processor_types();

    // Extract from logs/mod.rs
    let file_path = "src/logs/mod.rs";
    if let Ok(content) = fs::read_to_string(file_path) {
        extract_logs_struct_docs(&content, &mut docs, &registered_processor_types)?;
    }

    // Extract examples from implementation files
    extract_logs_examples_from_files(&mut docs)?;

    // Add common fields to all logs processors
    add_common_logs_fields(&mut docs);

    Ok(docs)
}

/// Extract task struct documentation from mod.rs
fn extract_task_struct_docs(
    content: &str,
    docs: &mut HashMap<String, TaskDocumentation>,
    task_type: &str,
) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for task struct definitions
        if lines[i].contains("#[derive") && lines[i + 1].contains("pub struct") {
            let struct_line = lines[i + 1];
            if let Some(struct_name) = extract_struct_name(struct_line) {
                // If it ends with Task, assume it's the main task struct for this file
                if struct_name.ends_with("Task") {
                    let mut task_doc = TaskDocumentation {
                        description: String::new(),
                        fields: HashMap::new(),
                        examples: Vec::new(),
                        register_outputs: HashMap::new(),
                    };

                    // Extract struct documentation (go backwards to find doc comments)
                    let mut doc_lines = Vec::new();
                    let mut j = i;
                    while j > 0 {
                        j -= 1;
                        let line = lines[j].trim();
                        if line.starts_with("///") {
                            let content = line.trim_start_matches("///").trim();
                            doc_lines.insert(0, content.to_string());
                        } else if !line.is_empty()
                            && !line.starts_with("//")
                            && !line.starts_with("#[")
                        {
                            break;
                        }
                    }

                    if !doc_lines.is_empty() {
                        // Separate description from register outputs
                        let mut description_lines = Vec::new();
                        let mut in_outputs_section = false;

                        for line in doc_lines {
                            let trimmed = line.trim();
                            if trimmed == "# Registered Outputs" {
                                in_outputs_section = true;
                                continue;
                            }

                            if in_outputs_section {
                                if trimmed.starts_with("- `") {
                                    if let Some(type_start) = trimmed.find("` (") {
                                        if let Some(type_end) = trimmed.find("): ") {
                                            let name = trimmed[3..type_start].to_string();
                                            let output_type =
                                                trimmed[type_start + 3..type_end].to_string();
                                            let desc = trimmed[type_end + 3..].to_string();
                                            task_doc.register_outputs.insert(
                                                name.clone(),
                                                RegisterOutputDocumentation {
                                                    name,
                                                    output_type,
                                                    description: desc,
                                                },
                                            );
                                        }
                                    } else if let Some(pos) = trimmed.find("`: ") {
                                        let name = trimmed[3..pos].to_string();
                                        let desc = trimmed[pos + 3..].to_string();
                                        task_doc.register_outputs.insert(
                                            name.clone(),
                                            RegisterOutputDocumentation {
                                                name,
                                                output_type: "Unknown".to_string(),
                                                description: desc,
                                            },
                                        );
                                    }
                                }
                            } else {
                                description_lines.push(line);
                            }
                        }
                        task_doc.description = description_lines.join("\n").trim().to_string();
                    }

                    // Extract field documentation
                    i += 2; // Skip the derive and struct lines
                    while i < lines.len() && !lines[i].contains('}') {
                        if lines[i].contains("pub ") && lines[i].contains(": ") {
                            if let Some(field_doc) = extract_field_doc(&lines, &mut i) {
                                task_doc.fields.insert(field_doc.name.clone(), field_doc);
                            }
                        } else {
                            i += 1;
                        }
                    }

                    docs.insert(task_type.to_string(), task_doc);
                }
            }
        }
        i += 1;
    }

    Ok(())
}

/// Extract struct name from a struct definition line
fn extract_struct_name(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 && parts[0] == "pub" && parts[1] == "struct" {
        Some(parts[2].to_string())
    } else {
        None
    }
}

/// Extract field documentation
fn extract_field_doc(lines: &[&str], i: &mut usize) -> Option<FieldDocumentation> {
    let field_line = lines[*i];
    let field_name = extract_field_name(field_line)?;

    // Extract field documentation (go backwards to find doc comments)
    let mut doc_lines = Vec::new();
    let mut j = *i;
    while j > 0 {
        j -= 1;
        let line = lines[j].trim();
        if line.starts_with("///") {
            doc_lines.insert(0, line.trim_start_matches("///").trim());
        } else if line.starts_with("#[") || line.is_empty() {
            // Skip attributes and empty lines, continue looking for docs
            continue;
        } else if !line.is_empty() && !line.starts_with("//") {
            // Stop at actual code that's not an attribute
            break;
        }
    }

    let description = if doc_lines.is_empty() {
        "No description available".to_string()
    } else {
        doc_lines.join("\n")
    };

    let required = !field_line.contains("Option<");
    let field_type = extract_field_type(field_line)?;

    *i += 1;

    Some(FieldDocumentation {
        name: field_name,
        field_type,
        required,
        description,
    })
}

/// Extract field name from a field definition line
fn extract_field_name(line: &str) -> Option<String> {
    let line = line.trim();
    line.find("pub ").and_then(|pub_pos| {
        let after_pub = &line[pub_pos + 4..];
        after_pub
            .find(": ")
            .map(|colon_pos| after_pub[..colon_pos].trim().to_string())
    })
}

/// Extract field type from a field definition line
fn extract_field_type(line: &str) -> Option<String> {
    let line = line.trim();
    if let Some(colon_pos) = line.find(": ") {
        let type_part = &line[colon_pos + 2..];

        // Find the comma that ends this field (not commas inside generics)
        let mut paren_depth = 0;
        let mut bracket_depth = 0;
        let mut angle_depth = 0;

        for (i, c) in type_part.char_indices() {
            match c {
                '(' => paren_depth += 1,
                ')' => paren_depth -= 1,
                '[' => bracket_depth += 1,
                ']' => bracket_depth -= 1,
                '<' => angle_depth += 1,
                '>' => angle_depth -= 1,
                ',' => {
                    // This is the field-ending comma if we're at depth 0
                    if paren_depth == 0 && bracket_depth == 0 && angle_depth == 0 {
                        let type_str = type_part[..i].trim().to_string();
                        return Some(simplify_type_name(&type_str));
                    }
                }
                _ => {}
            }
        }

        // No comma found, take everything until end (last field in struct)
        let type_str = type_part.trim().trim_end_matches(',').trim().to_string();
        Some(simplify_type_name(&type_str))
    } else {
        None
    }
}

/// Simplify type names by removing std:: prefixes and common simplifications
fn simplify_type_name(type_name: &str) -> String {
    let simplified = type_name
        .replace("std::collections::", "")
        .replace("std::option::", "")
        .replace("std::vec::", "")
        .replace("serde_yaml::", "")
        .replace("serde_json::", "");

    // Handle Option<T> -> Option<T> (keep as is, but simplify inner types)
    if simplified.starts_with("Option<") && simplified.ends_with('>') {
        let inner = &simplified[7..simplified.len() - 1];
        format!("Option<{}>", simplify_type_name(inner))
    } else if simplified.starts_with("Vec<") && simplified.ends_with('>') {
        let inner = &simplified[4..simplified.len() - 1];
        format!("Vec<{}>", simplify_type_name(inner))
    } else if simplified.starts_with("HashMap<") && simplified.ends_with('>') {
        let inner = &simplified[8..simplified.len() - 1];
        format!("HashMap<{}>", simplify_type_name(inner))
    } else {
        simplified
    }
}
fn extract_examples_from_files(docs: &mut HashMap<String, TaskDocumentation>) -> Result<()> {
    // Get all registered task types from the registry
    let registered_task_types = crate::apply::TaskRegistry::get_registered_task_types();

    // Generate file paths from task types using the filename mapping
    let task_files: Vec<(String, String)> = registered_task_types
        .iter()
        .map(|task_type| {
            let filename = crate::apply::TaskAction::task_filename(task_type);
            let file_path = format!("src/apply/{}.rs", filename);
            (task_type.clone(), file_path)
        })
        .collect();

    for (task_type, file_path) in task_files {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Some(examples) = extract_examples_from_file(&content) {
                if let Some(task_doc) = docs.get_mut(&task_type) {
                    task_doc.examples = examples;
                }
            }
        }
    }

    Ok(())
}

/// Extract examples from a single implementation file
fn extract_examples_from_file(content: &str) -> Option<Vec<TaskExample>> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let mut found_example = false;
        let mut description = String::new();

        // Look for example sections starting with //! ##
        if lines[i].starts_with("//! ## ") {
            description = lines[i].trim_start_matches("//! ## ").to_string();
            found_example = true;
            i += 1; // Move to next line
        }

        if found_example {
            // Skip description lines until we find **YAML Format:** or another header
            while i < lines.len()
                && !lines[i].contains("**YAML Format:**")
                && !lines[i].contains("## ")
            {
                i += 1;
            }

            if i >= lines.len() || lines[i].contains("## ") {
                continue;
            }

            i += 1; // Skip the **YAML Format:** line

            // Skip the opening ```yaml line
            if i < lines.len() && lines[i].starts_with("//! ```") {
                i += 1;
            }

            // Extract YAML block
            let mut yaml_lines = Vec::new();
            let mut min_indent = usize::MAX;

            // First pass: find minimum indentation
            let start_i = i;
            while i < lines.len() && !lines[i].starts_with("//! ```") {
                if lines[i].starts_with("//!") {
                    let line = lines[i].trim_start_matches("//!").trim_end();
                    if !line.is_empty() {
                        let indent = line.len() - line.trim_start().len();
                        min_indent = min_indent.min(indent);
                    }
                }
                i += 1;
            }

            // Reset to start and extract with proper indentation
            i = start_i;
            while i < lines.len() && !lines[i].starts_with("//! ```") {
                if lines[i].starts_with("//!") {
                    let line = lines[i].trim_start_matches("//!").trim_end();
                    if !line.is_empty() || yaml_lines.is_empty() {
                        // Remove the minimum indentation to preserve relative structure
                        let trimmed_line = if line.len() >= min_indent {
                            &line[min_indent..]
                        } else {
                            line.trim_start()
                        };
                        yaml_lines.push(trimmed_line.to_string());
                    }
                }
                i += 1;
            }
            i += 1; // Skip the closing ```

            // Skip to **JSON Format:** or another header
            while i < lines.len()
                && !lines[i].contains("**JSON Format:**")
                && !lines[i].contains("## ")
            {
                i += 1;
            }

            if i >= lines.len() || lines[i].contains("## ") {
                continue;
            }

            i += 1; // Skip the **JSON Format:** line

            // Skip the opening ```json line
            if i < lines.len() && lines[i].starts_with("//! ```") {
                i += 1;
            }

            // Extract JSON block
            let mut json_lines = Vec::new();
            let mut min_indent = usize::MAX;

            // First pass: find minimum indentation
            let start_i = i;
            while i < lines.len() && !lines[i].starts_with("//! ```") {
                if lines[i].starts_with("//!") {
                    let line = lines[i].trim_start_matches("//!").trim_end();
                    if !line.is_empty() {
                        let indent = line.len() - line.trim_start().len();
                        min_indent = min_indent.min(indent);
                    }
                }
                i += 1;
            }

            // Reset to start and extract with proper indentation
            i = start_i;
            while i < lines.len() && !lines[i].starts_with("//! ```") {
                if lines[i].starts_with("//!") {
                    let line = lines[i].trim_start_matches("//!").trim_end();
                    if !line.is_empty() || json_lines.is_empty() {
                        // Remove the minimum indentation to preserve relative structure
                        let trimmed_line = if line.len() >= min_indent {
                            &line[min_indent..]
                        } else {
                            line.trim_start()
                        };
                        json_lines.push(trimmed_line.to_string());
                    }
                }
                i += 1;
            }
            i += 1; // Skip the closing ```

            // Skip to **TOML Format:** or another header
            while i < lines.len()
                && !lines[i].contains("**TOML Format:**")
                && !lines[i].contains("## ")
            {
                i += 1;
            }

            if i >= lines.len() || lines[i].contains("## ") {
                continue;
            }

            i += 1; // Skip the **TOML Format:** line

            // Skip the opening ```toml line
            if i < lines.len() && lines[i].starts_with("//! ```") {
                i += 1;
            }

            // Extract TOML block
            let mut toml_lines = Vec::new();
            let mut min_indent = usize::MAX;

            // First pass: find minimum indentation
            let start_i = i;
            while i < lines.len() && !lines[i].starts_with("//! ```") {
                if lines[i].starts_with("//!") {
                    let line = lines[i].trim_start_matches("//!").trim_end();
                    if !line.is_empty() {
                        let indent = line.len() - line.trim_start().len();
                        min_indent = min_indent.min(indent);
                    }
                }
                i += 1;
            }

            // Reset to start and extract with proper indentation
            i = start_i;
            while i < lines.len() && !lines[i].starts_with("//! ```") {
                if lines[i].starts_with("//!") {
                    let line = lines[i].trim_start_matches("//!").trim_end();
                    if !line.is_empty() || toml_lines.is_empty() {
                        // Remove the minimum indentation to preserve relative structure
                        let trimmed_line = if line.len() >= min_indent {
                            &line[min_indent..]
                        } else {
                            line.trim_start()
                        };
                        toml_lines.push(trimmed_line.to_string());
                    }
                }
                i += 1;
            }
            i += 1; // Skip the closing ```

            let yaml = yaml_lines.join("\n");
            let json = json_lines.join("\n");
            let toml = toml_lines.join("\n");

            if !yaml.is_empty() && !json.is_empty() && !toml.is_empty() {
                examples.push(TaskExample {
                    description,
                    yaml,
                    json,
                    toml,
                });
            }
        } else {
            i += 1;
        }
    }

    if examples.is_empty() {
        None
    } else {
        Some(examples)
    }
}

/// Extract examples from facts implementation files
fn extract_facts_examples_from_files(docs: &mut HashMap<String, TaskDocumentation>) -> Result<()> {
    // Get all registered collector types from the registry
    let registered_collector_types = crate::facts::FactsRegistry::get_registered_collector_types();

    // Generate file paths from collector types using the filename mapping
    let collector_files: Vec<(String, String)> = registered_collector_types
        .iter()
        .map(|collector_type| {
            let filename = crate::facts::FactsRegistry::get_collector_filename(collector_type);
            let file_path = format!("src/facts/{}.rs", filename);
            (collector_type.clone(), file_path)
        })
        .collect();

    for (collector_type, file_path) in collector_files {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Some(examples) = extract_examples_from_file(&content) {
                if let Some(collector_doc) = docs.get_mut(&collector_type) {
                    collector_doc.examples = examples;
                }
            }
        }
    }

    Ok(())
}

/// Extract examples from logs implementation files
fn extract_logs_examples_from_files(docs: &mut HashMap<String, TaskDocumentation>) -> Result<()> {
    // Get all registered processor types from the registry
    let _registered_processor_types = crate::logs::LogsRegistry::get_registered_processor_types();

    // For logs, all processors are currently in mod.rs, so we check that file
    let file_path = "src/logs/mod.rs";
    if let Ok(content) = fs::read_to_string(file_path) {
        if let Some(examples) = extract_examples_from_file(&content) {
            // Since logs examples are not specific to processor types yet,
            // we'll add them to all processor docs for now
            for processor_doc in docs.values_mut() {
                processor_doc.examples = examples.clone();
            }
        }
    }

    Ok(())
}

/// Extract facts collector documentation from mod.rs
fn extract_facts_struct_docs(
    content: &str,
    docs: &mut HashMap<String, TaskDocumentation>,
    collector_types: &[String],
) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for collector struct definitions
        if lines[i].contains("#[derive") && lines[i + 1].contains("pub struct") {
            let struct_line = lines[i + 1];
            if let Some(struct_name) = extract_struct_name(struct_line) {
                // If it ends with Collector, assume it's a collector struct
                if struct_name.ends_with("Collector") {
                    let collector_type = struct_name.trim_end_matches("Collector").to_lowercase();
                    if collector_types.contains(&collector_type) {
                        let description =
                            crate::facts::FactsRegistry::get_collector_description(&collector_type);
                        let mut task_doc = TaskDocumentation {
                            description,
                            fields: HashMap::new(),
                            examples: Vec::new(),
                            register_outputs: HashMap::new(),
                        };

                        // Extract field documentation
                        extract_struct_fields(&lines, &mut i, &mut task_doc.fields)?;

                        docs.insert(collector_type, task_doc);
                    }
                }
            }
        }
        i += 1;
    }

    Ok(())
}

/// Extract all fields from a struct
fn extract_struct_fields(
    lines: &[&str],
    i: &mut usize,
    fields: &mut HashMap<String, FieldDocumentation>,
) -> Result<()> {
    // Skip the derive and struct lines
    *i += 2;

    while *i < lines.len() && !lines[*i].contains('}') {
        if lines[*i].contains("pub ") && lines[*i].contains(": ") {
            if let Some(field_doc) = extract_field_doc(lines, i) {
                fields.insert(field_doc.name.clone(), field_doc);
            }
        } else {
            *i += 1;
        }
    }

    Ok(())
}

/// Add common fields to all facts collectors
fn add_common_facts_fields(docs: &mut HashMap<String, TaskDocumentation>) {
    for task_doc in docs.values_mut() {
        // Add base collector fields that all collectors inherit
        task_doc.fields.insert(
            "name".to_string(),
            FieldDocumentation {
                name: "name".to_string(),
                field_type: "String".to_string(),
                required: true,
                description: "Collector name (used for metric names)".to_string(),
            },
        );

        task_doc.fields.insert(
            "enabled".to_string(),
            FieldDocumentation {
                name: "enabled".to_string(),
                field_type: "bool".to_string(),
                required: false,
                description: "Whether this collector is enabled (default: true)".to_string(),
            },
        );

        task_doc.fields.insert(
            "poll_interval".to_string(),
            FieldDocumentation {
                name: "poll_interval".to_string(),
                field_type: "u64".to_string(),
                required: true,
                description: "Poll interval in seconds (how often to collect this metric)"
                    .to_string(),
            },
        );

        task_doc.fields.insert(
            "labels".to_string(),
            FieldDocumentation {
                name: "labels".to_string(),
                field_type: "HashMap<String, String>".to_string(),
                required: false,
                description: "Additional labels for this collector".to_string(),
            },
        );
    }
}

/// Extract logs processor documentation from mod.rs
fn extract_logs_struct_docs(
    content: &str,
    docs: &mut HashMap<String, TaskDocumentation>,
    processor_types: &[String],
) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for log struct definitions
        if lines[i].contains("#[derive") && lines[i + 1].contains("pub struct") {
            let struct_line = lines[i + 1];
            if let Some(struct_name) = extract_struct_name(struct_line) {
                // Check if it's a LogSource or LogOutput struct
                let processor_type = if struct_name == "LogSource" {
                    // This is a generic source, skip for now
                    i += 1;
                    continue;
                } else if struct_name == "LogOutput" {
                    // This is a generic output, skip for now
                    i += 1;
                    continue;
                } else if struct_name.starts_with("Log") && struct_name.ends_with("Source") {
                    struct_name
                        .trim_start_matches("Log")
                        .trim_end_matches("Source")
                        .to_lowercase()
                } else if struct_name.starts_with("Log") && struct_name.ends_with("Output") {
                    struct_name
                        .trim_start_matches("Log")
                        .trim_end_matches("Output")
                        .to_lowercase()
                } else {
                    i += 1;
                    continue;
                };

                if processor_types.contains(&processor_type) {
                    let description =
                        crate::logs::LogsRegistry::get_processor_description(&processor_type);
                    let mut task_doc = TaskDocumentation {
                        description,
                        fields: HashMap::new(),
                        examples: Vec::new(),
                        register_outputs: HashMap::new(),
                    };

                    // Extract field documentation
                    extract_struct_fields(&lines, &mut i, &mut task_doc.fields)?;

                    docs.insert(processor_type, task_doc);
                }
            }
        }
        i += 1;
    }

    Ok(())
}

/// Add common fields to all logs processors
fn add_common_logs_fields(docs: &mut HashMap<String, TaskDocumentation>) {
    for task_doc in docs.values_mut() {
        // Add common log processor fields
        task_doc.fields.insert(
            "enabled".to_string(),
            FieldDocumentation {
                name: "enabled".to_string(),
                field_type: "bool".to_string(),
                required: false,
                description: "Whether this processor is enabled (default: true)".to_string(),
            },
        );

        task_doc.fields.insert(
            "name".to_string(),
            FieldDocumentation {
                name: "name".to_string(),
                field_type: "String".to_string(),
                required: true,
                description: "Processor name for identification".to_string(),
            },
        );
    }
}
