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

    // Generate file paths from task types using the filename mapping
    let task_files: Vec<String> = registered_task_types
        .iter()
        .map(|task_type| format!("src/apply/{}.rs", crate::apply::Task::task_filename(task_type)))
        .collect();

    // Extract from each task file
    for file_path in task_files {
        if let Ok(content) = fs::read_to_string(&file_path) {
            extract_task_struct_docs(&content, &mut docs)?;
        }
    }

    // Extract examples from implementation files
    extract_examples_from_files(&mut docs)?;

    Ok(docs)
}

/// Extract documentation for all facts collectors
pub fn extract_all_facts_docs() -> Result<HashMap<String, TaskDocumentation>> {
    let docs = HashMap::new();

    // For now, return empty docs since facts collectors are not yet implemented
    // When implemented, this will extract from src/facts/mod.rs similar to apply tasks

    Ok(docs)
}

/// Extract documentation for all logs sources and outputs
pub fn extract_all_logs_docs() -> Result<HashMap<String, TaskDocumentation>> {
    let docs = HashMap::new();

    // For now, return empty docs since logs functionality is not yet implemented
    // When implemented, this will extract from src/logs/mod.rs similar to apply tasks

    Ok(docs)
}

/// Extract task struct documentation from mod.rs
fn extract_task_struct_docs(
    content: &str,
    docs: &mut HashMap<String, TaskDocumentation>,
) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for task struct definitions
        if lines[i].contains("#[derive") && lines[i + 1].contains("pub struct") {
            let struct_line = lines[i + 1];
            if let Some(struct_name) = extract_struct_name(struct_line) {
                if let Some(task_type) = struct_name_to_task_type(&struct_name) {
                    let mut task_doc = TaskDocumentation {
                        description: String::new(),
                        fields: HashMap::new(),
                        examples: Vec::new(),
                    };

                    // Extract struct documentation (go backwards to find doc comments)
                    let mut doc_lines = Vec::new();
                    let mut j = i;
                    while j > 0 {
                        j -= 1;
                        let line = lines[j].trim();
                        if line.starts_with("///") {
                            let content = line.trim_start_matches("///").trim();
                            // Stop extracting when we hit any markdown heading (like # Examples)
                            if content.starts_with("#") {
                                break;
                            }
                            doc_lines.insert(0, content);
                        } else if !line.is_empty() && !line.starts_with("//") {
                            break;
                        }
                    }

                    if !doc_lines.is_empty() {
                        task_doc.description = doc_lines.join("\n");
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

                    docs.insert(task_type, task_doc);
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

/// Convert struct name to task type
fn struct_name_to_task_type(struct_name: &str) -> Option<String> {
    match struct_name {
        "FileTask" => Some("file".to_string()),
        "PackageTask" => Some("package".to_string()),
        "ServiceTask" => Some("service".to_string()),
        "UserTask" => Some("user".to_string()),
        "CommandTask" => Some("command".to_string()),
        "DirectoryTask" => Some("directory".to_string()),
        "GroupTask" => Some("group".to_string()),
        "CronTask" => Some("cron".to_string()),
        "MountTask" => Some("mount".to_string()),
        "FilesystemTask" => Some("filesystem".to_string()),
        "SysctlTask" => Some("sysctl".to_string()),
        "HostnameTask" => Some("hostname".to_string()),
        "TimezoneTask" => Some("timezone".to_string()),
        "RebootTask" => Some("reboot".to_string()),
        "ShutdownTask" => Some("shutdown".to_string()),
        "CopyTask" => Some("copy".to_string()),
        "TemplateTask" => Some("template".to_string()),
        "LineInFileTask" => Some("lineinfile".to_string()),
        "BlockInFileTask" => Some("blockinfile".to_string()),
        "ReplaceTask" => Some("replace".to_string()),
        "FetchTask" => Some("fetch".to_string()),
        "UriTask" => Some("uri".to_string()),
        "GetUrlTask" => Some("get_url".to_string()),
        "UnarchiveTask" => Some("unarchive".to_string()),
        "ArchiveTask" => Some("archive".to_string()),
        "StatTask" => Some("stat".to_string()),
        "AptTask" => Some("apt".to_string()),
        "YumTask" => Some("yum".to_string()),
        "PacmanTask" => Some("pacman".to_string()),
        "ZypperTask" => Some("zypper".to_string()),
        "PipTask" => Some("pip".to_string()),
        "NpmTask" => Some("npm".to_string()),
        "GemTask" => Some("gem".to_string()),
        "ScriptTask" => Some("script".to_string()),
        "RawTask" => Some("raw".to_string()),
        "DebugTask" => Some("debug".to_string()),
        "AssertTask" => Some("assert".to_string()),
        "FailTask" => Some("fail".to_string()),
        "WaitForTask" => Some("wait_for".to_string()),
        "PauseTask" => Some("pause".to_string()),
        "SetFactTask" => Some("set_fact".to_string()),
        "IncludeTasksTask" => Some("include_tasks".to_string()),
        "IncludeRoleTask" => Some("include_role".to_string()),
        "AuthorizedKeyTask" => Some("authorized_key".to_string()),
        "SudoersTask" => Some("sudoers".to_string()),
        "FirewalldTask" => Some("firewalld".to_string()),
        "UfwTask" => Some("ufw".to_string()),
        "SelinuxTask" => Some("selinux".to_string()),
        "IptablesTask" => Some("iptables".to_string()),
        "GitTask" => Some("git".to_string()),
        _ => None,
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
            let filename = crate::apply::Task::task_filename(task_type);
            let file_path = format!("src/apply/{}.rs", filename);
            (task_type.clone(), file_path)
        })
        .collect();

    for (task_type, file_path) in task_files {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Some(examples) = extract_examples_from_file(&content) {
                // Convert registry task type to documentation key for lookup
                let doc_key = match task_type.as_str() {
                    "geturl" => "get_url",
                    "waitfor" => "wait_for",
                    "setfact" => "set_fact",
                    "includetasks" => "include_tasks",
                    "includerole" => "include_role",
                    "authorizedkey" => "authorized_key",
                    _ => &task_type,
                };

                if let Some(task_doc) = docs.get_mut(doc_key) {
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
            // Skip description lines until we find **YAML Format:**
            while i < lines.len() && !lines[i].contains("**YAML Format:**") {
                i += 1;
            }

            if i >= lines.len() {
                break;
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

            // Skip to **JSON Format:**
            while i < lines.len() && !lines[i].contains("**JSON Format:**") {
                i += 1;
            }

            if i >= lines.len() {
                break;
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

            // Skip to **TOML Format:**
            while i < lines.len() && !lines[i].contains("**TOML Format:**") {
                i += 1;
            }

            if i >= lines.len() {
                break;
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
