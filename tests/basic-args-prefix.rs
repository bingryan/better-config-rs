use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "APP_"))]
pub struct AppConfig {
    #[conf(default = "app_api_key", from = "API_KEY")]
    pub api_key: String,
    #[conf(default = "8000", from = "PORT")]
    pub port: u16,
    #[conf(default = "false", from = "DEBUG")]
    pub debug: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn fixture() {
        env::remove_var("APP_API_KEY");
        env::remove_var("API_KEY");
        env::remove_var("APP_PORT");
        env::remove_var("PORT");
        env::remove_var("APP_DEBUG");
        env::remove_var("DEBUG");
    }

    #[test]
    #[serial]
    fn basic_args_defaults() {
        fixture();
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "app_api_key");
        assert_eq!(config.port, 8000);
        assert!(!config.debug);
    }

    #[test]
    #[serial]
    fn basic_args_env() {
        fixture();
        env::set_var("APP_API_KEY", "app_api_key");
        env::set_var("API_KEY", "api_key");
        env::set_var("APP_PORT", "8080");
        // tips: debug have prefix, so debug get from APP_DEBUG
        env::set_var("DEBUG", "true");

        let config = AppConfig::builder().build().unwrap();

        assert_eq!(config.api_key, "app_api_key");
        assert_eq!(config.port, 8080);
        // APP_DEBUG not set, and debug default is false
        assert!(!config.debug);
    }
}
