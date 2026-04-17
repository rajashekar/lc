## 2024-04-13 - [Performance] Prefer split_once over splitn(2)
**Learning:** For exactly-two-part string splitting, using `.splitn(2, delimiter).collect::<Vec<_>>()` forces intermediate heap allocations for the `Vec`. The memory instructions specified that `str::split_once()` avoids this and provides better performance and readability.
**Action:** Replaced usage of `.splitn(2, ...).collect::<Vec<_>>()` with `.split_once(...)` across the codebase.

## 2026-03-18 - Optimize ProviderConfig and EndpointTemplates regex compilation
**Learning:** In the template pipeline, regex pattern matching (e.g., matching models to templates) was previously recompiling the `regex::Regex` object for the exact same pattern on every resolution. Since regex compilation is relatively expensive, this caused an unnecessary performance penalty, particularly visible when checking numerous model pattern templates.
**Action:** Utilize `std::sync::OnceLock`, `std::sync::Mutex`, and `std::collections::HashMap` to create a thread-safe, centralized regex cache (in `src/utils/regex_cache.rs`) that prevents recompilation of previously seen patterns, thus optimizing template matching performance.

## 2026-03-18 - Optimize array iteration for LLVM auto-vectorization
**Learning:** Manual index and pointer arithmetic loops (e.g., `for i in 0..chunks`) hinder LLVM from effectively applying auto-vectorization and eliding bounds checks in Rust. Using iterator methods like `.chunks_exact(N)` combined with `.zip()` provides the compiler with structural guarantees, leading to significantly better vectorized performance.
**Action:** Replaced manual chunked indexing with `chunks_exact` and `.zip` for vector dot product operations in `cosine_similarity_simd` and `cosine_similarity_precomputed`.
