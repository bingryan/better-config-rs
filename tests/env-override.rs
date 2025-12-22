use better_config::{env, JsonConfig, TomlConfig};
use serial_test::serial;
use std::env::{remove_var, set_var};

// Test JSON configuration with environment override
#[env(JsonConfig(target = "config.json"))]
pub struct JsonAppConfig {
    #[conf(from = "name", default = "default-app")]
    pub app_name: String,
    #[conf(from = "public", default = "false")]
    pub public: bool,
    #[conf(from = "version", default = "0.0")]
    pub version: f64,
    #[conf(from = "scripts.echo", default = "default-echo", no_env_override)]
    pub echo: String,
}

// Test TOML configuration with environment override
#[env(TomlConfig(target = "config.toml"))]
pub struct TomlAppConfig {
    #[conf(from = "title", default = "default-title")]
    pub title: String,
    #[conf(from = "database.enabled", default = "false")]
    pub database_enabled: bool,
    #[conf(from = "servers.alpha.ip", default = "127.0.0.1")]
    pub alpha_ip: String,
    #[conf(from = "servers.beta.role", default = "unknown", no_env_override)]
    pub beta_role: String,
}

// Test nested structure with environment override
#[env(JsonConfig(target = "config-nested.json"))]
pub struct NestedAppConfig {
    #[conf(from = "debug", default = "false")]
    pub debug: bool,
    #[env]
    pub database: DatabaseConfig,
}

#[derive(Debug)]
#[env(JsonConfig(prefix = "database.", target = "config-nested.json"))]
pub struct DatabaseConfig {
    #[conf(from = "host", default = "localhost")]
    pub host: String,
    #[conf(from = "port", default = "3306")]
    pub port: u16,
    #[conf(from = "user", default = "root")]
    pub user: String,
    #[conf(from = "password", default = "123456", no_env_override)]
    pub password: String,
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn cleanup_env_vars() {
        // JSON config env vars
        remove_var("NAME");
        remove_var("PUBLIC");
        remove_var("VERSION");
        remove_var("SCRIPTS_ECHO");

        // TOML config env vars
        remove_var("TITLE");
        remove_var("DATABASE_ENABLED");
        remove_var("SERVERS_ALPHA_IP");
        remove_var("SERVERS_BETA_ROLE");

        // Nested config env vars
        remove_var("DEBUG");
        remove_var("DATABASE_HOST");
        remove_var("DATABASE_PORT");
        remove_var("DATABASE_USER");
        remove_var("DATABASE_PASSWORD");
    }

