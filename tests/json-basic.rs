use better_config::{env, JsonConfig};

#[env(JsonConfig)]
pub struct AppConfig {
    #[conf(default = "json_default_key")]
    pub api_key: String,
    #[conf(from = "name")]
    pub name: String,
    #[conf(from = "version")]
    pub version: f64,
    #[conf(from = "public", default = "false", no_env_override)]
    pub public: bool,
    #[conf(from = "scripts.echo", default = "default_echo")]
    pub echo: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn basic_defaults() {
        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "json_default_key");
        assert_eq!(config.name, "config.json");
        assert_eq!(config.version, 1.0);
        assert!(config.public);
        assert_eq!(config.echo, "echo");
    }
}
