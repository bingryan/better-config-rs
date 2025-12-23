use better_config_core::{merge_with_env_uppercase, AbstractConfig, Error, misc};
use std::collections::{HashMap, HashSet};
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
        Self::load_with_override(target, &HashSet::new())
    }

    /// Load specified YAML/YML file with explicit control over which keys should not be overridden.
    ///
    /// # Arguments
    /// * `target` - Path to the YAML/YML file.
    /// * `excluded_keys` - Keys that should not be overridden by environment variables.
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If the specified YAML/YML file cannot be loaded or parsed.
    fn load_with_override(target: Option<String>, excluded_keys: &HashSet<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        let target = target.or(Some("config.yml".to_string()));

        let mut yaml_map = HashMap::new();

        if let Some(target) = target {
            let file_paths = misc::validate_and_split_paths(&target)?;

            for file_path in file_paths {
                // Check file accessibility before reading
                misc::check_file_accessibility(&file_path)?;

                let content = fs::read_to_string(&file_path)
                    .map_err(|e| Error::IoError {
                        operation: format!("read file '{}'", file_path),
                        source: Some(Box::new(e)),
                    })?;

                let value: serde_yml::Value = serde_yml::from_str(&content)
                    .map_err(|e| Error::parse_yaml_error(&file_path, e))?;

                flatten_yml_value(&value, None, &mut yaml_map)
                    .map_err(|e| Error::value_conversion_error("yaml", "string", &format!("{}", e)))?;
            }
        }

        // Apply environment variable override with excluded keys
        let yaml_map = merge_with_env_uppercase(yaml_map, None, excluded_keys);

        Ok(yaml_map.into())
    }
}

fn flatten_yml_value(value: &serde_yml::Value, parent_key: Option<String>, map: &mut HashMap<String, String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match value {
        serde_yml::Value::Mapping(obj) => {
            for (key, val) in obj {
                let key_str = match key {
                    serde_yml::Value::String(s) => s.clone(),
                    _ => serde_yml::to_string(key).unwrap_or_else(|_| "unknown".to_string()),
                };
                let new_key = match &parent_key {
                    Some(parent) => format!("{}.{}", parent, key_str),
                    None => key_str,
                };
                flatten_yml_value(val, Some(new_key), map)?;
            }
        }
        serde_yml::Value::Sequence(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_key = match &parent_key {
                    Some(parent) => format!("{}[{}]", parent, i),
                    None => i.to_string(),
                };
                flatten_yml_value(val, Some(new_key), map)?;
            }
        }
        serde_yml::Value::String(s) => {
            if let Some(key) = parent_key {
                map.insert(key, s.to_string());
            }
        }
        serde_yml::Value::Number(n) => {
            if let Some(key) = parent_key {
                let num_str = if n.is_i64() {
                    n.as_i64()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| n.to_string())
                } else if n.is_f64() {
                    n.as_f64()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| n.to_string())
                } else {
                    n.to_string()
                };
                map.insert(key, num_str);
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

    Ok(())
}