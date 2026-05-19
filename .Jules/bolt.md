## 2024-05-24 - SIMD Auto-vectorization Optimization
**Learning:** For numerical calculations in Rust, inner iterator loops (like `zip`) can inhibit LLVM's auto-vectorizer from efficiently generating SIMD instructions.
**Action:** Explicitly unroll arithmetic operations manually (e.g., 8-element chunks) without inner iterators to allow LLVM to auto-vectorize the code more efficiently, yielding significantly better SIMD performance.
