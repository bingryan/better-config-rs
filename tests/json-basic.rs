use better_config::{env, JsonConfig};

#[env(JsonConfig)]
pub struct AppConfig {
    #[conf(default = "json_default_key")]
    api_key: String,
    #[conf(from = "name")]
    name: String,
    #[conf(from = "version")]
    version: f64,
    #[conf(from = "public")]
    public: bool,
    #[conf(from = "scripts.echo")]
    echo: String,
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
