use better_config::{env, JsonConfig};
use serial_test::serial;
use std::env::{remove_var, set_var};

#[env(JsonConfig(target = "config-nested.json"))]
pub struct AppConfig {
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
mod tests {
    use super::*;

    fn cleanup_env() {
        remove_var("DEBUG");
        remove_var("DATABASE_HOST");
        remove_var("DATABASE_PORT");
        remove_var("DATABASE_USER");
        remove_var("DATABASE_PASSWORD");
    }

    #[test]
    #[serial]
    fn test_nested_env_override() {
        cleanup_env();

        // Set environment variables that should override nested config
        set_var("DATABASE_HOST", "env-host");
        set_var("DATABASE_PORT", "9999");
        set_var("DATABASE_USER", "env-user");
        set_var("DATABASE_PASSWORD", "env-password"); // Should be ignored due to no_env_override

        let config = AppConfig::builder().build().unwrap();

        // Verify that nested config uses env vars (except password)
        assert_eq!(config.database.host, "env-host");
        assert_eq!(config.database.port, 9999);
        assert_eq!(config.database.user, "env-user");
        // Password should use file value, not env value (due to no_env_override)
        assert_eq!(config.database.password, "password");

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_nested_prefix_independence() {
        cleanup_env();

        // Set env vars for both parent and nested configs
        set_var("DEBUG", "true");
        set_var("DATABASE_HOST", "nested-host");

        let config = AppConfig::builder().build().unwrap();

        // Parent config should use its own env vars (no prefix)
        assert!(config.debug);

        // Nested config should use its own prefixed env vars
        assert_eq!(config.database.host, "nested-host");

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_nested_no_env_override() {
        cleanup_env();

        // Set env var that should be ignored for password field
        set_var("DATABASE_PASSWORD", "should-be-ignored");

        let config = AppConfig::builder().build().unwrap();

        // Password should use file value, not env value
        assert_eq!(config.database.password, "password");

        cleanup_env();
    }
}
