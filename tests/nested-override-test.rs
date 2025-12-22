use serial_test::serial;
use std::collections::{HashMap, HashSet};
use std::env;

/// Clean up environment variables used in tests
fn cleanup_env(keys: &[String]) {
    for key in keys {
        env::remove_var(key);
    }
}

#[cfg(test)]
mod nested_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_nested_override_independence() {
        cleanup_env(&[
            "PARENT_KEY".to_string(),
            "NESTED_KEY".to_string(),
            "NESTED_PREFIX_KEY".to_string(),
        ]);

        env::set_var("PARENT_KEY", "parent_value");
        env::set_var("NESTED_KEY", "wrong_prefix_value");
        env::set_var("NESTED_PREFIX_KEY", "correct_prefix_value");

        // Simulate nested config with prefix
        let mut nested_config = HashMap::new();
        nested_config.insert("KEY".to_string(), "file_value".to_string());

        // Apply merge with nested prefix
        let result = better_config_core::utils::override_env::merge_with_env(
            nested_config,
            Some("NESTED_PREFIX_"),
            &HashSet::new(),
        );

        // Should use the value from NESTED_PREFIX_KEY, not NESTED_KEY
        assert_eq!(result.get("KEY"), Some(&"correct_prefix_value".to_string()));

        cleanup_env(&[
            "PARENT_KEY".to_string(),
            "NESTED_KEY".to_string(),
            "NESTED_PREFIX_KEY".to_string(),
        ]);
    }

    #[test]
    #[serial]
    fn test_nested_excluded_keys_independence() {
        cleanup_env(&[
            "PARENT_SECRET".to_string(),
            "NESTED_PREFIX_SECRET".to_string(),
        ]);

        // Test that nested config's excluded keys are independent from parent
        env::set_var("PARENT_SECRET", "parent_env_secret");
        env::set_var("NESTED_PREFIX_SECRET", "nested_env_secret");

        // Parent config without exclusions
        let mut parent_config = HashMap::new();
        parent_config.insert("SECRET".to_string(), "parent_file_secret".to_string());

        let parent_result = better_config_core::utils::override_env::merge_with_env(
            parent_config,
            Some("PARENT_"),
            &HashSet::new(),
        );

        // Parent should use env value (no exclusions)
        assert_eq!(
            parent_result.get("SECRET"),
            Some(&"parent_env_secret".to_string())
        );

        // Nested config with exclusions
        let mut nested_config = HashMap::new();
        nested_config.insert("SECRET".to_string(), "nested_file_secret".to_string());

        let mut excluded = HashSet::new();
        excluded.insert("SECRET".to_string());

        let nested_result = better_config_core::utils::override_env::merge_with_env(
            nested_config,
            Some("NESTED_PREFIX_"),
            &excluded,
        );

        // Nested should use file value (excluded)
        assert_eq!(
            nested_result.get("SECRET"),
            Some(&"nested_file_secret".to_string())
        );

        cleanup_env(&[
            "PARENT_SECRET".to_string(),
            "NESTED_PREFIX_SECRET".to_string(),
        ]);
    }
}
