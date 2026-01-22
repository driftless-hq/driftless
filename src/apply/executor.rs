//! Apply task executor
//!
//! This module handles the actual execution of configuration tasks defined
//! in the apply schema.

use crate::apply::wait_for::ConnectionState;
use crate::apply::{variables::VariableContext, ApplyConfig, Task};
use anyhow::Result;
use tokio::net::TcpStream;

/// Executor for apply tasks
pub struct TaskExecutor {
    dry_run: bool,
    variables: VariableContext,
    config_dir: std::path::PathBuf,
}

impl TaskExecutor {
    #[cfg(test)]
    pub fn new(dry_run: bool) -> Self {
        Self {
            dry_run,
            variables: VariableContext::new(),
            config_dir: std::path::PathBuf::from("."),
        }
    }

    /// Create a new task executor with initial variables and pre-loaded context
    pub fn with_vars_from_context(
        dry_run: bool,
        vars: std::collections::HashMap<String, serde_yaml::Value>,
        mut context: VariableContext,
        config_dir: std::path::PathBuf,
    ) -> Self {
        // Process template expressions in variables when they're loaded
        for (key, value) in vars {
            match value {
                serde_yaml::Value::String(s) => {
                    // Check if the string contains template expressions
                    if s.contains("{{") && s.contains("}}") {
                        let processed = context.render_template(&s);
                        context.set(key, serde_yaml::Value::String(processed));
                    } else {
                        context.set(key, serde_yaml::Value::String(s));
                    }
                }
                _ => {
                    context.set(key, value);
                }
            }
        }

        Self {
            dry_run,
            variables: context,
            config_dir,
        }
    }

    /// Get the dry_run flag
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    /// Get the variables context
    pub fn variables(&self) -> &VariableContext {
        &self.variables
    }

    /// Get mutable access to variables context
    pub fn variables_mut(&mut self) -> &mut VariableContext {
        &mut self.variables
    }

