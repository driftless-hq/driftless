//! Tests for apply task functionality

#[cfg(test)]
use crate::apply::debug::DebugVerbosity;
#[cfg(test)]
use crate::apply::executor::TaskExecutor;
#[cfg(test)]
use crate::apply::variables::VariableContext;
#[cfg(test)]
use crate::apply::wait_for::ConnectionState;
#[cfg(test)]
use crate::apply::{
    AssertTask, DebugTask, FailTask, IncludeRoleTask, IncludeTasksTask, PauseTask, SetFactTask,
    Task, TaskAction, WaitForTask,
};
#[cfg(test)]
use serde_json;
#[cfg(test)]
use serde_yaml::Value;

#[test]
fn test_variable_context_basic() {
    let mut ctx = VariableContext::new();

    // Test setting and getting
    ctx.set("name".to_string(), Value::String("alice".to_string()));
    ctx.set("count".to_string(), Value::Number(42.into()));

    assert_eq!(ctx.get("name"), Some(&Value::String("alice".to_string())));
    assert_eq!(ctx.get("missing"), None);
}

#[test]
fn test_variable_template_rendering() {
    let mut ctx = VariableContext::new();
    ctx.set("user".to_string(), Value::String("bob".to_string()));
    ctx.set("count".to_string(), Value::Number(42.into()));

    assert_eq!(ctx.render_template("Hello {{ user }}!"), "Hello bob!");
    assert_eq!(ctx.render_template("Count: {{ count }}"), "Count: 42");
    assert_eq!(ctx.render_template("No vars"), "No vars");
}

#[test]
fn test_condition_evaluation() {
    let mut ctx = VariableContext::new();
    ctx.set("status".to_string(), Value::String("ready".to_string()));
    ctx.set("enabled".to_string(), Value::Bool(true));

    assert!(ctx.evaluate_condition("true"));
    assert!(!ctx.evaluate_condition("false"));
    assert!(ctx.evaluate_condition("{{ enabled }} == true"));
    assert!(!ctx.evaluate_condition("{{ status }} == pending"));
    assert!(ctx.evaluate_condition("{{ status }} == ready"));
}

#[test]
fn test_debug_task_variable_resolution() {
    let mut ctx = VariableContext::new();
    ctx.set(
        "test_var".to_string(),
        Value::String("test_value".to_string()),
    );

    let debug_task = DebugTask {
        description: None,
        msg: "Test message".to_string(),
        var: Some("test_var".to_string()),
        verbosity: DebugVerbosity::Normal,
    };

    // This would need tokio runtime for async test
    // For now, just test the structure exists
    assert_eq!(debug_task.var, Some("test_var".to_string()));
}

#[test]
fn test_assert_task_structure() {
    let assert_task = AssertTask {
        description: Some("Test assertion".to_string()),
        that: "{{ status }} == ready".to_string(),
        success_msg: Some("Success!".to_string()),
        fail_msg: Some("Failed!".to_string()),
        quiet: false,
    };

    assert_eq!(assert_task.that, "{{ status }} == ready");
    assert_eq!(assert_task.success_msg, Some("Success!".to_string()));
    assert!(!assert_task.quiet);
}

#[test]
fn test_fail_task_structure() {
    let fail_task = FailTask {
        description: Some("Test failure".to_string()),
        msg: "This should fail".to_string(),
    };

    assert_eq!(fail_task.msg, "This should fail");
}

#[test]
fn test_wait_for_task_structure() {
    let wait_for_task = WaitForTask {
        description: Some("Wait for service".to_string()),
        host: Some("localhost".to_string()),
        port: Some(8080),
        path: None,
        timeout: 30,
        delay: 1,
        state: ConnectionState::Started,
        active_connection: true,
    };

    assert_eq!(wait_for_task.host, Some("localhost".to_string()));
    assert_eq!(wait_for_task.port, Some(8080));
    assert_eq!(wait_for_task.timeout, 30);
}

#[test]
fn test_pause_task_structure() {
    let pause_task = PauseTask {
        description: Some("Pause for user".to_string()),
        prompt: "Press enter to continue...".to_string(),
        seconds: 30,
        minutes: 0,
    };

    assert_eq!(pause_task.prompt, "Press enter to continue...");
    assert_eq!(pause_task.seconds, 30);
    assert_eq!(pause_task.minutes, 0);
}

