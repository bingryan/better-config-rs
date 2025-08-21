use better_config::{env, EnvConfig};

#[env(EnvConfig(target = ".env.nested"))]
pub struct AppConfig {
    #[conf(default = "default_key")]
    pub api_key: String,
    #[conf(from = "DEBUG", default = "false")]
    pub debug: bool,
    #[env]
    pub database: DatabaseConfig,
}

#[env(EnvConfig(prefix = "DATABASE_", target = ".env.nested"))]
pub struct DatabaseConfig {
    #[conf(from = "HOST", default = "localhost")]
    pub host: String,
    #[conf(from = "PORT", default = "3306")]
    pub port: u16,
    #[conf(from = "USER", default = "root")]
    pub user: String,
    #[conf(from = "PASSWORD", default = "123456")]
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn fixture() {
        env::remove_var("DEBUG");
        env::remove_var("DATABASE_HOST");
        env::remove_var("DATABASE_PORT");
        env::remove_var("DATABASE_USER");
        env::remove_var("DATABASE_PASSWORD");
    }

    #[test]
    #[serial]
    fn basic_defaults() {
        fixture();

        let config: AppConfig = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "default_key");
        assert!(config.debug);
    }

    #[test]
    #[serial]
    fn basic_nested() {
        fixture();
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.database.host, "127.0.0.1");
        assert_eq!(config.database.port, 3307);
        assert_eq!(config.database.user, "admin");
        assert_eq!(config.database.password, "password");
    }
}
