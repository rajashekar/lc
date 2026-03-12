use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const MAX_CACHE_SIZE: usize = 100;

fn regex_cache() -> &'static Mutex<HashMap<String, Result<Regex, regex::Error>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Result<Regex, regex::Error>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::with_capacity(MAX_CACHE_SIZE)))
}

pub fn get_regex(pattern: &str) -> Result<Regex, regex::Error> {
    let mut cache = regex_cache().lock().unwrap();

    // Check cache first
    if let Some(cached) = cache.get(pattern) {
        return cached.clone();
    }

    // Not in cache, compile
    let re = Regex::new(pattern);

    // Simple bounded eviction (clear all if full, not true LRU but safe and fast)
    if cache.len() >= MAX_CACHE_SIZE {
        cache.clear();
    }

    cache.insert(pattern.to_string(), re.clone());
    re
}
