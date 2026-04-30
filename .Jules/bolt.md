## 2024-04-13 - [Performance] Prefer split_once over splitn(2)
**Learning:** For exactly-two-part string splitting, using `.splitn(2, delimiter).collect::<Vec<_>>()` forces intermediate heap allocations for the `Vec`. The memory instructions specified that `str::split_once()` avoids this and provides better performance and readability.
**Action:** Replaced usage of `.splitn(2, ...).collect::<Vec<_>>()` with `.split_once(...)` across the codebase.

## 2026-03-18 - Optimize ProviderConfig and EndpointTemplates regex compilation
**Learning:** In the template pipeline, regex pattern matching (e.g., matching models to templates) was previously recompiling the `regex::Regex` object for the exact same pattern on every resolution. Since regex compilation is relatively expensive, this caused an unnecessary performance penalty, particularly visible when checking numerous model pattern templates.
**Action:** Utilize `std::sync::OnceLock`, `std::sync::Mutex`, and `std::collections::HashMap` to create a thread-safe, centralized regex cache (in `src/utils/regex_cache.rs`) that prevents recompilation of previously seen patterns, thus optimizing template matching performance.

## 2024-05-20 - Array operations performance and bounds checking
**Learning:** Manual index arithmetic loops (e.g. `for i in 0..chunks`) in Rust can prevent LLVM from automatically vectorizing code and eliding bounds checks for array operations.
**Action:** Use standard library iterators combined with `.zip()`, such as `.chunks_exact(N)`, `.by_ref()`, and `.remainder()`, to ensure bounds checks are elided and LLVM optimizations are enabled for performance-critical code paths.

## 2024-04-26 - Double String Scanning in Text Parsing
**Learning:** In text parsing tasks (like extracting titles and snippets from search results), developers frequently use consecutive `.find()` operations with slicing instead of `.split_once()`. This results in scanning the string twice and creating intermediate String allocations.
**Action:** Always prefer `str::split_once()` when splitting a string into exactly two parts, as it eliminates redundant string scans and avoids unnecessary heap allocations, improving both execution speed and code readability.

## 2024-05-25 - Avoid Buffer Destruction in Stream Processing
**Learning:** In a continuous stream-processing loop reading SSE chunks, replacing `buffer.drain(..=newline_pos)` with `buffer = remaining` (where `remaining` is a newly allocated `String`) drops the original allocated capacity and creates a new one fitted perfectly to its current length. This causes a severe performance regression because the subsequent `buffer.push_str()` on the next chunk will immediately trigger a costly O(N) heap reallocation every single time.
**Action:** When popping data from the front of an active `String` buffer, always use `.drain(..=idx)` instead of recreating the string to preserve the buffer's capacity and prevent allocation churn in hot paths.
