#![allow(clippy::useless_format)]
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "BETTER_", target = ".env.prod,.env.staging,.env.dev"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    host: String,
    #[conf(from = "DB_PORT", default = "8000")]
    port: u16,
    #[conf(getter = "get_url")]
    url: String,
    #[conf(getter = "get_wrap_url")]
    wrap_url: WrapURL,
}

#[derive(Debug, PartialEq)]
struct WrapURL(String);

impl AppConfigBuilder {
    fn get_url(&self, params: &std::collections::HashMap<String, String>) -> String {
        format!(
            "{}",
            params
                .get("BETTER_DB_HOST")
                .unwrap_or(&"better".to_string())
        )
    }

    fn get_wrap_url(&self, p: &std::collections::HashMap<String, String>) -> WrapURL {
        WrapURL(format!(
            "{}",
            p.get("BETTER_DB_HOST")
                .unwrap_or(&"better wrap url".to_string())
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn fixture() {
        env::remove_var("BETTER_DB_HOST");
        env::remove_var("BETTER_DB_PORT");
    }

    #[test]
    #[serial]
    fn target_priority() {
        fixture();
        let config = AppConfig::builder().build().unwrap();
        // priority: .env.prod > .env.staging > .env.dev
        assert_eq!(config.host, "prod");
        assert_eq!(config.port, 8000);
        assert_eq!(config.url, "prod");
        assert_eq!(config.wrap_url, WrapURL("prod".to_string()));
    }
}
