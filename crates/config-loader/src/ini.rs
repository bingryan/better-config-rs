use better_config_core::{AbstractConfig, Error};
use ini::Ini;
use std::collections::HashMap;

/// Indicates that structure can be initialized from INI file.
pub trait IniConfig<T = HashMap<String, String>>: AbstractConfig<T> {
    /// Load specified INI file and initialize the structure.
    ///
    /// # Arguments
    /// * `target` - Path to the INI file.
    ///
    /// # Errors
    /// * `Error::LoadFileError` - If the specified INI file cannot be loaded or parsed.
    fn load(target: Option<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized,
    {
        let target = target.or(Some("config.ini".to_string()));

        let mut ini_map = HashMap::new();

        if let Some(target) = target {
            let file_paths: Vec<String> = target
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            for file_path in file_paths {
                let ini = Ini::load_from_file(&file_path).map_err(|e| Error::LoadFileError {
                    name: file_path.clone(),
                    source: Some(Box::new(e)),
                })?;

                for (section, props) in ini.iter() {
                    let section_prefix = match section {
                        Some(s) => format!("{}.", s),
                        None => String::new(),
                    };

                    for (key, value) in props.iter() {
                        ini_map.insert(format!("{}{}", section_prefix, key), value.to_string());
                    }
                }
            }
        }

        Ok(ini_map.into())
    }
}