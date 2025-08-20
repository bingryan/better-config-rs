use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error {
    /// Failed to load or read a configuration file
    LoadFileError {
        name: String,
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    /// Failed to parse configuration content (JSON, YAML, TOML, etc.)
    ParseError {
        name: String,
        format: String,
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    /// Invalid configuration value or type conversion error
    ValueError {
        key: String,
        expected_type: String,
        actual_value: String,
    },
    /// Invalid file path or configuration target
    InvalidPathError { path: String, reason: String },
    /// Configuration validation failed
    ValidationError { message: String },
    /// I/O operation failed
    IoError {
        operation: String,
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LoadFileError { name, source } => {
                write!(f, "Failed to load file {}", name)?;
                if let Some(source) = source {
                    write!(f, ": {}", source)?;
                }
                Ok(())
            }
            Error::ParseError {
                name,
                format,
                source,
            } => {
                write!(f, "Failed to parse {} file {} as {}", format, name, format)?;
                if let Some(source) = source {
                    write!(f, ": {}", source)?;
                }
                Ok(())
            }
            Error::ValueError {
                key,
                expected_type,
                actual_value,
            } => {
                write!(
                    f,
                    "Invalid value for key '{}': expected {}, got '{}'",
                    key, expected_type, actual_value
                )
            }
            Error::InvalidPathError { path, reason } => {
                write!(f, "Invalid path '{}': {}", path, reason)
            }
            Error::ValidationError { message } => {
                write!(f, "Configuration validation failed: {}", message)
            }
            Error::IoError { operation, source } => {
                write!(f, "I/O operation '{}' failed", operation)?;
                if let Some(source) = source {
                    write!(f, ": {}", source)?;
                }
                Ok(())
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::LoadFileError { source, .. } => {
                source.as_deref().map(|e| e as &(dyn StdError + 'static))
            }
            Error::ParseError { source, .. } => {
                source.as_deref().map(|e| e as &(dyn StdError + 'static))
            }
            Error::IoError { source, .. } => {
                source.as_deref().map(|e| e as &(dyn StdError + 'static))
            }
            _ => None,
        }
    }
}

// Helper constructors for creating common errors
impl Error {
    pub fn file_not_found(path: &str) -> Self {
        Error::LoadFileError {
            name: path.to_string(),
            source: Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File '{}' not found", path),
            ))),
        }
    }

    pub fn parse_json_error(path: &str, source: impl StdError + Send + Sync + 'static) -> Self {
        Error::ParseError {
            name: path.to_string(),
            format: "JSON".to_string(),
            source: Some(Box::new(source)),
        }
    }

    pub fn parse_yaml_error(path: &str, source: impl StdError + Send + Sync + 'static) -> Self {
        Error::ParseError {
            name: path.to_string(),
            format: "YAML".to_string(),
            source: Some(Box::new(source)),
        }
    }

    pub fn parse_toml_error(path: &str, source: impl StdError + Send + Sync + 'static) -> Self {
        Error::ParseError {
            name: path.to_string(),
            format: "TOML".to_string(),
            source: Some(Box::new(source)),
        }
    }

    pub fn invalid_path(path: &str, reason: &str) -> Self {
        Error::InvalidPathError {
            path: path.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn value_conversion_error(key: &str, expected: &str, actual: &str) -> Self {
        Error::ValueError {
            key: key.to_string(),
            expected_type: expected.to_string(),
            actual_value: actual.to_string(),
        }
    }
}
