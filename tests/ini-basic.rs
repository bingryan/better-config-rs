use better_config::{env, IniConfig};

#[env(IniConfig(target = "config.ini"))]
pub struct AppConfig {
    #[conf(default = "init_default_key")]
    pub api_key: String,
    #[conf(from = "title", default = "hello ini")]
    pub title: String,
    #[conf(from = "scripts.echo")]
    pub scripts_echo: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn basic_defaults() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "init_default_key");
    }

    #[test]
    #[serial]
    fn basic_ini() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.title, "INI Example");
        assert_eq!(config.scripts_echo, "echo");
    }
}