    /// Get the config directory
    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    /// Create a minimal task executor for included tasks
    pub fn minimal(
        variables: VariableContext,
        dry_run: bool,
        config_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            dry_run,
            variables,
            config_dir,
        }
    }

    /// Execute a single task
    pub async fn execute_single_task(&mut self, task: &Task) -> Result<()> {
        crate::apply::TaskRegistry::execute_task_minimal(
            task,
            &self.variables,
            self.dry_run,
            &self.config_dir,
        )
        .await
    }

    /// Execute all tasks in the configuration
    pub async fn execute(&mut self, config: &ApplyConfig) -> Result<()> {
        println!(
            "Executing {} tasks{}",
            config.tasks.len(),
            if self.dry_run { " (dry run)" } else { "" }
        );

        for (i, task) in config.tasks.iter().enumerate() {
            println!("Executing task {} of {}", i + 1, config.tasks.len());

            self.execute_single_task(task).await?;
        }

        println!(
            "All tasks completed{}",
            if self.dry_run { " (dry run)" } else { "" }
        );
        Ok(())
    }

    /// Validate tasks without executing them
    pub fn validate(&self, config: &ApplyConfig) -> Result<()> {
        println!("Validating {} tasks", config.tasks.len());

        for (i, task) in config.tasks.iter().enumerate() {
            crate::apply::TaskRegistry::validate_task(task, i)?;
        }

        println!("All tasks validated successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::cron::CronState;
    use crate::apply::file::FileState;
    use crate::apply::filesystem::FilesystemState;
    use crate::apply::group::GroupState;
    use crate::apply::mount::MountState;
    use crate::apply::sysctl::SysctlState;
    use crate::apply::{
        ApplyConfig, CronTask, FileTask, FilesystemTask, GroupTask, HostnameTask, MountTask,
        SysctlTask, Task, TimezoneTask,
    };

    #[tokio::test]
    async fn test_task_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![
                Task::File(FileTask {
                    description: None,
                    path: "/etc/hostname".to_string(),
                    state: FileState::Present,
                    content: Some("test-host".to_string()),
                    mode: Some("0644".to_string()),
                    owner: Some("root".to_string()),
                    group: Some("root".to_string()),
                    source: None,
                }),
                Task::File(FileTask {
                    description: None,
                    path: "".to_string(), // Invalid: empty path
                    state: FileState::Present,
                    content: None,
                    mode: None,
                    owner: None,
                    group: None,
                    source: None,
                }),
            ],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("file path cannot be empty"));
    }

    #[tokio::test]
    async fn test_group_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::Group(GroupTask {
                description: None,
                name: "".to_string(), // Invalid: empty name
                state: GroupState::Present,
                gid: None,
                system: false,
            })],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("group name cannot be empty"));
    }

    #[tokio::test]
    async fn test_cron_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::Cron(CronTask {
                description: None,
                name: "".to_string(), // Invalid: empty name
                state: CronState::Present,
                user: "root".to_string(),
                minute: "*".to_string(),
                hour: "*".to_string(),
                day: "*".to_string(),
                month: "*".to_string(),
                weekday: "*".to_string(),
                job: "".to_string(), // Invalid: empty job
                comment: None,
            })],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cron job command cannot be empty"));
    }

    #[tokio::test]
    async fn test_mount_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![
                Task::Mount(MountTask {
                    description: None,
                    path: "".to_string(), // Invalid: empty path
                    state: MountState::Mounted,
                    src: "/dev/sda1".to_string(),
                    fstype: None,
                    opts: vec![],
                    fstab: false,
                    recursive: false,
                }),
                Task::Mount(MountTask {
                    description: None,
                    path: "/mnt/test".to_string(),
                    state: MountState::Mounted,
                    src: "".to_string(), // Invalid: empty source
                    fstype: None,
                    opts: vec![],
                    fstab: false,
                    recursive: false,
                }),
            ],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("mount path cannot be empty")
                || error_msg.contains("mount source cannot be empty")
        );
    }

    #[tokio::test]
    async fn test_filesystem_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::Filesystem(FilesystemTask {
                description: None,
                dev: "".to_string(), // Invalid: empty device
                state: FilesystemState::Present,
                fstype: Some("ext4".to_string()),
                force: false,
                opts: vec![],
            })],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("filesystem device cannot be empty"));
    }

    #[tokio::test]
    async fn test_sysctl_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::Sysctl(SysctlTask {
                description: None,
                name: "".to_string(), // Invalid: empty name
                state: SysctlState::Present,
                value: "1".to_string(),
                persist: false,
                reload: false,
            })],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("sysctl parameter name cannot be empty"));
    }

    #[tokio::test]
    async fn test_hostname_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::Hostname(HostnameTask {
                description: None,
                name: "".to_string(), // Invalid: empty hostname
                persist: false,
            })],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("hostname cannot be empty"));
    }

    #[tokio::test]
    async fn test_timezone_validation() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::Timezone(TimezoneTask {
                description: None,
                name: "".to_string(), // Invalid: empty timezone
            })],
        };

        let executor = TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("timezone cannot be empty"));
    }

    #[tokio::test]
    async fn test_dry_run_execution() {
        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::File(FileTask {
                description: None,
                path: "/tmp/test.txt".to_string(),
                state: FileState::Present,
                content: Some("test content".to_string()),
                mode: None,
                owner: None,
                group: None,
                source: None,
            })],
        };

        let mut executor = TaskExecutor::new(true);
        let result = executor.execute(&config).await;
        assert!(result.is_ok());
    }
}

/// Execute debug task
pub async fn execute_debug_task(
    task: &crate::apply::DebugTask,
    variables: &VariableContext,
    _dry_run: bool,
) -> Result<()> {
    use crate::apply::debug::DebugVerbosity;

    let should_show = match task.verbosity {
        DebugVerbosity::Normal => true,
        DebugVerbosity::Verbose => true, // For now, always show verbose in debug mode
    };

    if should_show {
        if let Some(var_name) = &task.var {
            if let Some(value) = variables.get(var_name) {
                println!("DEBUG: {} = {:?}", var_name, value);
            } else {
                println!("DEBUG: {} = <undefined>", var_name);
            }
        } else {
            // Support variable templating in messages
            let rendered_msg = variables.render_template(&task.msg);
            println!("DEBUG: {}", rendered_msg);
        }
    }

    Ok(())
}

/// Execute assert task
pub async fn execute_assert_task(
    task: &crate::apply::AssertTask,
    variables: &VariableContext,
    _dry_run: bool,
) -> Result<()> {
    // Evaluate the condition using the variable context
    let condition_result = variables.evaluate_condition(&task.that);

    if condition_result {
        if !task.quiet {
            if let Some(msg) = &task.success_msg {
                let rendered_msg = variables.render_template(msg);
                println!("ASSERT: {}", rendered_msg);
            } else {
                println!("ASSERT: Condition '{}' passed", task.that);
            }
        }
    } else {
        let error_msg = if let Some(fail_msg) = &task.fail_msg {
            variables.render_template(fail_msg)
        } else {
            format!("Assertion failed: {}", task.that)
        };
        return Err(anyhow::anyhow!("{}", error_msg));
    }

    Ok(())
}

