use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::str::FromStr;

/// Get the value of the environment variable `key` as `T`.
/// If the variable is not set or cannot be parsed, it returns `None`.
/// /// # Example
/// ```rust
/// use std::env;
/// use std::str::FromStr;
/// use better_config_core::utils::env::get_optional;
///
/// let value: Option<u32> = get_optional("MY_ENV_VAR");
/// match value {
///     Some(v) => println!("Value: {}", v),
///     None => println!("Variable not set or cannot be parsed"),
/// }
/// ```
pub fn get_optional<K: AsRef<OsStr>, T: FromStr>(key: K) -> Option<T> {
    env::var(key).ok().and_then(|val| val.parse::<T>().ok())
}

/// Get the value of the environment variable `key` as `T`.
/// If the variable is not set or cannot be parsed, it returns `default`.
/// ## Example
/// ```rust
/// use std::env;
/// use std::str::FromStr;
/// use better_config_core::utils::env::get_optional_or;
///
/// let value: Option<u32> = get_optional_or("MY_ENV_VAR", 42);
/// match value {
///   Some(v) => println!("Value: {}", v),
///    None => println!("Variable not set or cannot be parsed, using default"),
/// }
///
/// ```
///
pub fn get_optional_or<K: AsRef<OsStr>, T: FromStr>(key: K, default: T) -> Option<T> {
    match env::var(key) {
        Ok(val) => val.parse::<T>().ok(),
        Err(_) => Some(default),
    }
}

/// Get the value of the environment variable `key` as `T`.
/// If the variable is not set or cannot be parsed, it returns the result of `f`.
/// ## Example
/// ```rust
/// use std::env;
/// use std::str::FromStr;
/// use better_config_core::utils::env::get_optional_or_else;
///
/// let value: Option<u32> = get_optional_or_else("MY_ENV_VAR", || 42);
/// match value {
///  Some(v) => println!("Value: {}", v),
///  None => println!("Variable not set or cannot be parsed, using fallback"),
/// }
///
/// ```
///
pub fn get_optional_or_else<K: AsRef<OsStr>, T: FromStr, F>(key: K, f: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    match env::var(key) {
        Ok(val) => val.parse::<T>().ok(),
        Err(_) => Some(f()),
    }
}

/// Get the value of the environment variable `key` as `T`.
/// If the variable is not set or cannot be parsed, it panics with a message.
/// ## Example
/// ```rust
/// use std::env;
/// use std::str::FromStr;
/// use better_config_core::utils::env::get;
///
/// env::set_var("MY_ENV_VAR", "42");
/// let value: u32 = get("MY_ENV_VAR");
/// println!("Value: {}", value);
///
/// ```
pub fn get<K: AsRef<OsStr>, T: FromStr>(key: K) -> T {
    get_optional(&key).unwrap_or_else(|| {
        panic!(
            "Environment variable '{}' is not set or cannot be parsed",
            &key.as_ref().to_str().unwrap()
        )
    })
}

