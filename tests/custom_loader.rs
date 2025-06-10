use better_config::{env, AbstractConfig, Error};
use std::collections::HashMap;

pub mod custom {
    use super::*;
    pub trait Config<T = HashMap<String, String>>: AbstractConfig {
        fn load(target: Option<String>) -> Result<T, Error>
        where
            T: Default,
            HashMap<String, String>: Into<T>,
            Self: Sized,
        {
            // step1: pre load environment variables from target file
            println!("Loading environment variables from target: {:?}", target);
            // step2: load result from environment variables and return
            // getter params from return HashMap
            let mut map = HashMap::new();
            for (key, value) in std::env::vars() {
                map.insert(key, value);
            }
            Ok(map.into())
        }
    }
}

#[env(custom::Config)]
pub struct AppConfig {
    #[conf(default = "default_key")]
    api_key: String,
    #[conf(default = "8000")]
    port: u16,
    #[conf(default = "false")]
    debug: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn basic_test_env_macro() {
        let config: AppConfig = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "default_key");
        assert_eq!(config.port, 8000);
        assert!(!config.debug);
    }
}
