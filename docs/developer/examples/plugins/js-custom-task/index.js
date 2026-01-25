/**
 * Example Custom Task Plugin for Driftless (JavaScript)
 *
 * This plugin demonstrates how to create custom tasks using JavaScript
 * that can be executed by the Driftless agent.
 */

// Plugin interface functions that must be exported
function get_task_definitions() {
    return JSON.stringify([
        {
            name: "js_echo",
            type: "apply",
            config_schema: {
                type: "object",
                properties: {
                    message: {
                        type: "string",
                        description: "Message to log"
                    },
                    level: {
                        type: "string",
                        enum: ["info", "warn", "error"],
                        default: "info",
                        description: "Log level"
                    }
                },
                required: ["message"]
            }
        },
        {
            name: "js_calculate",
            type: "apply",
            config_schema: {
                type: "object",
                properties: {
                    expression: {
                        type: "string",
                        description: "Mathematical expression to evaluate"
                    },
                    variable: {
                        type: "string",
                        description: "Variable name to store result"
                    }
                },
                required: ["expression"]
            }
        }
    ]);
}

function execute_task(name, configJson) {
    try {
        switch (name) {
            case "js_echo":
                return execute_echo_task(configJson);
            case "js_calculate":
                return execute_calculate_task(configJson);
            default:
                return JSON.stringify({
                    status: "error",
                    message: `Unknown task: ${name}`
                });
        }
    } catch (error) {
        return JSON.stringify({
            status: "error",
            message: `Task execution failed: ${error.message}`
        });
    }
}

function execute_echo_task(configJson) {
    const config = JSON.parse(configJson);
    const message = config.message || "No message";
    const level = config.level || "info";

    // Log the message (in WASM environment, this would use host logging)
    console.log(`JS ECHO [${level.toUpperCase()}]: ${message}`);

    return JSON.stringify({
        status: "success",
        message: `Logged message: ${message}`,
        level: level
    });
}

function execute_calculate_task(configJson) {
    const config = JSON.parse(configJson);
    const expression = config.expression;

    if (!expression) {
        return JSON.stringify({
            status: "error",
            message: "Expression is required"
        });
    }

    try {
        // Note: In a real implementation, use a safe expression evaluator
        // This is just for demonstration - eval is dangerous!
        const result = Function('"use strict"; return (' + expression + ')')();

        console.log(`JS CALCULATE: ${expression} = ${result}`);

        return JSON.stringify({
            status: "success",
            expression: expression,
            result: result,
            variable: config.variable
        });
    } catch (error) {
        return JSON.stringify({
            status: "error",
            message: `Invalid expression: ${error.message}`
        });
    }
}

// Other required plugin functions (return empty arrays)
function get_facts_collectors() {
    return "[]";
}

function get_template_extensions() {
    return "[]";
}

function get_log_sources() {
    return "[]";
}

function get_log_parsers() {
    return "[]";
}

function get_log_filters() {
    return "[]";
}

function get_log_outputs() {
    return "[]";
}

// Export functions for WASM binding
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        get_task_definitions,
        execute_task,
        get_facts_collectors,
        get_template_extensions,
        get_log_sources,
        get_log_parsers,
        get_log_filters,
        get_log_outputs
    };
}

// For browser/WASM environment
if (typeof window !== 'undefined') {
    window.get_task_definitions = get_task_definitions;
    window.execute_task = execute_task;
    window.get_facts_collectors = get_facts_collectors;
    window.get_template_extensions = get_template_extensions;
    window.get_log_sources = get_log_sources;
    window.get_log_parsers = get_log_parsers;
    window.get_log_filters = get_log_filters;
    window.get_log_outputs = get_log_outputs;
}