#[test]
fn test_set_fact_task_structure() {
    let set_fact_task = SetFactTask {
        description: Some("Set deployment info".to_string()),
        key: "deployment_id".to_string(),
        value: Value::String("abc123".to_string()),
        cacheable: true,
    };

    assert_eq!(set_fact_task.key, "deployment_id");
    assert_eq!(set_fact_task.value, Value::String("abc123".to_string()));
    assert!(set_fact_task.cacheable);
}

#[test]
fn test_include_tasks_structure() {
    let include_tasks = IncludeTasksTask {
        description: Some("Include database tasks".to_string()),
        file: "tasks/database.yml".to_string(),
        vars: std::collections::HashMap::new(),
    };

    assert_eq!(include_tasks.file, "tasks/database.yml");
}

#[test]
fn test_include_role_structure() {
    let include_role = IncludeRoleTask {
        description: Some("Include webserver role".to_string()),
        name: "webserver".to_string(),
        vars: std::collections::HashMap::new(),
        defaults: std::collections::HashMap::new(),
    };

    assert_eq!(include_role.name, "webserver");
}

#[tokio::test]
async fn test_include_tasks_successful_execution() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir(&tasks_dir).unwrap();

    // Create a tasks file to include
    let tasks_file = tasks_dir.join("common.yml");
    let tasks_content = r#"
---
- type: debug
  msg: "Included task executed"
- type: setfact
  key: "included_var"
  value: "test_value"
"#;
    fs::write(&tasks_file, tasks_content).unwrap();

    // Create executor
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        std::collections::HashMap::new(),
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include task
    let include_task = IncludeTasksTask {
        description: Some("Test include".to_string()),
        file: "tasks/common.yml".to_string(),

        vars: std::collections::HashMap::new(),
    };

    // Execute the include task
    let result = executor
        .execute_single_task(&Task::new(TaskAction::IncludeTasks(include_task)))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_include_tasks_with_condition_true() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir(&tasks_dir).unwrap();

    // Create a tasks file to include
    let tasks_file = tasks_dir.join("conditional.yml");
    let tasks_content = r#"
---
- type: debug
  msg: "Conditional task executed"
