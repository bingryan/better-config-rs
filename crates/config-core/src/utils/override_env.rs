use std::collections::{HashMap, HashSet};

/// Merge file-based configuration with environment variables.
/// Environment variables take precedence over file values.
///
/// # Arguments
/// * `file_config` - Configuration loaded from file
/// * `prefix` - Optional prefix for environment variable lookup
/// * `excluded_keys` - Keys that should not be overridden by env vars
///
/// # Returns
/// Merged configuration with env vars taking precedence
///
/// # Example
/// ```rust
/// use std::collections::{HashMap, HashSet};
/// use better_config_core::utils::merge_with_env;
///
/// let mut file_config = HashMap::new();
/// file_config.insert("DB_HOST".to_string(), "localhost".to_string());
/// file_config.insert("DB_PORT".to_string(), "5432".to_string());
///
/// // If env var DB_HOST is set to "production-db", it will override the file value
/// let merged = merge_with_env(file_config, None, &HashSet::new());
/// ```
pub fn merge_with_env(
    mut file_config: HashMap<String, String>,
    prefix: Option<&str>,
    excluded_keys: &HashSet<String>,
) -> HashMap<String, String> {
    for (key, value) in file_config.iter_mut() {
        // Skip keys that are excluded from env override
        if excluded_keys.contains(key) {
            continue;
        }

        // Build the environment variable key with optional prefix
        let env_key = match prefix {
            Some(p) => format!("{}{}", p, key),
            None => key.clone(),
        };

        // If env var exists, use its value instead
        if let Ok(env_value) = std::env::var(&env_key) {
            *value = env_value;
        }
    }

    file_config
}

/// Merge file-based configuration with environment variables using uppercase key conversion.
/// This variant converts keys to uppercase before looking up environment variables.
///
/// # Arguments
/// * `file_config` - Configuration loaded from file
/// * `prefix` - Optional prefix for environment variable lookup
/// * `excluded_keys` - Keys that should not be overridden by env vars
///
/// # Returns
/// Merged configuration with env vars taking precedence
pub fn merge_with_env_uppercase(
    mut file_config: HashMap<String, String>,
    prefix: Option<&str>,
    excluded_keys: &HashSet<String>,
) -> HashMap<String, String> {
    for (key, value) in file_config.iter_mut() {
        if excluded_keys.contains(key) {
            continue;
        }

        // Convert key to uppercase and replace dots with underscores for env var lookup
        let normalized_key = key.to_uppercase().replace('.', "_");
        let env_key = match prefix {
            Some(p) => format!("{}{}", p, normalized_key),
            None => normalized_key,
        };

        if let Ok(env_value) = std::env::var(&env_key) {
            *value = env_value;
        }
    }

    file_config
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn cleanup_env_vars() {
        env::remove_var("DB_HOST");
        env::remove_var("DB_PORT");
        env::remove_var("API_KEY");
        env::remove_var("APP_DB_HOST");
        env::remove_var("APP_DB_PORT");
        env::remove_var("DATABASE_HOST");
    }

    #[test]
    #[serial]
    fn test_merge_with_env_no_override() {
        cleanup_env_vars();

        let mut file_config = HashMap::new();
        file_config.insert("DB_HOST".to_string(), "localhost".to_string());
        file_config.insert("DB_PORT".to_string(), "5432".to_string());

        let result = merge_with_env(file_config, None, &HashSet::new());

        assert_eq!(result.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(result.get("DB_PORT"), Some(&"5432".to_string()));
    }

    #[test]
    #[serial]
    fn test_merge_with_env_override() {
        cleanup_env_vars();
        env::set_var("DB_HOST", "production-db");

        let mut file_config = HashMap::new();
        file_config.insert("DB_HOST".to_string(), "localhost".to_string());
        file_config.insert("DB_PORT".to_string(), "5432".to_string());

        let result = merge_with_env(file_config, None, &HashSet::new());

        assert_eq!(result.get("DB_HOST"), Some(&"production-db".to_string()));
        assert_eq!(result.get("DB_PORT"), Some(&"5432".to_string()));

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_merge_with_env_prefix() {
        cleanup_env_vars();
        env::set_var("APP_DB_HOST", "prefixed-host");

        let mut file_config = HashMap::new();
        file_config.insert("DB_HOST".to_string(), "localhost".to_string());
        file_config.insert("DB_PORT".to_string(), "5432".to_string());

        let result = merge_with_env(file_config, Some("APP_"), &HashSet::new());

        assert_eq!(result.get("DB_HOST"), Some(&"prefixed-host".to_string()));
        assert_eq!(result.get("DB_PORT"), Some(&"5432".to_string()));

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_merge_with_env_excluded_keys() {
        cleanup_env_vars();
        env::set_var("DB_HOST", "should-not-override");
        env::set_var("DB_PORT", "9999");

        let mut file_config = HashMap::new();
        file_config.insert("DB_HOST".to_string(), "localhost".to_string());
        file_config.insert("DB_PORT".to_string(), "5432".to_string());

        let mut excluded = HashSet::new();
        excluded.insert("DB_HOST".to_string());

        let result = merge_with_env(file_config, None, &excluded);

        // DB_HOST should NOT be overridden because it's excluded
        assert_eq!(result.get("DB_HOST"), Some(&"localhost".to_string()));
        // DB_PORT should be overridden
        assert_eq!(result.get("DB_PORT"), Some(&"9999".to_string()));

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_merge_with_env_prefix_and_excluded() {
        cleanup_env_vars();
        env::set_var("APP_DB_HOST", "prefixed-override");
        env::set_var("APP_API_KEY", "secret-key");

        let mut file_config = HashMap::new();
        file_config.insert("DB_HOST".to_string(), "localhost".to_string());
        file_config.insert("API_KEY".to_string(), "file-key".to_string());

        let mut excluded = HashSet::new();
        excluded.insert("API_KEY".to_string());

        let result = merge_with_env(file_config, Some("APP_"), &excluded);

        assert_eq!(
            result.get("DB_HOST"),
            Some(&"prefixed-override".to_string())
        );
        // API_KEY should NOT be overridden
        assert_eq!(result.get("API_KEY"), Some(&"file-key".to_string()));

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_merge_with_env_uppercase_dotted_keys() {
        cleanup_env_vars();
        env::set_var("DATABASE_HOST", "uppercase-host");

        let mut file_config = HashMap::new();
        file_config.insert("database.host".to_string(), "localhost".to_string());

        let result = merge_with_env_uppercase(file_config, None, &HashSet::new());

        assert_eq!(
            result.get("database.host"),
            Some(&"uppercase-host".to_string())
        );

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_merge_with_env_empty_config() {
        cleanup_env_vars();

        let file_config = HashMap::new();
        let result = merge_with_env(file_config, None, &HashSet::new());

        assert!(result.is_empty());
    }

    #[test]
    #[serial]
    fn test_merge_with_env_all_excluded() {
        cleanup_env_vars();
        env::set_var("DB_HOST", "should-not-override");
        env::set_var("DB_PORT", "9999");

        let mut file_config = HashMap::new();
        file_config.insert("DB_HOST".to_string(), "localhost".to_string());
        file_config.insert("DB_PORT".to_string(), "5432".to_string());

        let mut excluded = HashSet::new();
        excluded.insert("DB_HOST".to_string());
        excluded.insert("DB_PORT".to_string());

        let result = merge_with_env(file_config, None, &excluded);

        assert_eq!(result.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(result.get("DB_PORT"), Some(&"5432".to_string()));

        cleanup_env_vars();
    }
}
