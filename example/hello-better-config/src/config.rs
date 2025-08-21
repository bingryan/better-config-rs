#![allow(clippy::useless_format)]
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "BETTER_", target = ".env.prod,.env.staging,.env.dev"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    pub host: String,
    #[conf(from = "DB_PORT", default = "8000")]
    pub port: u16,
    #[conf(getter = "get_url")]
    pub url: String,
    #[conf(getter = "get_wrap_url")]
    pub wrap_url: WrapURL,
}

#[derive(Debug, PartialEq)]
pub struct WrapURL(String);

impl AppConfigBuilder {
    pub fn get_url(&self, params: &std::collections::HashMap<String, String>) -> String {
        format!(
            "{}",
            params
                .get("BETTER_DB_HOST")
                .unwrap_or(&"better".to_string())
        )
    }

    pub fn get_wrap_url(&self, p: &std::collections::HashMap<String, String>) -> WrapURL {
        WrapURL(format!(
            "{}",
            p.get("BETTER_DB_HOST")
                .unwrap_or(&"better wrap url".to_string())
        ))
    }
}
