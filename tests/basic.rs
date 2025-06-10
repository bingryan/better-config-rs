use better_config::{env, EnvConfig};

#[env(EnvConfig)]
pub struct AppConfig {
    #[conf(default = "default_key")]
    api_key: String,
    #[conf(default = "8000")]
    port: u16,
    #[conf(default = "false")]
    debug: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn fixture() {
        env::remove_var("API_KEY");
        env::remove_var("PORT");
        env::remove_var("DEBUG");
    }

    #[test]
    #[serial]
    fn basic_defaults() {
        fixture();

        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "default_key");
        assert_eq!(config.port, 8000);
        assert!(!config.debug);
    }

    #[test]
    #[serial]
    fn basic_env() {
        fixture();
        env::set_var("API_KEY", "test_key");
        env::set_var("PORT", "8080");
        env::set_var("DEBUG", "true");

        let config = AppConfig::builder().build().unwrap();

        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.port, 8080);
        assert!(config.debug);
    }
}
