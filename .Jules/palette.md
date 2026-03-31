# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-24 - Multi-byte Character Boundaries
**Learning:** In interactive CLI prompts, users often input emojis or multi-byte characters. When using custom input loops tracking cursor position, treating character indices as byte indices causes a panic (`assertion failed: self.is_char_boundary(idx)`) when attempting to slice or mutate strings.
**Action:** Always map the logical cursor position (character index) to the correct byte index (`.chars().take(idx).map(|c| c.len_utf8()).sum()`) before calling string operations like `insert`, `remove`, or slicing.
