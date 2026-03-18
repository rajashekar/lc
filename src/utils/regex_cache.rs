use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Thread-safe cache for compiled regex patterns
static REGEX_CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();

/// Gets a compiled Regex from the cache or compiles and caches it if not present.
/// Returns an error if the pattern is invalid.
pub fn get_regex(pattern: &str) -> Result<Regex, regex::Error> {
    let cache_mutex = REGEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Attempt to read from the cache first
    {
        let cache = cache_mutex.lock().unwrap();
        if let Some(re) = cache.get(pattern) {
            return Ok(re.clone());
        }
    }

    // Compile outside the lock to avoid holding the lock during computation
    let re = Regex::new(pattern)?;

    // Write to the cache
    {
        let mut cache = cache_mutex.lock().unwrap();
        // Insert and return the clone
        cache.insert(pattern.to_string(), re.clone());
    }

    Ok(re)
}
