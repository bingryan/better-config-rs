# Better-Config for Rust

`better-config` is a library for the configuration of Rust. It is designed to be simple, flexible, and easy to use.

## Features

-   Not need to implement `FromStr` trait for struct
-   Not need to add `Option` for field type
-   Support multiple env files
-   Support getter for struct and return field type directly
-   Support nested struct
-   Environment variable override for all config formats
-   Flexible architecture, supporting custom loaders

## Supported loader

[✓] env : `EnvConfig` -> load from env file

[✓] toml : `TomlConfig` -> load from toml file

[✓] json : `JsonConfig` -> load from json file

[✓] yaml/yml : `YmlConfig` -> load from yaml/yml file

[✓] ini : `IniConfig` -> load from ini file

[✗] More...

## Installation

Run the following Cargo command in your project directory:

```
cargo add better-config
```

Or add the following line to your Cargo.toml:

```toml
better-config = "0.2"
```

crate features:

-   `env` : for load from env file, default target is `.env`
-   `toml` : for load from toml file, default target is `config.toml`
-   `json` : for load from json file, default target is `config.json`
-   `yml` : for load from yaml/yml file, default target is `config.yml`
-   `ini` : for load from ini file, default target is `config.ini`
-   `full` : for all features

## Usage

### Basic Usage

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig)]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    pub host: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    // load from .env file default
    // if not found, use default value
    assert_eq!(config.host, "env");
}

```

### Add prefix and multiple env files

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "BETTER_", target = ".env.prod,.env.staging,.env.dev"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    pub host: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    // priority: .env.prod > .env.staging > .env.dev
    assert_eq!(config.host, "prod");
}

```

### FromStr for custom Type

if your custom type implement `FromStr`, you can use it directly.

```rust
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
    pub api_key: String,
    #[conf(default = "8000")]
    pub port: u16,
    #[conf(default = "false")]
    pub debug: bool,
    #[conf(from = "ADDRESS")]
    pub address: Address,
}


fn main() {
    let config: AppConfig = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "default_key");
    assert_eq!(config.port, 8080);
    assert!(!config.debug);
    assert_eq!(config.address, Address { ip: "127.0.0.1".to_string(), port: 8080 });
}
```

### Getter for custom struct

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "BETTER_", target = ".env.prod,.env.staging,.env.dev"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    pub host: String,
    #[conf(from = "DB_PORT", default = "8000")]
    pub port: u16,
    #[conf(getter = "get_url")]
    pub url: String,
    #[conf(getter = "get_wrap_url")]
    pub wrap_url: WrapURL,
}

#[derive(Debug, PartialEq)]
pub struct WrapURL(String);

impl AppConfigBuilder {
    pub fn get_url(&self, params: &std::collections::HashMap<String, String>) -> String {
        format!(
            "{}",
            params
                .get("BETTER_DB_HOST")
                .unwrap_or(&"better".to_string())
        )
    }

    pub fn get_wrap_url(&self, p: &std::collections::HashMap<String, String>) -> WrapURL {
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

### Nested struct

#### Nested for EnvConfig

```rust
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

// the target can be diffrent from the AppConfig's target, if you want to split the config file
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


fn main() {
    let config: AppConfig = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "default_key");
    assert!(config.debug);
    assert_eq!(config.database.host, "127.0.0.1");
    assert_eq!(config.database.port, 3307);
    assert_eq!(config.database.user, "admin");
    assert_eq!(config.database.password, "password");

}
```

#### Nested for JsonConfig

```rust
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

// the target can be diffrent from the AppConfig's target, if you want to split the config file
#[env(JsonConfig(prefix = "database.", target = "config-nested.json"))]
pub struct DatabaseConfig {
    #[conf(from = "host", default = "localhost")]
    pub host: String,
    #[conf(from = "port", default = "3306")]
    pub port: u16,
    #[conf(from = "user", default = "root")]
    pub user: String,
    #[conf(from = "password", default = "123456")]
    pub password: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "default_key");
    assert!(config.debug)
    assert_eq!(config.database.host, "127.0.0.1");
    assert_eq!(config.database.port, 3307);
    assert_eq!(config.database.user, "admin");
    assert_eq!(config.database.password, "password");
}
```

### Environment Variable Override

Environment variables can override values from configuration files for all supported formats. This provides a flexible way to customize configuration in different deployment environments without modifying config files.

#### Basic Environment Override

```rust
use better_config::{env, JsonConfig};

#[env(JsonConfig)]
pub struct AppConfig {
    #[conf(from = "database.host", default = "localhost")]
    pub db_host: String,
    #[conf(from = "database.port", default = "5432")]
    pub db_port: u16,
    #[conf(from = "api.key", default = "default-key")]
    pub api_key: String,
}

fn main() {
    // Set environment variables to override config file values
    std::env::set_var("database.host", "prod-server.com");
    std::env::set_var("database.port", "3306");

    let config = AppConfig::builder().build().unwrap();

    // Environment variables take priority over config file values
    assert_eq!(config.db_host, "prod-server.com");  // from env var
    assert_eq!(config.db_port, 3306);               // from env var
    assert_eq!(config.api_key, "file-api-key");     // from config file
}
```

#### Environment Override with Prefix

When using a prefix, environment variables must include the prefix to override config values:

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "APP_"))]
pub struct AppConfig {
    #[conf(from = "DB_HOST", default = "localhost")]
    pub db_host: String,
    #[conf(from = "DB_PORT", default = "5432")]
    pub db_port: u16,
}

