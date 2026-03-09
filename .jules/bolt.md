
## 2024-05-19 - Fast Vector Similarity with Cached Norms
**Learning:** Pre-computing and caching L2 norms during vector insertion (and lazy instantiation) eliminates the need for expensive square root arithmetic in the linear search loop during similarity calculation. `.iter().zip()` combined with a precomputed norm provides an optimal, highly-vectorizable O(N) calculation loop instead of manual chunking.
**Action:** When implementing mathematical aggregations in tight loops, look for invariances like norms that can be precalculated at the data entry phase. Use idiomatic Rust `.iter().zip()` loops.