    #[test]
    #[serial]
    fn test_json_config_env_override() {
        cleanup_env_vars();

        // Set environment variables to override JSON config
        set_var("NAME", "env-app-name");
        set_var("PUBLIC", "false");
        set_var("VERSION", "2.5");
        set_var("SCRIPTS_ECHO", "env-echo"); // Should be ignored due to no_env_override

        let config = JsonAppConfig::builder().build().unwrap();

        // Verify environment variables override file values
        assert_eq!(config.app_name, "env-app-name");
        assert!(!config.public);
        assert_eq!(config.version, 2.5);

        // Echo should use file value, not env value (due to no_env_override)
        assert_eq!(config.echo, "echo"); // From config.json

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_json_config_no_env_vars() {
        cleanup_env_vars();

        let config = JsonAppConfig::builder().build().unwrap();

        // Should use file values when no env vars are set
        assert_eq!(config.app_name, "config.json"); // From config.json
        assert!(config.public); // From config.json
        assert_eq!(config.version, 1.0); // From config.json
        assert_eq!(config.echo, "echo"); // From config.json

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_toml_config_env_override() {
        cleanup_env_vars();

        // Set environment variables to override TOML config
        set_var("TITLE", "env-title");
        set_var("DATABASE_ENABLED", "false");
        set_var("SERVERS_ALPHA_IP", "192.168.1.1");
        set_var("SERVERS_BETA_ROLE", "env-role"); // Should be ignored

        let config = TomlAppConfig::builder().build().unwrap();

        // Verify environment variables override file values
        assert_eq!(config.title, "env-title");
        assert!(!config.database_enabled);
        assert_eq!(config.alpha_ip, "192.168.1.1");

        // Beta role should use file value, not env value (due to no_env_override)
        assert_eq!(config.beta_role, "backend"); // From config.toml

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_toml_config_no_env_vars() {
        cleanup_env_vars();

        let config = TomlAppConfig::builder().build().unwrap();

        // Should use file values when no env vars are set
        assert_eq!(config.title, "TOML Example"); // From config.toml
        assert!(config.database_enabled); // From config.toml
        assert_eq!(config.alpha_ip, "10.0.0.1"); // From config.toml
        assert_eq!(config.beta_role, "backend"); // From config.toml

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_nested_config_env_override() {
        cleanup_env_vars();

        // Set environment variables for nested config
        set_var("DEBUG", "false"); // Override parent config
        set_var("DATABASE_HOST", "env-db-host");
        set_var("DATABASE_PORT", "5432");
        set_var("DATABASE_USER", "env-user");
        set_var("DATABASE_PASSWORD", "env-password"); // Should be ignored

        let config = NestedAppConfig::builder().build().unwrap();

        // Parent config should use env vars
        assert!(!config.debug);

        // Nested config should use env vars (except password)
        assert_eq!(config.database.host, "env-db-host");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.database.user, "env-user");

        // Password should use file value (due to no_env_override)
        assert_eq!(config.database.password, "password"); // From config-nested.json

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_nested_config_no_env_vars() {
        cleanup_env_vars();

        let config = NestedAppConfig::builder().build().unwrap();

        // Should use file values when no env vars are set
        assert!(config.debug); // From config-nested.json
        assert_eq!(config.database.host, "127.0.0.1"); // From config-nested.json
        assert_eq!(config.database.port, 3307); // From config-nested.json
        assert_eq!(config.database.user, "admin"); // From config-nested.json
        assert_eq!(config.database.password, "password"); // From config-nested.json

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_no_env_override_attribute() {
        cleanup_env_vars();

        // Set env vars for fields with and without no_env_override
        set_var("NAME", "env-name");
        set_var("SCRIPTS_ECHO", "env-echo"); // Should be ignored
        set_var("DATABASE_HOST", "env-host");
        set_var("DATABASE_PASSWORD", "env-password"); // Should be ignored

        let json_config = JsonAppConfig::builder().build().unwrap();
        let nested_config = NestedAppConfig::builder().build().unwrap();

        // Fields without no_env_override should use env values
        assert_eq!(json_config.app_name, "env-name");
        assert_eq!(nested_config.database.host, "env-host");

        // Fields with no_env_override should use file values
        assert_eq!(json_config.echo, "echo"); // From config.json
        assert_eq!(nested_config.database.password, "password"); // From config-nested.json

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_nested_prefix_independence() {
        cleanup_env_vars();

        // Set env vars with different prefixes
        set_var("DEBUG", "false"); // Parent config (no prefix)
        set_var("HOST", "wrong-host"); // Should not affect nested config
        set_var("DATABASE_HOST", "correct-host"); // Should affect nested config

        let config = NestedAppConfig::builder().build().unwrap();

        // Parent should use DEBUG (no prefix)
        assert!(!config.debug);

        // Nested should use DATABASE_HOST (with prefix), not HOST
        assert_eq!(config.database.host, "correct-host");

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_type_conversion() {
        cleanup_env_vars();

        // Test type conversion from string env vars
        set_var("PUBLIC", "false"); // String to bool
        set_var("VERSION", "2.71"); // String to f64
        set_var("DATABASE_PORT", "5432"); // String to u16

        let json_config = JsonAppConfig::builder().build().unwrap();
        let nested_config = NestedAppConfig::builder().build().unwrap();

        // Verify correct type conversion
        assert!(!json_config.public);
        assert_eq!(json_config.version, 2.71); // Use a different value to avoid PI constant warning
        assert_eq!(nested_config.database.port, 5432);

        cleanup_env_vars();
    }
}
