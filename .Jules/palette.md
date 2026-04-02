# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2025-02-28 - Multi-byte Character Handling in Custom Input UIs
**Learning:** In Rust (and many other languages), terminal user interfaces that manage their own cursor position and string modification often fall into the trap of using logical character indices interchangeably with byte indices. This causes `is_char_boundary` panics when users type multi-byte characters (like emojis or non-English text) because string operations like `insert` and `remove` expect byte indices.
**Action:** When implementing or modifying custom input handlers, always explicitly calculate the byte index from the logical character index (e.g., via `str.chars().take(logical_idx).map(|c| c.len_utf8()).sum::<usize>()`) before modifying strings. Ensure bounds checks use `.chars().count()` instead of `.len()`.