/// Execute fail task
pub async fn execute_fail_task(
    task: &crate::apply::FailTask,
    variables: &VariableContext,
    _dry_run: bool,
) -> Result<()> {
    if let Some(when_condition) = &task.when {
        // Evaluate the when condition using variable context
        let should_fail = variables.evaluate_condition(when_condition);
        if !should_fail {
            return Ok(());
        }
    }

    let msg = variables.render_template(&task.msg);
    Err(anyhow::anyhow!("{}", msg))
}

/// Execute wait_for task
pub async fn execute_wait_for_task(task: &crate::apply::WaitForTask, dry_run: bool) -> Result<()> {
    use std::time::Duration;
    use tokio::time::sleep;

    async fn check_port_connectivity(host: &str, port: u16) -> bool {
        TcpStream::connect((host, port)).await.is_ok()
    }

    async fn check_file_exists(path: &str) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    if let (Some(host), Some(port)) = (&task.host, task.port) {
        println!(
            "Waiting for {}:{} to be {}",
            host,
            port,
            if matches!(task.state, ConnectionState::Started) {
                "available"
            } else {
                "unavailable"
            }
        );

        if dry_run {
            println!("DRY RUN: Would wait up to {} seconds", task.timeout);
            return Ok(());
        }

        let start_time = std::time::Instant::now();
        let mut attempts = 0;

        loop {
            attempts += 1;
            let is_connected = if task.active_connection {
                check_port_connectivity(host, port).await
            } else {
                // Simple connectivity check - just try to connect once
                check_port_connectivity(host, port).await
            };

            let should_be_connected = matches!(task.state, ConnectionState::Started);

            if is_connected == should_be_connected {
                println!(
                    "Condition met for {}:{} after {} attempts ({} seconds)",
                    host,
                    port,
                    attempts,
                    start_time.elapsed().as_secs()
                );
                return Ok(());
            }

            if start_time.elapsed().as_secs() >= task.timeout {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for {}:{} after {} attempts ({} seconds)",
                    host,
                    port,
                    attempts,
                    task.timeout
                ));
            }

            sleep(Duration::from_secs(task.delay)).await;
        }
    } else if let Some(path) = &task.path {
        println!("Waiting for path '{}' to exist", path);

        if dry_run {
            println!("DRY RUN: Would wait up to {} seconds", task.timeout);
            return Ok(());
        }

        let start_time = std::time::Instant::now();
        let mut attempts = 0;

        loop {
            attempts += 1;
            let file_exists = check_file_exists(path).await;

            let should_exist = matches!(task.state, ConnectionState::Started);

            if file_exists == should_exist {
                println!(
                    "Condition met for path '{}' after {} attempts ({} seconds)",
                    path,
                    attempts,
                    start_time.elapsed().as_secs()
                );
                return Ok(());
            }

            if start_time.elapsed().as_secs() >= task.timeout {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for path '{}' after {} attempts ({} seconds)",
                    path,
                    attempts,
                    task.timeout
                ));
            }

            sleep(Duration::from_secs(task.delay)).await;
        }
    } else {
        Err(anyhow::anyhow!(
            "wait_for requires either host+port or path"
        ))
    }
}

/// Execute pause task
pub async fn execute_pause_task(task: &crate::apply::PauseTask, dry_run: bool) -> Result<()> {
    use std::time::Duration;
    use tokio::time::sleep;

    let total_seconds = task.seconds + (task.minutes * 60);

    if total_seconds > 0 {
        if dry_run {
            println!(
                "DRY RUN: Would pause for {} seconds with message: {}",
                total_seconds, task.prompt
            );
            return Ok(());
        }

        println!("{}", task.prompt);

        // In a real implementation, this might wait for user input
        // For now, just sleep
        sleep(Duration::from_secs(total_seconds)).await;
    }

    Ok(())
}

/// Execute set_fact task
pub async fn execute_set_fact_task(
    task: &crate::apply::SetFactTask,
    variables: &mut VariableContext,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("DRY RUN: Would set fact '{}' = {:?}", task.key, task.value);
        return Ok(());
    }

    variables.set(task.key.clone(), task.value.clone());

    if task.cacheable {
        println!("Set cached fact '{}' = {:?}", task.key, task.value);
    } else {
        println!("Set fact '{}' = {:?}", task.key, task.value);
    }

    Ok(())
}

