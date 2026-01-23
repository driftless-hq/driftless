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

### List Operations

#### `batch`

Batch items in a list into groups of a specified size

**Arguments**:

- `size: integer - Size of each batch`
- `fill_with: any (optional) - Value to fill incomplete batches`

**Usage**:

```jinja2
{{ value | batch(size, fill_with) }}
```
#### `first`

Get the first item from a list

**Usage**:

```jinja2
{{ value | first }}
```

#### `join`

Join a list of strings with a separator

**Arguments**:

- `separator: string (optional) - String to join with (default: empty string)`

**Usage**:

```jinja2
{{ value | join(separator) }}
```
#### `last`

Get the last item from a list

**Usage**:

```jinja2
{{ value | last }}
```

#### `reverse`

Reverse the order of items in a list

**Usage**:

```jinja2
{{ value | reverse }}
```

#### `sort`

Sort items in a list

**Arguments**:

- `reverse: boolean (optional) - Sort in reverse order (default: false)`
- `case_sensitive: boolean (optional) - Case sensitive sorting for strings (default: true)`

**Usage**:

```jinja2
{{ value | sort(reverse, case_sensitive) }}
```
#### `unique`

Remove duplicate items from a list

**Usage**:

```jinja2
{{ value | unique }}
```

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

#### `center`

Center a string in a field of given width

**Arguments**:

- `width: integer - Width of the field`
- `fillchar: string (optional) - Character to fill with (default: space)`

**Usage**:

```jinja2
{{ value | center(width, fillchar) }}
```
#### `indent`

Indent each line of a string

**Arguments**:

- `width: integer - Number of spaces to indent`
- `indentfirst: boolean (optional) - Whether to indent the first line (default: false)`

**Usage**:

```jinja2
{{ value | indent(width, indentfirst) }}
```
#### `ljust`

Left-justify a string in a field of given width

**Arguments**:

- `width: integer - Width of the field`
- `fillchar: string (optional) - Character to fill with (default: space)`

**Usage**:

```jinja2
{{ value | ljust(width, fillchar) }}
```
#### `lower`

Convert a string to lowercase

**Usage**:

```jinja2
{{ value | lower }}
```

#### `lstrip`

Remove leading whitespace from a string

**Usage**:

```jinja2
{{ value | lstrip }}
```

#### `rjust`

Right-justify a string in a field of given width

**Arguments**:

- `width: integer - Width of the field`
- `fillchar: string (optional) - Character to fill with (default: space)`

**Usage**:

```jinja2
{{ value | rjust(width, fillchar) }}
```
#### `rstrip`

Remove trailing whitespace from a string

**Usage**:

```jinja2
{{ value | rstrip }}
```

#### `splitlines`

Split a string into a list of lines

**Usage**:

```jinja2
{{ value | splitlines }}
```

#### `title`

Convert a string to title case

**Usage**:

```jinja2
{{ value | title }}
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

#### `wordcount`

Count the number of words in a string

**Usage**:

```jinja2
{{ value | wordcount }}
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

