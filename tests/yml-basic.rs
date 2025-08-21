use better_config::{env, YmlConfig};

#[env(YmlConfig)]
pub struct AppConfig {
    #[conf(default = "yml_default_key")]
    pub api_key: String,
    #[conf(from = "title", default = "hello yml")]
    pub title: String,
    #[conf(from = "database.host", default = "localhost")]
    pub database_host: String,
    #[conf(from = "database.port")]
    pub database_port: u16,
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
    fn basic_yml() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.title, "Yml Example");
        assert_eq!(config.database_host, "127.0.0.1");
        assert_eq!(config.database_port, 3306);
    }
}
