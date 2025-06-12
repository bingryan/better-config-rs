use better_config::{env, YmlConfig};

#[env(YmlConfig)]
pub struct AppConfig {
    #[conf(default = "yml_default_key")]
    api_key: String,
    #[conf(from = "title", default = "hello yml")]
    title: String,
    #[conf(from = "database.host", default = "localhost")]
    database_host: String,
    #[conf(from = "database.port")]
    database_port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn basic_defaults() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "yml_default_key");
    }

    #[test]
    #[serial]
    fn basic_toml() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.title, "Yml Example");
        assert_eq!(config.database_host, "127.0.0.1");
        assert_eq!(config.database_port, 3306);
    }
}
