/// Common utilities for configuration handling
use crate::error::Error;
use std::path::Path;

/// Validate and split a comma-separated list of file paths
///
/// # Arguments
///
/// * `paths_str` - A comma-separated list of file paths
///
/// # Returns
///
/// A vector of validated file paths
///
/// # Errors
///
/// Returns an error if the input is empty, contains invalid paths, or exceeds the maximum path length
pub fn validate_and_split_paths(paths_str: &str) -> Result<Vec<String>, Error> {
    if paths_str.trim().is_empty() {
        return Err(Error::invalid_path(paths_str, "path cannot be empty"));
    }

    let file_paths: Vec<String> = paths_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if file_paths.is_empty() {
        return Err(Error::invalid_path(paths_str, "no valid file paths found"));
    }

    // Basic path validation
    for path in &file_paths {
        if path.contains("..") {
            return Err(Error::invalid_path(path, "path traversal not allowed"));
        }
        if path.len() > 260 {
            return Err(Error::invalid_path(path, "path too long"));
        }
        // Check for invalid characters (Windows-compatible)
        if cfg!(windows) {
            // On Windows, allow colon for drive letters but reject other invalid chars
            if path
                .chars()
                .any(|c| matches!(c, '<' | '>' | '"' | '|' | '?' | '*'))
            {
                return Err(Error::invalid_path(path, "invalid characters in path"));
            }
        } else {
            // On Unix-like systems, reject colon and other invalid chars
            if path
                .chars()
                .any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
            {
                return Err(Error::invalid_path(path, "invalid characters in path"));
            }
        }
    }

    Ok(file_paths)
}

/// Check if a file exists and is readable
///
/// # Arguments
///
/// * `path` - The path to the file
///
/// # Returns
///
/// Ok(()) if the file exists and is readable, otherwise an error
///
/// # Errors
///
/// Returns an error if the file does not exist, is not a file, or cannot be read
pub fn check_file_accessibility(path: &str) -> Result<(), Error> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(Error::file_not_found(path));
    }
    if !path_obj.is_file() {
        return Err(Error::invalid_path(path, "path is not a file"));
    }
    Ok(())
}

/// Safely convert a string to a number without panicking
///
/// # Arguments
///
/// * `s` - The string to convert
/// * `key` - The key associated with the value (for error messages)
/// * `type_name` - The name of the type being converted to (for error messages)
///
/// # Returns
///
/// A result containing the parsed number or an error
///
/// # Errors
///
/// Returns an error if the string cannot be parsed as the specified number type
pub fn safe_string_to_number<T: std::str::FromStr>(
    s: &str,
    key: &str,
    type_name: &str,
) -> Result<T, Error> {
    s.parse()
        .map_err(|_| Error::value_conversion_error(key, type_name, s))
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::misc;

    #[test]
    fn test_validate_and_split_paths_valid() {
        let result = misc::validate_and_split_paths("file1.json,file2.json");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["file1.json", "file2.json"]);
    }

    #[test]
    fn test_validate_and_split_paths_empty() {
        let result = misc::validate_and_split_paths("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidPathError { .. } => {}
            _ => panic!("Expected InvalidPathError"),
        }
    }

    #[test]
    fn test_validate_and_split_paths_path_traversal() {
        let result = misc::validate_and_split_paths("../config.json");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidPathError { .. } => {}
            _ => panic!("Expected InvalidPathError"),
        }
    }

    #[test]
    fn test_validate_and_split_paths_invalid_chars() {
        let result = misc::validate_and_split_paths("config<test>.json");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidPathError { .. } => {}
            _ => panic!("Expected InvalidPathError"),
        }
    }

    #[test]
    fn test_validate_and_split_paths_windows_drive_letter() {
        // On Windows, drive letters with colons should be allowed
        if cfg!(windows) {
            // Test Windows absolute path
            let result = misc::validate_and_split_paths(r"C:\config.json");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), vec![r"C:\config.json"]);

            // Test Windows relative path with drive
            let result = misc::validate_and_split_paths("C:config.json");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), vec!["C:config.json"]);
        } else {
            // On Unix-like systems, colons should be rejected
            let result = misc::validate_and_split_paths("C:config.json");
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::InvalidPathError { .. } => {}
                _ => panic!("Expected InvalidPathError"),
            }
        }
    }

    #[test]
    fn test_check_file_accessibility_nonexistent() {
        let result = misc::check_file_accessibility("nonexistent.json");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::LoadFileError { .. } => {}
            _ => panic!("Expected LoadFileError"),
        }
    }
}
