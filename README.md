# Better-Config for Rust

`better-config` is a library for the configuration of Rust. It is designed to be simple, flexible, and easy to use.

## Features

-   Not need to implement `FromStr` trait for struct
-   Not need to add `Option` for field type
-   Support multiple env files
-   Support getter for struct and return field type directly
-   Flexible architecture, supporting custom loaders

`supported loaders:`

[✓] env : `EnvConfig` -> load from env file

[✗] More...

## Installation

Run the following Cargo command in your project directory:

```
cargo add better-config
```

Or add the following line to your Cargo.toml:

```toml
better-config = "0.1"
```

## Examples

### Basic Usage

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig)]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    host: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    // load from .env file default
    // if not found, use default value
    assert_eq!(config.host, "env");
}

```

### Add Prefix and multiple env files

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "BETTER_", target = ".env.prod,.env.staging,.env.dev"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    host: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    // priority: .env.prod > .env.staging > .env.dev
    assert_eq!(config.host, "prod");
}

```

### Getter for custom struct

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "BETTER_", target = ".env.prod,.env.staging,.env.dev"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    host: String,
    #[conf(from = "DB_PORT", default = "8000")]
    port: u16,
    #[conf(getter = "get_url")]
    url: String,
    #[conf(getter = "get_wrap_url")]
    wrap_url: WrapURL,
}

#[derive(Debug, PartialEq)]
struct WrapURL(String);

impl AppConfigBuilder {
    fn get_url(&self, params: &std::collections::HashMap<String, String>) -> String {
        format!(
            "{}",
            params
                .get("BETTER_DB_HOST")
                .unwrap_or(&"better".to_string())
        )
    }

    fn get_wrap_url(&self, p: &std::collections::HashMap<String, String>) -> WrapURL {
        WrapURL(format!(
            "{}",
            p.get("BETTER_DB_HOST")
                .unwrap_or(&"better wrap url".to_string())
        ))
    }
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    // priority: .env.prod > .env.staging > .env.dev
    assert_eq!(config.host, "prod");
    assert_eq!(config.port, 8000);
    assert_eq!(config.url, "prod");
    assert_eq!(config.wrap_url, WrapURL("prod".to_string()));
}
```

## Custom loader

if you want to custom loader, you can implement `AbstractConfig` trait and custom load function.

```rust
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
            // step2: load result from you logic, getter params from return HashMap
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

fn main() {
    let config: AppConfig = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "default_key");
    assert_eq!(config.port, 8000);
    assert_eq!(config.debug, false);
}
```

## Contributing

Contributors are welcomed to join this project. Please check [CONTRIBUTING](./CONTRIBUTING.md) about how to contribute to this project.
