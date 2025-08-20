use better_config_core::{AbstractConfig, Error};
use better_config_core::misc;
use std::collections::HashMap;
use std::fs;

/// Indicates that structure can be initialized from JSON file.
pub trait JsonConfig<T = HashMap<String, String>>: AbstractConfig<T> {
    /// Load specified JSON file and initialize the structure.
    ///
    /// # Arguments
    /// * `target` - Path to the JSON file.
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If the specified JSON file cannot be loaded or parsed.
    fn load(target: Option<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        let target = target.or(Some("config.json".to_string()));

        let mut json_map = HashMap::new();

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

                let value: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| Error::parse_json_error(&file_path, e))?;

                flatten_json_value(&value, None, &mut json_map)
                    .map_err(|e| Error::value_conversion_error("json", "string", &format!("{}", e)))?;
            }
        }

        Ok(json_map.into())
    }
}

fn flatten_json_value(value: &serde_json::Value, parent_key: Option<String>, map: &mut HashMap<String, String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let new_key = match &parent_key {
                    Some(parent) => format!("{}.{}", parent, key),
                    None => key.to_string(),
                };
                flatten_json_value(val, Some(new_key), map)?;
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_key = match &parent_key {
                    Some(parent) => format!("{}[{}]", parent, i),
                    None => i.to_string(),
                };
                flatten_json_value(val, Some(new_key), map)?;
            }
        }
        serde_json::Value::String(s) => {
            if let Some(key) = parent_key {
                map.insert(key, s.to_string());
            }
        }
        serde_json::Value::Number(n) => {
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
        serde_json::Value::Bool(b) => {
            if let Some(key) = parent_key {
                map.insert(key, b.to_string());
            }
        }
        serde_json::Value::Null => {}
    }

    Ok(())
}