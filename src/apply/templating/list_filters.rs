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
        vec![(
            "test".to_string(),
            "Test to apply (currently supports 'defined' and 'truthy')".to_string(),
        )],
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
        vec![(
            "test".to_string(),
            "Test to apply (currently supports 'defined' and 'truthy')".to_string(),
        )],
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
                        _ => false, // Default to exclude none
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
}
