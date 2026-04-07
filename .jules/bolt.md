## 2026-04-07 - String prefix and suffix separation
**Learning:** To optimize parsing an owned string into a prefix (title) and suffix (snippet) separated by a delimiter, using `split_off` to extract the suffix and `truncate` to clean the prefix reduces the total number of allocations from two to one by reusing the original string's capacity for the prefix. This is especially useful in parsing loops, like search result processing.
**Action:** Use `split_off` combined with `truncate` when splitting owned strings in performance-critical code paths to minimize unnecessary allocations.
