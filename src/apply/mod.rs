use crate::apply::executor::TaskExecutor;
use crate::logs::default_compression_level;
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Type alias for task execution functions
type TaskExecutorFn = Arc<
    dyn for<'a> Fn(
            &'a Task,
            &'a mut crate::apply::executor::TaskExecutor,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<serde_yaml::Value, anyhow::Error>>
                    + Send
                    + 'a,
            >,
        > + Send
        + Sync,
>;

// Type alias for task validation functions
type TaskValidatorFn = Arc<dyn Fn(&Task, usize) -> Result<()> + Send + Sync>;

// Task registry entry containing both executor and validator
#[derive(Clone)]
pub(crate) struct TaskRegistryEntry {
    executor: TaskExecutorFn,
    validator: Option<TaskValidatorFn>,
    category: String,
    filename: String,
}

// Global task registry for extensible task execution
static TASK_REGISTRY: Lazy<RwLock<HashMap<String, TaskRegistryEntry>>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    // Initialize with built-in executors
    TaskRegistry::initialize_builtin_executors(&mut registry);

    RwLock::new(registry)
});

/// Task executor registry for runtime extensibility
pub struct TaskRegistry;

impl TaskRegistry {
    /// Register a task executor function
    pub(crate) fn register(
        registry: &mut HashMap<String, TaskRegistryEntry>,
        task_type: &str,
        category: &str,
        filename: &str,
        executor: TaskExecutorFn,
    ) {
        let entry = TaskRegistryEntry {
            executor,
            validator: None,
            category: category.to_string(),
            filename: filename.to_string(),
        };
        registry.insert(task_type.to_string(), entry);
    }

    /// Register a task executor function with validation
    pub(crate) fn register_with_validator(
        registry: &mut HashMap<String, TaskRegistryEntry>,
        task_type: &str,
        category: &str,
        filename: &str,
        executor: TaskExecutorFn,
        validator: TaskValidatorFn,
    ) {
        let entry = TaskRegistryEntry {
            executor,
            validator: Some(validator),
            category: category.to_string(),
            filename: filename.to_string(),
        };
        registry.insert(task_type.to_string(), entry);
    }

