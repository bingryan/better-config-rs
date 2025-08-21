use better_config::{env, TomlConfig};

#[env(TomlConfig)]
pub struct AppConfig {
    #[conf(default = "default_key")]
    pub api_key: String,
    #[conf(from = "title", default = "hello toml")]
    pub title: String,
    #[conf(from = "database.enabled", default = "false")]
    pub database_enabled: bool,

    #[conf(from = "database.ports")]
    pub database_ports: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn basic_defaults() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "default_key");
    }

    #[test]
    #[serial]
    fn basic_toml() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.title, "TOML Example");
        assert!(config.database_enabled);
        assert_eq!(config.database_ports, "[8000, 8001, 8002]");
    }
}
