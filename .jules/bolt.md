
## 2024-05-14 - Optimize string parsing allocations using split_off and truncate
**Learning:** Parsing an owned string into two distinct strings (e.g. prefix/suffix) using standard slices and `.to_string()` requires two new allocations. This codebase frequently parses titles and snippets from a single string.
**Action:** Use `.into_iter()` to take ownership of objects rather than references when parsing structures, and use `.split_off(pos)` to extract the suffix into a new allocation while using `.truncate(pos)` on the original string. This drops total allocations per parse from two (or more) down to just one while reusing the original string capacity for the prefix.
