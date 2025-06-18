use better_config_core::{AbstractConfig, Error};
use std::collections::HashMap;
use std::fs;

/// Indicates that structure can be initialized from YAML/YML file.
pub trait YmlConfig<T = HashMap<String, String>>: AbstractConfig<T> {
    /// Load specified YAML/YML file and initialize the structure.
    ///
    /// # Arguments
    /// * `target` - Path to the YAML/YML file.
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If the specified YAML/YML file cannot be loaded or parsed.
    fn load(target: Option<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        let target = target.or(Some("config.yml".to_string()));

        let mut yaml_map = HashMap::new();

        if let Some(target) = target {
            let file_paths: Vec<String> = target
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            for file_path in file_paths {
                let content = fs::read_to_string(&file_path).map_err(|e| Error::LoadFileError {
                    name: file_path.clone(),
                    source: Some(Box::new(e)),
                })?;

                let value: serde_yml::Value = serde_yml::from_str(&content).map_err(|e| Error::LoadFileError {
                    name: file_path.clone(),
                    source: Some(Box::new(e)),
                })?;

                flatten_yml_value(&value, None, &mut yaml_map);
            }
        }

        Ok(yaml_map.into())
    }
}

fn flatten_yml_value(value: &serde_yml::Value, parent_key: Option<String>, map: &mut HashMap<String, String>) {
    match value {
        serde_yml::Value::Mapping(obj) => {
            for (key, val) in obj {
                let key_str = match key {
                    serde_yml::Value::String(s) => s.clone(),
                    _ => serde_yml::to_string(key).unwrap_or_default(),
                };
                let new_key = match &parent_key {
                    Some(parent) => format!("{}.{}", parent, key_str),
                    None => key_str,
                };
                flatten_yml_value(val, Some(new_key), map);
            }
        }
        serde_yml::Value::Sequence(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_key = match &parent_key {
                    Some(parent) => format!("{}[{}]", parent, i),
                    None => i.to_string(),
                };
                flatten_yml_value(val, Some(new_key), map);
            }
        }
        serde_yml::Value::String(s) => {
            if let Some(key) = parent_key {
                map.insert(key, s.to_string());
            }
        }
        serde_yml::Value::Number(n) => {
            if let Some(key) = parent_key {
                if n.is_i64() {
                    map.insert(key, n.as_i64().unwrap().to_string());
                } else if n.is_f64() {
                    map.insert(key, n.as_f64().unwrap().to_string());
                } else {
                    map.insert(key, n.to_string());
                }
            }
        }
        serde_yml::Value::Bool(b) => {
            if let Some(key) = parent_key {
                map.insert(key, b.to_string());
            }
        }
        serde_yml::Value::Null => {}
        _ => {}
    }
}