/// Get the value of the environment variable `key` as `T`.
/// If the variable is not set or cannot be parsed, it returns `default`.
/// ## Example
/// ```rust
/// use std::env;
/// use std::str::FromStr;
/// use better_config_core::utils::env::get_or;
///
/// let value: u32 = get_or("MY_ENV_VAR", 42);
/// println!("Value: {}", value);
///
/// ```
pub fn get_or<K: AsRef<OsStr>, T: FromStr>(key: K, default: T) -> T {
    match env::var(key) {
        Ok(val) => val.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}

/// Get the value of the environment variable `key` as `T`.
/// If the variable is not set or cannot be parsed, it returns the result of `f`.
/// ## Example
/// ```rust
/// use std::env;
/// use std::str::FromStr;
/// use better_config_core::utils::env::get_or_else;
///
/// let value: u32 = get_or_else("MY_ENV_VAR", || 42);
/// println!("Value: {}", value);
///
/// ```
pub fn get_or_else<K: AsRef<OsStr>, T: FromStr, F>(key: K, f: F) -> T
where
    F: FnOnce() -> T,
{
    match env::var(key) {
        Ok(val) => val.parse::<T>().unwrap_or_else(|_| f()),
        Err(_) => f(),
    }
}

/// Get the value of the environment variable `key` as `T` from a provided hashmap.
/// If the variable is not set or cannot be parsed, it returns `default`.
/// ## Example
/// ```rust
/// use std::collections::HashMap;
/// use std::env;
/// use std::ffi::OsStr;
/// use std::str::FromStr;
/// use better_config_core::utils::env::get_or_with_hashmap;
///
/// let mut hashmap = HashMap::new();
/// hashmap.insert("MY_ENV_VAR".to_string(), "42".to_string());
/// let value: u32 = get_or_with_hashmap("MY_ENV_VAR", 0, Some(&hashmap));
/// println!("Value: {}", value);
///
/// ```
pub fn get_or_with_hashmap<K: AsRef<OsStr>, T, S: ::std::hash::BuildHasher>(
    key: K,
    default: T,
    hashmap: Option<&HashMap<String, String, S>>,
) -> T
where
    T: FromStr,
{
    match hashmap {
        None => get_or(key, default),
        Some(hashmap) => {
            if let Some(value) = hashmap.get(key.as_ref().to_str().unwrap()) {
                value.parse::<T>().unwrap_or(default)
            } else {
                default
            }
        }
    }
}

/// Get the value of the environment variable `key` as `T` from a provided hashmap.
/// If the variable is not set or cannot be parsed, it returns the result of `f`.
/// ## Example
/// ```rust
/// use std::env;
/// use std::ffi::OsStr;
/// use std::str::FromStr;
/// use std::collections::HashMap;
/// use better_config_core::utils::env::get_or_else_with_hashmap;
///
/// let mut hashmap = HashMap::new();
/// hashmap.insert("MY_ENV_VAR".to_string(), "42".to_string());
/// let value: u32 = get_or_else_with_hashmap("MY_ENV_VAR", || 0, Some(&hashmap));
/// println!("Value: {}", value);
///
/// ```
///
pub fn get_or_else_with_hashmap<K: AsRef<OsStr>, T: FromStr, F, S: ::std::hash::BuildHasher>(
    key: K,
    f: F,
    hashmap: Option<&HashMap<String, String, S>>,
) -> T
where
    F: FnOnce() -> T,
{
    match hashmap {
        None => get_or_else(key, f),
        Some(hashmap) => {
            if let Some(value) = hashmap.get(key.as_ref().to_str().unwrap()) {
                value.parse::<T>().unwrap_or_else(|_| f())
            } else {
                f()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_existing_var() {
        env::set_var("TEST_VAR", "42");
        let result: u32 = get("TEST_VAR");
        assert_eq!(result, 42);
        env::remove_var("TEST_VAR");
    }

    #[test]
    #[should_panic(expected = "Environment variable 'MISSING_VAR' is not set or cannot be parsed")]
    fn test_get_missing_var() {
        let _: u32 = get("MISSING_VAR");
    }

    #[test]
    #[should_panic(expected = "cannot be parsed")]
    fn test_get_invalid_format() {
        env::set_var("INVALID_VAR", "not_a_number");
        let _: u32 = get("INVALID_VAR");
        env::remove_var("INVALID_VAR");
    }

    #[test]
    fn test_get_with_different_types() {
        env::set_var("STR_VAR", "hello");
        env::set_var("BOOL_VAR", "true");
        env::set_var("FLOAT_VAR", "1.23");

        let s: String = get("STR_VAR");
        let b: bool = get("BOOL_VAR");
        let f: f64 = get("FLOAT_VAR");

        assert_eq!(s, "hello");
        assert!(b);
        assert_eq!(f, 1.23);

        env::remove_var("STR_VAR");
        env::remove_var("BOOL_VAR");
        env::remove_var("FLOAT_VAR");
    }

    #[test]
    fn test_get_optional_existing_var() {
        env::set_var("GET_EXISTING_VAR", "42");
        let result: Option<u32> = get_optional("GET_EXISTING_VAR");
        assert_eq!(result, Some(42));
        env::remove_var("GET_EXISTING_VAR");
    }

    #[test]
    fn test_get_optional_missing_var() {
        let result: Option<u32> = get_optional("MISSING_VAR");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_optional_invalid_format() {
        env::set_var("INVALID_VAR", "not_a_number");
        let result: Option<u32> = get_optional("INVALID_VAR");
        assert_eq!(result, None);
        env::remove_var("INVALID_VAR");
    }

    #[test]
    fn test_get_optional_or_existing_var() {
        env::set_var("EXISTING_VAR", "42");
        let result: Option<u32> = get_optional_or("EXISTING_VAR", 0);
        assert_eq!(result, Some(42));
        env::remove_var("EXISTING_VAR");
    }

    #[test]
    fn test_get_optional_or_missing_var() {
        let result: Option<u32> = get_optional_or("MISSING_VAR", 0);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_get_optional_or_else_existing_var() {
        env::set_var("EXISTING_VAR", "42");
        let result: Option<u32> = get_optional_or_else("EXISTING_VAR", || 0);
        assert_eq!(result, Some(42));
        env::remove_var("EXISTING_VAR");
    }

    #[test]
    fn test_get_optional_or_else_missing_var() {
        let result: Option<u32> = get_optional_or_else("MISSING_VAR", || 0);
        assert_eq!(result, Some(0));
    }

    #[test]
    #[should_panic]
    fn test_get_optional_or_else_panic() {
        let result: Option<u32> = get_optional_or_else("PANIC_VAR", || panic!("This should panic"));
        assert!(result.is_none());
        env::remove_var("PANIC_VAR");
    }

    #[test]
    fn test_get_or_existing_var() {
        env::set_var("EXISTING_VAR", "42");
        let result: u32 = get_or("EXISTING_VAR", 0);
        assert_eq!(result, 42);
        env::remove_var("EXISTING_VAR");
    }

    #[test]
    fn test_get_or_missing_var() {
        let result: u32 = get_or("MISSING_VAR", 0);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_or_else_existing_var() {
        env::set_var("EXISTING_VAR", "42");
        let result: u32 = get_or_else("EXISTING_VAR", || 0);
        assert_eq!(result, 42);
        env::remove_var("EXISTING_VAR");
    }

    #[test]
    fn test_get_or_else_missing_var() {
        let result: u32 = get_or_else("MISSING_VAR", || 0);
        assert_eq!(result, 0);
    }

    #[test]
    #[should_panic]
    fn test_get_or_else_panic() {
        let result: u32 = get_or_else("PANIC_VAR", || panic!("This should panic"));
        assert_eq!(result, 0);
        env::remove_var("PANIC_VAR");
    }

    #[test]
    fn test_get_with_hashmap_existing_var() {
        let mut hashmap = HashMap::new();
        hashmap.insert("EXISTING_VAR".to_string(), "42".to_string());
        let result: u32 = get_or_with_hashmap("EXISTING_VAR", 0, Some(&hashmap));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_get_with_hashmap_missing_var() {
        let mut hashmap = HashMap::new();
        hashmap.insert("EXISTING_VAR".to_string(), "42".to_string());
        let result: u32 = get_or_with_hashmap("MISSING_VAR", 0, Some(&hashmap));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_with_hashmap_none() {
        let result: u32 = get_or_with_hashmap::<_, _, std::collections::hash_map::RandomState>(
            "EXISTING_VAR",
            0,
            None,
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_or_else_with_hashmap_existing_var() {
        let mut hashmap = HashMap::new();
        hashmap.insert("EXISTING_VAR".to_string(), "42".to_string());
        let result: u32 = get_or_else_with_hashmap("EXISTING_VAR", || 0, Some(&hashmap));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_get_or_else_with_hashmap_missing_var() {
        let mut hashmap = HashMap::new();
        hashmap.insert("EXISTING_VAR".to_string(), "42".to_string());
        let result: u32 = get_or_else_with_hashmap("MISSING_VAR", || 0, Some(&hashmap));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_or_else_with_hashmap_none() {
        let result: u32 = get_or_else_with_hashmap::<
            _,
            _,
            _,
            std::collections::hash_map::RandomState,
        >("EXISTING_VAR", || 0, None);
        assert_eq!(result, 0);
    }

    #[test]
    #[should_panic]
    fn test_get_or_else_with_hashmap_panic() {
        let mut hashmap = HashMap::new();
        hashmap.insert("PANIC_VAR".to_string(), "42".to_string());
        let result: u32 = get_or_else_with_hashmap::<
            _,
            _,
            _,
            std::collections::hash_map::RandomState,
        >("PANIC_VAR", || panic!("This should panic"), Some(&hashmap));
        assert_eq!(result, 0);
        env::remove_var("PANIC_VAR");
    }

    #[test]
    fn test_get_with_hashmap_with_different_types() {
        let mut hashmap = HashMap::new();
        hashmap.insert("STR_VAR".to_string(), "hello".to_string());
        hashmap.insert("BOOL_VAR".to_string(), "true".to_string());
        hashmap.insert("FLOAT_VAR".to_string(), "1.23".to_string());
        let s: String =
            get_or_with_hashmap::<_, _, _>("STR_VAR", "default".to_string(), Some(&hashmap));
        let b: bool = get_or_with_hashmap::<_, _, _>("BOOL_VAR", false, Some(&hashmap));
        let f: f64 = get_or_with_hashmap::<_, _, _>("FLOAT_VAR", 0.0, Some(&hashmap));
        assert_eq!(s, "hello");
        assert!(b);
        assert_eq!(f, 1.23);
    }
}
