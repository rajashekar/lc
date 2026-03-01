

## 2024-05-15 - [TokenCounter Caching Initialization Fix]
**Learning:** Found an issue where the `ENCODER_CACHE` mapping logic used a `CoreBPE` value directly rather than wrapped in an `Arc`. The `TokenCounter::new` function previously created `encoder: CoreBPE` and put it into the cache directly (`cache.put(..., new_encoder.clone())`), and also cloned it out. `CoreBPE` can be somewhat heavy, and `clone()` on it might involve more overhead than cloning an `Arc`. We wrap it in an `Arc` to make caching much cheaper, which also reduces the struct footprint. The cache values themselves are now `Arc<CoreBPE>`, meaning we only clone the Arc, not the underlying BPE struct.
**Action:** When working with types that have potentially expensive `clone()` operations like Tokenizers/BPEs, verify if we can share them via `Arc` inside lazy_static caches instead of cloning the full structures.
