//! List and dictionary manipulation filters for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use minijinja::Value as JinjaValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Register list and dictionary manipulation filters
pub fn register_list_filters(
    registry: &mut std::collections::HashMap<String, crate::apply::templating::TemplateFilterEntry>,
) {
    // combine filter
    TemplateRegistry::register_filter(
        registry,
        "combine",
        "Combine multiple dictionaries into one. Later dictionaries override earlier ones.",
        "List/Dict Operations",
        vec![(
            "dictionaries".to_string(),
            "Additional dictionaries to combine".to_string(),
        )],
        Arc::new(|value, args| {
            let mut result = HashMap::new();

            // Function to add a dict to result
            let mut add_dict = |val: &JinjaValue| {
                if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(val) {
                    for (k, v) in map {
                        let jinja_val = JinjaValue::from_serialize(&v);
                        result.insert(k, jinja_val);
                    }
                }
            };

            // Add base value
            add_dict(&value);

            // Add additional args
            for arg in args {
                add_dict(arg);
            }

            JinjaValue::from(result)
        }),
    );

    // dict2items filter
    TemplateRegistry::register_filter(
        registry,
        "dict2items",
        "Convert a dictionary to a list of items with 'key' and 'value' fields.",
        "List/Dict Operations",
        vec![],
        Arc::new(|value, _args| {
            if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(&value) {
                let mut items = Vec::new();
                for (k, v) in map {
                    let val = JinjaValue::from_serialize(&v);
                    let mut item = HashMap::new();
                    item.insert("key".to_string(), JinjaValue::from(k));
                    item.insert("value".to_string(), val);
                    items.push(JinjaValue::from(item));
                }
                JinjaValue::from(items)
            } else {
                JinjaValue::from(Vec::<JinjaValue>::new())
            }
        }),
    );

    // items2dict filter
    TemplateRegistry::register_filter(
        registry,
        "items2dict",
        "Convert a list of items with 'key' and 'value' fields back to a dictionary.",
        "List/Dict Operations",
        vec![],
        Arc::new(|value, _args| {
            if let Ok(iter) = value.try_iter() {
                let mut result = HashMap::new();
                for item in iter {
                    if let Ok(serde_json::Value::Object(item_map)) = serde_json::to_value(&item) {
                        if let Some(serde_json::Value::String(key)) = item_map.get("key") {
                            if let Some(v) = item_map.get("value") {
                                let val = JinjaValue::from_serialize(v);
                                result.insert(key.clone(), val);
                            }
                        }
                    }
                }
                JinjaValue::from(result)
            } else {
                JinjaValue::from(HashMap::<String, JinjaValue>::new())
            }
        }),
    );

    // flatten filter
    TemplateRegistry::register_filter(
        registry,
        "flatten",
        "Flatten a nested list structure.",
        "List/Dict Operations",
        vec![],
        Arc::new(|value, _args| {
            fn flatten_recursive(val: &JinjaValue, result: &mut Vec<JinjaValue>) {
                if let Ok(iter) = val.try_iter() {
                    for item in iter {
                        flatten_recursive(&item, result);
                    }
                } else {
                    result.push(val.clone());
                }
            }

            let mut result = Vec::new();
            flatten_recursive(&value, &mut result);
            JinjaValue::from(result)
        }),
    );

    // map filter
    TemplateRegistry::register_filter(
        registry,
        "map",
        "Apply an attribute or filter to each item in a list.",
        "List/Dict Operations",
        vec![(
            "attribute".to_string(),
            "Attribute name or filter to apply".to_string(),
        )],
        Arc::new(|value, args| {
            if args.is_empty() {
                return value.clone();
            }

            let attribute = args[0].as_str().unwrap_or("");
            if attribute.is_empty() {
                return value.clone();
            }

            if let Ok(iter) = value.try_iter() {
                let mut result = Vec::new();
                for item in iter {
                    if let Ok(serde_json::Value::Object(item_map)) = serde_json::to_value(&item) {
                        if let Some(v) = item_map.get(attribute) {
                            let val = JinjaValue::from_serialize(v);
                            result.push(val);
                        } else {
                            result.push(JinjaValue::from(()));
                        }
                    } else {
                        result.push(item.clone());
                    }
                }
                JinjaValue::from(result)
            } else {
                value.clone()
            }
        }),
    );

    // select filter
    TemplateRegistry::register_filter(
        registry,
        "select",
        "Select items from a list that match a test.",
        "List/Dict Operations",
        vec![
            (
                "test".to_string(),
                "Test to apply (supports: defined, truthy, undefined, none, falsy, equalto, match, search, version_compare)".to_string(),
            ),
            (
                "arg".to_string(),
                "Optional argument for tests that require it (e.g., value for equalto, regex for match)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            if args.is_empty() {
                return value.clone();
            }

            let test = args[0].as_str().unwrap_or("");

            if let Ok(iter) = value.try_iter() {
                let mut result = Vec::new();
                for item in iter {
                    let include = match test {
                        "defined" => !item.is_undefined() && !item.is_none(),
                        "truthy" => {
                            let is_falsy = item.is_undefined()
                                || item == JinjaValue::from(false)
                                || item == JinjaValue::from(0)
                                || item == JinjaValue::from(0.0)
                                || item.as_str().map(|s| s.is_empty()).unwrap_or(false)
                                || item
                                    .try_iter()
                                    .map(|mut i| i.next().is_none())
                                    .unwrap_or(false);
                            !is_falsy
                        }
                        "undefined" => item.is_undefined(),
                        "none" => item.is_none(),
                        "falsy" => {
                            item.is_undefined()
                                || item == JinjaValue::from(false)
                                || item == JinjaValue::from(0)
                                || item == JinjaValue::from(0.0)
                                || item.as_str().map(|s| s.is_empty()).unwrap_or(false)
                                || item
                                    .try_iter()
                                    .map(|mut i| i.next().is_none())
                                    .unwrap_or(false)
                        }
                        "equalto" => {
                            if args.len() > 1 {
                                item == args[1]
                            } else {
                                false
                            }
                        }
                        "match" => {
                            if args.len() > 1 {
                                if let (Some(item_str), Some(pattern)) = (item.as_str(), args[1].as_str()) {
                                    regex::Regex::new(pattern).map(|re| re.is_match(item_str)).unwrap_or(false)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        "search" => {
                            if args.len() > 1 {
                                if let (Some(item_str), Some(pattern)) = (item.as_str(), args[1].as_str()) {
                                    regex::Regex::new(pattern).map(|re| re.find(item_str).is_some()).unwrap_or(false)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        "version_compare" => {
                            // Simple version comparison - assumes semantic versions
                            if args.len() > 1 {
                                if let (Some(item_str), Some(compare_str)) = (item.as_str(), args[1].as_str()) {
                                    // For simplicity, just compare as strings for now
                                    // A full implementation would parse versions
                                    item_str == compare_str
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        _ => true, // Default to include all
                    };
                    if include {
                        result.push(item);
                    }
                }
                JinjaValue::from(result)
            } else {
                value.clone()
            }
        }),
    );

    // reject filter
    TemplateRegistry::register_filter(
        registry,
        "reject",
        "Reject items from a list that match a test.",
        "List/Dict Operations",
        vec![
            (
                "test".to_string(),
                "Test to apply (supports: defined, truthy, undefined, none, falsy, equalto, match, search, version_compare)".to_string(),
            ),
            (
                "arg".to_string(),
                "Optional argument for tests that require it (e.g., value for equalto, regex for match)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            if args.is_empty() {
                return value.clone();
            }

            let test = args[0].as_str().unwrap_or("");

            if let Ok(iter) = value.try_iter() {
                let mut result = Vec::new();
                for item in iter {
                    let exclude = match test {
                        "defined" => !item.is_undefined() && !item.is_none(),
                        "truthy" => {
                            let is_falsy = item.is_undefined()
                                || item == JinjaValue::from(false)
                                || item == JinjaValue::from(0)
                                || item == JinjaValue::from(0.0)
                                || item.as_str().map(|s| s.is_empty()).unwrap_or(false)
                                || item
                                    .try_iter()
                                    .map(|mut i| i.next().is_none())
                                    .unwrap_or(false);
                            !is_falsy
                        }
                        "undefined" => item.is_undefined(),
                        "none" => item.is_none(),
                        "falsy" => {
                            item.is_undefined()
                                || item == JinjaValue::from(false)
                                || item == JinjaValue::from(0)
                                || item == JinjaValue::from(0.0)
                                || item.as_str().map(|s| s.is_empty()).unwrap_or(false)
                                || item
                                    .try_iter()
                                    .map(|mut i| i.next().is_none())
                                    .unwrap_or(false)
                        }
                        "equalto" => {
                            if args.len() > 1 {
                                item == args[1]
                            } else {
                                false
                            }
                        }
                        "match" => {
                            if args.len() > 1 {
                                if let (Some(item_str), Some(pattern)) = (item.as_str(), args[1].as_str()) {
                                    regex::Regex::new(pattern).map(|re| re.is_match(item_str)).unwrap_or(false)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        "search" => {
                            if args.len() > 1 {
                                if let (Some(item_str), Some(pattern)) = (item.as_str(), args[1].as_str()) {
                                    regex::Regex::new(pattern).map(|re| re.find(item_str).is_some()).unwrap_or(false)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        "version_compare" => {
                            // Simple version comparison - assumes semantic versions
                            if args.len() > 1 {
                                if let (Some(item_str), Some(compare_str)) = (item.as_str(), args[1].as_str()) {
                                    // For simplicity, just compare as strings for now
                                    // A full implementation would parse versions
                                    item_str == compare_str
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        _ => false, // Default to exclude nothing
                    };
                    if !exclude {
                        result.push(item);
                    }
                }
                JinjaValue::from(result)
            } else {
                value.clone()
            }
        }),
    );

    // zip filter
    TemplateRegistry::register_filter(
        registry,
        "zip",
        "Zip multiple lists together into a list of tuples.",
        "List/Dict Operations",
        vec![(
            "lists".to_string(),
            "Additional lists to zip with".to_string(),
        )],
        Arc::new(|value, args| {
            let mut lists = vec![value];
            lists.extend_from_slice(args);

            // Convert all to sequences
            let sequences: Vec<Vec<JinjaValue>> = lists
                .iter()
                .filter_map(|v| {
                    if let Ok(iter) = v.try_iter() {
                        Some(iter.collect())
                    } else {
                        None
                    }
                })
                .collect();

            if sequences.is_empty() {
                return JinjaValue::from(Vec::<JinjaValue>::new());
            }

            let min_len = sequences.iter().map(|s| s.len()).min().unwrap_or(0);
            let mut result = Vec::new();

            for i in 0..min_len {
                let mut tuple = Vec::new();
                for seq in &sequences {
                    tuple.push(seq[i].clone());
                }
                result.push(JinjaValue::from(tuple));
            }

            JinjaValue::from(result)
        }),
    );

    // dictsort filter
    TemplateRegistry::register_filter(
        registry,
        "dictsort",
        "Sort a dictionary by keys or values",
        "List/Dict Operations",
        vec![
            (
                "case_sensitive".to_string(),
                "boolean: Whether sorting is case sensitive (optional, default: false)".to_string(),
            ),
            (
                "by".to_string(),
                "string: Sort by 'key' or 'value' (optional, default: 'key')".to_string(),
            ),
            (
                "reverse".to_string(),
                "boolean: Reverse the sort order (optional, default: false)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let case_sensitive = args.first().map(|v| v.is_true()).unwrap_or(false);
            let by = args.get(1).and_then(|v| v.as_str()).unwrap_or("key");
            let reverse = args.get(2).map(|v| v.is_true()).unwrap_or(false);

            if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(&value) {
                let mut items: Vec<_> = map.into_iter().collect();

                items.sort_by(|a, b| {
                    let cmp = match by {
                        "value" => {
                            let a_val = &a.1;
                            let b_val = &b.1;
                            match (a_val, b_val) {
                                (
                                    serde_json::Value::String(ref a_str),
                                    serde_json::Value::String(ref b_str),
                                ) => {
                                    if case_sensitive {
                                        a_str.cmp(b_str)
                                    } else {
                                        a_str.to_lowercase().cmp(&b_str.to_lowercase())
                                    }
                                }
                                _ => a_val.to_string().cmp(&b_val.to_string()),
                            }
                        }
                        _ => {
                            // "key"
                            let a_key = &a.0;
                            let b_key = &b.0;
                            if case_sensitive {
                                a_key.cmp(b_key)
                            } else {
                                a_key.to_lowercase().cmp(&b_key.to_lowercase())
                            }
                        }
                    };
                    if reverse {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });

                let sorted_map: std::collections::HashMap<String, serde_json::Value> =
                    items.into_iter().collect();
                JinjaValue::from_serialize(&sorted_map)
            } else {
                // Not a dict, return as is
                value
            }
        }),
    );

    // slice filter
    TemplateRegistry::register_filter(
        registry,
        "slice",
        "Slice a list into sublists of a specified size",
        "List/Dict Operations",
        vec![(
            "size".to_string(),
            "integer: Size of each slice".to_string(),
        )],
        Arc::new(|value, args| {
            let size = args.first().and_then(|v| v.as_i64()).unwrap_or(1) as usize;
            if size == 0 {
                return JinjaValue::from(Vec::<JinjaValue>::new());
            }

            if let Ok(seq) = value.try_iter() {
                let items: Vec<JinjaValue> = seq.collect();
                let mut result = Vec::new();
                for chunk in items.chunks(size) {
                    result.push(JinjaValue::from(chunk.to_vec()));
                }
                JinjaValue::from(result)
            } else {
                // Not a sequence, return empty list
                JinjaValue::from(Vec::<JinjaValue>::new())
            }
        }),
    );
}
