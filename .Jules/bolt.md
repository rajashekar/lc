## 2024-05-14 - Precomputing L2 Norms for Cosine Similarity
**Learning:** Precomputing L2 norms of vectors and storing them on `VectorEntry` instances skips expensive square root operations and redundant math during each cosine similarity comparison when performing linear scans over large datasets.
**Action:** When performing heavily repetitive math over stored data objects, cache the calculation on the data model if it's static over the object's lifetime to avoid recomputation on O(N) comparisons.