    /// Initialize the registry with built-in task executors
    pub(crate) fn initialize_builtin_executors(registry: &mut HashMap<String, TaskRegistryEntry>) {
        // File operations
        TaskRegistry::register_with_validator(
            registry,
            "file",
            "File Operations",
            "file",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::File(file_task) = &task.action {
                        crate::apply::file::execute_file_task(file_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for file executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::File(file_task) = &task.action {
                    if file_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: file path cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "directory",
            "File Operations",
            "directory",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Directory(dir_task) = &task.action {
                        crate::apply::directory::execute_directory_task(
                            dir_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for directory executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Directory(dir_task) = &task.action {
                    if dir_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: directory path cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "copy",
            "File Operations",
            "copy",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Copy(copy_task) = &task.action {
                        crate::apply::copy::execute_copy_task(copy_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for copy executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Copy(copy_task) = &task.action {
                    if copy_task.src.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: copy source cannot be empty",
                            task_index + 1
                        ));
                    }
                    if copy_task.dest.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: copy destination cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Package management
        TaskRegistry::register_with_validator(
            registry,
            "package",
            "Package Management",
            "package",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Package(pkg_task) = &task.action {
                        crate::apply::package::execute_package_task(pkg_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for package executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Package(pkg_task) = &task.action {
                    if pkg_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "apt",
            "Package Management",
            "apt",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Apt(apt_task) = &task.action {
                        crate::apply::apt::execute_apt_task(apt_task, executor.dry_run()).await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for apt executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Apt(apt_task) = &task.action {
                    if apt_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: apt package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Service management
        TaskRegistry::register_with_validator(
            registry,
            "service",
            "System Administration",
            "service",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Service(service_task) = &task.action {
                        crate::apply::service::execute_service_task(
                            service_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for service executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Service(service_task) = &task.action {
                    if service_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: service name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // User management
        TaskRegistry::register_with_validator(
            registry,
            "user",
            "System Administration",
            "user",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::User(user_task) = &task.action {
                        crate::apply::user::execute_user_task(user_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for user executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::User(user_task) = &task.action {
                    if user_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: user name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "group",
            "System Administration",
            "group",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Group(group_task) = &task.action {
                        crate::apply::group::execute_group_task(group_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for group executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Group(group_task) = &task.action {
                    if group_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: group name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Command execution
        TaskRegistry::register_with_validator(
            registry,
            "command",
            "Command Execution",
            "command",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Command(cmd_task) = &task.action {
                        crate::apply::command::execute_command_task(cmd_task, executor).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for command executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Command(cmd_task) = &task.action {
                    if cmd_task.command.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: command cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "script",
            "Command Execution",
            "script",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Script(script_task) = &task.action {
                        crate::apply::script::execute_script_task(script_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for script executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Script(script_task) = &task.action {
                    if script_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: script path cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "raw",
            "Command Execution",
            "raw",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Raw(raw_task) = &task.action {
                        crate::apply::raw::execute_raw_task(raw_task, executor.dry_run()).await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for raw executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Raw(raw_task) = &task.action {
                    if raw_task.executable.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: raw executable cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Control flow tasks
        TaskRegistry::register_with_validator(
            registry,
            "debug",
            "Utility/Control",
            "debug",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Debug(debug_task) = &task.action {
                        crate::apply::executor::execute_debug_task(
                            debug_task,
                            executor.variables(),
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for debug executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Debug(debug_task) = &task.action {
                    if debug_task.msg.is_empty() && debug_task.var.is_none() {
                        return Err(anyhow::anyhow!(
                            "Task {}: debug task must have either msg or var",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "assert",
            "Utility/Control",
            "assert",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Assert(assert_task) = &task.action {
                        crate::apply::executor::execute_assert_task(
                            assert_task,
                            executor.variables(),
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for assert executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Assert(assert_task) = &task.action {
                    if assert_task.that.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: assert that condition cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "fail",
            "Utility/Control",
            "fail",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Fail(fail_task) = &task.action {
                        crate::apply::executor::execute_fail_task(
                            fail_task,
                            executor.variables(),
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for fail executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Fail(fail_task) = &task.action {
                    if fail_task.msg.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: fail message cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register(
            registry,
            "pause",
            "Utility/Control",
            "pause",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Pause(pause_task) = &task.action {
                        crate::apply::executor::execute_pause_task(pause_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for pause executor"))
                    }
                })
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "setfact",
            "Utility/Control",
            "set_fact",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::SetFact(set_fact_task) = &task.action {
                        let dry_run = executor.dry_run();
                        crate::apply::executor::execute_set_fact_task(
                            set_fact_task,
                            executor.variables_mut(),
                            dry_run,
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for setfact executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::SetFact(set_fact_task) = &task.action {
                    if set_fact_task.key.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: set_fact key cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Include functionality
        TaskRegistry::register_with_validator(
            registry,
            "includetasks",
            "Utility/Control",
            "include_tasks",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::IncludeTasks(include_tasks_task) = &task.action {
                        crate::apply::executor::execute_include_tasks_task(
                            include_tasks_task,
                            executor.variables(),
                            executor.dry_run(),
                            executor.config_dir(),
                            executor.plugin_manager().clone(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for includetasks executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::IncludeTasks(include_tasks_task) = &task.action {
                    if include_tasks_task.file.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: include_tasks file cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "includerole",
            "Utility/Control",
            "include_role",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::IncludeRole(include_role_task) = &task.action {
                        crate::apply::executor::execute_include_role_task(
                            include_role_task,
                            executor.variables(),
                            executor.dry_run(),
                            executor.config_dir(),
                            executor.plugin_manager().clone(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for includerole executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::IncludeRole(include_role_task) = &task.action {
                    if include_role_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: include_role name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // System management
        TaskRegistry::register_with_validator(
            registry,
            "cron",
            "System Administration",
            "cron",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Cron(cron_task) = &task.action {
                        crate::apply::cron::execute_cron_task(cron_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for cron executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Cron(cron_task) = &task.action {
                    if cron_task.job.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: cron job command cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "mount",
            "System Administration",
            "mount",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Mount(mount_task) = &task.action {
                        crate::apply::mount::execute_mount_task(mount_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for mount executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Mount(mount_task) = &task.action {
                    if mount_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: mount path cannot be empty",
                            task_index + 1
                        ));
                    }
                    if mount_task.src.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: mount source cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "filesystem",
            "System Administration",
            "filesystem",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Filesystem(fs_task) = &task.action {
                        crate::apply::filesystem::execute_filesystem_task(
                            fs_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for filesystem executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Filesystem(fs_task) = &task.action {
                    if fs_task.dev.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: filesystem device cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "sysctl",
            "System Administration",
            "sysctl",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Sysctl(sysctl_task) = &task.action {
                        crate::apply::sysctl::execute_sysctl_task(sysctl_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for sysctl executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Sysctl(sysctl_task) = &task.action {
                    if sysctl_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: sysctl parameter name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "hostname",
            "System Administration",
            "hostname",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Hostname(hostname_task) = &task.action {
                        crate::apply::hostname::execute_hostname_task(
                            hostname_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for hostname executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Hostname(hostname_task) = &task.action {
                    if hostname_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: hostname cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "timezone",
            "System Administration",
            "timezone",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Timezone(timezone_task) = &task.action {
                        crate::apply::timezone::execute_timezone_task(
                            timezone_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for timezone executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Timezone(timezone_task) = &task.action {
                    if timezone_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: timezone cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register(
            registry,
            "reboot",
            "System Administration",
            "reboot",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Reboot(reboot_task) = &task.action {
                        crate::apply::reboot::execute_reboot_task(reboot_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for reboot executor"))
                    }
                })
            }),
        );

        TaskRegistry::register(
            registry,
            "shutdown",
            "System Administration",
            "shutdown",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Shutdown(shutdown_task) = &task.action {
                        crate::apply::shutdown::execute_shutdown_task(
                            shutdown_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for shutdown executor"))
                    }
                })
            }),
        );

        // File operations
        TaskRegistry::register_with_validator(
            registry,
            "template",
            "File Operations",
            "template",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Template(template_task) = &task.action {
                        crate::apply::template::execute_template_task(
                            template_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for template executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Template(template_task) = &task.action {
                    if template_task.src.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: template source cannot be empty",
                            task_index + 1
                        ));
                    }
                    if template_task.dest.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: template destination cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "lineinfile",
            "File Operations",
            "lineinfile",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::LineInFile(line_task) = &task.action {
                        crate::apply::lineinfile::execute_lineinfile_task(
                            line_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for lineinfile executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::LineInFile(line_task) = &task.action {
                    if line_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: lineinfile path cannot be empty",
                            task_index + 1
                        ));
                    }
                    if line_task.line.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: lineinfile line cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "blockinfile",
            "File Operations",
            "blockinfile",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::BlockInFile(block_task) = &task.action {
                        crate::apply::blockinfile::execute_blockinfile_task(
                            block_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for blockinfile executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::BlockInFile(block_task) = &task.action {
                    if block_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: blockinfile path cannot be empty",
                            task_index + 1
                        ));
                    }
                    if block_task.block.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: blockinfile block cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "replace",
            "File Operations",
            "replace",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Replace(replace_task) = &task.action {
                        crate::apply::replace::execute_replace_task(
                            replace_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for replace executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Replace(replace_task) = &task.action {
                    if replace_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: replace path cannot be empty",
                            task_index + 1
                        ));
                    }
                    // Either regexp or before must be specified
                    if replace_task.regexp.is_none() && replace_task.before.is_none() {
                        return Err(anyhow::anyhow!(
                            "Task {}: replace must specify either 'regexp' or 'before'",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Network/file operations
        TaskRegistry::register_with_validator(
            registry,
            "fetch",
            "File Operations",
            "fetch",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Fetch(fetch_task) = &task.action {
                        crate::apply::fetch::execute_fetch_task(fetch_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for fetch executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Fetch(fetch_task) = &task.action {
                    if fetch_task.url.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: fetch url cannot be empty",
                            task_index + 1
                        ));
                    }
                    if fetch_task.dest.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: fetch destination cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "uri",
            "Network Operations",
            "uri",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Uri(uri_task) = &task.action {
                        crate::apply::uri::execute_uri_task(uri_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for uri executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Uri(uri_task) = &task.action {
                    if uri_task.url.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: uri url cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "geturl",
            "Network Operations",
            "get_url",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::GetUrl(get_url_task) = &task.action {
                        crate::apply::get_url::execute_get_url_task(
                            get_url_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for geturl executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::GetUrl(get_url_task) = &task.action {
                    if get_url_task.url.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: get_url url cannot be empty",
                            task_index + 1
                        ));
                    }
                    if get_url_task.dest.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: get_url destination cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Archive operations
        TaskRegistry::register_with_validator(
            registry,
            "unarchive",
            "File Operations",
            "unarchive",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Unarchive(unarchive_task) = &task.action {
                        crate::apply::unarchive::execute_unarchive_task(
                            unarchive_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for unarchive executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Unarchive(unarchive_task) = &task.action {
                    if unarchive_task.src.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: unarchive source cannot be empty",
                            task_index + 1
                        ));
                    }
                    if unarchive_task.dest.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: unarchive destination cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "archive",
            "File Operations",
            "archive",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Archive(archive_task) = &task.action {
                        crate::apply::archive::execute_archive_task(
                            archive_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for archive executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Archive(archive_task) = &task.action {
                    if archive_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: archive path cannot be empty",
                            task_index + 1
                        ));
                    }
                    if archive_task.sources.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: archive sources cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Information gathering
        TaskRegistry::register_with_validator(
            registry,
            "stat",
            "File Operations",
            "stat",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Stat(stat_task) = &task.action {
                        crate::apply::stat::execute_stat_task(stat_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for stat executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Stat(stat_task) = &task.action {
                    if stat_task.path.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: stat path cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Additional package managers
        TaskRegistry::register_with_validator(
            registry,
            "yum",
            "Package Management",
            "yum",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Yum(yum_task) = &task.action {
                        crate::apply::yum::execute_yum_task(yum_task, executor.dry_run()).await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for yum executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Yum(yum_task) = &task.action {
                    if yum_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: yum package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "pacman",
            "Package Management",
            "pacman",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Pacman(pacman_task) = &task.action {
                        crate::apply::pacman::execute_pacman_task(pacman_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for pacman executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Pacman(pacman_task) = &task.action {
                    if pacman_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: pacman package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "zypper",
            "Package Management",
            "zypper",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Zypper(zypper_task) = &task.action {
                        crate::apply::zypper::execute_zypper_task(zypper_task, executor.dry_run())
                            .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for zypper executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Zypper(zypper_task) = &task.action {
                    if zypper_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: zypper package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Language package managers
        TaskRegistry::register_with_validator(
            registry,
            "pip",
            "Package Management",
            "pip",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Pip(pip_task) = &task.action {
                        crate::apply::pip::execute_pip_task(pip_task, executor.dry_run()).await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for pip executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Pip(pip_task) = &task.action {
                    if pip_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: pip package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "npm",
            "Package Management",
            "npm",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Npm(npm_task) = &task.action {
                        crate::apply::npm::execute_npm_task(npm_task, executor.dry_run()).await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for npm executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Npm(npm_task) = &task.action {
                    if npm_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: npm package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "gem",
            "Package Management",
            "gem",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Gem(gem_task) = &task.action {
                        crate::apply::gem::execute_gem_task(gem_task, executor.dry_run()).await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for gem executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Gem(gem_task) = &task.action {
                    if gem_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: gem package name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Control flow
        TaskRegistry::register_with_validator(
            registry,
            "waitfor",
            "Utility/Control",
            "wait_for",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::WaitFor(wait_for_task) = &task.action {
                        crate::apply::executor::execute_wait_for_task(
                            wait_for_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for waitfor executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::WaitFor(wait_for_task) = &task.action {
                    if wait_for_task.host.is_none() || wait_for_task.port.is_none() {
                        return Err(anyhow::anyhow!(
                            "Task {}: wait_for requires both host and port",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Security
        TaskRegistry::register_with_validator(
            registry,
            "authorizedkey",
            "Security & Access",
            "authorized_key",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::AuthorizedKey(authorized_key_task) = &task.action {
                        crate::apply::authorized_key::execute_authorized_key_task(
                            authorized_key_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for authorizedkey executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::AuthorizedKey(authorized_key_task) = &task.action {
                    if authorized_key_task.user.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: authorized_key user cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "sudoers",
            "Security & Access",
            "sudoers",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Sudoers(sudoers_task) = &task.action {
                        crate::apply::sudoers::execute_sudoers_task(
                            sudoers_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for sudoers executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Sudoers(sudoers_task) = &task.action {
                    if sudoers_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: sudoers name cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Firewalls
        TaskRegistry::register_with_validator(
            registry,
            "firewalld",
            "Security & Access",
            "firewalld",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Firewalld(firewalld_task) = &task.action {
                        crate::apply::firewalld::execute_firewalld_task(
                            firewalld_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for firewalld executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Firewalld(firewalld_task) = &task.action {
                    if firewalld_task.zone.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: firewalld zone cannot be empty",
                            task_index + 1
                        ));
                    }
                    let rule_count = firewalld_task.service.is_some() as u8
                        + firewalld_task.port.is_some() as u8
                        + firewalld_task.rich_rule.is_some() as u8;
                    if rule_count != 1 {
                        return Err(anyhow::anyhow!(
                            "Task {}: firewalld requires exactly one of service, port, or rich_rule",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "ufw",
            "Security & Access",
            "ufw",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Ufw(ufw_task) = &task.action {
                        crate::apply::ufw::execute_ufw_task(ufw_task, executor.dry_run()).await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for ufw executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Ufw(ufw_task) = &task.action {
                    match ufw_task.state {
                        crate::apply::ufw::UfwState::Logging => {
                            if ufw_task.logging.is_none() {
                                return Err(anyhow::anyhow!(
                                    "Task {}: ufw logging requires logging level",
                                    task_index + 1
                                ));
                            }
                        }
                        crate::apply::ufw::UfwState::Default => {
                            if ufw_task.default.is_none() {
                                return Err(anyhow::anyhow!(
                                    "Task {}: ufw default requires default policy",
                                    task_index + 1
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "selinux",
            "Security & Access",
            "selinux",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Selinux(selinux_task) = &task.action {
                        crate::apply::selinux::execute_selinux_task(
                            selinux_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for selinux executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Selinux(selinux_task) = &task.action {
                    match selinux_task.state {
                        crate::apply::selinux::SelinuxState::On
                        | crate::apply::selinux::SelinuxState::Off => {
                            if selinux_task.boolean.is_none() {
                                return Err(anyhow::anyhow!(
                                    "Task {}: selinux boolean name required",
                                    task_index + 1
                                ));
                            }
                        }
                        crate::apply::selinux::SelinuxState::Context
                        | crate::apply::selinux::SelinuxState::Restorecon => {
                            if selinux_task.target.is_none() {
                                return Err(anyhow::anyhow!(
                                    "Task {}: selinux target path required",
                                    task_index + 1
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "iptables",
            "Security & Access",
            "iptables",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Iptables(iptables_task) = &task.action {
                        crate::apply::iptables::execute_iptables_task(
                            iptables_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for iptables executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Iptables(iptables_task) = &task.action {
                    if iptables_task.target.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: iptables target cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Version control
        TaskRegistry::register_with_validator(
            registry,
            "git",
            "Source Control",
            "git",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Git(git_task) = &task.action {
                        crate::apply::git::execute_git_task(git_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for git executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Git(git_task) = &task.action {
                    if git_task.repo.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: git repo cannot be empty",
                            task_index + 1
                        ));
                    }
                    if git_task.dest.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: git dest cannot be empty",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        // Monitoring & Logging
        TaskRegistry::register_with_validator(
            registry,
            "logrotate",
            "Monitoring & Logging",
            "logrotate",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Logrotate(logrotate_task) = &task.action {
                        crate::apply::logrotate::execute_logrotate_task(
                            logrotate_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for logrotate executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Logrotate(logrotate_task) = &task.action {
                    if logrotate_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: logrotate name cannot be empty",
                            task_index + 1
                        ));
                    }
                    if logrotate_task.state == crate::apply::logrotate::LogrotateState::Present
                        && logrotate_task.path.is_none()
                    {
                        return Err(anyhow::anyhow!(
                            "Task {}: logrotate path is required when state is present",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "rsyslog",
            "Monitoring & Logging",
            "rsyslog",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Rsyslog(rsyslog_task) = &task.action {
                        crate::apply::rsyslog::execute_rsyslog_task(
                            rsyslog_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for rsyslog executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Rsyslog(rsyslog_task) = &task.action {
                    if rsyslog_task.name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Task {}: rsyslog name cannot be empty",
                            task_index + 1
                        ));
                    }
                    if rsyslog_task.state == crate::apply::rsyslog::RsyslogState::Present
                        && rsyslog_task.config.is_none()
                    {
                        return Err(anyhow::anyhow!(
                            "Task {}: rsyslog config is required when state is present",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "journald",
            "Monitoring & Logging",
            "journald",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let TaskAction::Journald(journald_task) = &task.action {
                        crate::apply::journald::execute_journald_task(
                            journald_task,
                            executor.dry_run(),
                        )
                        .await?;
                        Ok(serde_yaml::Value::Null)
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for journald executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let TaskAction::Journald(journald_task) = &task.action {
                    if journald_task.state == crate::apply::journald::JournaldState::Present
                        && journald_task.config.is_empty()
                    {
                        return Err(anyhow::anyhow!(
                            "Task {}: journald config is required when state is present",
                            task_index + 1
                        ));
                    }
                }
                Ok(())
            }),
        );
    }

    /// Execute a task using the registry with minimal context (for includes)
    pub async fn execute_task_minimal(
        task: &Task,
        variables: &crate::apply::variables::VariableContext,
        dry_run: bool,
        config_dir: &std::path::Path,
        plugin_manager: Option<std::sync::Arc<std::sync::RwLock<crate::plugins::PluginManager>>>,
    ) -> Result<serde_yaml::Value> {
        let task_type = task.task_type();

        // Handle plugin tasks specially
        if let TaskAction::Plugin(plugin_task) = &task.action {
            if let Some(pm) = &plugin_manager {
                let pm_read = pm.read().map_err(|_| {
                    anyhow::anyhow!(
                        "Failed to acquire read lock on plugin manager - lock is poisoned"
                    )
                })?;
                // Extract plugin name and task name from the plugin task
                // For now, assume the format is "plugin_name.task_name"
                let (plugin_name, task_name) =
                    crate::plugins::parse_plugin_component_name(&plugin_task.name)?;
                let config_json = serde_json::to_value(&plugin_task.config)?;
                match pm_read.execute_apply_task(plugin_name, task_name, &config_json) {
                    Ok(result) => return Ok(result),
                    Err(e) => return Err(anyhow::anyhow!("Plugin task execution failed: {}", e)),
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Plugin task '{}' requires plugin manager, but none is available",
                    plugin_task.name
                ));
            }
        }

        let entry = {
            let registry = TASK_REGISTRY.read().unwrap();
            registry.get(task_type.as_str()).cloned()
        };

        if let Some(entry) = entry {
            // Create a minimal executor context for included tasks
            let mut minimal_executor = crate::apply::executor::TaskExecutor::minimal(
                variables.clone(),
                dry_run,
                config_dir.to_path_buf(),
                plugin_manager,
                ApplyConfig {
                    vars: std::collections::HashMap::new(),
                    tasks: Vec::new(),
                    state_dir: crate::apply::default_state_dir(),
                },
            );
            (entry.executor)(task, &mut minimal_executor).await
        } else {
            Err(anyhow::anyhow!(
                "No executor registered for task type: {}",
                task_type
            ))
        }
    }

    /// Validate a task using the registry
    pub fn validate_task(task: &Task, task_index: usize) -> Result<()> {
        // Handle plugin tasks specially - plugins handle their own validation
        if let TaskAction::Plugin(_) = &task.action {
            return Ok(());
        }

        let task_type = task.task_type();

        let entry = {
            let registry = TASK_REGISTRY.read().unwrap();
            registry.get(task_type.as_str()).cloned()
        };

        if let Some(entry) = entry {
            if let Some(validator) = entry.validator {
                validator(task, task_index)
            } else {
                // No validator registered, task is considered valid
                Ok(())
            }
        } else {
            Err(anyhow::anyhow!(
                "No validator registered for task type: {}",
                task_type
            ))
        }
    }

    /// Get all registered task types
    pub fn get_registered_task_types() -> Vec<String> {
        let registry = TASK_REGISTRY.read().unwrap();
        registry.keys().cloned().collect()
    }

    /// Get the category for a task type
    pub fn get_task_category(task_type: &str) -> String {
        // Check if this is a plugin task (contains a dot)
        if task_type.contains('.') {
            return "Plugin Tasks".to_string();
        }

        let registry = TASK_REGISTRY.read().unwrap();
        registry
            .get(task_type)
            .map(|e| e.category.clone())
            .unwrap_or_else(|| "Other".to_string())
    }

    /// Get the filename for a task type
    pub fn get_task_filename(task_type: &str) -> String {
        // Check if this is a plugin task (contains a dot)
        if task_type.contains('.') {
            return "plugin".to_string();
        }

        let registry = TASK_REGISTRY.read().unwrap();
        registry
            .get(task_type)
            .map(|e| e.filename.clone())
            .unwrap_or_else(|| task_type.to_string())
    }

    /// Register a plugin-provided task executor at runtime
    #[allow(dead_code)]
    pub fn register_plugin_task(task_name: &str, executor: TaskExecutorFn) -> Result<()> {
        let mut registry = TASK_REGISTRY.write().unwrap();
        if registry.contains_key(task_name) {
            return Err(anyhow::anyhow!(
                "Task type '{}' is already registered",
                task_name
            ));
        }
        let entry = TaskRegistryEntry {
            executor,
            validator: None, // Plugins handle their own validation
            category: "Plugin Tasks".to_string(),
            filename: "plugin".to_string(),
        };
        registry.insert(task_name.to_string(), entry);
        Ok(())
    }
}

// Re-export task types for convenience
pub use apt::AptTask;
pub use archive::ArchiveTask;
pub use assert::AssertTask;
pub use authorized_key::AuthorizedKeyTask;
pub use blockinfile::BlockInFileTask;
pub use command::CommandTask;
pub use copy::CopyTask;
pub use cron::CronTask;
pub use debug::DebugTask;
pub use directory::DirectoryTask;
pub use fail::FailTask;
pub use fetch::FetchTask;
pub use file::FileTask;
pub use filesystem::FilesystemTask;
pub use firewalld::FirewalldTask;
pub use gem::GemTask;
pub use get_url::GetUrlTask;
pub use git::GitTask;
pub use group::GroupTask;
pub use hostname::HostnameTask;
pub use include_role::IncludeRoleTask;
pub use include_tasks::IncludeTasksTask;
pub use iptables::IptablesTask;
pub use journald::JournaldTask;
pub use lineinfile::LineInFileTask;
pub use logrotate::LogrotateTask;
pub use mount::MountTask;
pub use npm::NpmTask;
pub use package::{PackageState, PackageTask};
pub use pacman::PacmanTask;
pub use pause::PauseTask;
pub use pip::PipTask;
pub use raw::RawTask;
pub use reboot::RebootTask;
pub use replace::ReplaceTask;
pub use rsyslog::RsyslogTask;
pub use script::ScriptTask;
pub use selinux::SelinuxTask;
pub use service::ServiceTask;
pub use set_fact::SetFactTask;
pub use shutdown::ShutdownTask;
pub use stat::StatTask;
pub use sudoers::SudoersTask;
pub use sysctl::SysctlTask;
pub use template::TemplateTask;
pub use timezone::TimezoneTask;
pub use ufw::UfwTask;
pub use unarchive::UnarchiveTask;
pub use uri::UriTask;
pub use user::UserTask;
pub use wait_for::WaitForTask;
pub use yum::YumTask;
pub use zypper::ZypperTask;

// Public modules
pub mod apt;
pub mod archive;
pub mod assert;
pub mod authorized_key;
pub mod blockinfile;
pub mod command;
pub mod copy;
pub mod cron;
pub mod debug;
pub mod directory;
pub mod executor;
pub mod fail;
pub mod fetch;
pub mod file;
pub mod filesystem;
pub mod firewalld;
pub mod gem;
pub mod get_url;
pub mod git;
pub mod group;
pub mod hostname;
pub mod include_role;
pub mod include_tasks;
pub mod iptables;
pub mod journald;
pub mod lineinfile;
pub mod logrotate;
pub mod mount;
pub mod npm;
pub mod package;
pub mod pacman;
pub mod pause;
pub mod pip;
pub mod raw;
pub mod reboot;
pub mod replace;
pub mod rsyslog;
pub mod script;
pub mod selinux;
pub mod service;
pub mod set_fact;
pub mod shutdown;
pub mod stat;
pub mod sudoers;
pub mod sysctl;
pub mod template;
pub mod templating;
pub mod timezone;
pub mod ufw;
pub mod unarchive;
pub mod uri;
pub mod user;
pub mod variables;
pub mod wait_for;
pub mod yum;
pub mod zypper;

#[cfg(test)]
pub mod tests;

// Default value functions
/// Default true value
pub fn default_true() -> bool {
    true
}

/// Default state directory
pub fn default_state_dir() -> String {
    "/var/lib/driftless/state".to_string()
}

/// Main apply configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyConfig {
    /// Variables available to all tasks
    #[serde(default)]
    pub vars: std::collections::HashMap<String, serde_yaml::Value>,

    /// List of configuration operations to execute
    pub tasks: Vec<Task>,

    /// Directory for storing command execution state
    #[serde(default = "default_state_dir")]
    pub state_dir: String,
}

impl ApplyConfig {
    /// Merge another ApplyConfig into this one
    pub fn merge(&mut self, other: ApplyConfig) {
        // Merge variables (other variables take precedence)
        for (key, value) in other.vars {
            self.vars.insert(key, value);
        }

        // Merge tasks (extend the list)
        self.tasks.extend(other.tasks);

        // Merge state directory (other takes precedence if not default)
        if other.state_dir != default_state_dir() {
            self.state_dir = other.state_dir;
        }
    }
}

/// Generic task wrapper that includes common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// The specific task to execute
    #[serde(flatten)]
    pub action: TaskAction,

    /// Optional variable name to register the task result in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register: Option<String>,

    /// Optional condition to determine if the task should run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

impl Task {
    /// Create a new task with the given action
    #[allow(dead_code)]
    pub fn new(action: TaskAction) -> Self {
        Self {
            action,
            register: None,
            when: None,
        }
    }

    /// Set the register field
    #[allow(dead_code)]
    pub fn with_register(mut self, register: &str) -> Self {
        self.register = Some(register.to_string());
        self
    }

    /// Set the condition field
    #[allow(dead_code)]
    pub fn with_when(mut self, when: &str) -> Self {
        self.when = Some(when.to_string());
        self
    }

    /// Get the string representation of the task type
    pub fn task_type(&self) -> String {
        self.action.task_type()
    }
}

/// Types of configuration operations
///
/// These operations define desired system state and are executed idempotently.
/// Each operation type corresponds to a specific aspect of system configuration.
///
/// Configuration operations are distinct from:
/// - **Facts collectors**: Gather system metrics and information
/// - **Log sources/outputs**: Handle log collection and forwarding
/// - **Plugin-provided task configuration**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTask {
    /// The plugin task name
    pub name: String,
    /// Task-specific configuration
    #[serde(flatten)]
    pub config: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TaskAction {
    /// File system operations (create, modify, delete files/directories)
    File(FileTask),
    /// Software package management (install, remove, update packages)
    Package(PackageTask),
    /// System service management (start, stop, enable, disable services)
    Service(ServiceTask),
    /// User account management
    User(UserTask),
    /// Execute shell commands
    Command(CommandTask),
    /// Directory management operations
    Directory(DirectoryTask),
    /// User group management
    Group(GroupTask),
    /// Scheduled task (cron job) management
    Cron(CronTask),
    /// Filesystem mount operations
    Mount(MountTask),
    /// Filesystem creation and management
    Filesystem(FilesystemTask),
    /// Kernel parameter (sysctl) management
    Sysctl(SysctlTask),
    /// System hostname configuration
    Hostname(HostnameTask),
    /// System timezone configuration
    Timezone(TimezoneTask),
    /// System reboot operations
    Reboot(RebootTask),
    /// System shutdown operations
    Shutdown(ShutdownTask),
    /// File copy operations
    Copy(CopyTask),
    /// Template file rendering
    Template(TemplateTask),
    /// Single line file modifications
    LineInFile(LineInFileTask),
    /// Multi-line block file modifications
    BlockInFile(BlockInFileTask),
    /// Text replacement in files
    Replace(ReplaceTask),
    /// Remote file fetching (SCP/SFTP)
    Fetch(FetchTask),
    /// Web service interactions (HTTP API calls)
    Uri(UriTask),
    /// File downloading from URLs (HTTP/HTTPS/FTP)
    GetUrl(GetUrlTask),
    /// Archive extraction operations
    Unarchive(UnarchiveTask),
    /// Archive files
    Archive(ArchiveTask),
    /// Get file/directory statistics
    Stat(StatTask),
    /// Debian/Ubuntu package management
    Apt(AptTask),
    /// RHEL/CentOS/Fedora package management
    Yum(YumTask),
    /// Arch Linux package management
    Pacman(PacmanTask),
    /// SUSE package management
    Zypper(ZypperTask),
    /// Python package management
    Pip(PipTask),
    /// Node.js package management
    Npm(NpmTask),
    /// Ruby gem management
    Gem(GemTask),
    /// Execute local scripts
    Script(ScriptTask),
    /// Execute commands without shell processing
    Raw(RawTask),
    /// Debug output for troubleshooting
    Debug(DebugTask),
    /// Assert conditions are met
    Assert(AssertTask),
    /// Force failure with custom message
    Fail(FailTask),
    /// Wait for conditions to be met
    WaitFor(WaitForTask),
    /// Pause execution
    Pause(PauseTask),
    /// Set facts for later use
    SetFact(SetFactTask),
    /// Include external task files
    IncludeTasks(IncludeTasksTask),
    /// Include roles
    IncludeRole(IncludeRoleTask),
    /// SSH authorized keys management
    AuthorizedKey(AuthorizedKeyTask),
    /// Sudoers configuration management
    Sudoers(SudoersTask),
    /// Firewalld firewall management
    Firewalld(FirewalldTask),
    /// UFW firewall management
    Ufw(UfwTask),
    /// SELinux policy management
    Selinux(SelinuxTask),
    /// iptables firewall management
    Iptables(IptablesTask),
    /// Git repository management
    Git(GitTask),
    /// Logrotate configuration management
    Logrotate(LogrotateTask),
    /// Rsyslog configuration management
    Rsyslog(RsyslogTask),
    /// systemd journal configuration management
    Journald(JournaldTask),
    /// Plugin-provided tasks
    Plugin(PluginTask),
}

impl TaskAction {
    /// Get the string representation of the task type
    pub fn task_type(&self) -> String {
        match self {
            TaskAction::File(_) => "file".to_string(),
            TaskAction::Package(_) => "package".to_string(),
            TaskAction::Service(_) => "service".to_string(),
            TaskAction::User(_) => "user".to_string(),
            TaskAction::Command(_) => "command".to_string(),
            TaskAction::Directory(_) => "directory".to_string(),
            TaskAction::Group(_) => "group".to_string(),
            TaskAction::Cron(_) => "cron".to_string(),
            TaskAction::Mount(_) => "mount".to_string(),
            TaskAction::Filesystem(_) => "filesystem".to_string(),
            TaskAction::Sysctl(_) => "sysctl".to_string(),
            TaskAction::Hostname(_) => "hostname".to_string(),
            TaskAction::Timezone(_) => "timezone".to_string(),
            TaskAction::Reboot(_) => "reboot".to_string(),
            TaskAction::Shutdown(_) => "shutdown".to_string(),
            TaskAction::Copy(_) => "copy".to_string(),
            TaskAction::Template(_) => "template".to_string(),
            TaskAction::LineInFile(_) => "lineinfile".to_string(),
            TaskAction::BlockInFile(_) => "blockinfile".to_string(),
            TaskAction::Replace(_) => "replace".to_string(),
            TaskAction::Fetch(_) => "fetch".to_string(),
            TaskAction::Uri(_) => "uri".to_string(),
            TaskAction::GetUrl(_) => "geturl".to_string(),
            TaskAction::Unarchive(_) => "unarchive".to_string(),
            TaskAction::Archive(_) => "archive".to_string(),
            TaskAction::Stat(_) => "stat".to_string(),
            TaskAction::Apt(_) => "apt".to_string(),
            TaskAction::Yum(_) => "yum".to_string(),
            TaskAction::Pacman(_) => "pacman".to_string(),
            TaskAction::Zypper(_) => "zypper".to_string(),
            TaskAction::Pip(_) => "pip".to_string(),
            TaskAction::Npm(_) => "npm".to_string(),
            TaskAction::Gem(_) => "gem".to_string(),
            TaskAction::Script(_) => "script".to_string(),
            TaskAction::Raw(_) => "raw".to_string(),
            TaskAction::Debug(_) => "debug".to_string(),
            TaskAction::Assert(_) => "assert".to_string(),
            TaskAction::Fail(_) => "fail".to_string(),
            TaskAction::WaitFor(_) => "waitfor".to_string(),
            TaskAction::Pause(_) => "pause".to_string(),
            TaskAction::SetFact(_) => "setfact".to_string(),
            TaskAction::IncludeTasks(_) => "includetasks".to_string(),
            TaskAction::IncludeRole(_) => "includerole".to_string(),
            TaskAction::AuthorizedKey(_) => "authorizedkey".to_string(),
            TaskAction::Sudoers(_) => "sudoers".to_string(),
            TaskAction::Firewalld(_) => "firewalld".to_string(),
            TaskAction::Ufw(_) => "ufw".to_string(),
            TaskAction::Selinux(_) => "selinux".to_string(),
            TaskAction::Iptables(_) => "iptables".to_string(),
            TaskAction::Git(_) => "git".to_string(),
            TaskAction::Logrotate(_) => "logrotate".to_string(),
            TaskAction::Rsyslog(_) => "rsyslog".to_string(),
            TaskAction::Journald(_) => "journald".to_string(),
            TaskAction::Plugin(task) => task.name.clone(),
        }
    }

    /// Get the filename for this task type
    pub fn task_filename(task_type: &str) -> String {
        TaskRegistry::get_task_filename(task_type)
    }
}
