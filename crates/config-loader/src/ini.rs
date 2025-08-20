use better_config_core::{AbstractConfig, Error};
use better_config_core::misc;
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
            let file_paths = misc::validate_and_split_paths(&target)?;

            for file_path in file_paths {
                // Check file accessibility before reading
                misc::check_file_accessibility(&file_path)?;

                let ini = Ini::load_from_file(&file_path)
                    .map_err(|e| Error::IoError {
                        operation: format!("load INI file '{}'", file_path),
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