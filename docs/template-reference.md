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

### Encoding/Decoding

#### `b64decode`

Decode a base64 encoded string.

**Usage**:

```jinja2
{{ value | b64decode }}
```

#### `b64encode`

Encode a string using base64 encoding.

**Usage**:

```jinja2
{{ value | b64encode }}
```

#### `from_json`

Parse a JSON string into a value.

**Usage**:

```jinja2
{{ value | from_json }}
```

#### `from_yaml`

Parse a YAML string into a value.

**Usage**:

```jinja2
{{ value | from_yaml }}
```

#### `to_json`

Serialize a value to JSON string.

**Arguments**:

- `indent`: Number of spaces for indentation (optional)

**Usage**:

```jinja2
{{ value | to_json(indent) }}
```
#### `to_yaml`

Serialize a value to YAML string.

**Usage**:

```jinja2
{{ value | to_yaml }}
```

### List Operations

#### `batch`

Batch items in a list into groups of a specified size

**Arguments**:

- `size` (integer): Size of each batch
- `fill_with` (any): Value to fill incomplete batches (optional)

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

- `separator` (string): String to join with (optional, default: empty string)

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

- `reverse` (boolean): Sort in reverse order (optional, default: false)
- `case_sensitive` (boolean): Case sensitive sorting for strings (optional, default: true)

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

### List/Dict Operations

#### `combine`

Combine multiple dictionaries into one. Later dictionaries override earlier ones.

**Arguments**:

- `dictionaries`: Additional dictionaries to combine

**Usage**:

```jinja2
{{ value | combine(dictionaries) }}
```
#### `dict2items`

Convert a dictionary to a list of items with 'key' and 'value' fields.

**Usage**:

```jinja2
{{ value | dict2items }}
```

#### `flatten`

Flatten a nested list structure.

**Usage**:

```jinja2
{{ value | flatten }}
```

#### `items2dict`

Convert a list of items with 'key' and 'value' fields back to a dictionary.

**Usage**:

```jinja2
{{ value | items2dict }}
```

#### `map`

Apply an attribute or filter to each item in a list.

**Arguments**:

- `attribute`: Attribute name or filter to apply

**Usage**:

```jinja2
{{ value | map(attribute) }}
```
#### `reject`

Reject items from a list that match a test.

**Arguments**:

- `test`: Test to apply (currently supports 'defined' and 'truthy')

**Usage**:

```jinja2
{{ value | reject(test) }}
```
#### `select`

Select items from a list that match a test.

**Arguments**:

- `test`: Test to apply (currently supports 'defined' and 'truthy')

**Usage**:

```jinja2
{{ value | select(test) }}
```
#### `zip`

Zip multiple lists together into a list of tuples.

**Arguments**:

- `lists`: Additional lists to zip with

**Usage**:

```jinja2
{{ value | zip(lists) }}
```
### Math/Logic Operations

#### `abs`

Return the absolute value of a number

**Usage**:

```jinja2
{{ value | abs }}
```

#### `bool`

Convert value to boolean

**Usage**:

```jinja2
{{ value | bool }}
```

#### `random`

Return a random number, optionally within a specified range

**Arguments**:

- `start` (integer): The starting value of the range (optional)
- `end` (integer): The ending value of the range (optional)

**Usage**:

```jinja2
{{ value | random(start, end) }}
```
#### `round`

Round a number to a given precision (default 0 decimal places)

**Arguments**:

- `precision` (integer): The number of decimal places to round to (optional, default: 0)

**Usage**:

```jinja2
{{ value | round(precision) }}
```
#### `ternary`

Return one of two values based on condition (true_val if condition is true, false_val if false)

**Arguments**:

- `true_val` (any): The value to return if the condition is true
- `false_val` (any): The value to return if the condition is false

**Usage**:

```jinja2
{{ value | ternary(true_val, false_val) }}
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

#### `expanduser`

Expand a path containing a tilde (~) to the user's home directory.

**Usage**:

```jinja2
{{ value | expanduser }}
```

#### `realpath`

Return the canonical absolute path, resolving symlinks and relative components.

**Usage**:

```jinja2
{{ value | realpath }}
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

- `width` (integer): Width of the field
- `fillchar` (string): Character to fill with (optional, default: space)

**Usage**:

```jinja2
{{ value | center(width, fillchar) }}
```
#### `indent`

Indent each line of a string

**Arguments**:

- `width` (integer): Number of spaces to indent (optional, default: 0)
- `indentfirst` (boolean): Whether to indent the first line (optional, default: false)

**Usage**:

```jinja2
{{ value | indent(width, indentfirst) }}
```
#### `ljust`

Left-justify a string in a field of given width

**Arguments**:

- `width` (integer): Width of the field
- `fillchar` (string): Character to fill with (optional, default: space)

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

- `width` (integer): Width of the field
- `fillchar` (string): Character to fill with (optional, default: space)

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

- `length` (integer): Maximum length of the resulting string
- `killwords` (boolean): If true, truncate at character boundary; if false, try to truncate at word boundary (optional, default: false)
- `end` (string): String to append when truncation occurs (optional, default: "...")

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

### Generator Functions

#### `random`

Generate random numbers.

**Arguments**:

- `max` (int): The maximum value (exclusive) or minimum value if second arg provided
- `max` (int): The maximum value (exclusive)

**Usage**:

```jinja2
{{ random(max, max) }}
```
#### `range`

Generate a sequence of numbers.

**Arguments**:

- `end_or_start` (int): The end value (exclusive) for single arg, or start value for multiple args
- `end` (int): The end value (exclusive)
- `step` (int): The step value (optional, defaults to 1)

**Usage**:

```jinja2
{{ range(end_or_start, end, step) }}
```
### Lookup Functions

#### `lookup`

Look up values from various sources (env, file, etc.)

**Arguments**:

- `type` (string): The lookup type (currently only 'env' is supported)
- `key` (string): The key to look up

**Usage**:

```jinja2
{{ lookup('env', 'HOME') }}
{{ lookup('env', 'USER') }}
```
### Path Operations

#### `basename`

Return the basename of a path

**Arguments**:

- `path` (string): The path to extract the basename from

**Usage**:

```jinja2
{{ basename('/path/to/file.txt') }}
{{ basename(path_variable) }}
```
#### `dirname`

Return the directory name of a path

**Arguments**:

- `path` (string): The path to extract the directory name from

**Usage**:

```jinja2
{{ dirname('/path/to/file.txt') }}
{{ dirname(path_variable) }}
```
### Utility Functions

#### `length`

Return the length of a string, array, or object

**Arguments**:

- `value` (any): The value to get the length of (string, array, or object)

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

