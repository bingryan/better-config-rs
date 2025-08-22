use better_config::{env, EnvConfig};
use serde::{Deserialize, Serialize};

#[env(EnvConfig)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[conf(default = "default_key")]
    pub api_key: String,
    #[conf(default = "8000")]
    pub port: u16,
    #[conf(default = "false")]
    pub debug: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    trait HasDebug {}
    impl<T: std::fmt::Debug> HasDebug for T {}

    fn check_debug<T: HasDebug>() -> bool {
        true
    }

    trait HasClone {}
    impl<T: Clone> HasClone for T {}
    fn check_clone<T: HasClone>() -> bool {
        true
    }



    #[test]
    #[serial]
    fn basic_with_derive_clone() {
        assert!(check_clone::<AppConfig>());
    }

    #[test]
    #[serial]
    fn basic_with_derive_debug() {
        assert!(check_debug::<AppConfig>());
    }
}
