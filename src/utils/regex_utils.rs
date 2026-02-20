use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// Global regex cache: pattern -> Option<Regex>
// Storing Option<Regex> allows caching compilation failures to avoid retrying invalid patterns
static REGEX_CACHE: OnceLock<Mutex<HashMap<String, Option<Regex>>>> = OnceLock::new();

/// Get a compiled regex from the cache, or compile and cache it.
/// Returns None if the regex is invalid.
pub fn get_cached_regex(pattern: &str) -> Option<Regex> {
    let cache = REGEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    let mut map = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(), // Handle poisoned mutex gracefully
    };

    if let Some(re) = map.get(pattern) {
        return re.clone();
    }

    let re = Regex::new(pattern).ok();
    map.insert(pattern.to_string(), re.clone());
    re
}
