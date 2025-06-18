use std::collections::HashMap;

use crate::error::Error;

/// The trait for configuration loader.
pub trait AbstractConfig<T = HashMap<String, String>> {
    /// Load target file and initialize the structure.
    fn load(target: Option<String>) -> Result<T, Error>
    where
        T: Default,
        HashMap<String, String>: Into<T>,
        Self: Sized;
}
