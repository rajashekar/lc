## 2024-04-13 - [Performance] Prefer split_once over splitn(2)
**Learning:** For exactly-two-part string splitting, using `.splitn(2, delimiter).collect::<Vec<_>>()` forces intermediate heap allocations for the `Vec`. The memory instructions specified that `str::split_once()` avoids this and provides better performance and readability.
**Action:** Replaced usage of `.splitn(2, ...).collect::<Vec<_>>()` with `.split_once(...)` across the codebase.

## 2026-03-18 - Optimize ProviderConfig and EndpointTemplates regex compilation
**Learning:** In the template pipeline, regex pattern matching (e.g., matching models to templates) was previously recompiling the `regex::Regex` object for the exact same pattern on every resolution. Since regex compilation is relatively expensive, this caused an unnecessary performance penalty, particularly visible when checking numerous model pattern templates.
**Action:** Utilize `std::sync::OnceLock`, `std::sync::Mutex`, and `std::collections::HashMap` to create a thread-safe, centralized regex cache (in `src/utils/regex_cache.rs`) that prevents recompilation of previously seen patterns, thus optimizing template matching performance.

## 2024-04-20 - Allow LLVM Auto-vectorization with Exact Chunks Zipping
**Learning:** In performance-critical array iterations like cosine similarity calculation, using manual index loops (e.g. `for i in 0..chunks`) on sliced chunks prevents the compiler from mathematically proving that array accesses won't go out-of-bounds, hindering its ability to apply auto-vectorization optimizations.
**Action:** Use idiomatic iterator zipping with `.chunks_exact(N)` instead of indexing to iterate through parallel collections. Extract `.by_ref()` and zip it to ensure bounds checking is inherently eliminated for the chunks, freeing LLVM to automatically apply SIMD instructions safely.
