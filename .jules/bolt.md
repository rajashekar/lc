## 2024-04-02 - In-place String Manipulation with split_off
**Learning:** To avoid memory allocations when parsing title/snippet strings separated by a delimiter, iterating over `Vec<String>` with `.into_iter()` allows taking ownership of the string.
**Action:** Use `.into_iter()` and combine `split_off(index + delimiter_len)` to extract the suffix and `truncate(index)` to clean the prefix, reducing allocations from two to one by reusing the original string's capacity for the prefix.
