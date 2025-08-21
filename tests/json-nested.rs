use better_config::{env, JsonConfig};

#[env(JsonConfig(target = "config-nested.json"))]
pub struct AppConfig {
    #[conf(default = "default_key")]
    pub api_key: String,
    #[conf(from = "debug", default = "false")]
    pub debug: bool,
    #[env]
    pub database: DatabaseConfig,
}

#[derive(Debug)]
#[env(JsonConfig(prefix = "database.", target = "config-nested.json"))]
pub struct DatabaseConfig {
    #[conf(from = "host", default = "localhost")]
    host: String,
    #[conf(from = "port", default = "3306")]
    port: u16,
    #[conf(from = "user", default = "root")]
    user: String,
    #[conf(from = "password", default = "123456")]
    password: String,
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
        assert!(config.debug)
    }

    #[test]
    #[serial]
    fn basic_json_nested() {
        let config = AppConfig::builder().build().unwrap();
        println!("config:{:?}", config.database);
        assert_eq!(config.database.host, "127.0.0.1");
        assert_eq!(config.database.port, 3307);
        assert_eq!(config.database.user, "admin");
        assert_eq!(config.database.password, "password");
    }
}
