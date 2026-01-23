# Driftless Template Reference

Comprehensive reference for all available Jinja2 template filters and functions in Driftless.

This documentation is auto-generated from the Rust source code.

## Overview

Driftless uses Jinja2 templating for dynamic configuration values. Templates support both filters (applied with `|` syntax) and functions (called directly).

### Template Syntax

```jinja2
{{ variable | filter_name(arg1, arg2) }}
{{ function_name(arg1, arg2) }}
```

## Template Filters

Filters transform values in templates using the `|` syntax.

### Path Operations

#### `basename`

Return the basename of a path

**Usage**:

```jinja2
{{ value | basename }}
```

#### `dirname`

Return the directory name of a path

**Usage**:

```jinja2
{{ value | dirname }}
```

### String Operations

#### `capitalize`

Capitalize the first character of a string

**Usage**:

```jinja2
{{ value | capitalize }}
```

#### `lower`

Convert a string to lowercase

**Usage**:

```jinja2
{{ value | lower }}
```

#### `truncate`

Truncate a string to a specified length

**Arguments**:

- `length: integer - Maximum length of the resulting string`
- `killwords: boolean (optional) - If true, truncate at character boundary; if false, try to truncate at word boundary`
- `end: string (optional) - String to append when truncation occurs (default: "...")`

**Usage**:

```jinja2
{{ value | truncate(50) }}
{{ value | truncate(20, "...") }}
{{ value | truncate(30, true, "[truncated]") }}
```
#### `upper`

Convert a string to uppercase

**Usage**:

```jinja2
{{ value | upper }}
```

### String/List Operations

#### `length`

Return the length of a string, list, or object

**Usage**:

```jinja2
{{ value | length }}
```

## Template Functions

Functions perform operations and return values in templates.

### Lookup Functions

#### `lookup`

Look up values from various sources (env, file, etc.)

**Arguments**:

- `type: string - The lookup type (currently only 'env' is supported)`
- `key: string - The key to look up`

**Usage**:

```jinja2
{{ lookup('env', 'HOME') }}
{{ lookup('env', 'USER') }}
```
### Path Operations

#### `basename`

Return the basename of a path

**Arguments**:

- `path: string - The path to extract the basename from`

**Usage**:

```jinja2
{{ basename('/path/to/file.txt') }}
{{ basename(path_variable) }}
```
#### `dirname`

Return the directory name of a path

**Arguments**:

- `path: string - The path to extract the directory name from`

**Usage**:

```jinja2
{{ dirname('/path/to/file.txt') }}
{{ dirname(path_variable) }}
```
### Utility Functions

#### `length`

Return the length of a string, array, or object

**Arguments**:

- `value: any - The value to get the length of (string, array, or object)`

**Usage**:

```jinja2
{{ length('hello') }}
{{ length(items) }}
{{ length(my_object) }}
```
## Examples

```yaml
# Using filters
path: "/home/{{ username | lower }}"
config: "{{ app_name | upper }}.conf"
truncated: "{{ long_text | truncate(50) }}"

# Using functions
length: "{{ length(items) }}"
basename: "{{ basename('/path/to/file.txt') }}"
env_var: "{{ lookup('env', 'HOME') }}"
```

