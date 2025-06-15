use better_config::{env, EnvConfig};
use std::str::FromStr;

#[derive(Debug, Default, PartialEq)]
pub struct Address {
    pub ip: String,
    pub port: u16,
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(":").collect();
        if parts.len() != 2 {
            return Err("Invalid address format".to_string());
        }

        let ip = parts[0].to_string();
        let port = parts[1]
            .parse::<u16>()
            .map_err(|_| "Invalid port number".to_string())?;

        Ok(Address { ip, port })
    }
}

#[env(EnvConfig)]
pub struct AppConfig {
    #[conf(default = "default_key")]
    api_key: String,
    #[conf(default = "8000")]
    port: u16,
    #[conf(default = "false")]
    debug: bool,
    #[conf(from = "ADDRESS")]
    address: Address,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn fixture() {
        env::remove_var("API_KEY");
        env::remove_var("PORT");
        env::remove_var("DEBUG");
        env::remove_var("ADDRESS");
    }

    #[test]
    #[serial]
    fn basic_env() {
        fixture();
        env::set_var("API_KEY", "test_key");
        env::set_var("PORT", "8080");
        env::set_var("DEBUG", "true");
        env::set_var("ADDRESS", "127.0.0.1:8080");

        let config = AppConfig::builder().build().unwrap();

        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.port, 8080);
        assert!(config.debug);
        assert_eq!(
            config.address,
            Address {
                ip: "127.0.0.1".to_string(),
                port: 8080
            }
        );
    }

    #[test]
    #[serial]
    fn basic_from_str() {
        fixture();
        env::set_var("ADDRESS", "192.168.1.1:9000");

        let config = AppConfig::builder().build().unwrap();

        assert_eq!(
            config.address,
            Address {
                ip: "192.168.1.1".to_string(),
                port: 9000
            }
        );
    }
}
