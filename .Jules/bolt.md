## 2024-04-13 - [Performance] Prefer split_once over splitn(2)
**Learning:** For exactly-two-part string splitting, using `.splitn(2, delimiter).collect::<Vec<_>>()` forces intermediate heap allocations for the `Vec`. The memory instructions specified that `str::split_once()` avoids this and provides better performance and readability.
**Action:** Replaced usage of `.splitn(2, ...).collect::<Vec<_>>()` with `.split_once(...)` across the codebase.

## 2026-03-18 - Optimize ProviderConfig and EndpointTemplates regex compilation
**Learning:** In the template pipeline, regex pattern matching (e.g., matching models to templates) was previously recompiling the `regex::Regex` object for the exact same pattern on every resolution. Since regex compilation is relatively expensive, this caused an unnecessary performance penalty, particularly visible when checking numerous model pattern templates.
**Action:** Utilize `std::sync::OnceLock`, `std::sync::Mutex`, and `std::collections::HashMap` to create a thread-safe, centralized regex cache (in `src/utils/regex_cache.rs`) that prevents recompilation of previously seen patterns, thus optimizing template matching performance.
## 2024-04-14 - [Performance] Double scan optimization in parsers
**Learning:** In string parsing heavy sections (like JSON/text API response handling), repeated index scanning via `.find()` to extract adjacent components (e.g., parsing `title - snippet`) results in unneeded O(N) operations. Using `.split_once()` completely avoids redundant scans and eliminates intermediate byte math.
**Action:** Always prefer `str::split_once()` when cleanly extracting exactly two components from string API responses to minimize scan passes and manual slicing bugs.
