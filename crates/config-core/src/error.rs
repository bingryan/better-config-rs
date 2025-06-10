use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error {
    LoadFileError {
        name: String,
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
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::LoadFileError { source, .. } => {
                source.as_deref().map(|e| e as &(dyn StdError + 'static))
            }
        }
    }
}
