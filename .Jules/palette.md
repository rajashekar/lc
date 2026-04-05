# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-24 - UTF-8 Character vs Byte Indexing in CLI Input
**Learning:** Rust strings index into bytes, not characters. In CLI manual multi-line input handling, operations like `insert`, `remove`, and cursor left/right bounds checking based on a purely incremental byte `cursor_pos` will lead to `is_char_boundary` panics when users type multi-byte characters like emojis.
**Action:** When working with CLI interactive inputs, use `.chars().count()` for bounds checking and calculate actual byte insertion indices by summing `len_utf8()` for the sequence of characters preceding the cursor character index.
