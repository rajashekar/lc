## 2024-04-06 - String Parsing Optimization (split_off and truncate)
**Learning:** When separating a delimiter-separated string into prefix and suffix, using `.split()` often involves cloning strings and creating extra allocations.
**Action:** Use `.into_iter()` to take ownership of strings inside collections and apply `split_off(pos + delimiter_len)` to extract the suffix while modifying the string in-place, and `truncate(pos)` to clean up the prefix. This minimizes string allocations from 2 to 1 and reuses the string's capacity for the prefix.
