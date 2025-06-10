use better_config_core::{AbstractConfig, Error};
use std::collections::HashMap;
use std::fs;
use toml::Value;

/// Indicates that structure can be initialized from TOML files.
pub trait TomlConfig<T = HashMap<String, String>>: AbstractConfig<T> {
    /// Load specified TOML files and initialize the structure.
    ///
    /// # Arguments
    /// * `target` - A comma-separated string of TOML file paths, e.g., "config.toml,local.toml".
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If any of the specified TOML files cannot be loaded or parsed.
    fn load(target: Option<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        let target = target.or(Some("config.toml".to_string()));

        let mut toml_map = HashMap::new();

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

                let value: Value = toml::from_str(&content).map_err(|e| Error::LoadFileError {
                    name: file_path.clone(),
                    source: Some(Box::new(e)),
                })?;

                if let Some(table) = value.as_table() {
                    flatten_table(table, None, &mut toml_map);
                }
            }
        }

        Ok(toml_map.into())
    }
}

fn flatten_table(
    table: &toml::value::Table,
    prefix: Option<&str>,
    map: &mut HashMap<String, String>,
) {
    for (key, value) in table {
        let full_key = match prefix {
            Some(p) => format!("{}.{}", p, key),
            None => key.clone(),
        };
        match value {
            toml::Value::Table(t) => flatten_table(t, Some(&full_key), map),
            toml::Value::Array(_arr) => {
                map.insert(full_key, value.to_string());
            }
            toml::Value::String(s) => {
                map.insert(full_key, s.clone());
            }
            _ => {
                map.insert(full_key, value.to_string());
            }
        }
    }
}
