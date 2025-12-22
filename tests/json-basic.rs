use better_config::{env, JsonConfig};

#[env(JsonConfig(target = "config.json"))]
pub struct AppConfig {
    #[conf(default = "json_default_key")]
    pub api_key: String,
    #[conf(from = "name", default = "default.json")]
    pub name: String,
    #[conf(from = "version", default = "0.0")]
    pub version: f64,
    #[conf(from = "public", default = "false")]
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
        // Verify config.json exists and is readable
        let config_path = "config.json";

        // Debug: Print current working directory on failure
        if !std::path::Path::new(config_path).exists() {
            if let Ok(cwd) = std::env::current_dir() {
                eprintln!("Current working directory: {:?}", cwd);
                if let Ok(entries) = std::fs::read_dir(&cwd) {
                    eprintln!("Directory contents:");
                    for entry in entries.flatten() {
                        eprintln!("  {:?}", entry.file_name());
                    }
                }
            }
            panic!("config.json file does not exist in current directory");
        }

        let content = std::fs::read_to_string(config_path)
            .expect("Failed to read config.json");

        // Verify JSON content is valid
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .expect("Failed to parse config.json as valid JSON");

        // Verify the expected fields exist in JSON
        assert!(json_value.get("public").is_some(), "JSON missing 'public' field");
        assert_eq!(json_value["public"], true, "JSON 'public' field should be true");

        let config = AppConfig::builder().build().unwrap();
        assert_eq!(config.api_key, "json_default_key");
        assert_eq!(config.name, "config.json");
        assert_eq!(config.version, 1.0);
        assert!(config.public, "Expected config.public to be true, but got false. _params: {:?}", config._params);
        assert_eq!(config.echo, "echo");
    }
}