"#;
    fs::write(&tasks_file, tasks_content).unwrap();

    // Create executor with variables
    let mut vars = std::collections::HashMap::new();
    vars.insert("should_include".to_string(), serde_yaml::Value::Bool(true));
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        vars,
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include task with condition
    let include_task = IncludeTasksTask {
        description: Some("Conditional include".to_string()),
        file: "tasks/conditional.yml".to_string(),
        vars: std::collections::HashMap::new(),
    };

    // Execute the include task
    let result = executor
        .execute_single_task(
            &Task::new(TaskAction::IncludeTasks(include_task))
                .with_when("{{ should_include }} == true"),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_include_tasks_with_condition_false() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir(&tasks_dir).unwrap();

    // Create a tasks file to include
    let tasks_file = tasks_dir.join("skipped.yml");
    let tasks_content = r#"
---
- type: debug
  msg: "This should not execute"
"#;
    fs::write(&tasks_file, tasks_content).unwrap();

    // Create executor with variables
    let mut vars = std::collections::HashMap::new();
    vars.insert("should_include".to_string(), serde_yaml::Value::Bool(false));
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        vars,
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include task with condition that evaluates to false
    let include_task = IncludeTasksTask {
        description: Some("Skipped include".to_string()),
        file: "tasks/skipped.yml".to_string(),
        vars: std::collections::HashMap::new(),
    };

    // Execute the include task - should succeed but not execute included tasks
    let result = executor
        .execute_single_task(
            &Task::new(TaskAction::IncludeTasks(include_task))
                .with_when("{{ should_include }} == true"),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_include_tasks_missing_file() {
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();

    // Create executor
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        std::collections::HashMap::new(),
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include task pointing to non-existent file
    let include_task = IncludeTasksTask {
        description: Some("Missing file test".to_string()),
        file: "tasks/missing.yml".to_string(),
        vars: std::collections::HashMap::new(),
    };

    // Execute the include task - should fail
    let result = executor
        .execute_single_task(&Task::new(TaskAction::IncludeTasks(include_task)))
        .await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to read task file"));
}

#[tokio::test]
async fn test_include_role_successful_execution() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let roles_dir = temp_dir.path().join("roles");
    let webserver_dir = roles_dir.join("webserver");
    let defaults_dir = webserver_dir.join("defaults");
    let tasks_dir = webserver_dir.join("tasks");
    fs::create_dir_all(&defaults_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    // Create role defaults
    let defaults_file = defaults_dir.join("main.yml");
    let defaults_content = r#"
---
role_var: "default_value"
"#;
    fs::write(&defaults_file, defaults_content).unwrap();

    // Create role tasks
    let tasks_file = tasks_dir.join("main.yml");
    let tasks_content = r#"
---
- type: debug
  msg: "Role task executed with {{ role_var }}"
- type: setfact
  key: "role_test"
  value: "executed"
"#;
    fs::write(&tasks_file, tasks_content).unwrap();

    // Create executor
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        std::collections::HashMap::new(),
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include role task
    let include_role = IncludeRoleTask {
        description: Some("Test role".to_string()),
        name: "webserver".to_string(),

        vars: std::collections::HashMap::new(),
        defaults: std::collections::HashMap::new(),
    };

    // Execute the include role task
    let result = executor
        .execute_single_task(&Task::new(TaskAction::IncludeRole(include_role)))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_include_role_missing_role() {
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();

    // Create executor
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        std::collections::HashMap::new(),
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include role task pointing to non-existent role
    let include_role = IncludeRoleTask {
        description: Some("Missing role test".to_string()),
        name: "nonexistent".to_string(),
        vars: std::collections::HashMap::new(),
        defaults: std::collections::HashMap::new(),
    };

    // Execute the include role task - should fail
    let result = executor
        .execute_single_task(&Task::new(TaskAction::IncludeRole(include_role)))
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_include_role_missing_tasks_file() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory structure with defaults but no tasks
    let temp_dir = TempDir::new().unwrap();
    let roles_dir = temp_dir.path().join("roles");
    let webserver_dir = roles_dir.join("webserver");
    let defaults_dir = webserver_dir.join("defaults");
    fs::create_dir_all(&defaults_dir).unwrap();

    // Create role defaults
    let defaults_file = defaults_dir.join("main.yml");
    fs::write(&defaults_file, "role_var: default").unwrap();

    // Create executor
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        std::collections::HashMap::new(),
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include role task
    let include_role = IncludeRoleTask {
        description: Some("Missing tasks test".to_string()),
        name: "webserver".to_string(),

        vars: std::collections::HashMap::new(),
        defaults: std::collections::HashMap::new(),
    };

    // Execute the include role task - should fail
    let result = executor
        .execute_single_task(&Task::new(TaskAction::IncludeRole(include_role)))
        .await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("missing tasks/main.yml"));
}

#[tokio::test]
async fn test_include_tasks_variable_passing() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir(&tasks_dir).unwrap();

    // Create a tasks file that uses variables
    let tasks_file = tasks_dir.join("vars.yml");
    let tasks_content = r#"
---
- type: debug
  msg: "Variable value: {{ passed_var }}"
- type: setfact
  key: "result"
  value: "{{ passed_var }}_processed"
"#;
    fs::write(&tasks_file, tasks_content).unwrap();

    // Create executor with variables
    let mut vars = std::collections::HashMap::new();
    vars.insert(
        "passed_var".to_string(),
        serde_yaml::Value::String("test_value".to_string()),
    );
    let mut executor = TaskExecutor::with_vars_from_context(
        true, // dry_run
        vars,
        VariableContext::new(),
        temp_dir.path().to_path_buf(),
    );

    // Create include task with vars
    let mut include_vars = std::collections::HashMap::new();
    include_vars.insert(
        "passed_var".to_string(),
        serde_json::Value::String("override_value".to_string()),
    );
    let include_task = IncludeTasksTask {
        description: Some("Variable passing test".to_string()),
        file: "tasks/vars.yml".to_string(),
        vars: include_vars,
    };

    // Execute the include task
    let result = executor
        .execute_single_task(&Task::new(TaskAction::IncludeTasks(include_task)))
        .await;
    assert!(result.is_ok());
}