/// Execute include_tasks task
pub async fn execute_include_tasks_task(
    task: &crate::apply::IncludeTasksTask,
    variables: &VariableContext,
    dry_run: bool,
    config_dir: &std::path::Path,
) -> Result<()> {
    // Check conditional inclusion
    if let Some(when_condition) = &task.when {
        let should_include = variables.evaluate_condition(when_condition);
        if !should_include {
            println!(
                "Skipping task inclusion '{}' due to condition: {}",
                task.file, when_condition
            );
            return Ok(());
        }
    }

    println!("Including tasks from file: {}", task.file);

    // Resolve the file path relative to the config directory
    let file_path = config_dir.join(&task.file);

    // Load the external task file
    let content = match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => content,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to read task file '{}': {}",
                file_path.display(),
                e
            ));
        }
    };

    // Parse the tasks
    let external_tasks: Vec<crate::apply::Task> =
        match file_path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse JSON task file '{}': {}",
                    file_path.display(),
                    e
                )
            })?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse YAML task file '{}': {}",
                    file_path.display(),
                    e
                )
            })?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported task file format: {}",
                    task.file
                ));
            }
        };

    // Execute the included tasks directly (avoid recursion)
    for (i, included_task) in external_tasks.iter().enumerate() {
        println!(
            "Executing included task {} of {} from {}",
            i + 1,
            external_tasks.len(),
            task.file
        );

        // Execute each task using the registry
        crate::apply::TaskRegistry::execute_task_minimal(
            included_task,
            variables,
            dry_run,
            config_dir,
        )
        .await?;
    }

    println!(
        "Completed inclusion of {} tasks from {}",
        external_tasks.len(),
        task.file
    );
    Ok(())
}

/// Execute include_role task
pub async fn execute_include_role_task(
    task: &crate::apply::IncludeRoleTask,
    variables: &VariableContext,
    dry_run: bool,
    config_dir: &std::path::Path,
) -> Result<()> {
    // Check conditional inclusion
    if let Some(when_condition) = &task.when {
        let should_include = variables.evaluate_condition(when_condition);
        if !should_include {
            println!(
                "Skipping role inclusion '{}' due to condition: {}",
                task.name, when_condition
            );
            return Ok(());
        }
    }

    println!("Including role: {}", task.name);

    // Look for role in roles/ directory relative to config directory
    let role_path = config_dir.join("roles").join(&task.name);

    if !role_path.exists() {
        return Err(anyhow::anyhow!(
            "Role '{}' not found at {}",
            task.name,
            role_path.display()
        ));
    }

    // Load role defaults if they exist
    let defaults_path = role_path.join("defaults/main.yml");
    if defaults_path.exists() {
        let defaults_content = tokio::fs::read_to_string(&defaults_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read role defaults: {}", e))?;

        let role_defaults: std::collections::HashMap<String, serde_yaml::Value> =
            serde_yaml::from_str(&defaults_content)
                .map_err(|e| anyhow::anyhow!("Failed to parse role defaults: {}", e))?;

        // Merge role defaults with provided variables
        let mut merged_vars = variables.clone();
        for (key, value) in role_defaults {
            if !variables.contains(&key) {
                // Don't override explicit vars
                merged_vars.set(key, value);
            }
        }

        // Load and execute role tasks
        let tasks_path = role_path.join("tasks/main.yml");
        if tasks_path.exists() {
            let tasks_content = tokio::fs::read_to_string(&tasks_path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read role tasks: {}", e))?;

            let role_tasks: Vec<crate::apply::Task> = serde_yaml::from_str(&tasks_content)
                .map_err(|e| anyhow::anyhow!("Failed to parse role tasks: {}", e))?;

            // Execute the role tasks directly (avoid recursion)
            for (i, role_task) in role_tasks.iter().enumerate() {
                println!(
                    "Executing role task {} of {} from role '{}'",
                    i + 1,
                    role_tasks.len(),
                    task.name
                );

                // Execute each task using the registry
                crate::apply::TaskRegistry::execute_task_minimal(
                    role_task,
                    &merged_vars,
                    dry_run,
                    config_dir,
                )
                .await?;
            }

            println!("Completed execution of role '{}'", task.name);
        } else {
            return Err(anyhow::anyhow!(
                "Role '{}' missing tasks/main.yml",
                task.name
            ));
        }
    } else {
        return Err(anyhow::anyhow!(
            "Role '{}' missing defaults/main.yml",
            task.name
        ));
    }

    Ok(())
}
