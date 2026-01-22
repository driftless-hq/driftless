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
            Box<dyn std::future::Future<Output = Result<(), anyhow::Error>> + Send + 'a>,
        > + Send
        + Sync,
>;

// Type alias for task validation functions
type TaskValidatorFn = Arc<dyn Fn(&Task, usize) -> Result<()> + Send + Sync>;

// Task registry entry containing both executor and validator
#[derive(Clone)]
struct TaskRegistryEntry {
    executor: TaskExecutorFn,
    validator: Option<TaskValidatorFn>,
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
    pub fn register(
        registry: &mut HashMap<String, TaskRegistryEntry>,
        task_type: &str,
        executor: TaskExecutorFn,
    ) {
        let entry = TaskRegistryEntry {
            executor,
            validator: None,
        };
        registry.insert(task_type.to_string(), entry);
    }

    /// Register a task executor function with validation
    pub fn register_with_validator(
        registry: &mut HashMap<String, TaskRegistryEntry>,
        task_type: &str,
        executor: TaskExecutorFn,
        validator: TaskValidatorFn,
    ) {
        let entry = TaskRegistryEntry {
            executor,
            validator: Some(validator),
        };
        registry.insert(task_type.to_string(), entry);
    }

    /// Initialize the registry with built-in task executors
    pub fn initialize_builtin_executors(registry: &mut HashMap<String, TaskRegistryEntry>) {
        // File operations
        TaskRegistry::register_with_validator(
            registry,
            "file",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::File(file_task) = task {
                        crate::apply::file::execute_file_task(&file_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for file executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::File(file_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Directory(dir_task) = task {
                        crate::apply::directory::execute_directory_task(
                            &dir_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for directory executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Directory(dir_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Copy(copy_task) = task {
                        crate::apply::copy::execute_copy_task(&copy_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for copy executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Copy(copy_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Package(pkg_task) = task {
                        crate::apply::package::execute_package_task(&pkg_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for package executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Package(pkg_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Apt(apt_task) = task {
                        crate::apply::apt::execute_apt_task(&apt_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for apt executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Apt(apt_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Service(service_task) = task {
                        crate::apply::service::execute_service_task(
                            &service_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for service executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Service(service_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::User(user_task) = task {
                        crate::apply::user::execute_user_task(&user_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for user executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::User(user_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Group(group_task) = task {
                        crate::apply::group::execute_group_task(&group_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for group executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Group(group_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Command(cmd_task) = task {
                        crate::apply::command::execute_command_task(&cmd_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for command executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Command(cmd_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Script(script_task) = task {
                        crate::apply::script::execute_script_task(&script_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for script executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Script(script_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Raw(raw_task) = task {
                        crate::apply::raw::execute_raw_task(&raw_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for raw executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Raw(raw_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Debug(debug_task) = task {
                        crate::apply::executor::execute_debug_task(
                            &debug_task,
                            executor.variables(),
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for debug executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Debug(debug_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Assert(assert_task) = task {
                        crate::apply::executor::execute_assert_task(
                            &assert_task,
                            executor.variables(),
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for assert executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Assert(assert_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Fail(fail_task) = task {
                        crate::apply::executor::execute_fail_task(
                            &fail_task,
                            executor.variables(),
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for fail executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Fail(fail_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Pause(pause_task) = task {
                        crate::apply::executor::execute_pause_task(&pause_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for pause executor"))
                    }
                })
            }),
        );

        TaskRegistry::register_with_validator(
            registry,
            "setfact",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::SetFact(set_fact_task) = task {
                        let dry_run = executor.dry_run();
                        crate::apply::executor::execute_set_fact_task(
                            &set_fact_task,
                            executor.variables_mut(),
                            dry_run,
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for setfact executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::SetFact(set_fact_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::IncludeTasks(include_tasks_task) = task {
                        crate::apply::executor::execute_include_tasks_task(
                            &include_tasks_task,
                            executor.variables(),
                            executor.dry_run(),
                            executor.config_dir(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for includetasks executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::IncludeTasks(include_tasks_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::IncludeRole(include_role_task) = task {
                        crate::apply::executor::execute_include_role_task(
                            &include_role_task,
                            executor.variables(),
                            executor.dry_run(),
                            executor.config_dir(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for includerole executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::IncludeRole(include_role_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Cron(cron_task) = task {
                        crate::apply::cron::execute_cron_task(&cron_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for cron executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Cron(cron_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Mount(mount_task) = task {
                        crate::apply::mount::execute_mount_task(&mount_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for mount executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Mount(mount_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Filesystem(fs_task) = task {
                        crate::apply::filesystem::execute_filesystem_task(
                            &fs_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for filesystem executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Filesystem(fs_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Sysctl(sysctl_task) = task {
                        crate::apply::sysctl::execute_sysctl_task(&sysctl_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for sysctl executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Sysctl(sysctl_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Hostname(hostname_task) = task {
                        crate::apply::hostname::execute_hostname_task(
                            &hostname_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for hostname executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Hostname(hostname_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Timezone(timezone_task) = task {
                        crate::apply::timezone::execute_timezone_task(
                            &timezone_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for timezone executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Timezone(timezone_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Reboot(reboot_task) = task {
                        crate::apply::reboot::execute_reboot_task(&reboot_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for reboot executor"))
                    }
                })
            }),
        );

        TaskRegistry::register(
            registry,
            "shutdown",
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Shutdown(shutdown_task) = task {
                        crate::apply::shutdown::execute_shutdown_task(
                            &shutdown_task,
                            executor.dry_run(),
                        )
                        .await
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Template(template_task) = task {
                        crate::apply::template::execute_template_task(
                            &template_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for template executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Template(template_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::LineInFile(line_task) = task {
                        crate::apply::lineinfile::execute_lineinfile_task(
                            &line_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for lineinfile executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::LineInFile(line_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::BlockInFile(block_task) = task {
                        crate::apply::blockinfile::execute_blockinfile_task(
                            &block_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for blockinfile executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::BlockInFile(block_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Replace(replace_task) = task {
                        crate::apply::replace::execute_replace_task(
                            &replace_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for replace executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Replace(replace_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Fetch(fetch_task) = task {
                        crate::apply::fetch::execute_fetch_task(&fetch_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for fetch executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Fetch(fetch_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Uri(uri_task) = task {
                        crate::apply::uri::execute_uri_task(&uri_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for uri executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Uri(uri_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::GetUrl(get_url_task) = task {
                        crate::apply::get_url::execute_get_url_task(
                            &get_url_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for geturl executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::GetUrl(get_url_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Unarchive(unarchive_task) = task {
                        crate::apply::unarchive::execute_unarchive_task(
                            &unarchive_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for unarchive executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Unarchive(unarchive_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Archive(archive_task) = task {
                        crate::apply::archive::execute_archive_task(
                            &archive_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for archive executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Archive(archive_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Stat(stat_task) = task {
                        crate::apply::stat::execute_stat_task(&stat_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for stat executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Stat(stat_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Yum(yum_task) = task {
                        crate::apply::yum::execute_yum_task(&yum_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for yum executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Yum(yum_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Pacman(pacman_task) = task {
                        crate::apply::pacman::execute_pacman_task(&pacman_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for pacman executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Pacman(pacman_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Zypper(zypper_task) = task {
                        crate::apply::zypper::execute_zypper_task(&zypper_task, executor.dry_run())
                            .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for zypper executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Zypper(zypper_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Pip(pip_task) = task {
                        crate::apply::pip::execute_pip_task(&pip_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for pip executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Pip(pip_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Npm(npm_task) = task {
                        crate::apply::npm::execute_npm_task(&npm_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for npm executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Npm(npm_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Gem(gem_task) = task {
                        crate::apply::gem::execute_gem_task(&gem_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for gem executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Gem(gem_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::WaitFor(wait_for_task) = task {
                        crate::apply::executor::execute_wait_for_task(
                            &wait_for_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for waitfor executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::WaitFor(wait_for_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::AuthorizedKey(authorized_key_task) = task {
                        crate::apply::authorized_key::execute_authorized_key_task(
                            &authorized_key_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!(
                            "Invalid task type for authorizedkey executor"
                        ))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::AuthorizedKey(authorized_key_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Sudoers(sudoers_task) = task {
                        crate::apply::sudoers::execute_sudoers_task(
                            &sudoers_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for sudoers executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Sudoers(sudoers_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Firewalld(firewalld_task) = task {
                        crate::apply::firewalld::execute_firewalld_task(
                            &firewalld_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for firewalld executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Firewalld(firewalld_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Ufw(ufw_task) = task {
                        crate::apply::ufw::execute_ufw_task(&ufw_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for ufw executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Ufw(ufw_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Selinux(selinux_task) = task {
                        crate::apply::selinux::execute_selinux_task(
                            &selinux_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for selinux executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Selinux(selinux_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Iptables(iptables_task) = task {
                        crate::apply::iptables::execute_iptables_task(
                            &iptables_task,
                            executor.dry_run(),
                        )
                        .await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for iptables executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Iptables(iptables_task) = task {
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
            Arc::new(|task, executor: &mut TaskExecutor| {
                Box::pin(async move {
                    if let Task::Git(git_task) = task {
                        crate::apply::git::execute_git_task(&git_task, executor.dry_run()).await
                    } else {
                        Err(anyhow::anyhow!("Invalid task type for git executor"))
                    }
                })
            }),
            Arc::new(|task, task_index| {
                if let Task::Git(git_task) = task {
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
    }

    /// Execute a task using the registry with minimal context (for includes)
    pub async fn execute_task_minimal(
        task: &Task,
        variables: &crate::apply::variables::VariableContext,
        dry_run: bool,
        config_dir: &std::path::Path,
    ) -> Result<()> {
        let task_type = task.task_type();

        let entry = {
            let registry = TASK_REGISTRY.read().unwrap();
            registry.get(task_type).cloned()
        };

        if let Some(entry) = entry {
            // Create a minimal executor context for included tasks
            let mut minimal_executor = crate::apply::executor::TaskExecutor::minimal(
                variables.clone(),
                dry_run,
                config_dir.to_path_buf(),
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
        let task_type = task.task_type();

        let entry = {
            let registry = TASK_REGISTRY.read().unwrap();
            registry.get(task_type).cloned()
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
pub use lineinfile::LineInFileTask;
pub use mount::MountTask;
pub use npm::NpmTask;
pub use package::{PackageState, PackageTask};
pub use pacman::PacmanTask;
pub use pause::PauseTask;
pub use pip::PipTask;
pub use raw::RawTask;
pub use reboot::RebootTask;
pub use replace::ReplaceTask;
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
pub mod lineinfile;
pub mod mount;
pub mod npm;
pub mod package;
pub mod pacman;
pub mod pause;
pub mod pip;
pub mod raw;
pub mod reboot;
pub mod replace;
pub mod script;
pub mod selinux;
pub mod service;
pub mod set_fact;
pub mod shutdown;
pub mod stat;
pub mod sudoers;
pub mod sysctl;
pub mod template;
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

/// Main apply configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyConfig {
    /// Variables available to all tasks
    #[serde(default)]
    pub vars: std::collections::HashMap<String, serde_yaml::Value>,

    /// List of configuration operations to execute
    pub tasks: Vec<Task>,
}

/// Types of configuration operations
///
/// These operations define desired system state and are executed idempotently.
/// Each operation type corresponds to a specific aspect of system configuration.
///
/// Configuration operations are distinct from:
/// - **Facts collectors**: Gather system metrics and information
/// - **Log sources/outputs**: Handle log collection and forwarding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Task {
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
}

impl Task {
    /// Get the string representation of the task type
    pub fn task_type(&self) -> &'static str {
        match self {
            Task::File(_) => "file",
            Task::Package(_) => "package",
            Task::Service(_) => "service",
            Task::User(_) => "user",
            Task::Command(_) => "command",
            Task::Directory(_) => "directory",
            Task::Group(_) => "group",
            Task::Cron(_) => "cron",
            Task::Mount(_) => "mount",
            Task::Filesystem(_) => "filesystem",
            Task::Sysctl(_) => "sysctl",
            Task::Hostname(_) => "hostname",
            Task::Timezone(_) => "timezone",
            Task::Reboot(_) => "reboot",
            Task::Shutdown(_) => "shutdown",
            Task::Copy(_) => "copy",
            Task::Template(_) => "template",
            Task::LineInFile(_) => "lineinfile",
            Task::BlockInFile(_) => "blockinfile",
            Task::Replace(_) => "replace",
            Task::Fetch(_) => "fetch",
            Task::Uri(_) => "uri",
            Task::GetUrl(_) => "geturl",
            Task::Unarchive(_) => "unarchive",
            Task::Archive(_) => "archive",
            Task::Stat(_) => "stat",
            Task::Apt(_) => "apt",
            Task::Yum(_) => "yum",
            Task::Pacman(_) => "pacman",
            Task::Zypper(_) => "zypper",
            Task::Pip(_) => "pip",
            Task::Npm(_) => "npm",
            Task::Gem(_) => "gem",
            Task::Script(_) => "script",
            Task::Raw(_) => "raw",
            Task::Debug(_) => "debug",
            Task::Assert(_) => "assert",
            Task::Fail(_) => "fail",
            Task::WaitFor(_) => "waitfor",
            Task::Pause(_) => "pause",
            Task::SetFact(_) => "setfact",
            Task::IncludeTasks(_) => "includetasks",
            Task::IncludeRole(_) => "includerole",
            Task::AuthorizedKey(_) => "authorizedkey",
            Task::Sudoers(_) => "sudoers",
            Task::Firewalld(_) => "firewalld",
            Task::Ufw(_) => "ufw",
            Task::Selinux(_) => "selinux",
            Task::Iptables(_) => "iptables",
            Task::Git(_) => "git",
        }
    }

    /// Get the filename for this task type
    pub fn task_filename(task_type: &str) -> String {
        match task_type {
            "file" => "file".to_string(),
            "package" => "package".to_string(),
            "service" => "service".to_string(),
            "user" => "user".to_string(),
            "command" => "command".to_string(),
            "directory" => "directory".to_string(),
            "group" => "group".to_string(),
            "cron" => "cron".to_string(),
            "mount" => "mount".to_string(),
            "filesystem" => "filesystem".to_string(),
            "sysctl" => "sysctl".to_string(),
            "hostname" => "hostname".to_string(),
            "timezone" => "timezone".to_string(),
            "reboot" => "reboot".to_string(),
            "shutdown" => "shutdown".to_string(),
            "copy" => "copy".to_string(),
            "template" => "template".to_string(),
            "lineinfile" => "lineinfile".to_string(),
            "blockinfile" => "blockinfile".to_string(),
            "replace" => "replace".to_string(),
            "fetch" => "fetch".to_string(),
            "uri" => "uri".to_string(),
            "geturl" => "get_url".to_string(),
            "unarchive" => "unarchive".to_string(),
            "archive" => "archive".to_string(),
            "stat" => "stat".to_string(),
            "apt" => "apt".to_string(),
            "yum" => "yum".to_string(),
            "pacman" => "pacman".to_string(),
            "zypper" => "zypper".to_string(),
            "pip" => "pip".to_string(),
            "npm" => "npm".to_string(),
            "gem" => "gem".to_string(),
            "script" => "script".to_string(),
            "raw" => "raw".to_string(),
            "debug" => "debug".to_string(),
            "assert" => "assert".to_string(),
            "fail" => "fail".to_string(),
            "waitfor" => "wait_for".to_string(),
            "pause" => "pause".to_string(),
            "setfact" => "set_fact".to_string(),
            "includetasks" => "include_tasks".to_string(),
            "includerole" => "include_role".to_string(),
            "authorizedkey" => "authorized_key".to_string(),
            "sudoers" => "sudoers".to_string(),
            "firewalld" => "firewalld".to_string(),
            "ufw" => "ufw".to_string(),
            "selinux" => "selinux".to_string(),
            "iptables" => "iptables".to_string(),
            "git" => "git".to_string(),
            _ => task_type.to_string(),
        }
    }
}
