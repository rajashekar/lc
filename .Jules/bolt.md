## 2024-03-24 - [Initialization]

## 2024-03-24 - [Avoid manual unrolling in Rust iterators]
**Learning:** Manual loop chunking in math/vector logic (like cosine similarity) using `for i in 0..chunks` with index accesses often results in slower or identical performance compared to idiomatic `.iter().zip()` loops, due to missed auto-vectorization bounds check elisions by LLVM.
**Action:** Always prefer idiomatic chained iterator patterns (`a.iter().zip(b.iter())`) for simple math reductions over manual C-style unrolling, as it's cleaner, safer, and typically faster.

## 2024-03-24 - [Precompute norms for repeated vector similarities]
**Learning:** Calculating L2 norms repeatedly for the same vectors inside a search fallback (`find_similar_linear_optimized`) adds unnecessary overhead.
**Action:** Precalculate static mathematical data like vector norms upon insertion, store them alongside the vector (`#[serde(default)]` helps backwards compat), and utilize them in specific fast-path functions (`cosine_similarity_fast`).
