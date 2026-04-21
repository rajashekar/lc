## 2024-04-13 - [Performance] Prefer split_once over splitn(2)
**Learning:** For exactly-two-part string splitting, using `.splitn(2, delimiter).collect::<Vec<_>>()` forces intermediate heap allocations for the `Vec`. The memory instructions specified that `str::split_once()` avoids this and provides better performance and readability.
**Action:** Replaced usage of `.splitn(2, ...).collect::<Vec<_>>()` with `.split_once(...)` across the codebase.

## 2026-03-18 - Optimize ProviderConfig and EndpointTemplates regex compilation
**Learning:** In the template pipeline, regex pattern matching (e.g., matching models to templates) was previously recompiling the `regex::Regex` object for the exact same pattern on every resolution. Since regex compilation is relatively expensive, this caused an unnecessary performance penalty, particularly visible when checking numerous model pattern templates.
**Action:** Utilize `std::sync::OnceLock`, `std::sync::Mutex`, and `std::collections::HashMap` to create a thread-safe, centralized regex cache (in `src/utils/regex_cache.rs`) that prevents recompilation of previously seen patterns, thus optimizing template matching performance.

## 2024-05-18 - Allow LLVM auto-vectorization for performance-critical iterators
**Learning:** To enable LLVM auto-vectorization and elide bounds checks in performance-critical Rust array operations, use standard library iterators like `.chunks_exact(N)` combined with `.zip()` instead of manual index arithmetic loops (e.g., `for i in 0..chunks { a[i] }`). Use `.by_ref()` on the iterators before zipping to prevent them from being consumed, allowing you to use `.remainder()` afterward to process any remaining elements.
**Action:** Replaced manual array index iteration with `.chunks_exact(4)` and `.zip()` in performance-critical SIMD-like operations such as `cosine_similarity_simd`.
