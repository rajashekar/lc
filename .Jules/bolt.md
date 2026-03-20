## 2024-05-18 - Optimized Cosine Similarity Calculations

**Learning:** In Rust, manual loop unrolling and chunking for mathematical operations on vectors (like calculating cosine similarity) can actually hinder performance compared to using idiomatic iterators. By using `.iter().zip()`, LLVM can automatically and reliably apply SIMD vectorization and loop unrolling, bypassing manual implementations.

**Action:** Replace manual loop unrolling and chunking loops with idiomatic `.iter().zip()` in vector and matrix mathematics for both improved performance and better readability. Let the Rust compiler and LLVM optimize these patterns automatically.
