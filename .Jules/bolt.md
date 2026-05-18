
## 2024-05-18 - Optimize SIMD vector calculations
**Learning:** For floating point SIMD calculations in Rust (like cosine similarity), inner iterators like `.zip()` can obscure loop structures from LLVM. Explicitly unrolling the mathematical operations manually (e.g., in 8-element chunks) allows for much better auto-vectorization, yielding roughly a 2x performance increase over zip-based iteration.
**Action:** Always manually unroll math-heavy numeric loops in performance-critical code paths to give the compiler maximum visibility for emitting optimal SIMD instructions.
