# Better-Config for Rust

`better-config` is a library for the configuration of Rust. It is designed to be simple, flexible, and easy to use.

## Features

-   Not need to implement `FromStr` trait for struct
-   Not need to add `Option` for field type
-   Support multiple env files
-   Support getter for struct and return field type directly
-   Flexible architecture, supporting custom loaders

## Supported loader

[✓] env : `EnvConfig` -> load from env file

[✓] toml : `TomlConfig` -> load from toml file

[✓] json : `JsonConfig` -> load from json file

[✓] yaml/yml : `YmlConfig` -> load from yaml/yml file

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

crate features:

-   `env` : for load from env file, default target is `.env`
-   `toml` : for load from toml file, default target is `config.toml`
-   `json` : for load from json file, default target is `config.json`
-   `yml` : for load from yaml/yml file, default target is `config.yml`
-   `full` : for all features

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

## Toml loader

> [!NOTE]
>
> `toml` feature is required
>
> from format: `from = "key"`, key is a dot-separated flattened key path.

```rust
use better_config::{env, TomlConfig};

#[env(TomlConfig)]
pub struct AppConfig {
    #[conf(default = "default_key")]
    api_key: String,
    #[conf(from = "title", default = "hello toml")]
    title: String,
    #[conf(from = "database.enabled", default = "false")]
    database_enabled: bool,

    #[conf(from = "database.ports")]
    database_ports: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "default_key");
    assert_eq!(config.title, "TOML Example");
    assert!(config.database_enabled);
    assert_eq!(config.database_ports, "[8000, 8001, 8002]");
}
```

## Json loader

> [!NOTE]
>
> `json` feature is required
>
> from format: `from = "key"`, key is a dot-separated flattened key path.

```rust
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

fn main() {
    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "json_default_key");
    assert_eq!(config.name, "config.json");
    assert_eq!(config.version, 1.0);
    assert!(config.public);
    assert_eq!(config.echo, "echo");
}
```

## Yaml/yml loader

> [!NOTE]
>
> `yml` feature is required

```rust
use better_config::{env, YmlConfig};

#[env(YmlConfig)]
pub struct AppConfig {
    #[conf(default = "yml_default_key")]
    api_key: String,
    #[conf(from = "title", default = "hello yml")]
    title: String,
    #[conf(from = "database.host", default = "localhost")]
    database_host: String,
    #[conf(from = "database.port")]
    database_port: u16,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "yml_default_key");
    assert_eq!(config.title, "Yml Example");
    assert_eq!(config.database_host, "127.0.0.1");
    assert_eq!(config.database_port, 3306);
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
