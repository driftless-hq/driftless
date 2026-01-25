/**
 * Example Template Extension Plugin for Driftless (TypeScript)
 *
 * This plugin demonstrates how to create custom Jinja2 filters and functions
 * for use in Driftless templates using TypeScript.
 */

interface TemplateExtension {
    name: string;
    type: 'filter' | 'function';
    config_schema: any;
    description: string;
    category: string;
    arguments: [string, string][];
}

interface TaskDefinition {
    name: string;
    type: 'apply' | 'facts' | 'logs';
    config_schema: any;
}

// Plugin interface functions that must be exported
export function get_template_extensions(): string {
    const extensions: TemplateExtension[] = [
        {
            name: "uppercase_words",
            type: "filter",
            config_schema: {
                type: "object",
                properties: {}
            },
            description: "Capitalize first letter of each word",
            category: "text",
            arguments: [["input", "String to transform"]]
        },
        {
            name: "extract_domain",
            type: "filter",
            config_schema: {
                type: "object",
                properties: {}
            },
            description: "Extract domain from email or URL",
            category: "text",
            arguments: [["input", "Email address or URL"]]
        },
        {
            name: "format_date",
            type: "function",
            config_schema: {
                type: "object",
                properties: {
                    format: {
                        type: "string",
                        default: "YYYY-MM-DD",
                        description: "Date format string"
                    }
                }
            },
            description: "Format current date/time",
            category: "date",
            arguments: [["format", "Date format (optional)"]]
        },
        {
            name: "uuid_v4",
            type: "function",
            config_schema: {
                type: "object",
                properties: {}
            },
            description: "Generate a random UUID v4",
            category: "utility",
            arguments: []
        }
    ];

    return JSON.stringify(extensions);
}

export function execute_template_filter(
    name: string,
    _configJson: string,
    valueJson: string,
    _argsJson: string
): string {
    try {
        switch (name) {
            case "uppercase_words":
                return execute_uppercase_words_filter(valueJson);
            case "extract_domain":
                return execute_extract_domain_filter(valueJson);
            default:
                return JSON.stringify({ error: `Unknown filter: ${name}` });
        }
    } catch (error) {
        return JSON.stringify({ error: `Filter execution failed: ${(error as Error).message}` });
    }
}

export function execute_template_function(
    name: string,
    configJson: string,
    argsJson: string
): string {
    try {
        switch (name) {
            case "format_date":
                return execute_format_date_function(configJson, argsJson);
            case "uuid_v4":
                return execute_uuid_v4_function();
            default:
                return JSON.stringify({ error: `Unknown function: ${name}` });
        }
    } catch (error) {
        return JSON.stringify({ error: `Function execution failed: ${(error as Error).message}` });
    }
}

function execute_uppercase_words_filter(valueJson: string): string {
    const value = JSON.parse(valueJson);

    if (typeof value !== 'string') {
        return JSON.stringify({ error: "Filter input must be a string" });
    }

    const result = value.replace(/\b\w/g, (char) => char.toUpperCase());
    return JSON.stringify(result);
}

function execute_extract_domain_filter(valueJson: string): string {
    const value = JSON.parse(valueJson);

    if (typeof value !== 'string') {
        return JSON.stringify({ error: "Filter input must be a string" });
    }

    // Simple domain extraction - in real implementation, use proper URL parsing
    const emailMatch = value.match(/@([^@]+)$/);
    if (emailMatch) {
        return JSON.stringify(emailMatch[1]);
    }

    try {
        const url = new URL(value.startsWith('http') ? value : `http://${value}`);
        return JSON.stringify(url.hostname);
    } catch {
        return JSON.stringify({ error: "Invalid email or URL format" });
    }
}

function execute_format_date_function(configJson: string, argsJson: string): string {
    const config = JSON.parse(configJson);
    const args = JSON.parse(argsJson);

    const format = args[0] || config.format || "YYYY-MM-DD";
    const now = new Date();

    // Simple date formatting - in real implementation, use a proper date library
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    const hours = String(now.getHours()).padStart(2, '0');
    const minutes = String(now.getMinutes()).padStart(2, '0');
    const seconds = String(now.getSeconds()).padStart(2, '0');

    const formatted = format
        .replace('YYYY', year.toString())
        .replace('MM', month)
        .replace('DD', day)
        .replace('HH', hours)
        .replace('mm', minutes)
        .replace('ss', seconds);

    return JSON.stringify(formatted);
}

function execute_uuid_v4_function(): string {
    // Simple UUID v4 generation - in real implementation, use crypto.randomUUID() if available
    const randomBytes = new Uint8Array(16);
    for (let i = 0; i < 16; i++) {
        randomBytes[i] = Math.floor(Math.random() * 256);
    }

    // Set version (4) and variant bits
    randomBytes[6] = (randomBytes[6] & 0x0f) | 0x40;
    randomBytes[8] = (randomBytes[8] & 0x3f) | 0x80;

    const hex = Array.from(randomBytes, byte => byte.toString(16).padStart(2, '0')).join('');
    const uuid = `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;

    return JSON.stringify(uuid);
}

// Other required plugin functions (return empty arrays)
export function get_task_definitions(): string {
    return "[]";
}

export function get_facts_collectors(): string {
    return "[]";
}

export function get_log_sources(): string {
    return "[]";
}

export function get_log_parsers(): string {
    return "[]";
}

export function get_log_filters(): string {
    return "[]";
}

export function get_log_outputs(): string {
    return "[]";
}

// For browser/WASM environment
if (typeof window !== 'undefined') {
    (window as any).get_template_extensions = get_template_extensions;
    (window as any).execute_template_filter = execute_template_filter;
    (window as any).execute_template_function = execute_template_function;
    (window as any).get_task_definitions = get_task_definitions;
    (window as any).get_facts_collectors = get_facts_collectors;
    (window as any).get_log_sources = get_log_sources;
    (window as any).get_log_parsers = get_log_parsers;
    (window as any).get_log_filters = get_log_filters;
    (window as any).get_log_outputs = get_log_outputs;
}