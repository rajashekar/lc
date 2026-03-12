
## 2024-03-12 - [Centralize and Cache Regex Creation]
**Learning:** Repetitive creation of identical Regex instances, particularly inside loops parsing configs or templates, imposes unnecessary compilation overhead. `Regex::new` is expensive.
**Action:** Implemented a centralized `get_regex` function in `src/utils/regex_cache.rs` backed by a `std::sync::OnceLock` and a `std::sync::Mutex` wrapping a `HashMap` to prevent memory leaks and avoid external dependencies like `lazy_static` or `lru`. Updated `TemplateProcessor` and `ProviderConfig` to use `crate::utils::regex_cache::get_regex` rather than `regex::Regex::new`, skipping re-compilations.
