use better_config_core::{AbstractConfig, Error};
use dotenvy::from_filename;
use std::collections::{HashMap, HashSet};

/// Indicates that structure can be initialize from environment variables.
pub trait EnvConfig<T = HashMap<String, String>>: AbstractConfig<T> {
    /// Load specified env files to environment variables and initialize the structure.
    ///
    /// # Arguments
    /// * `target` - A comma-separated string of env file paths, e.g., ".env,.env.local".
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If any of the specified env files cannot be loaded.
    fn load(target: Option<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        Self::load_with_override(target, &HashSet::new())
    }

    /// Load specified env files with explicit control over which keys should not be overridden.
    /// Note: For EnvConfig, excluded_keys has no effect since env vars are the source of truth.
    ///
    /// # Arguments
    /// * `target` - A comma-separated string of env file paths, e.g., ".env,.env.local".
    /// * `_excluded_keys` - Ignored for EnvConfig (env vars are already the override source).
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If any of the specified env files cannot be loaded.
    fn load_with_override(target: Option<String>, _excluded_keys: &HashSet<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        let target = target.or(Some(".env".to_string()));

        if let Some(target) = target {
            let file_paths: Vec<String> = target
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            for file_path in file_paths {
                if let Err(e) = from_filename(&file_path) {
                    return Err(Error::LoadFileError {
                        name: file_path,
                        source: Some(Box::new(e)),
                    });
                }
            }
        }

        let mut env_map = HashMap::new();
        for (key, value) in std::env::vars() {
            env_map.insert(key, value);
        }

        Ok(env_map.into())
    }
}
