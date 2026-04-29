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

## 2024-05-30 - Off-by-one errors in text parsing bounds
**Learning:** In code attempting to parse specific formatted strings (e.g., `] URL Source: `) using `.find()` and manual string slicing, off-by-one byte boundary calculation errors are easy to introduce, leading to artifacts like leading whitespace or panics.
**Action:** Replace manual slicing and `find()` usage with `.split_once()` or similar standard library string parsing functions. These are safer, inherently handle correct byte boundary extraction without explicit math, and prevent double-scanning the string.
