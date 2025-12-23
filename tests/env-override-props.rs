use serial_test::serial;
use std::collections::{HashMap, HashSet};
use std::env;

// Global proptest imports for the main property tests
use proptest::prelude::*;

/// Helper to generate valid config keys (alphanumeric with underscores)
fn config_key_strategy() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,10}".prop_map(|s| s.to_string())
}

/// Helper to generate config values
fn config_value_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,20}".prop_map(|s| s.to_string())
}

/// Clean up environment variables used in tests
fn cleanup_env(keys: &[String]) {
    for key in keys {
        env::remove_var(key);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    #[serial]
    fn prop_override_priority_env_over_file(
        key in config_key_strategy(),
        file_value in config_value_strategy(),
        env_value in config_value_strategy(),
    ) {
        // Setup: clean environment
        cleanup_env(std::slice::from_ref(&key));

        // Set environment variable
        env::set_var(&key, &env_value);

        // Create file config
        let mut file_config = HashMap::new();
        file_config.insert(key.clone(), file_value.clone());

        // Apply merge
        let result = better_config::utils::merge_with_env(
            file_config,
            None,
            &HashSet::new(),
        );

        // Property: env value should override file value
        prop_assert_eq!(
            result.get(&key),
            Some(&env_value),
            "Environment variable should override file value"
        );

        // Cleanup
        cleanup_env(std::slice::from_ref(&key));
    }

    #[test]
    #[serial]
    fn prop_override_priority_file_when_no_env(
        key in config_key_strategy(),
        file_value in config_value_strategy(),
    ) {
        // Setup: ensure no env var exists
        cleanup_env(std::slice::from_ref(&key));

        // Create file config
        let mut file_config = HashMap::new();
        file_config.insert(key.clone(), file_value.clone());

        // Apply merge
        let result = better_config::utils::merge_with_env(
            file_config,
            None,
            &HashSet::new(),
        );

        // Property: file value should be used when no env var exists
        prop_assert_eq!(
            result.get(&key),
            Some(&file_value),
            "File value should be used when no environment variable exists"
        );
    }

    #[test]
    #[serial]
    fn prop_override_priority_multiple_keys(
        keys in prop::collection::hash_set(config_key_strategy(), 1..5),
        file_values in prop::collection::vec(config_value_strategy(), 5),
        env_values in prop::collection::vec(config_value_strategy(), 5),
        override_mask in prop::collection::vec(any::<bool>(), 5),
    ) {
        // Convert HashSet to Vec to ensure unique keys
        let keys: Vec<_> = keys.into_iter().take(5).collect();

        if keys.is_empty() {
            return Ok(());
        }

        // Setup: clean environment
        cleanup_env(&keys);

        // Create file config and selectively set env vars
        let mut file_config = HashMap::new();
        let mut expected = HashMap::new();

        for (i, key) in keys.iter().enumerate() {
            let file_val = file_values.get(i).cloned().unwrap_or_default();
            let env_val = env_values.get(i).cloned().unwrap_or_default();
            let should_override = override_mask.get(i).copied().unwrap_or(false);

            file_config.insert(key.clone(), file_val.clone());

            if should_override {
                env::set_var(key, &env_val);
                expected.insert(key.clone(), env_val);
            } else {
                expected.insert(key.clone(), file_val);
            }
        }

        // Apply merge
        let result = better_config::utils::merge_with_env(
            file_config,
            None,
            &HashSet::new(),
        );

        // Property: each key should have the expected value based on override mask
        for key in &keys {
            prop_assert_eq!(
                result.get(key),
                expected.get(key),
                "Key {} should have correct priority-based value",
                key
            );
        }

        // Cleanup
        cleanup_env(&keys);
    }

    // For any field marked with `no_env_override`, even if a matching environment variable exists,
    // the file value (or default) SHALL be used instead of the environment variable value.
    #[test]
    #[serial]
    fn prop_no_override_opt_out(
        key in config_key_strategy(),
        file_value in config_value_strategy(),
        env_value in config_value_strategy(),
    ) {
        // Setup: clean environment
        cleanup_env(std::slice::from_ref(&key));

        // Set environment variable
        env::set_var(&key, &env_value);

        // Create file config
        let mut file_config = HashMap::new();
        file_config.insert(key.clone(), file_value.clone());

        // Create excluded keys set (simulating no_env_override)
        let mut excluded_keys = HashSet::new();
        excluded_keys.insert(key.clone());

        // Apply merge with excluded keys
        let result = better_config::utils::merge_with_env(
            file_config,
            None,
            &excluded_keys,
        );

        // Property: file value should be used even when env var exists (because key is excluded)
        prop_assert_eq!(
            result.get(&key),
            Some(&file_value),
            "File value should be used for excluded keys even when environment variable exists"
        );

        // Cleanup
        cleanup_env(std::slice::from_ref(&key));
    }

    #[test]
    #[serial]
    fn prop_no_override_mixed_keys(
        keys in prop::collection::vec(config_key_strategy(), 2..6),
        file_values in prop::collection::vec(config_value_strategy(), 6),
        env_values in prop::collection::vec(config_value_strategy(), 6),
        exclude_mask in prop::collection::vec(any::<bool>(), 6),
    ) {
        // Ensure we have enough values
        let keys: Vec<_> = keys.into_iter().take(6).collect();
        if keys.len() < 2 {
            return Ok(());
        }

        // Setup: clean environment
        cleanup_env(&keys);

        // Create file config and set all env vars
        let mut file_config = HashMap::new();
        let mut excluded_keys = HashSet::new();
        let mut expected = HashMap::new();

        for (i, key) in keys.iter().enumerate() {
            let file_val = file_values.get(i).cloned().unwrap_or_default();
            let env_val = env_values.get(i).cloned().unwrap_or_default();
            let should_exclude = exclude_mask.get(i).copied().unwrap_or(false);

            file_config.insert(key.clone(), file_val.clone());
            env::set_var(key, &env_val);

            if should_exclude {
                excluded_keys.insert(key.clone());
                expected.insert(key.clone(), file_val); // Should use file value
            } else {
                expected.insert(key.clone(), env_val); // Should use env value
            }
        }

        // Apply merge with excluded keys
        let result = better_config::utils::merge_with_env(
            file_config,
            None,
            &excluded_keys,
        );

        // Property: each key should have the expected value based on exclude mask
        for key in &keys {
            prop_assert_eq!(
                result.get(key),
                expected.get(key),
                "Key {} should have correct value based on exclusion status",
                key
            );
        }

        // Cleanup
        cleanup_env(&keys);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_priority_chain_example() {
        cleanup_env(&["TEST_KEY".to_string()]);

        // Case 1: Env overrides file
        env::set_var("TEST_KEY", "from_env");
        let mut config = HashMap::new();
        config.insert("TEST_KEY".to_string(), "from_file".to_string());

        let result = better_config::utils::merge_with_env(config, None, &HashSet::new());
        assert_eq!(result.get("TEST_KEY"), Some(&"from_env".to_string()));

        // Case 2: File used when no env
        cleanup_env(&["TEST_KEY".to_string()]);
        let mut config = HashMap::new();
        config.insert("TEST_KEY".to_string(), "from_file".to_string());

        let result = better_config::utils::merge_with_env(config, None, &HashSet::new());
        assert_eq!(result.get("TEST_KEY"), Some(&"from_file".to_string()));

        cleanup_env(&["TEST_KEY".to_string()]);
    }

    #[test]
    #[serial]
    fn test_no_override_opt_out_example() {
        cleanup_env(&["SECRET_KEY".to_string()]);

        // Set env var that should be ignored
        env::set_var("SECRET_KEY", "env_secret");
        let mut config = HashMap::new();
        config.insert("SECRET_KEY".to_string(), "file_secret".to_string());

        // Mark key as excluded
        let mut excluded = HashSet::new();
        excluded.insert("SECRET_KEY".to_string());

        let result = better_config::utils::merge_with_env(config, None, &excluded);

        // Should use file value, not env value
        assert_eq!(result.get("SECRET_KEY"), Some(&"file_secret".to_string()));

        cleanup_env(&["SECRET_KEY".to_string()]);
    }
}

#[cfg(test)]
mod nested_tests {
    use super::*;

    // Unit test for nested override independence
    // This test verifies that nested structures use their own prefix for environment variable lookups
    #[test]
    #[serial]
    fn test_nested_override_independence() {
        cleanup_env(&[
            "PARENT_KEY".to_string(),
            "NESTED_KEY".to_string(),
            "NESTED_PREFIX_KEY".to_string(),
        ]);

        // Test scenario: Parent has no prefix, nested has "NESTED_PREFIX_"
        // Setting NESTED_PREFIX_KEY should affect nested config
        // Setting NESTED_KEY should NOT affect nested config (wrong prefix)

        env::set_var("PARENT_KEY", "parent_value");
        env::set_var("NESTED_KEY", "wrong_prefix_value");
        env::set_var("NESTED_PREFIX_KEY", "correct_prefix_value");

        // Simulate nested config with prefix
        let mut nested_config = HashMap::new();
        nested_config.insert("KEY".to_string(), "file_value".to_string());

        // Apply merge with nested prefix
        let result = better_config::utils::merge_with_env(
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

        let parent_result =
            better_config::utils::merge_with_env(parent_config, Some("PARENT_"), &HashSet::new());

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

        let nested_result =
            better_config::utils::merge_with_env(nested_config, Some("NESTED_PREFIX_"), &excluded);

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

    #[test]
    #[serial]
    fn test_multiple_nested_levels() {
        cleanup_env(&[
            "LEVEL1_KEY".to_string(),
            "LEVEL2_KEY".to_string(),
            "LEVEL3_KEY".to_string(),
        ]);

        // Test multiple levels of nesting with different prefixes
        env::set_var("LEVEL1_KEY", "level1_env");
        env::set_var("LEVEL2_KEY", "level2_env");
        env::set_var("LEVEL3_KEY", "level3_env");

        // Level 1 config
        let mut level1_config = HashMap::new();
        level1_config.insert("KEY".to_string(), "level1_file".to_string());

        let level1_result =
            better_config::utils::merge_with_env(level1_config, Some("LEVEL1_"), &HashSet::new());

        // Level 2 config
        let mut level2_config = HashMap::new();
        level2_config.insert("KEY".to_string(), "level2_file".to_string());

        let level2_result =
            better_config::utils::merge_with_env(level2_config, Some("LEVEL2_"), &HashSet::new());

        // Level 3 config
        let mut level3_config = HashMap::new();
        level3_config.insert("KEY".to_string(), "level3_file".to_string());

        let level3_result =
            better_config::utils::merge_with_env(level3_config, Some("LEVEL3_"), &HashSet::new());

        // Each level should use its own prefixed env var
        assert_eq!(level1_result.get("KEY"), Some(&"level1_env".to_string()));
        assert_eq!(level2_result.get("KEY"), Some(&"level2_env".to_string()));
        assert_eq!(level3_result.get("KEY"), Some(&"level3_env".to_string()));

        cleanup_env(&[
            "LEVEL1_KEY".to_string(),
            "LEVEL2_KEY".to_string(),
            "LEVEL3_KEY".to_string(),
        ]);
    }
}

#[cfg(test)]
mod nested_prop_tests {
    use super::*;

    proptest! {
        #[test]
        #[serial]
        fn prop_nested_prefix_isolation(
            parent_prefix in "[A-Z_]{1,8}",
            nested_prefix in "[A-Z_]{1,8}",
            key in "[A-Z_]{1,6}",
            file_value in "[a-z-]{1,15}",
            parent_env_value in "[a-z-]{1,15}",
            nested_env_value in "[a-z-]{1,15}"
        ) {
            // Ensure prefixes are different
            prop_assume!(parent_prefix != nested_prefix);

            let parent_env_key = format!("{}{}", parent_prefix, key);
            let nested_env_key = format!("{}{}", nested_prefix, key);

            // Clean up
            env::remove_var(&parent_env_key);
            env::remove_var(&nested_env_key);

            // Set both env vars
            env::set_var(&parent_env_key, &parent_env_value);
            env::set_var(&nested_env_key, &nested_env_value);

            // Test parent config (should use parent prefix)
            let mut parent_config = HashMap::new();
            parent_config.insert(key.clone(), file_value.clone());

            let parent_result = better_config::utils::merge_with_env(
                parent_config,
                Some(&parent_prefix),
                &HashSet::new(),
            );

            // Test nested config (should use nested prefix)
            let mut nested_config = HashMap::new();
            nested_config.insert(key.clone(), file_value.clone());

            let nested_result = better_config::utils::merge_with_env(
                nested_config,
                Some(&nested_prefix),
                &HashSet::new(),
            );

            // Each should use its own prefixed env var
            prop_assert_eq!(parent_result.get(&key), Some(&parent_env_value));
            prop_assert_eq!(nested_result.get(&key), Some(&nested_env_value));

            // Clean up
            env::remove_var(&parent_env_key);
            env::remove_var(&nested_env_key);
        }
    }
}

#[cfg(test)]
mod prefix_prop_tests {
    use super::*;

    proptest! {
        #[test]
        #[serial]
        fn prop_prefix_handling_consistency(
            prefix in "[A-Z_]{1,8}",
            key in "[A-Z_]{1,6}",
            file_value in "[a-z-]{1,15}",
            env_value in "[a-z-]{1,15}"
        ) {
            let prefixed_env_key = format!("{}{}", prefix, key);
            let unprefixed_env_key = key.clone();

            // Clean up
            env::remove_var(&prefixed_env_key);
            env::remove_var(&unprefixed_env_key);

            // Set both prefixed and unprefixed env vars
            env::set_var(&prefixed_env_key, &env_value);
            env::set_var(&unprefixed_env_key, "wrong-value");

            // Test with prefix - should use prefixed env var
            let mut config_with_prefix = HashMap::new();
            config_with_prefix.insert(key.clone(), file_value.clone());

            let result_with_prefix = better_config::utils::merge_with_env(
                config_with_prefix,
                Some(&prefix),
                &HashSet::new(),
            );

            // Should use the prefixed env var value
            prop_assert_eq!(result_with_prefix.get(&key), Some(&env_value));

            // Test without prefix - should use unprefixed env var
            let mut config_without_prefix = HashMap::new();
            config_without_prefix.insert(key.clone(), file_value.clone());

            let result_without_prefix = better_config::utils::merge_with_env(
                config_without_prefix,
                None,
                &HashSet::new(),
            );

            // Should use the unprefixed env var value
            let wrong_value = "wrong-value".to_string();
            prop_assert_eq!(result_without_prefix.get(&key), Some(&wrong_value));

            // Clean up
            env::remove_var(&prefixed_env_key);
            env::remove_var(&unprefixed_env_key);
        }

        #[test]
        #[serial]
        fn prop_prefix_exact_match(
            prefix in "[A-Z_]{1,8}",
            key in "[A-Z_]{1,6}",
            file_value in "[a-z-]{1,15}",
            env_value in "[a-z-]{1,15}"
        ) {
            let correct_env_key = format!("{}{}", prefix, key);
            let wrong_prefix_key = format!("WRONG_{}{}", prefix, key);

            // Clean up
            env::remove_var(&correct_env_key);
            env::remove_var(&wrong_prefix_key);

            // Set env var with wrong prefix
            env::set_var(&wrong_prefix_key, "wrong-value");

            // Test with correct prefix - should NOT use wrong prefix env var
            let mut config = HashMap::new();
            config.insert(key.clone(), file_value.clone());

            let result = better_config::utils::merge_with_env(
                config,
                Some(&prefix),
                &HashSet::new(),
            );

            // Should use file value because correct prefixed env var doesn't exist
            prop_assert_eq!(result.get(&key), Some(&file_value));

            // Now set the correct prefixed env var
            env::set_var(&correct_env_key, &env_value);

            let mut config2 = HashMap::new();
            config2.insert(key.clone(), file_value.clone());

            let result2 = better_config::utils::merge_with_env(
                config2,
                Some(&prefix),
                &HashSet::new(),
            );

            // Should use env value from correctly prefixed var
            prop_assert_eq!(result2.get(&key), Some(&env_value));

            // Clean up
            env::remove_var(&correct_env_key);
            env::remove_var(&wrong_prefix_key);
        }
    }
}

#[cfg(test)]
mod type_conversion_prop_tests {
    use super::*;

    proptest! {
        #[test]
        #[serial]
        fn prop_type_conversion_string_values(
            key in "[A-Z_]{1,6}",
            string_value in "[a-zA-Z0-9-_]{1,20}"
        ) {
            let env_key = key.clone();

            // Clean up
            env::remove_var(&env_key);

            // Set string env var
            env::set_var(&env_key, &string_value);

            // Test string type conversion (should work for any string)
            let mut config = HashMap::new();
            config.insert(key.clone(), "file-value".to_string());

            let result = better_config::utils::merge_with_env(
                config,
                None,
                &HashSet::new(),
            );

            // Should use env value as string
            prop_assert_eq!(result.get(&key), Some(&string_value));

            // Clean up
            env::remove_var(&env_key);
        }

        #[test]
        #[serial]
        fn prop_type_conversion_boolean_values(
            key in "[A-Z_]{1,6}",
            bool_value in prop::bool::ANY
        ) {
            let env_key = key.clone();
            let bool_string = bool_value.to_string();

            // Clean up
            env::remove_var(&env_key);

            // Set boolean env var
            env::set_var(&env_key, &bool_string);

            // Test that env override provides the string representation
            let mut config = HashMap::new();
            config.insert(key.clone(), "file-value".to_string());

            let result = better_config::utils::merge_with_env(
                config,
                None,
                &HashSet::new(),
            );

            // Should use env value as string (type conversion happens later)
            prop_assert_eq!(result.get(&key), Some(&bool_string));

            // Clean up
            env::remove_var(&env_key);
        }

        #[test]
        #[serial]
        fn prop_type_conversion_numeric_values(
            key in "[A-Z_]{1,6}",
            numeric_value in 0u32..10000u32
        ) {
            let env_key = key.clone();
            let numeric_string = numeric_value.to_string();

            // Clean up
            env::remove_var(&env_key);

            // Set numeric env var
            env::set_var(&env_key, &numeric_string);

            // Test that env override provides the string representation
            let mut config = HashMap::new();
            config.insert(key.clone(), "999".to_string());

            let result = better_config::utils::merge_with_env(
                config,
                None,
                &HashSet::new(),
            );

            // Should use env value as string (type conversion happens later)
            prop_assert_eq!(result.get(&key), Some(&numeric_string));

            // Clean up
            env::remove_var(&env_key);
        }


        #[test]
        #[serial]
        fn prop_type_conversion_consistency_with_file(
            key in "[A-Z_]{1,6}",
            value in "[a-zA-Z0-9-_.]{1,15}"
        ) {
            let env_key = key.clone();

            // Clean up
            env::remove_var(&env_key);

            // Test 1: File value only
            let mut config1 = HashMap::new();
            config1.insert(key.clone(), value.clone());

            let result1 = better_config::utils::merge_with_env(
                config1,
                None,
                &HashSet::new(),
            );

            // Test 2: Same value from env var
            env::set_var(&env_key, &value);

            let mut config2 = HashMap::new();
            config2.insert(key.clone(), "different-file-value".to_string());

            let result2 = better_config::utils::merge_with_env(
                config2,
                None,
                &HashSet::new(),
            );

            // Both should produce the same string value
            prop_assert_eq!(result1.get(&key), result2.get(&key));
            prop_assert_eq!(result1.get(&key), Some(&value));
            prop_assert_eq!(result2.get(&key), Some(&value));

            // Clean up
            env::remove_var(&env_key);
        }
    }
}
