use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "BETTER_", target = ".env.prod,.env.staging,.env.dev"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    pub host: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn fixture() {
        env::remove_var("BETTER_DB_HOST");
    }

    #[test]
    #[serial]
    fn target_priority() {
        fixture();
        let config = AppConfig::builder().build().unwrap();
        // priority: .env.prod > .env.staging > .env.dev
        assert_eq!(config.host, "prod");
    }
}
