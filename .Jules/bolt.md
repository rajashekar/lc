## 2024-05-18 - Avoid manual loop unrolling/chunking
**Learning:** Manual loop unrolling/chunking for SIMD optimization (like "process in chunks of 4" in `cosine_similarity_fast`) sacrifices readability for micro-optimizations that the Rust compiler's autovectorizer likely handles automatically anyway. It violates the strict directive to not sacrifice readability for micro-optimizations.
**Action:** Use idiomatic, readable iterators (e.g., `a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()`) for mathematical operations unless there is a proven, measured bottleneck that iterators cannot solve.