fn main() {
    // Environment variables must include the prefix
    std::env::set_var("APP_DB_HOST", "production.db.com");
    std::env::set_var("APP_DB_PORT", "3306");

    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.db_host, "production.db.com");
    assert_eq!(config.db_port, 3306);
}
```

#### Disabling Environment Override for Specific Fields

Use the `no_env_override` attribute to prevent specific fields from being overridden by environment variables:

```rust
use better_config::{env, JsonConfig};

#[env(JsonConfig)]
pub struct AppConfig {
    #[conf(from = "database.host", default = "localhost")]
    pub db_host: String,

    // This field cannot be overridden by environment variables
    #[conf(from = "security.secret", default = "default-secret", no_env_override)]
    pub secret_key: String,
}

fn main() {
    // Set environment variables
    std::env::set_var("database.host", "env-host.com");
    std::env::set_var("security.secret", "env-secret");

    let config = AppConfig::builder().build().unwrap();

    assert_eq!(config.db_host, "env-host.com");      // overridden by env var
    assert_eq!(config.secret_key, "file-secret");    // NOT overridden (uses config file value)
}
```

#### Environment Override with Nested Structures

Nested structures maintain independent environment variable handling with their own prefixes:

```rust
use better_config::{env, EnvConfig};

#[env(EnvConfig(prefix = "APP_"))]
pub struct AppConfig {
    #[conf(from = "NAME", default = "MyApp")]
    pub app_name: String,

    #[env]
    pub database: DatabaseConfig,
}

#[env(EnvConfig(prefix = "DB_"))]
pub struct DatabaseConfig {
    #[conf(from = "HOST", default = "localhost")]
    pub host: String,

    #[conf(from = "PORT", default = "5432")]
    pub port: u16,

    // This field in nested struct cannot be overridden
    #[conf(from = "PASSWORD", default = "default-pass", no_env_override)]
    pub password: String,
}

fn main() {
    // Parent struct uses APP_ prefix
    std::env::set_var("APP_NAME", "ProductionApp");

    // Nested struct uses its own DB_ prefix
    std::env::set_var("DB_HOST", "prod-db.com");
    std::env::set_var("DB_PORT", "3306");
    std::env::set_var("DB_PASSWORD", "env-password");  // This won't override due to no_env_override

    let config = AppConfig::builder().build().unwrap();

    assert_eq!(config.app_name, "ProductionApp");           // overridden by APP_NAME
    assert_eq!(config.database.host, "prod-db.com");       // overridden by DB_HOST
    assert_eq!(config.database.port, 3306);                // overridden by DB_PORT
    assert_eq!(config.database.password, "file-password"); // NOT overridden
}
```

#### Priority Order

The configuration value resolution follows this priority order (highest to lowest):

1. **Environment Variables** (when not excluded by `no_env_override`)
2. **Configuration File Values**
3. **Default Values** (specified in `default` attribute)

This works consistently across all supported configuration formats (JSON, TOML, YAML, INI, ENV).

## More Examples

<details>
<summary>Click to expand examples</summary>

### Toml loader

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
    pub api_key: String,
    #[conf(from = "title", default = "hello toml")]
    pub title: String,
    #[conf(from = "database.enabled", default = "false")]
    pub database_enabled: bool,

    #[conf(from = "database.ports")]
    pub database_ports: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "default_key");
    assert_eq!(config.title, "TOML Example");
    assert!(config.database_enabled);
    assert_eq!(config.database_ports, "[8000, 8001, 8002]");
}
```

### Json loader

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
    pub api_key: String,
    #[conf(from = "name")]
    pub name: String,
    #[conf(from = "version")]
    pub version: f64,
    #[conf(from = "public")]
    pub public: bool,
    #[conf(from = "scripts.echo")]
    pub echo: String,
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

### Yaml/yml loader

> [!NOTE]
>
> `yml` feature is required

```rust
use better_config::{env, YmlConfig};

#[env(YmlConfig)]
pub struct AppConfig {
    #[conf(default = "yml_default_key")]
    pub api_key: String,
    #[conf(from = "title", default = "hello yml")]
    pub title: String,
    #[conf(from = "database.host", default = "localhost")]
    pub database_host: String,
    #[conf(from = "database.port")]
    pub database_port: u16,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "yml_default_key");
    assert_eq!(config.title, "Yml Example");
    assert_eq!(config.database_host, "127.0.0.1");
    assert_eq!(config.database_port, 3306);
}
```

### Ini loader

> [!NOTE]
>
> `ini` feature is required
>
> from format: `from = "key"`, key is a dot-separated flattened key path.

```rust
use better_config::{env, IniConfig};

#[env(IniConfig)]
pub struct AppConfig {
    #[conf(default = "ini_default_key")]
    pub api_key: String,
    #[conf(from = "title", default = "hello ini")]
    pub title: String,
    #[conf(from = "scripts.echo")]
    pub scripts_echo: String,
}

fn main() {
    let config = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "ini_default_key");
    assert_eq!(config.title, "INI Example");
    assert_eq!(config.scripts_echo, "echo");
}
```

### Custom loader

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
    pub api_key: String,
    #[conf(default = "8000")]
    pub port: u16,
    #[conf(default = "false")]
    pub debug: bool,
}

fn main() {
    let config: AppConfig = AppConfig::builder().build().unwrap();
    assert_eq!(config.api_key, "default_key");
    assert_eq!(config.port, 8000);
    assert_eq!(config.debug, false);
}
```

</details>

## Contributing

Contributors are welcomed to join this project. Please check [CONTRIBUTING](./CONTRIBUTING.md) about how to contribute to this project.
