## 2024-05-24 - Idiomatic Rust Loops for Better SIMD
**Learning:** In Rust, manual loop unrolling and chunking (e.g., `let chunks = a.len() / 4; for i in 0..chunks { ... }`) often defeats LLVM's auto-vectorizer and introduces unavoidable runtime bounds-checking overhead.
**Action:** Always prefer idiomatic iterators like `a.iter().zip(b.iter())` for vector mathematics. Under the hood, this leverages `TrustedRandomAccess`, allowing the compiler to completely elide bounds checks and reliably apply automatic SIMD vectorization and loop unrolling, resulting in faster and safer code.
