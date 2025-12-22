use better_config_core::override_env::merge_with_env_uppercase;
use better_config_core::{AbstractConfig, Error};
use better_config_core::misc;
use std::collections::{HashMap, HashSet};
use std::fs;
use toml::Value;

/// Indicates that structure can be initialized from TOML file.
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
        Self::load_with_override(target, &HashSet::new())
    }

    /// Load specified TOML files with explicit control over which keys should not be overridden.
    ///
    /// # Arguments
    /// * `target` - A comma-separated string of TOML file paths, e.g., "config.toml,local.toml".
    /// * `excluded_keys` - Keys that should not be overridden by environment variables.
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If any of the specified TOML files cannot be loaded or parsed.
    fn load_with_override(target: Option<String>, excluded_keys: &HashSet<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        let target = target.or(Some("config.toml".to_string()));

        let mut toml_map = HashMap::new();

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

                let value: Value = toml::from_str(&content)
                    .map_err(|e| Error::parse_toml_error(&file_path, e))?;

                if let Some(table) = value.as_table() {
                    flatten_table(table, None, &mut toml_map)
                        .map_err(|e| Error::value_conversion_error("toml", "string", &format!("{}", e)))?;
                }
            }
        }

        // Apply environment variable override with excluded keys
        let toml_map = merge_with_env_uppercase(toml_map, None, excluded_keys);

        Ok(toml_map.into())
    }
}

fn flatten_table(
    table: &toml::value::Table,
    prefix: Option<&str>,
    map: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (key, value) in table {
        let full_key = match prefix {
            Some(p) => format!("{}.{}", p, key),
            None => key.clone(),
        };
        match value {
            toml::Value::Table(t) => flatten_table(t, Some(&full_key), map)?,
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

    Ok(())
}
